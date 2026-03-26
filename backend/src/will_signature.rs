//! # Digital Signature Service (Task 4)
//!
//! Enables users to sign legal will documents using their crypto wallet.
//! Uses Ed25519 (Stellar) signatures, binds to document hash + vault ID,
//! and prevents replay attacks via a nonce system.

use crate::api_error::ApiError;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Duration, Utc};
use ring::digest::{digest, SHA256};
use ring::signature;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use stellar_strkey::Strkey;
use uuid::Uuid;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningChallengeRequest {
    pub document_id: Uuid,
    pub vault_id: String,
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningChallenge {
    pub challenge_id: Uuid,
    pub message: String,
    pub message_hash: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitSignatureRequest {
    pub challenge_id: Uuid,
    pub wallet_address: String,
    /// Hex-encoded Ed25519 signature over the challenge message
    pub signature_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillSignatureRecord {
    pub id: Uuid,
    pub document_id: Uuid,
    pub vault_id: String,
    pub wallet_address: String,
    pub document_hash: String,
    pub signature_hex: String,
    pub signed_at: DateTime<Utc>,
}

// ─── Service ──────────────────────────────────────────────────────────────────

pub struct WillSignatureService;

impl WillSignatureService {
    /// Step 1: Generate a signing challenge (message + nonce) for the user.
    /// The message binds document_hash + vault_id + nonce to prevent replay attacks.
    pub async fn create_challenge(
        db: &PgPool,
        req: &SigningChallengeRequest,
    ) -> Result<SigningChallenge, ApiError> {
        // Fetch document hash from DB
        let doc_hash: Option<String> =
            sqlx::query_scalar("SELECT will_hash FROM will_documents WHERE id = $1")
                .bind(req.document_id)
                .fetch_optional(db)
                .await?;

        let document_hash = doc_hash.ok_or_else(|| {
            ApiError::NotFound(format!("Will document {} not found", req.document_id))
        })?;

        let challenge_id = Uuid::new_v4();
        let nonce = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::minutes(10);

        // Canonical message: "INHERITX_WILL_SIGN:{doc_hash}:{vault_id}:{nonce}"
        let message = format!(
            "INHERITX_WILL_SIGN:{}:{}:{}",
            document_hash, req.vault_id, nonce
        );

        // Hash the message for reference
        let hash_bytes = digest(&SHA256, message.as_bytes());
        let message_hash = hex::encode(hash_bytes.as_ref());

        // Persist challenge
        sqlx::query(
            r#"
            INSERT INTO will_signing_challenges
                (id, document_id, vault_id, wallet_address, message, message_hash, nonce, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(challenge_id)
        .bind(req.document_id)
        .bind(&req.vault_id)
        .bind(&req.wallet_address)
        .bind(&message)
        .bind(&message_hash)
        .bind(&nonce)
        .bind(expires_at)
        .execute(db)
        .await?;

        Ok(SigningChallenge {
            challenge_id,
            message,
            message_hash,
            expires_at,
        })
    }

    /// Step 2: Verify the wallet signature and store the binding.
    pub async fn verify_and_store(
        db: &PgPool,
        req: &SubmitSignatureRequest,
    ) -> Result<WillSignatureRecord, ApiError> {
        let mut tx = db.begin().await?;

        // Fetch and lock challenge row (prevents concurrent replay)
        #[derive(sqlx::FromRow)]
        struct ChallengeRow {
            document_id: Uuid,
            vault_id: String,
            wallet_address: String,
            message: String,
            message_hash: String,
            expires_at: DateTime<Utc>,
            used: bool,
        }

        let row = sqlx::query_as::<_, ChallengeRow>(
            "SELECT document_id, vault_id, wallet_address, message, message_hash, expires_at, used \
             FROM will_signing_challenges WHERE id = $1 FOR UPDATE",
        )
        .bind(req.challenge_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::NotFound("Signing challenge not found".to_string()))?;

        // Replay attack prevention: challenge must not be used
        if row.used {
            return Err(ApiError::BadRequest(
                "Signing challenge already used".to_string(),
            ));
        }

        // Expiry check
        if row.expires_at < Utc::now() {
            return Err(ApiError::BadRequest(
                "Signing challenge has expired".to_string(),
            ));
        }

        // Wallet address must match the challenge
        if row.wallet_address.to_lowercase() != req.wallet_address.to_lowercase() {
            return Err(ApiError::Unauthorized);
        }

        // Decode public key from wallet address
        let public_key_bytes = Self::decode_public_key(&req.wallet_address)?;

        // Decode signature
        let sig_bytes = hex::decode(&req.signature_hex)
            .map_err(|_| ApiError::BadRequest("Invalid signature hex".to_string()))?;

        // Verify Ed25519 signature over the challenge message
        let peer_public_key =
            signature::UnparsedPublicKey::new(&signature::ED25519, &public_key_bytes);
        peer_public_key
            .verify(row.message.as_bytes(), &sig_bytes)
            .map_err(|_| ApiError::Unauthorized)?;

        // Mark challenge as used (replay prevention)
        sqlx::query("UPDATE will_signing_challenges SET used = true WHERE id = $1")
            .bind(req.challenge_id)
            .execute(&mut *tx)
            .await?;

        // Persist signature record
        let record_id = Uuid::new_v4();
        let signed_at = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO will_signatures
                (id, document_id, vault_id, wallet_address, document_hash, signature_hex, signed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(record_id)
        .bind(row.document_id)
        .bind(&row.vault_id)
        .bind(&req.wallet_address)
        .bind(&row.message_hash)
        .bind(&req.signature_hex)
        .bind(signed_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Fetch plan_id for event
        let plan_id: Option<Uuid> =
            sqlx::query_scalar("SELECT plan_id FROM will_documents WHERE id = $1")
                .bind(row.document_id)
                .fetch_optional(db)
                .await?;

        // Emit WillSigned event
        if let Some(plan_id) = plan_id {
            let sig_hash =
                ring::digest::digest(&ring::digest::SHA256, req.signature_hex.as_bytes());
            let event = crate::will_events::WillEvent::WillSigned {
                vault_id: row.vault_id.clone(),
                document_id: row.document_id,
                plan_id,
                signer: req.wallet_address.clone(),
                signature_hash: hex::encode(sig_hash.as_ref()),
                timestamp: signed_at,
            };
            if let Err(e) = crate::will_events::WillEventService::emit(db, event).await {
                tracing::warn!("Failed to emit WillSigned event: {}", e);
            }
        }

        Ok(WillSignatureRecord {
            id: record_id,
            document_id: row.document_id,
            vault_id: row.vault_id,
            wallet_address: req.wallet_address.clone(),
            document_hash: row.message_hash,
            signature_hex: req.signature_hex.clone(),
            signed_at,
        })
    }

    /// Retrieve all signatures for a document.
    pub async fn get_signatures_for_document(
        db: &PgPool,
        document_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<WillSignatureRecord>, ApiError> {
        // Verify the document belongs to this user
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM will_documents WHERE id = $1 AND user_id = $2)",
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_one(db)
        .await?;

        if !exists {
            return Err(ApiError::NotFound(format!(
                "Will document {document_id} not found"
            )));
        }

        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            document_id: Uuid,
            vault_id: String,
            wallet_address: String,
            document_hash: String,
            signature_hex: String,
            signed_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, document_id, vault_id, wallet_address, document_hash, signature_hex, signed_at \
             FROM will_signatures WHERE document_id = $1 ORDER BY signed_at DESC",
        )
        .bind(document_id)
        .fetch_all(db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| WillSignatureRecord {
                id: r.id,
                document_id: r.document_id,
                vault_id: r.vault_id,
                wallet_address: r.wallet_address,
                document_hash: r.document_hash,
                signature_hex: r.signature_hex,
                signed_at: r.signed_at,
            })
            .collect())
    }

    /// Decode a Stellar G-address or hex string into 32 raw public key bytes.
    fn decode_public_key(wallet_address: &str) -> Result<[u8; 32], ApiError> {
        if wallet_address.starts_with('G') && wallet_address.len() == 56 {
            let strkey = Strkey::from_string(wallet_address)
                .map_err(|_| ApiError::BadRequest("Invalid Stellar address".to_string()))?;
            match strkey {
                Strkey::PublicKeyEd25519(pk) => Ok(pk.0),
                _ => Err(ApiError::BadRequest(
                    "Only Ed25519 public keys supported".to_string(),
                )),
            }
        } else {
            // Fallback: hex-encoded raw key (used in tests)
            let bytes = hex::decode(wallet_address)
                .map_err(|_| ApiError::BadRequest("Invalid wallet address format".to_string()))?;
            bytes
                .try_into()
                .map_err(|_| ApiError::BadRequest("Invalid public key length".to_string()))
        }
    }

    /// Build the canonical signing message without DB (for client-side use).
    pub fn build_message(document_hash: &str, vault_id: &str, nonce: &str) -> String {
        format!("INHERITX_WILL_SIGN:{document_hash}:{vault_id}:{nonce}")
    }

    /// Verify a signature without DB interaction (stateless check).
    pub fn verify_signature(
        wallet_address: &str,
        message: &str,
        signature_hex: &str,
    ) -> Result<(), ApiError> {
        let public_key_bytes = Self::decode_public_key(wallet_address)?;
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|_| ApiError::BadRequest("Invalid signature hex".to_string()))?;
        let peer_public_key =
            signature::UnparsedPublicKey::new(&signature::ED25519, &public_key_bytes);
        peer_public_key
            .verify(message.as_bytes(), &sig_bytes)
            .map_err(|_| ApiError::Unauthorized)
    }

    /// Encode raw bytes as base64 (utility for clients).
    pub fn encode_base64(data: &[u8]) -> String {
        BASE64.encode(data)
    }
}

// ─── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ring::signature::KeyPair;

    fn generate_keypair() -> (ring::signature::Ed25519KeyPair, Vec<u8>) {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let kp = ring::signature::Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let pub_key = kp.public_key().as_ref().to_vec();
        (kp, pub_key)
    }

    #[test]
    fn test_build_message_format() {
        let msg = WillSignatureService::build_message("hash123", "vault-1", "nonce-abc");
        assert_eq!(msg, "INHERITX_WILL_SIGN:hash123:vault-1:nonce-abc");
    }

    #[test]
    fn test_verify_valid_signature() {
        let (kp, pub_key_bytes) = generate_keypair();
        let wallet_address = hex::encode(&pub_key_bytes);
        let message = "INHERITX_WILL_SIGN:abc:vault-1:nonce-xyz";
        let sig = kp.sign(message.as_bytes());
        let sig_hex = hex::encode(sig.as_ref());
        assert!(WillSignatureService::verify_signature(&wallet_address, message, &sig_hex).is_ok());
    }

    #[test]
    fn test_reject_invalid_signature() {
        let (kp, pub_key_bytes) = generate_keypair();
        let wallet_address = hex::encode(&pub_key_bytes);
        let message = "INHERITX_WILL_SIGN:abc:vault-1:nonce-xyz";
        let sig = kp.sign(message.as_bytes());
        let mut sig_bytes = sig.as_ref().to_vec();
        // Tamper with signature
        sig_bytes[0] ^= 0xFF;
        let bad_sig_hex = hex::encode(&sig_bytes);
        assert!(
            WillSignatureService::verify_signature(&wallet_address, message, &bad_sig_hex).is_err()
        );
    }

    #[test]
    fn test_reject_wrong_message() {
        let (kp, pub_key_bytes) = generate_keypair();
        let wallet_address = hex::encode(&pub_key_bytes);
        let message = "INHERITX_WILL_SIGN:abc:vault-1:nonce-xyz";
        let sig = kp.sign(message.as_bytes());
        let sig_hex = hex::encode(sig.as_ref());
        // Verify against different message
        assert!(WillSignatureService::verify_signature(
            &wallet_address,
            "INHERITX_WILL_SIGN:different:vault-1:nonce-xyz",
            &sig_hex
        )
        .is_err());
    }

    #[test]
    fn test_invalid_hex_signature_rejected() {
        let (_, pub_key_bytes) = generate_keypair();
        let wallet_address = hex::encode(&pub_key_bytes);
        assert!(WillSignatureService::verify_signature(
            &wallet_address,
            "some message",
            "not-valid-hex!!"
        )
        .is_err());
    }

    #[test]
    fn test_base64_encode_utility() {
        let data = b"hello world";
        let encoded = WillSignatureService::encode_base64(data);
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_decode_public_key_wrong_length() {
        // hex of 16 bytes (too short)
        let short_hex = hex::encode([0u8; 16]);
        assert!(WillSignatureService::verify_signature(&short_hex, "msg", "aabb").is_err());
    }
}

//! # Witness Verification System (Issue #331)
//!
//! Allows witnesses to review and sign legal will documents.
//! Witnesses are invited via wallet address or email, can sign using
//! Ed25519 signatures, and their status is tracked (pending/signed/declined).

use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use ring::signature;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use stellar_strkey::Strkey;
use uuid::Uuid;

// --- Types -------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessRecord {
    pub id: Uuid,
    pub document_id: Uuid,
    pub wallet_address: Option<String>,
    pub email: Option<String>,
    pub status: String,
    pub signature_hex: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
    pub invited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessStatusSummary {
    pub document_id: Uuid,
    pub total: i64,
    pub signed: i64,
    pub pending: i64,
    pub declined: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InviteWitnessRequest {
    pub wallet_address: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WitnessSignRequest {
    pub wallet_address: String,
    pub signature_hex: String,
}

// --- Service -----------------------------------------------------------------

pub struct WitnessService;

impl WitnessService {
    /// Invite a witness to a will document. The document must belong to the inviting user.
    pub async fn invite_witness(
        db: &PgPool,
        user_id: Uuid,
        document_id: Uuid,
        wallet_address: Option<String>,
        email: Option<String>,
    ) -> Result<WitnessRecord, ApiError> {
        if wallet_address.is_none() && email.is_none() {
            return Err(ApiError::BadRequest(
                "Either wallet_address or email must be provided".to_string(),
            ));
        }

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

        let id = Uuid::new_v4();
        let invited_at = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO will_witnesses
                (id, document_id, inviter_user_id, wallet_address, email, status, invited_at)
            VALUES ($1, $2, $3, $4, $5, 'pending', $6)
            "#,
        )
        .bind(id)
        .bind(document_id)
        .bind(user_id)
        .bind(&wallet_address)
        .bind(&email)
        .bind(invited_at)
        .execute(db)
        .await?;

        Ok(WitnessRecord {
            id,
            document_id,
            wallet_address,
            email,
            status: "pending".to_string(),
            signature_hex: None,
            signed_at: None,
            invited_at,
        })
    }

    /// List all witnesses for a document. The document must belong to the requesting user.
    pub async fn get_witnesses(
        db: &PgPool,
        user_id: Uuid,
        document_id: Uuid,
    ) -> Result<Vec<WitnessRecord>, ApiError> {
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
            wallet_address: Option<String>,
            email: Option<String>,
            status: String,
            signature_hex: Option<String>,
            signed_at: Option<DateTime<Utc>>,
            invited_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, document_id, wallet_address, email, status, signature_hex, signed_at, invited_at \
             FROM will_witnesses WHERE document_id = $1 ORDER BY invited_at DESC",
        )
        .bind(document_id)
        .fetch_all(db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| WitnessRecord {
                id: r.id,
                document_id: r.document_id,
                wallet_address: r.wallet_address,
                email: r.email,
                status: r.status,
                signature_hex: r.signature_hex,
                signed_at: r.signed_at,
                invited_at: r.invited_at,
            })
            .collect())
    }

    /// Witness signs a document. Verifies the Ed25519 signature against the document hash.
    pub async fn sign_as_witness(
        db: &PgPool,
        witness_id: Uuid,
        wallet_address: &str,
        signature_hex: &str,
    ) -> Result<WitnessRecord, ApiError> {
        let mut tx = db.begin().await?;

        #[derive(sqlx::FromRow)]
        struct WitnessRow {
            id: Uuid,
            document_id: Uuid,
            wallet_address: Option<String>,
            email: Option<String>,
            status: String,
            invited_at: DateTime<Utc>,
        }

        let witness = sqlx::query_as::<_, WitnessRow>(
            "SELECT id, document_id, wallet_address, email, status, invited_at \
             FROM will_witnesses WHERE id = $1 FOR UPDATE",
        )
        .bind(witness_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::NotFound("Witness record not found".to_string()))?;

        if witness.status != "pending" {
            return Err(ApiError::BadRequest(format!(
                "Witness has already {}",
                witness.status
            )));
        }

        // Verify the wallet address matches the invited witness
        if let Some(ref invited_wallet) = witness.wallet_address {
            if invited_wallet.to_lowercase() != wallet_address.to_lowercase() {
                return Err(ApiError::Unauthorized);
            }
        }

        // Fetch the document hash to verify the signature against
        let doc_hash: String =
            sqlx::query_scalar("SELECT will_hash FROM will_documents WHERE id = $1")
                .bind(witness.document_id)
                .fetch_one(&mut *tx)
                .await?;

        // Verify Ed25519 signature over the document hash
        let public_key_bytes = decode_public_key(wallet_address)?;
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|_| ApiError::BadRequest("Invalid signature hex".to_string()))?;
        let peer_public_key =
            signature::UnparsedPublicKey::new(&signature::ED25519, &public_key_bytes);
        peer_public_key
            .verify(doc_hash.as_bytes(), &sig_bytes)
            .map_err(|_| ApiError::Unauthorized)?;

        let signed_at = Utc::now();

        sqlx::query(
            "UPDATE will_witnesses SET status = 'signed', signature_hex = $1, signed_at = $2 WHERE id = $3",
        )
        .bind(signature_hex)
        .bind(signed_at)
        .bind(witness_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(WitnessRecord {
            id: witness.id,
            document_id: witness.document_id,
            wallet_address: witness.wallet_address,
            email: witness.email,
            status: "signed".to_string(),
            signature_hex: Some(signature_hex.to_string()),
            signed_at: Some(signed_at),
            invited_at: witness.invited_at,
        })
    }

    /// Witness declines to sign.
    pub async fn decline_witness(db: &PgPool, witness_id: Uuid) -> Result<WitnessRecord, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            document_id: Uuid,
            wallet_address: Option<String>,
            email: Option<String>,
            status: String,
            signature_hex: Option<String>,
            signed_at: Option<DateTime<Utc>>,
            invited_at: DateTime<Utc>,
        }

        let witness = sqlx::query_as::<_, Row>(
            "SELECT id, document_id, wallet_address, email, status, signature_hex, signed_at, invited_at \
             FROM will_witnesses WHERE id = $1",
        )
        .bind(witness_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Witness record not found".to_string()))?;

        if witness.status != "pending" {
            return Err(ApiError::BadRequest(format!(
                "Witness has already {}",
                witness.status
            )));
        }

        sqlx::query("UPDATE will_witnesses SET status = 'declined' WHERE id = $1")
            .bind(witness_id)
            .execute(db)
            .await?;

        Ok(WitnessRecord {
            id: witness.id,
            document_id: witness.document_id,
            wallet_address: witness.wallet_address,
            email: witness.email,
            status: "declined".to_string(),
            signature_hex: witness.signature_hex,
            signed_at: witness.signed_at,
            invited_at: witness.invited_at,
        })
    }

    /// Get a summary of witness statuses for a document.
    pub async fn get_witness_status(
        db: &PgPool,
        document_id: Uuid,
    ) -> Result<WitnessStatusSummary, ApiError> {
        #[derive(sqlx::FromRow)]
        struct CountRow {
            total: Option<i64>,
            signed: Option<i64>,
            pending: Option<i64>,
            declined: Option<i64>,
        }

        let row = sqlx::query_as::<_, CountRow>(
            r#"
            SELECT
                COUNT(*) AS total,
                COUNT(*) FILTER (WHERE status = 'signed') AS signed,
                COUNT(*) FILTER (WHERE status = 'pending') AS pending,
                COUNT(*) FILTER (WHERE status = 'declined') AS declined
            FROM will_witnesses
            WHERE document_id = $1
            "#,
        )
        .bind(document_id)
        .fetch_one(db)
        .await?;

        Ok(WitnessStatusSummary {
            document_id,
            total: row.total.unwrap_or(0),
            signed: row.signed.unwrap_or(0),
            pending: row.pending.unwrap_or(0),
            declined: row.declined.unwrap_or(0),
        })
    }
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
        let bytes = hex::decode(wallet_address)
            .map_err(|_| ApiError::BadRequest("Invalid wallet address format".to_string()))?;
        bytes
            .try_into()
            .map_err(|_| ApiError::BadRequest("Invalid public key length".to_string()))
    }
}

// --- Unit Tests --------------------------------------------------------------

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
    fn test_witness_status_summary_defaults() {
        let summary = WitnessStatusSummary {
            document_id: Uuid::new_v4(),
            total: 0,
            signed: 0,
            pending: 0,
            declined: 0,
        };
        assert_eq!(summary.total, 0);
        assert_eq!(summary.signed, 0);
        assert_eq!(summary.pending, 0);
        assert_eq!(summary.declined, 0);
    }

    #[test]
    fn test_witness_record_creation() {
        let now = Utc::now();
        let doc_id = Uuid::new_v4();
        let record = WitnessRecord {
            id: Uuid::new_v4(),
            document_id: doc_id,
            wallet_address: Some("GABCDEF".to_string()),
            email: None,
            status: "pending".to_string(),
            signature_hex: None,
            signed_at: None,
            invited_at: now,
        };
        assert_eq!(record.document_id, doc_id);
        assert_eq!(record.status, "pending");
        assert!(record.wallet_address.is_some());
        assert!(record.email.is_none());
        assert!(record.signature_hex.is_none());
        assert!(record.signed_at.is_none());
    }

    #[test]
    fn test_decode_public_key_hex() {
        let (_, pub_key_bytes) = generate_keypair();
        let wallet_address = hex::encode(&pub_key_bytes);
        let decoded = decode_public_key(&wallet_address).unwrap();
        assert_eq!(decoded.as_slice(), pub_key_bytes.as_slice());
    }

    #[test]
    fn test_decode_public_key_invalid_hex() {
        let result = decode_public_key("not-valid-hex!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_public_key_wrong_length() {
        let short_hex = hex::encode([0u8; 16]);
        let result = decode_public_key(&short_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_invite_witness_request_deserialization() {
        let json = r#"{"wallet_address": "GABCDEF", "email": null}"#;
        let req: InviteWitnessRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.wallet_address, Some("GABCDEF".to_string()));
        assert!(req.email.is_none());
    }

    #[test]
    fn test_witness_sign_request_deserialization() {
        let json = r#"{"wallet_address": "GABCDEF", "signature_hex": "aabb"}"#;
        let req: WitnessSignRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.wallet_address, "GABCDEF");
        assert_eq!(req.signature_hex, "aabb");
    }

    #[test]
    fn test_signature_verification_via_decode_and_ring() {
        let (kp, pub_key_bytes) = generate_keypair();
        let wallet_address = hex::encode(&pub_key_bytes);
        let message = "test-document-hash-abc123";
        let sig = kp.sign(message.as_bytes());
        let sig_hex = hex::encode(sig.as_ref());

        let public_key_bytes = decode_public_key(&wallet_address).unwrap();
        let peer_public_key =
            signature::UnparsedPublicKey::new(&signature::ED25519, &public_key_bytes);
        assert!(peer_public_key
            .verify(message.as_bytes(), sig.as_ref())
            .is_ok());

        // Also verify hex decoding path
        let decoded_sig = hex::decode(&sig_hex).unwrap();
        assert!(peer_public_key
            .verify(message.as_bytes(), &decoded_sig)
            .is_ok());
    }
}

//! Encrypted document storage with backup support.
//!
//! Provides AES-256-GCM encryption for will documents at rest,
//! per-user access control, and a backup mechanism.

use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::digest::{digest, SHA256};
use ring::hkdf::{Salt, HKDF_SHA256};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

// ---------------------------------------------------------------------------
// Encryption helpers
// ---------------------------------------------------------------------------

fn derive_key(secret: &[u8]) -> Result<LessSafeKey, ApiError> {
    let salt = Salt::new(HKDF_SHA256, b"inheritx-document-encryption");
    let prk = salt.extract(secret);
    let okm = prk
        .expand(&[b"aes-256-gcm-key"], &AES_256_GCM)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Key derivation failed")))?;
    let mut key_bytes = [0u8; KEY_LEN];
    okm.fill(&mut key_bytes)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Key material extraction failed")))?;
    let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to create encryption key")))?;
    Ok(LessSafeKey::new(unbound))
}

fn load_encryption_secret() -> Vec<u8> {
    std::env::var("DOCUMENT_ENCRYPTION_KEY")
        .unwrap_or_default()
        .into_bytes()
}

fn encrypt_bytes(plaintext: &[u8], secret: &[u8]) -> Result<(Vec<u8>, Vec<u8>), ApiError> {
    let key = derive_key(secret)?;
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to generate nonce")))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Encryption failed")))?;

    Ok((in_out, nonce_bytes.to_vec()))
}

fn decrypt_bytes(
    ciphertext: &[u8],
    nonce_bytes: &[u8],
    secret: &[u8],
) -> Result<Vec<u8>, ApiError> {
    let key = derive_key(secret)?;
    let mut nonce_arr = [0u8; NONCE_LEN];
    if nonce_bytes.len() != NONCE_LEN {
        return Err(ApiError::Internal(anyhow::anyhow!("Invalid nonce length")));
    }
    nonce_arr.copy_from_slice(nonce_bytes);
    let nonce = Nonce::assume_unique_for_key(nonce_arr);

    let mut in_out = ciphertext.to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Decryption failed")))?;
    Ok(plaintext.to_vec())
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: Uuid,
    pub document_id: Uuid,
    pub user_id: Uuid,
    pub backup_hash: String,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct DocumentStorageService;

impl DocumentStorageService {
    /// Encrypt an existing document's content and store the ciphertext.
    pub async fn store_encrypted(
        db: &PgPool,
        user_id: Uuid,
        document_id: Uuid,
        content_bytes: &[u8],
    ) -> Result<(), ApiError> {
        // Verify ownership
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM will_documents WHERE id = $1 AND user_id = $2)",
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_one(db)
        .await?;

        if !exists {
            return Err(ApiError::Forbidden(
                "Not authorized to access this document".to_string(),
            ));
        }

        let secret = load_encryption_secret();
        if secret.is_empty() {
            return Err(ApiError::Internal(anyhow::anyhow!(
                "DOCUMENT_ENCRYPTION_KEY is not configured"
            )));
        }

        let (ciphertext, nonce) = encrypt_bytes(content_bytes, &secret)?;

        sqlx::query(
            "UPDATE will_documents \
             SET encrypted_content = $1, encryption_nonce = $2, is_encrypted = TRUE \
             WHERE id = $3 AND user_id = $4",
        )
        .bind(&ciphertext)
        .bind(&nonce)
        .bind(document_id)
        .bind(user_id)
        .execute(db)
        .await?;

        Ok(())
    }

    /// Retrieve and decrypt a document's encrypted content.
    pub async fn retrieve_decrypted(
        db: &PgPool,
        user_id: Uuid,
        document_id: Uuid,
    ) -> Result<Vec<u8>, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            encrypted_content: Option<Vec<u8>>,
            encryption_nonce: Option<Vec<u8>>,
            is_encrypted: bool,
        }

        let row = sqlx::query_as::<_, Row>(
            "SELECT encrypted_content, encryption_nonce, is_encrypted \
             FROM will_documents WHERE id = $1 AND user_id = $2",
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Document {document_id} not found")))?;

        if !row.is_encrypted {
            return Err(ApiError::BadRequest(
                "Document is not encrypted".to_string(),
            ));
        }

        let ciphertext = row
            .encrypted_content
            .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("Missing encrypted content")))?;
        let nonce = row
            .encryption_nonce
            .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("Missing encryption nonce")))?;

        let secret = load_encryption_secret();
        if secret.is_empty() {
            return Err(ApiError::Internal(anyhow::anyhow!(
                "DOCUMENT_ENCRYPTION_KEY is not configured"
            )));
        }

        decrypt_bytes(&ciphertext, &nonce, &secret)
    }

    /// Create an encrypted backup of a document.
    pub async fn create_backup(
        db: &PgPool,
        user_id: Uuid,
        document_id: Uuid,
    ) -> Result<BackupRecord, ApiError> {
        #[derive(sqlx::FromRow)]
        struct DocRow {
            encrypted_content: Option<Vec<u8>>,
            encryption_nonce: Option<Vec<u8>>,
            is_encrypted: bool,
        }

        let doc = sqlx::query_as::<_, DocRow>(
            "SELECT encrypted_content, encryption_nonce, is_encrypted \
             FROM will_documents WHERE id = $1 AND user_id = $2",
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Document {document_id} not found")))?;

        if !doc.is_encrypted {
            return Err(ApiError::BadRequest(
                "Document must be encrypted before creating a backup".to_string(),
            ));
        }

        let ciphertext = doc
            .encrypted_content
            .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("Missing encrypted content")))?;
        let nonce = doc
            .encryption_nonce
            .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("Missing encryption nonce")))?;

        let hash_bytes = digest(&SHA256, &ciphertext);
        let backup_hash = hex::encode(hash_bytes.as_ref());

        let backup_id = Uuid::new_v4();
        let created_at = Utc::now();

        sqlx::query(
            "INSERT INTO document_backups \
             (id, document_id, user_id, backup_hash, encrypted_content, encryption_nonce, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(backup_id)
        .bind(document_id)
        .bind(user_id)
        .bind(&backup_hash)
        .bind(&ciphertext)
        .bind(&nonce)
        .bind(created_at)
        .execute(db)
        .await?;

        Ok(BackupRecord {
            id: backup_id,
            document_id,
            user_id,
            backup_hash,
            created_at,
        })
    }

    /// List all backups for a given document.
    pub async fn list_backups(
        db: &PgPool,
        user_id: Uuid,
        document_id: Uuid,
    ) -> Result<Vec<BackupRecord>, ApiError> {
        // Verify ownership
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM will_documents WHERE id = $1 AND user_id = $2)",
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_one(db)
        .await?;

        if !exists {
            return Err(ApiError::Forbidden(
                "Not authorized to access this document".to_string(),
            ));
        }

        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            document_id: Uuid,
            user_id: Uuid,
            backup_hash: String,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, document_id, user_id, backup_hash, created_at \
             FROM document_backups \
             WHERE document_id = $1 AND user_id = $2 \
             ORDER BY created_at DESC",
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_all(db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| BackupRecord {
                id: r.id,
                document_id: r.document_id,
                user_id: r.user_id,
                backup_hash: r.backup_hash,
                created_at: r.created_at,
            })
            .collect())
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_secret() -> Vec<u8> {
        b"test-encryption-key-for-unit-tests".to_vec()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"Sensitive will document content for testing.";
        let secret = test_secret();

        let (ciphertext, nonce) = encrypt_bytes(plaintext, &secret).unwrap();
        assert_ne!(ciphertext.as_slice(), plaintext);
        assert_eq!(nonce.len(), NONCE_LEN);

        let decrypted = decrypt_bytes(&ciphertext, &nonce, &secret).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let plaintext = b"Secret data";
        let secret = test_secret();
        let wrong_secret = b"wrong-key-that-should-not-work!!".to_vec();

        let (ciphertext, nonce) = encrypt_bytes(plaintext, &secret).unwrap();
        let result = decrypt_bytes(&ciphertext, &nonce, &wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_invalid_nonce_fails() {
        let plaintext = b"Secret data";
        let secret = test_secret();

        let (ciphertext, _) = encrypt_bytes(plaintext, &secret).unwrap();
        let bad_nonce = vec![0u8; NONCE_LEN];
        let result = decrypt_bytes(&ciphertext, &bad_nonce, &secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_nonce_length_fails() {
        let secret = test_secret();
        let result = decrypt_bytes(&[0u8; 32], &[0u8; 5], &secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_plaintexts_produce_different_ciphertexts() {
        let secret = test_secret();
        let (ct1, _) = encrypt_bytes(b"Document A", &secret).unwrap();
        let (ct2, _) = encrypt_bytes(b"Document B", &secret).unwrap();
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_key_derivation_is_deterministic() {
        let secret = test_secret();
        let plaintext = b"Deterministic test";

        // Encrypt twice with same nonce to verify key derivation consistency
        let key1 = derive_key(&secret).unwrap();
        let key2 = derive_key(&secret).unwrap();

        let nonce_bytes = [1u8; NONCE_LEN];

        let mut buf1 = plaintext.to_vec();
        key1.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::empty(),
            &mut buf1,
        )
        .unwrap();

        let mut buf2 = plaintext.to_vec();
        key2.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::empty(),
            &mut buf2,
        )
        .unwrap();

        assert_eq!(buf1, buf2);
    }
}

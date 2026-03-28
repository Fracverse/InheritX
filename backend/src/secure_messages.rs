use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::hkdf::{Salt, HKDF_SHA256};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLegacyMessageRequest {
    pub beneficiary_contact: String,
    pub message: String,
    pub unlock_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyMessage {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub beneficiary_contact: String,
    pub key_version: i32,
    pub unlock_at: DateTime<Utc>,
    pub status: String,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEncryptionKey {
    pub id: Uuid,
    pub key_version: i32,
    pub status: String,
    pub created_by_admin_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryResult {
    pub processed: usize,
    pub delivered: usize,
    pub failed: usize,
}

#[derive(sqlx::FromRow)]
struct DueMessage {
    id: Uuid,
    owner_user_id: Uuid,
    beneficiary_contact: String,
    encrypted_payload: Vec<u8>,
    payload_nonce: Vec<u8>,
    key_version: i32,
}

fn derive_key(secret: &[u8], context: &'static [u8]) -> Result<LessSafeKey, ApiError> {
    let salt = Salt::new(HKDF_SHA256, b"inheritx-message-encryption");
    let prk = salt.extract(secret);
    let info = [context];
    let okm = prk
        .expand(&info, &AES_256_GCM)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Key derivation failed")))?;
    let mut key_bytes = [0u8; KEY_LEN];
    okm.fill(&mut key_bytes)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Unable to materialize key")))?;
    let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Unable to create key")))?;
    Ok(LessSafeKey::new(unbound))
}

fn load_wrapping_secret() -> Result<Vec<u8>, ApiError> {
    let secret = std::env::var("MESSAGE_KEY_ENCRYPTION_KEY").unwrap_or_default();
    if secret.is_empty() {
        return Err(ApiError::Internal(anyhow::anyhow!(
            "MESSAGE_KEY_ENCRYPTION_KEY must be set"
        )));
    }
    Ok(secret.into_bytes())
}

fn encrypt_with_key(key: &LessSafeKey, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), ApiError> {
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

fn decrypt_with_key(
    key: &LessSafeKey,
    ciphertext: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, ApiError> {
    if nonce.len() != NONCE_LEN {
        return Err(ApiError::Internal(anyhow::anyhow!("Invalid nonce length")));
    }
    let mut nonce_arr = [0u8; NONCE_LEN];
    nonce_arr.copy_from_slice(nonce);
    let nonce = Nonce::assume_unique_for_key(nonce_arr);
    let mut in_out = ciphertext.to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Decryption failed")))?;
    Ok(plaintext.to_vec())
}

pub struct MessageKeyService;

impl MessageKeyService {
    pub async fn ensure_active_key(db: &PgPool) -> Result<(), ApiError> {
        let has_active: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM message_encryption_keys WHERE status = 'active')",
        )
        .fetch_one(db)
        .await?;

        if !has_active {
            let _ = Self::create_new_key(db, None).await?;
        }

        Ok(())
    }

    pub async fn list_keys(db: &PgPool) -> Result<Vec<MessageEncryptionKey>, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            key_version: i32,
            status: String,
            created_by_admin_id: Option<Uuid>,
            created_at: DateTime<Utc>,
            rotated_at: Option<DateTime<Utc>>,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, key_version, status, created_by_admin_id, created_at, rotated_at \
             FROM message_encryption_keys ORDER BY key_version DESC",
        )
        .fetch_all(db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| MessageEncryptionKey {
                id: r.id,
                key_version: r.key_version,
                status: r.status,
                created_by_admin_id: r.created_by_admin_id,
                created_at: r.created_at,
                rotated_at: r.rotated_at,
            })
            .collect())
    }

    pub async fn rotate_active_key(
        db: &PgPool,
        admin_id: Uuid,
    ) -> Result<MessageEncryptionKey, ApiError> {
        let mut tx = db.begin().await?;

        sqlx::query(
            "UPDATE message_encryption_keys \
             SET status = 'retired', rotated_at = NOW() \
             WHERE status = 'active'",
        )
        .execute(&mut *tx)
        .await?;

        let key = Self::create_new_key_tx(&mut tx, Some(admin_id)).await?;
        tx.commit().await?;
        Ok(key)
    }

    pub async fn active_data_key_material(db: &PgPool) -> Result<(i32, Vec<u8>), ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            key_version: i32,
            encrypted_key: Vec<u8>,
            wrapping_nonce: Vec<u8>,
        }
        let row = sqlx::query_as::<_, Row>(
            "SELECT key_version, encrypted_key, wrapping_nonce \
             FROM message_encryption_keys WHERE status = 'active' \
             ORDER BY key_version DESC LIMIT 1",
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("No active message key found".to_string()))?;

        let wrapping_secret = load_wrapping_secret()?;
        let wrapping_key = derive_key(&wrapping_secret, b"wrap-message-data-key")?;
        let data_key = decrypt_with_key(&wrapping_key, &row.encrypted_key, &row.wrapping_nonce)?;
        Ok((row.key_version, data_key))
    }

    pub async fn key_material_by_version(
        db: &PgPool,
        key_version: i32,
    ) -> Result<Vec<u8>, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            encrypted_key: Vec<u8>,
            wrapping_nonce: Vec<u8>,
        }
        let row = sqlx::query_as::<_, Row>(
            "SELECT encrypted_key, wrapping_nonce \
             FROM message_encryption_keys WHERE key_version = $1",
        )
        .bind(key_version)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("Message key version {} not found", key_version))
        })?;

        let wrapping_secret = load_wrapping_secret()?;
        let wrapping_key = derive_key(&wrapping_secret, b"wrap-message-data-key")?;
        decrypt_with_key(&wrapping_key, &row.encrypted_key, &row.wrapping_nonce)
    }

    async fn create_new_key(
        db: &PgPool,
        created_by_admin_id: Option<Uuid>,
    ) -> Result<MessageEncryptionKey, ApiError> {
        let mut tx = db.begin().await?;
        let key = Self::create_new_key_tx(&mut tx, created_by_admin_id).await?;
        tx.commit().await?;
        Ok(key)
    }

    async fn create_new_key_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        created_by_admin_id: Option<Uuid>,
    ) -> Result<MessageEncryptionKey, ApiError> {
        let next_version: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(key_version), 0) + 1 FROM message_encryption_keys",
        )
        .fetch_one(&mut **tx)
        .await?;

        let mut raw_key = [0u8; KEY_LEN];
        SystemRandom::new()
            .fill(&mut raw_key)
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to generate data key")))?;

        let wrapping_secret = load_wrapping_secret()?;
        let wrapping_key = derive_key(&wrapping_secret, b"wrap-message-data-key")?;
        let (encrypted_key, wrapping_nonce) = encrypt_with_key(&wrapping_key, &raw_key)?;

        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            key_version: i32,
            status: String,
            created_by_admin_id: Option<Uuid>,
            created_at: DateTime<Utc>,
            rotated_at: Option<DateTime<Utc>>,
        }

        let row = sqlx::query_as::<_, Row>(
            "INSERT INTO message_encryption_keys \
             (key_version, encrypted_key, wrapping_nonce, status, created_by_admin_id) \
             VALUES ($1, $2, $3, 'active', $4) \
             RETURNING id, key_version, status, created_by_admin_id, created_at, rotated_at",
        )
        .bind(next_version)
        .bind(&encrypted_key)
        .bind(&wrapping_nonce)
        .bind(created_by_admin_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(MessageEncryptionKey {
            id: row.id,
            key_version: row.key_version,
            status: row.status,
            created_by_admin_id: row.created_by_admin_id,
            created_at: row.created_at,
            rotated_at: row.rotated_at,
        })
    }
}

pub struct MessageEncryptionService;

impl MessageEncryptionService {
    pub async fn create_encrypted_message(
        db: &PgPool,
        owner_user_id: Uuid,
        req: &CreateLegacyMessageRequest,
    ) -> Result<LegacyMessage, ApiError> {
        if req.unlock_at <= Utc::now() {
            return Err(ApiError::BadRequest(
                "unlock_at must be in the future".to_string(),
            ));
        }
        if req.message.trim().is_empty() {
            return Err(ApiError::BadRequest("message cannot be empty".to_string()));
        }
        if req.beneficiary_contact.trim().is_empty() {
            return Err(ApiError::BadRequest(
                "beneficiary_contact cannot be empty".to_string(),
            ));
        }

        MessageKeyService::ensure_active_key(db).await?;
        let (key_version, data_key) = MessageKeyService::active_data_key_material(db).await?;
        let payload_key = derive_key(&data_key, b"legacy-message-payload-key")?;
        let (encrypted_payload, payload_nonce) =
            encrypt_with_key(&payload_key, req.message.as_bytes())?;

        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            owner_user_id: Uuid,
            beneficiary_contact: String,
            key_version: i32,
            unlock_at: DateTime<Utc>,
            status: String,
            delivered_at: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            "INSERT INTO legacy_messages \
             (owner_user_id, beneficiary_contact, encrypted_payload, payload_nonce, key_version, unlock_at, status) \
             VALUES ($1, $2, $3, $4, $5, $6, 'pending') \
             RETURNING id, owner_user_id, beneficiary_contact, key_version, unlock_at, status, delivered_at, created_at",
        )
        .bind(owner_user_id)
        .bind(req.beneficiary_contact.trim())
        .bind(&encrypted_payload)
        .bind(&payload_nonce)
        .bind(key_version)
        .bind(req.unlock_at)
        .fetch_one(db)
        .await?;

        Ok(LegacyMessage {
            id: row.id,
            owner_user_id: row.owner_user_id,
            beneficiary_contact: row.beneficiary_contact,
            key_version: row.key_version,
            unlock_at: row.unlock_at,
            status: row.status,
            delivered_at: row.delivered_at,
            created_at: row.created_at,
        })
    }

    pub async fn list_owner_messages(
        db: &PgPool,
        owner_user_id: Uuid,
    ) -> Result<Vec<LegacyMessage>, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            owner_user_id: Uuid,
            beneficiary_contact: String,
            key_version: i32,
            unlock_at: DateTime<Utc>,
            status: String,
            delivered_at: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, owner_user_id, beneficiary_contact, key_version, unlock_at, status, delivered_at, created_at \
             FROM legacy_messages WHERE owner_user_id = $1 ORDER BY created_at DESC",
        )
        .bind(owner_user_id)
        .fetch_all(db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| LegacyMessage {
                id: r.id,
                owner_user_id: r.owner_user_id,
                beneficiary_contact: r.beneficiary_contact,
                key_version: r.key_version,
                unlock_at: r.unlock_at,
                status: r.status,
                delivered_at: r.delivered_at,
                created_at: r.created_at,
            })
            .collect())
    }
}

pub struct LegacyMessageDeliveryService {
    db: PgPool,
}

impl LegacyMessageDeliveryService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = self.process_due_messages().await {
                    error!("Legacy Message Delivery Service error: {}", e);
                }
            }
        });
    }

    pub async fn process_due_messages(&self) -> Result<DeliveryResult, ApiError> {
        let due_messages = sqlx::query_as::<_, DueMessage>(
            "SELECT id, owner_user_id, beneficiary_contact, encrypted_payload, payload_nonce, key_version \
             FROM legacy_messages \
             WHERE status = 'pending' AND unlock_at <= NOW() \
             ORDER BY unlock_at ASC LIMIT 100",
        )
        .fetch_all(&self.db)
        .await?;

        let mut delivered = 0usize;
        let mut failed = 0usize;

        for row in &due_messages {
            match self.deliver_single(row).await {
                Ok(()) => {
                    delivered += 1;
                }
                Err(e) => {
                    failed += 1;
                    warn!("Failed delivering legacy message {}: {}", row.id, e);
                    sqlx::query("UPDATE legacy_messages SET status = 'failed', updated_at = NOW() WHERE id = $1")
                        .bind(row.id)
                        .execute(&self.db)
                        .await?;
                }
            }
        }

        if delivered > 0 {
            info!("Delivered {} legacy messages", delivered);
        }

        Ok(DeliveryResult {
            processed: due_messages.len(),
            delivered,
            failed,
        })
    }

    async fn deliver_single(&self, row: &DueMessage) -> Result<(), ApiError> {
        let key_material =
            MessageKeyService::key_material_by_version(&self.db, row.key_version).await?;
        let payload_key = derive_key(&key_material, b"legacy-message-payload-key")?;
        let decrypted = decrypt_with_key(&payload_key, &row.encrypted_payload, &row.payload_nonce)?;
        let decrypted_payload = String::from_utf8(decrypted)
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("Invalid UTF-8 payload")))?;

        let mut tx = self.db.begin().await?;

        sqlx::query(
            "INSERT INTO legacy_message_deliveries \
             (message_id, owner_user_id, beneficiary_contact, decrypted_payload, delivered_at) \
             VALUES ($1, $2, $3, $4, NOW())",
        )
        .bind(row.id)
        .bind(row.owner_user_id)
        .bind(&row.beneficiary_contact)
        .bind(&decrypted_payload)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE legacy_messages \
             SET status = 'delivered', delivered_at = NOW(), updated_at = NOW() \
             WHERE id = $1",
        )
        .bind(row.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_encrypt_decrypt_roundtrip() {
        let base_secret = b"unit-test-secret-for-message-key".to_vec();
        let key = derive_key(&base_secret, b"legacy-message-payload-key").unwrap();
        let plaintext = b"legacy beneficiary message payload";
        let (ciphertext, nonce) = encrypt_with_key(&key, plaintext).unwrap();
        let decrypted = decrypt_with_key(&key, &ciphertext, &nonce).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}

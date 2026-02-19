use crate::api_error::ApiError;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

const OTP_LENGTH: usize = 6;
const OTP_EXPIRY_MINUTES: i64 = 5;
const MAX_ATTEMPTS: i32 = 3;

#[derive(Debug, sqlx::FromRow)]
pub struct TwoFaRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub otp_hash: String,
    pub expires_at: DateTime<Utc>,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
}

/// Generate a 6-digit OTP
pub fn generate_otp() -> String {
    let mut rng = rand::thread_rng();
    let otp: u32 = rng.gen_range(100000..999999);
    otp.to_string()
}

/// Hash OTP using bcrypt
pub fn hash_otp(otp: &str) -> Result<String, ApiError> {
    bcrypt::hash(otp, bcrypt::DEFAULT_COST)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to hash OTP: {}", e)))
}

/// Verify OTP against hash
pub fn verify_otp(otp: &str, hash: &str) -> Result<bool, ApiError> {
    bcrypt::verify(otp, hash)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to verify OTP: {}", e)))
}

/// Store OTP in database
pub async fn store_otp(pool: &PgPool, user_id: Uuid, otp: &str) -> Result<(), ApiError> {
    // Clean up any existing OTPs for this user
    sqlx::query("DELETE FROM user_2fa WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    let otp_hash = hash_otp(otp)?;
    let expires_at = Utc::now() + Duration::minutes(OTP_EXPIRY_MINUTES);

    sqlx::query(
        "INSERT INTO user_2fa (user_id, otp_hash, expires_at, attempts) 
         VALUES ($1, $2, $3, 0)"
    )
    .bind(user_id)
    .bind(otp_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Verify OTP from database
pub async fn verify_otp_from_db(
    pool: &PgPool,
    user_id: Uuid,
    otp: &str,
) -> Result<bool, ApiError> {
    // Fetch the OTP record
    let record: Option<TwoFaRecord> = sqlx::query_as(
        "SELECT id, user_id, otp_hash, expires_at, attempts, created_at 
         FROM user_2fa 
         WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let record = match record {
        Some(r) => r,
        None => return Err(ApiError::BadRequest("No OTP found for user".to_string())),
    };

    // Check if OTP has expired
    if record.expires_at < Utc::now() {
        // Clean up expired OTP
        sqlx::query("DELETE FROM user_2fa WHERE id = $1")
            .bind(record.id)
            .execute(pool)
            .await?;
        return Err(ApiError::BadRequest("OTP has expired".to_string()));
    }

    // Check if max attempts exceeded
    if record.attempts >= MAX_ATTEMPTS {
        // Clean up after max attempts
        sqlx::query("DELETE FROM user_2fa WHERE id = $1")
            .bind(record.id)
            .execute(pool)
            .await?;
        return Err(ApiError::BadRequest("Too many attempts. Please request a new OTP".to_string()));
    }

    // Verify the OTP
    let is_valid = verify_otp(otp, &record.otp_hash)?;

    if is_valid {
        // Delete the OTP record after successful verification
        sqlx::query("DELETE FROM user_2fa WHERE id = $1")
            .bind(record.id)
            .execute(pool)
            .await?;
        Ok(true)
    } else {
        // Increment attempt count
        sqlx::query("UPDATE user_2fa SET attempts = attempts + 1 WHERE id = $1")
            .bind(record.id)
            .execute(pool)
            .await?;
        Ok(false)
    }
}

/// Clean up expired OTPs (can be called periodically)
pub async fn cleanup_expired_otps(pool: &PgPool) -> Result<u64, ApiError> {
    let result = sqlx::query("DELETE FROM user_2fa WHERE expires_at < $1")
        .bind(Utc::now())
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

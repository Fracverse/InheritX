use crate::api_error::ApiError;
use crate::two_fa::models::User2FA;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

const OTP_EXPIRY_MINUTES: i64 = 5;
const MAX_ATTEMPTS: i32 = 3;

pub struct TwoFAService;

impl TwoFAService {
    /// Generate a 6-digit OTP
    pub fn generate_otp() -> String {
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..1000000))
    }

    /// Hash the OTP using bcrypt
    pub fn hash_otp(otp: &str) -> Result<String, ApiError> {
        hash(otp, DEFAULT_COST)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to hash OTP: {}", e)))
    }

    /// Verify OTP against hash
    pub fn verify_otp(otp: &str, hash: &str) -> Result<bool, ApiError> {
        verify(otp, hash)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to verify OTP: {}", e)))
    }

    /// Store OTP in database
    pub async fn store_otp(
        db: &PgPool,
        user_id: Uuid,
        otp: &str,
    ) -> Result<DateTime<chrono::Utc>, ApiError> {
        let otp_hash = Self::hash_otp(otp)?;
        let expires_at = Utc::now() + Duration::minutes(OTP_EXPIRY_MINUTES);

        // Delete any existing OTPs for this user
        sqlx::query("DELETE FROM user_2fa WHERE user_id = $1")
            .bind(user_id)
            .execute(db)
            .await?;

        // Insert new OTP
        sqlx::query(
            "INSERT INTO user_2fa (user_id, otp_hash, expires_at, attempts) 
             VALUES ($1, $2, $3, 0)",
        )
        .bind(user_id)
        .bind(otp_hash)
        .bind(expires_at)
        .execute(db)
        .await?;

        Ok(expires_at)
    }

    /// Verify OTP from database
    pub async fn verify_otp_from_db(
        db: &PgPool,
        user_id: Uuid,
        otp: &str,
    ) -> Result<bool, ApiError> {
        // Fetch the OTP record
        let record: Option<User2FA> = sqlx::query_as(
            "SELECT * FROM user_2fa WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(db)
        .await?;

        let Some(record) = record else {
            return Err(ApiError::BadRequest(
                "No OTP found for this user".to_string(),
            ));
        };

        // Check if OTP has expired
        if Utc::now() > record.expires_at {
            // Clean up expired OTP
            sqlx::query("DELETE FROM user_2fa WHERE id = $1")
                .bind(record.id)
                .execute(db)
                .await?;
            return Err(ApiError::BadRequest("OTP has expired".to_string()));
        }

        // Check if max attempts exceeded
        if record.attempts >= MAX_ATTEMPTS {
            // Clean up after max attempts
            sqlx::query("DELETE FROM user_2fa WHERE id = $1")
                .bind(record.id)
                .execute(db)
                .await?;
            return Err(ApiError::BadRequest(
                "Maximum verification attempts exceeded".to_string(),
            ));
        }

        // Verify the OTP
        let is_valid = Self::verify_otp(otp, &record.otp_hash)?;

        if is_valid {
            // Delete the OTP after successful verification
            sqlx::query("DELETE FROM user_2fa WHERE id = $1")
                .bind(record.id)
                .execute(db)
                .await?;
            Ok(true)
        } else {
            // Increment attempts
            sqlx::query("UPDATE user_2fa SET attempts = attempts + 1 WHERE id = $1")
                .bind(record.id)
                .execute(db)
                .await?;
            Err(ApiError::BadRequest("Invalid OTP".to_string()))
        }
    }

    /// Get user email from database (assumes users table exists with email column)
    pub async fn get_user_email(db: &PgPool, user_id: Uuid) -> Result<String, ApiError> {
        let email: (String,) = sqlx::query_as("SELECT email FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(db)
            .await
            .map_err(|_| ApiError::NotFound("User not found".to_string()))?;

        Ok(email.0)
    }

    /// Clean up expired OTPs (can be called periodically)
    pub async fn cleanup_expired_otps(db: &PgPool) -> Result<u64, ApiError> {
        let result = sqlx::query("DELETE FROM user_2fa WHERE expires_at < NOW()")
            .execute(db)
            .await?;

        Ok(result.rows_affected())
    }
}

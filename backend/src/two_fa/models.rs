use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct User2FA {
    pub id: Uuid,
    pub user_id: Uuid,
    pub otp_hash: String,
    pub expires_at: DateTime<Utc>,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct Send2FARequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct Send2FAResponse {
    pub message: String,
    pub expires_in_seconds: i64,
}

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    pub user_id: Uuid,
    pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct Verify2FAResponse {
    pub message: String,
    pub verified: bool,
}

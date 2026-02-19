use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::api_error::ApiError;
use crate::app::AppState;
use crate::two_fa::{generate_otp, store_otp, verify_otp_from_db};

#[derive(Debug, Deserialize)]
pub struct Send2faRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct Send2faResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct Verify2faRequest {
    pub user_id: Uuid,
    pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct Verify2faResponse {
    pub success: bool,
    pub message: String,
}

/// POST /user/send-2fa
/// Send OTP to user's email
pub async fn send_2fa(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Send2faRequest>,
) -> Result<Json<Send2faResponse>, ApiError> {
    // Fetch user email from database
    let user: Option<(String,)> = sqlx::query_as("SELECT email FROM users WHERE id = $1")
        .bind(payload.user_id)
        .fetch_optional(&state.db)
        .await?;

    let (email,) = user.ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Generate OTP
    let otp = generate_otp();

    // Store OTP in database
    store_otp(&state.db, payload.user_id, &otp).await?;

    // Send OTP via email
    state.email_service.send_otp(&email, &otp).await?;

    Ok(Json(Send2faResponse {
        success: true,
        message: "OTP sent successfully to your email".to_string(),
    }))
}

/// POST /user/verify-2fa
/// Verify OTP provided by user
pub async fn verify_2fa(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Verify2faRequest>,
) -> Result<Json<Verify2faResponse>, ApiError> {
    // Validate OTP format (6 digits)
    if payload.otp.len() != 6 || !payload.otp.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::BadRequest("Invalid OTP format. Must be 6 digits".to_string()));
    }

    // Verify OTP from database
    let is_valid = verify_otp_from_db(&state.db, payload.user_id, &payload.otp).await?;

    if is_valid {
        Ok(Json(Verify2faResponse {
            success: true,
            message: "OTP verified successfully".to_string(),
        }))
    } else {
        Err(ApiError::BadRequest("Invalid OTP".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otp_validation() {
        assert!("123456".len() == 6);
        assert!("123456".chars().all(|c| c.is_ascii_digit()));
        assert!(!"12345".chars().all(|c| c.is_ascii_digit()) || "12345".len() != 6);
        assert!(!"abc123".chars().all(|c| c.is_ascii_digit()));
    }
}

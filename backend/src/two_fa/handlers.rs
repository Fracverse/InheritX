use crate::api_error::ApiError;
use crate::app::AppState;
use crate::email_service::EmailService;
use crate::two_fa::models::{Send2FARequest, Send2FAResponse, Verify2FARequest, Verify2FAResponse};
use crate::two_fa::service::TwoFAService;
use axum::{extract::State, Json};
use std::sync::Arc;
use tracing::{error, info};

/// POST /user/send-2fa
/// Send OTP to user's email
pub async fn send_2fa_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Send2FARequest>,
) -> Result<Json<Send2FAResponse>, ApiError> {
    info!("Sending 2FA OTP to user: {}", payload.user_id);

    // Get user email
    let email = TwoFAService::get_user_email(&state.db, payload.user_id).await?;

    // Generate OTP
    let otp = TwoFAService::generate_otp();

    // Store OTP in database
    let expires_at = TwoFAService::store_otp(&state.db, payload.user_id, &otp).await?;

    // Send email with OTP
    EmailService::send_otp_email(&email, &otp).await?;

    info!("OTP sent successfully to user: {}", payload.user_id);

    let expires_in_seconds = (expires_at - chrono::Utc::now()).num_seconds();

    Ok(Json(Send2FAResponse {
        message: format!("OTP sent to {}", email),
        expires_in_seconds,
    }))
}

/// POST /user/verify-2fa
/// Verify OTP provided by user
pub async fn verify_2fa_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Verify2FARequest>,
) -> Result<Json<Verify2FAResponse>, ApiError> {
    info!("Verifying 2FA OTP for user: {}", payload.user_id);

    // Verify OTP
    match TwoFAService::verify_otp_from_db(&state.db, payload.user_id, &payload.otp).await {
        Ok(true) => {
            info!("2FA verification successful for user: {}", payload.user_id);
            Ok(Json(Verify2FAResponse {
                message: "OTP verified successfully".to_string(),
                verified: true,
            }))
        }
        Ok(false) => {
            error!("Invalid OTP for user: {}", payload.user_id);
            Err(ApiError::BadRequest("Invalid OTP".to_string()))
        }
        Err(e) => {
            error!("2FA verification error for user {}: {}", payload.user_id, e);
            Err(e)
        }
    }
}

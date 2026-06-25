use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::api::AppState;
use crate::ws::KycUpdateEvent;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum KycStatusPayload {
    Pending,
    Submitted,
    Approved,
    Rejected,
}

impl KycStatusPayload {
    fn as_db_str(&self) -> &str {
        match self {
            KycStatusPayload::Pending => "pending",
            KycStatusPayload::Submitted => "submitted",
            KycStatusPayload::Approved => "approved",
            KycStatusPayload::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KycWebhookPayload {
    pub wallet_address: String,
    pub status: KycStatusPayload,
    pub provider_reference: Option<String>,
    pub event_type: String,
}

#[derive(Serialize)]
pub struct WebhookResponse {
    pub success: bool,
    pub message: String,
}

fn verify_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    let sig_bytes = match hex::decode(signature.trim_start_matches("sha256=")) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    mac.verify_slice(&sig_bytes).is_ok()
}

pub async fn kyc_webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let secret = state.kyc_webhook_secret.as_deref().unwrap_or("");
    let signature = headers
        .get("x-kyc-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !secret.is_empty() && !verify_signature(secret, &body, signature) {
        warn!(signature = %signature, "KYC webhook rejected: invalid signature");
        return (
            StatusCode::UNAUTHORIZED,
            Json(WebhookResponse {
                success: false,
                message: "Invalid webhook signature".to_string(),
            }),
        )
            .into_response();
    }

    let payload: KycWebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "KYC webhook: failed to parse payload");
            return (
                StatusCode::BAD_REQUEST,
                Json(WebhookResponse {
                    success: false,
                    message: format!("Invalid payload: {}", e),
                }),
            )
                .into_response();
        }
    };

    info!(
        wallet_address = %payload.wallet_address,
        status = ?payload.status,
        event_type = %payload.event_type,
        "KYC webhook received"
    );

    let kyc_status_str = payload.status.as_db_str();
    let raw_payload =
        serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null);

    let update_result = sqlx::query(
        r#"
        INSERT INTO users (wallet_address, kyc_status)
        VALUES ($1, $2::kyc_status)
        ON CONFLICT (wallet_address)
        DO UPDATE SET kyc_status = $2::kyc_status
        "#,
    )
    .bind(&payload.wallet_address)
    .bind(kyc_status_str)
    .execute(&state.db_pool)
    .await;

    let (success, error_message) = match update_result {
        Ok(_) => {
            info!(
                wallet_address = %payload.wallet_address,
                kyc_status = %kyc_status_str,
                "KYC status updated successfully"
            );
            // Broadcast to WebSocket subscribers
            let event = KycUpdateEvent {
                wallet_address: payload.wallet_address.clone(),
                kyc_status: kyc_status_str.to_string(),
                event_type: payload.event_type.clone(),
            };
            if let Err(e) = state.kyc_tx.send(event) {
                tracing::debug!("No WebSocket subscribers for KYC event: {}", e);
            }
            (true, None::<String>)
        }
        Err(e) => {
            error!(
                wallet_address = %payload.wallet_address,
                error = %e,
                "Failed to update KYC status in database"
            );
            (false, Some(e.to_string()))
        }
    };

    let log_result = sqlx::query(
        r#"
        INSERT INTO kyc_webhook_logs
            (wallet_address, provider_reference, event_type, kyc_status, raw_payload, success, error_message)
        VALUES ($1, $2, $3, $4::kyc_status, $5, $6, $7)
        "#,
    )
    .bind(&payload.wallet_address)
    .bind(&payload.provider_reference)
    .bind(&payload.event_type)
    .bind(kyc_status_str)
    .bind(&raw_payload)
    .bind(success)
    .bind(&error_message)
    .execute(&state.db_pool)
    .await;

    if let Err(e) = log_result {
        error!(error = %e, "Failed to write KYC webhook log");
    }

    if success {
        (
            StatusCode::OK,
            Json(WebhookResponse {
                success: true,
                message: format!(
                    "KYC status updated to '{}' for wallet {}",
                    kyc_status_str, payload.wallet_address
                ),
            }),
        )
            .into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(WebhookResponse {
                success: false,
                message: "Failed to update KYC status".to_string(),
            }),
        )
            .into_response()
    }
}

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

/// Header the KYC provider carries the request signature in.
pub const KYC_SIGNATURE_HEADER: &str = "x-kyc-signature";

/// Length of an HMAC-SHA256 digest in bytes.
const SIGNATURE_LEN: usize = 32;

/// Why a webhook request failed signature verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureError {
    /// No `kyc_webhook_secret` is configured, so nothing can be verified.
    SecretNotConfigured,
    /// The signature header is absent or not valid UTF-8.
    MissingSignature,
    /// The header is present but is not a hex-encoded HMAC-SHA256 digest.
    MalformedSignature,
    /// The signature does not match the body under the configured secret.
    Mismatch,
}

impl SignatureError {
    /// A missing secret is our misconfiguration, not the caller's: answer 503 so
    /// the provider retries once the deployment is fixed rather than dropping
    /// the event as permanently rejected.
    pub fn status(&self) -> StatusCode {
        match self {
            SignatureError::SecretNotConfigured => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::UNAUTHORIZED,
        }
    }

    /// Message returned to the caller. Deliberately coarse — a caller must not
    /// be able to tell a malformed signature from a wrong one.
    pub fn client_message(&self) -> &'static str {
        match self {
            SignatureError::SecretNotConfigured => "Webhook verification is not configured",
            _ => "Invalid webhook signature",
        }
    }
}

/// Verify an inbound webhook against the configured shared secret.
///
/// The signature is an HMAC-SHA256 of the **raw** request body, hex encoded and
/// optionally prefixed with `sha256=`. Verification fails closed: a missing or
/// blank secret rejects every request instead of waving it through.
pub fn verify_webhook_signature(
    secret: Option<&str>,
    body: &[u8],
    signature: Option<&str>,
) -> Result<(), SignatureError> {
    let secret = secret
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(SignatureError::SecretNotConfigured)?;

    let signature = signature
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(SignatureError::MissingSignature)?;

    let hex_digest = strip_sha256_prefix(signature);
    let expected = hex::decode(hex_digest).map_err(|_| SignatureError::MalformedSignature)?;
    if expected.len() != SIGNATURE_LEN {
        return Err(SignatureError::MalformedSignature);
    }

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| SignatureError::Mismatch)?;
    mac.update(body);
    // `verify_slice` is constant time, so this leaks nothing about the digest.
    mac.verify_slice(&expected)
        .map_err(|_| SignatureError::Mismatch)
}

fn strip_sha256_prefix(signature: &str) -> &str {
    const PREFIX: &[u8] = b"sha256=";
    // Compare as bytes: a non-ASCII header would make byte-index slicing panic
    // on a char boundary, and matching the ASCII prefix proves byte 7 is one.
    match signature.as_bytes().get(..PREFIX.len()) {
        Some(head) if head.eq_ignore_ascii_case(PREFIX) => &signature[PREFIX.len()..],
        _ => signature,
    }
}

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

pub async fn kyc_webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let signature = headers
        .get(KYC_SIGNATURE_HEADER)
        .and_then(|v| v.to_str().ok());

    // Verify before parsing: an unauthenticated caller must not reach the
    // deserializer, the database, or the WebSocket broadcast.
    if let Err(err) = verify_webhook_signature(
        state.kyc_webhook_secret.as_deref(),
        &body,
        signature,
    ) {
        warn!(reason = ?err, "KYC webhook rejected");
        return (
            err.status(),
            Json(WebhookResponse {
                success: false,
                message: err.client_message().to_string(),
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
                    message: format!("Invalid payload: {e}"),
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

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "kyc-webhook-secret";
    const BODY: &[u8] = br#"{"wallet_address":"GDTEST123","status":"approved"}"#;

    fn sign(secret: &str, body: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        hex::encode(mac.finalize().into_bytes())
    }

    #[test]
    fn accepts_signature_with_and_without_prefix() {
        let digest = sign(SECRET, BODY);
        for signature in [
            digest.clone(),
            format!("sha256={digest}"),
            format!("SHA256={digest}"),
            format!("  sha256={digest}  "),
            format!("sha256={}", digest.to_uppercase()),
        ] {
            assert_eq!(
                verify_webhook_signature(Some(SECRET), BODY, Some(&signature)),
                Ok(()),
                "should have accepted {signature}"
            );
        }
    }

    #[test]
    fn rejects_signature_from_a_different_secret() {
        let signature = sign("other-secret", BODY);
        assert_eq!(
            verify_webhook_signature(Some(SECRET), BODY, Some(&signature)),
            Err(SignatureError::Mismatch)
        );
    }

    #[test]
    fn rejects_body_tampered_after_signing() {
        let signature = sign(SECRET, BODY);
        let tampered = br#"{"wallet_address":"GDATTACKER","status":"approved"}"#;
        assert_eq!(
            verify_webhook_signature(Some(SECRET), tampered, Some(&signature)),
            Err(SignatureError::Mismatch)
        );
    }

    #[test]
    fn rejects_missing_or_blank_signature() {
        assert_eq!(
            verify_webhook_signature(Some(SECRET), BODY, None),
            Err(SignatureError::MissingSignature)
        );
        assert_eq!(
            verify_webhook_signature(Some(SECRET), BODY, Some("   ")),
            Err(SignatureError::MissingSignature)
        );
        assert_eq!(
            verify_webhook_signature(Some(SECRET), BODY, Some("sha256=")),
            Err(SignatureError::MalformedSignature)
        );
    }

    #[test]
    fn rejects_malformed_signature() {
        // Not hex, and a hex digest of the wrong length.
        for signature in ["sha256=not-hex-at-all", "sha256=abcd", &sign(SECRET, BODY)[..62]] {
            assert_eq!(
                verify_webhook_signature(Some(SECRET), BODY, Some(signature)),
                Err(SignatureError::MalformedSignature),
                "should have rejected {signature}"
            );
        }
    }

    /// The whole point of the issue: no secret must mean no access, not open access.
    #[test]
    fn fails_closed_when_secret_is_not_configured() {
        let signature = sign(SECRET, BODY);
        for secret in [None, Some(""), Some("   ")] {
            assert_eq!(
                verify_webhook_signature(secret, BODY, Some(&signature)),
                Err(SignatureError::SecretNotConfigured)
            );
        }
    }

    #[test]
    fn error_statuses_distinguish_client_from_server_fault() {
        assert_eq!(
            SignatureError::SecretNotConfigured.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        for err in [
            SignatureError::MissingSignature,
            SignatureError::MalformedSignature,
            SignatureError::Mismatch,
        ] {
            assert_eq!(err.status(), StatusCode::UNAUTHORIZED);
            // Callers must not learn *why* the signature was rejected.
            assert_eq!(err.client_message(), "Invalid webhook signature");
        }
    }
}

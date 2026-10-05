use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use dashmap::DashSet;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Maximum age of a signed request, in seconds. A signature whose signed
/// timestamp is older than this is rejected (fail-closed).
pub const SIGNATURE_MAX_AGE_SECS: i64 = 5 * 60;

/// Maximum clock skew, in seconds, allowed for a signature whose signed
/// timestamp is in the future relative to the server clock.
pub const SIGNATURE_MAX_SKEW_SECS: i64 = 60;

/// Process-wide cache of SHA-256 fingerprints of signatures that have already
/// been accepted. A second submission of the same signature is a replay.
///
/// Entries are kept forever (the [`SIGNATURE_MAX_AGE_SECS`] window bounds how
/// long a signature is *usable*); operators can swap this for a bounded/TTL
/// cache without changing the [`signature_auth_middleware`] contract.
static PROCESSED_SIGNATURES: Lazy<DashSet<[u8; 32]>> = Lazy::new(DashSet::new);

/// Builds the canonical byte-string that a client signs for signature auth.
///
/// The timestamp is bound into the signed payload (not merely sent alongside
/// it) so an attacker cannot extend a captured signature's lifetime by
/// rewriting the `X-Timestamp` header. Format: `"{unix_seconds}.{body}"`.
pub fn canonical_signed_payload(timestamp: i64, body: &str) -> String {
    format!("{timestamp}.{body}")
}

/// Stable SHA-256 fingerprint of a raw signature.
fn signature_fingerprint(signature: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(signature);
    hasher.finalize().into()
}

/// Returns `true` when this signature has already been accepted (i.e. a
/// replay attempt). Exposed for tests and operational introspection.
pub fn is_signature_processed(signature: &[u8]) -> bool {
    PROCESSED_SIGNATURES.contains(&signature_fingerprint(signature))
}

/// Enforces the timestamp window. `now` is injected for deterministic tests.
fn verify_timestamp_window(timestamp: i64, now: i64) -> Result<(), AuthError> {
    if timestamp < now - SIGNATURE_MAX_AGE_SECS {
        return Err(AuthError::SignatureExpired);
    }
    if timestamp > now + SIGNATURE_MAX_SKEW_SECS {
        return Err(AuthError::SignatureFromFuture);
    }
    Ok(())
}

/// Records a signature that has already been cryptographically verified.
///
/// Returns `true` when the signature was newly recorded and may proceed;
/// `false` when it was already present (a replay).
fn record_signature(signature: &[u8]) -> bool {
    PROCESSED_SIGNATURES.insert(signature_fingerprint(signature))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub role: String,
}

impl axum::extract::FromRequestParts<()> for UserContext {
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &(),
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let ctx = parts.extensions.get::<UserContext>().cloned();
        Box::pin(async move { ctx.ok_or(StatusCode::INTERNAL_SERVER_ERROR) })
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingHeader,
    #[error("Invalid authorization header format")]
    InvalidHeaderFormat,
    #[error("Missing token")]
    MissingToken,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Missing signature timestamp")]
    MissingTimestamp,
    #[error("Invalid signature timestamp")]
    InvalidTimestamp,
    #[error("Signature timestamp outside the allowed window")]
    SignatureExpired,
    #[error("Signature timestamp is in the future")]
    SignatureFromFuture,
    #[error("Signature has already been used")]
    SignatureReplayed,
    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.to_string() });
        (StatusCode::UNAUTHORIZED, Json(body)).into_response()
    }
}

pub async fn jwt_auth_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(AuthError::MissingHeader)?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| AuthError::InvalidHeaderFormat)?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AuthError::InvalidHeaderFormat);
    }

    let token = auth_str.trim_start_matches("Bearer ").trim();
    if token.is_empty() {
        return Err(AuthError::MissingToken);
    }

    let secret = std::env::var("JWT_SECRET").map_err(|_| AuthError::InvalidToken)?;

    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map_err(|_| AuthError::InvalidToken)?;

    if token_data.claims.role != "admin" {
        return Err(AuthError::Unauthorized);
    }

    let user_context = UserContext {
        user_id: token_data.claims.sub,
        role: token_data.claims.role,
    };

    req.extensions_mut().insert(user_context);

    Ok(next.run(req).await)
}

/// Accepts either an admin JWT (`Authorization: Bearer`) or an ed25519
/// request signature (`X-Public-Key` + `X-Signature`). JWT takes precedence
/// when both are present so browser clients that attach a token keep working
/// even if signature headers are incomplete (empty POST bodies are not signed).
pub async fn jwt_or_signature_auth_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let has_bearer = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("Bearer "));

    if has_bearer {
        jwt_auth_middleware(req, next).await
    } else {
        signature_auth_middleware(req, next).await
    }
}

pub async fn signature_auth_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let (parts, body) = req.into_parts();

    let public_key_hex = parts
        .headers
        .get("X-Public-Key")
        .ok_or(AuthError::MissingHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaderFormat)?;

    let signature_hex = parts
        .headers
        .get("X-Signature")
        .ok_or(AuthError::MissingHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaderFormat)?;

    let timestamp = parts
        .headers
        .get("X-Timestamp")
        .ok_or(AuthError::MissingTimestamp)?
        .to_str()
        .map_err(|_| AuthError::InvalidTimestamp)?
        .trim()
        .parse::<i64>()
        .map_err(|_| AuthError::InvalidTimestamp)?;

    let public_key_bytes = hex::decode(public_key_hex.trim_start_matches("0x"))
        .map_err(|_| AuthError::InvalidSignature)?;

    let signature_bytes = hex::decode(signature_hex.trim_start_matches("0x"))
        .map_err(|_| AuthError::InvalidSignature)?;

    if public_key_bytes.len() != 32 {
        return Err(AuthError::InvalidSignature);
    }

    let public_key_array: [u8; 32] = public_key_bytes
        .try_into()
        .map_err(|_| AuthError::InvalidSignature)?;

    let verifying_key =
        VerifyingKey::from_bytes(&public_key_array).map_err(|_| AuthError::InvalidSignature)?;

    let signature = Signature::from_slice(signature_bytes.as_slice())
        .map_err(|_| AuthError::InvalidSignature)?;

    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| AuthError::InvalidSignature)?;

    let body_str =
        String::from_utf8(body_bytes.to_vec()).map_err(|_| AuthError::InvalidSignature)?;

    // The timestamp is part of the signed payload, so a valid signature
    // proves the caller intended this exact timestamp (and body).
    let canonical_payload = canonical_signed_payload(timestamp, &body_str);
    verifying_key
        .verify(canonical_payload.as_bytes(), &signature)
        .map_err(|_| AuthError::InvalidSignature)?;

    // Invariant: verify-then-nonce. The signature must authenticate the
    // timestamp and body *before* either the timestamp window is evaluated or
    // the process-wide cache is touched, so unauthenticated input can never
    // poison the replay set. Both checks fail closed.
    verify_timestamp_window(timestamp, Utc::now().timestamp())?;
    if !record_signature(signature_bytes.as_slice()) {
        return Err(AuthError::SignatureReplayed);
    }

    let user_context = UserContext {
        user_id: public_key_hex.to_string(),
        role: "user".to_string(),
    };

    let mut new_req = Request::from_parts(parts, Body::from(body_str));
    new_req.extensions_mut().insert(user_context);

    Ok(next.run(new_req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    /// Signs the canonical payload for `(timestamp, body)` with a fresh random
    /// key. Returns the raw signature bytes.
    fn sign_payload(timestamp: i64, body: &str) -> (SigningKey, Vec<u8>) {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let canonical = canonical_signed_payload(timestamp, body);
        let signature = signing_key.sign(canonical.as_bytes()).to_bytes().to_vec();
        (signing_key, signature)
    }

    #[test]
    fn fresh_and_exact_boundary_timestamps_are_accepted() {
        let now = 1_700_000_000_i64;
        assert!(verify_timestamp_window(now, now).is_ok());
        // Exactly 5 minutes old is still inside the window (inclusive).
        assert!(verify_timestamp_window(now - SIGNATURE_MAX_AGE_SECS, now).is_ok());
    }

    #[test]
    fn timestamps_older_than_five_minutes_are_rejected() {
        let now = 1_700_000_000_i64;
        assert!(matches!(
            verify_timestamp_window(now - SIGNATURE_MAX_AGE_SECS - 1, now),
            Err(AuthError::SignatureExpired)
        ));
    }

    #[test]
    fn future_timestamps_beyond_skew_are_rejected() {
        let now = 1_700_000_000_i64;
        assert!(verify_timestamp_window(now + SIGNATURE_MAX_SKEW_SECS, now).is_ok());
        assert!(matches!(
            verify_timestamp_window(now + SIGNATURE_MAX_SKEW_SECS + 1, now),
            Err(AuthError::SignatureFromFuture)
        ));
    }

    #[test]
    fn valid_signature_is_accepted_once_then_rejected_as_replay() {
        let timestamp = Utc::now().timestamp();
        let body = "unit-test-fresh-signature";
        let (signing_key, signature) = sign_payload(timestamp, body);

        let canonical = canonical_signed_payload(timestamp, body);
        assert!(signing_key
            .verifying_key()
            .verify(
                canonical.as_bytes(),
                &Signature::from_slice(&signature).unwrap()
            )
            .is_ok());

        assert!(record_signature(&signature), "first use must be accepted");
        assert!(
            !record_signature(&signature),
            "second use of the same signature must be rejected as a replay"
        );
        assert!(is_signature_processed(&signature));
    }

    #[test]
    fn invalid_signature_is_not_recorded() {
        // A signature over a different payload must fail verification, and the
        // verify-then-nonce ordering means it is never inserted into the set.
        let timestamp = Utc::now().timestamp();
        let (signing_key, signature) = sign_payload(timestamp, "original body");
        let tampered = canonical_signed_payload(timestamp, "tampered body");

        assert!(signing_key
            .verifying_key()
            .verify(
                tampered.as_bytes(),
                &Signature::from_slice(&signature).unwrap()
            )
            .is_err());
        assert!(!is_signature_processed(&signature));
    }
}

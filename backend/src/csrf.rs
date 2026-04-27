//! CSRF protection middleware — Issue #434
//!
//! Implements the synchroniser token pattern:
//!  1. The client receives a CSRF token via `GET /api/v1/csrf-token`.
//!  2. All state-changing requests (POST / PUT / PATCH / DELETE) must include
//!     the token in the `X-CSRF-Token` header.
//!  3. Requests that fail validation receive `403 Forbidden`.
//!
//! Tokens are random 32-byte values encoded as hex strings stored in the
//! `csrf_tokens` table, tied to the requesting user's session.

use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use chrono::{Duration, Utc};
use ring::rand::{SecureRandom, SystemRandom};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::api_error::ApiError;
use crate::app::AppState;
use crate::auth::UserClaims;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Generate a cryptographically random 32-byte hex token using ring.
pub fn generate_csrf_token() -> String {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes).expect("CSRF token generation failed");
    hex::encode(bytes)
}

/// Validate a JWT and return its claims — used by the CSRF token handler.
fn decode_user_claims(token: &str, secret: &str) -> Result<UserClaims, ApiError> {
    jsonwebtoken::decode::<UserClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| ApiError::Unauthorized)
}

// ── HTTP handler ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct CsrfTokenResponse {
    pub csrf_token: String,
    pub expires_at: String,
}

/// `GET /api/v1/csrf-token`
///
/// Issues a fresh single-use CSRF token for the authenticated user.
/// Requires `Authorization: Bearer <jwt>`. Token expires in 60 minutes.
pub async fn get_csrf_token(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Result<Json<CsrfTokenResponse>, ApiError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::Unauthorized);
    }
    let token_str = auth_header.strip_prefix("Bearer ").unwrap();
    let claims = decode_user_claims(token_str, &state.config.jwt_secret)?;

    let csrf_token = generate_csrf_token();
    let expires_at = Utc::now() + Duration::minutes(60);

    sqlx::query(
        r#"
        INSERT INTO csrf_tokens (id, user_id, token, expires_at, used)
        VALUES ($1, $2, $3, $4, FALSE)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(claims.user_id)
    .bind(&csrf_token)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    Ok(Json(CsrfTokenResponse {
        csrf_token,
        expires_at: expires_at.to_rfc3339(),
    }))
}

// ── Middleware ────────────────────────────────────────────────────────────────

/// Validates `X-CSRF-Token` for all state-changing HTTP methods.
///
/// GET / HEAD / OPTIONS pass through (safe/idempotent).
/// Requests with **no** `Authorization` header pass through — the auth
/// middleware downstream will return 401 for unauthenticated requests, so
/// we must not intercept them with 403 here.
/// Only authenticated state-changing requests are required to carry a
/// valid CSRF token.
pub async fn csrf_protection_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let method = req.method().clone();

    if method == Method::GET || method == Method::HEAD || method == Method::OPTIONS {
        return next.run(req).await;
    }

    // No Authorization header → not authenticated; let auth middleware handle it
    let has_auth = req.headers().contains_key("Authorization");
    if !has_auth {
        return next.run(req).await;
    }

    let csrf_token = req
        .headers()
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let token = match csrf_token {
        Some(t) => t,
        None => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Missing X-CSRF-Token header" })),
            )
                .into_response();
        }
    };

    let valid = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM csrf_tokens
            WHERE token = $1
              AND expires_at > NOW()
              AND used = FALSE
        )
        "#,
    )
    .bind(&token)
    .fetch_one(&state.db)
    .await;

    match valid {
        Ok(true) => {
            // Single-use: mark consumed immediately before processing the request
            let _ = sqlx::query("UPDATE csrf_tokens SET used = TRUE WHERE token = $1")
                .bind(&token)
                .execute(&state.db)
                .await;
            next.run(req).await
        }
        Ok(false) => (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Invalid or expired CSRF token" })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "CSRF validation failed" })),
        )
            .into_response(),
    }
}

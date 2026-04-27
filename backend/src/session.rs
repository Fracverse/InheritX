//! Session Management & Revocation — Issue #436
//!
//! Provides database-backed JWT session tracking so tokens can be explicitly
//! revoked. Solves the stateless JWT limitation: once a token is revoked via
//! `POST /api/v1/auth/logout` or `POST /api/v1/auth/logout-all`, subsequent
//! requests carrying that token are rejected at the middleware layer.
//!
//! Each login call should create a session row via `create_session`. The
//! `session_guard_middleware` rejects requests whose token appears in the
//! `revoked_sessions` table.

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::api_error::ApiError;
use crate::app::AppState;
use crate::auth::UserClaims;

// ── Domain types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    /// SHA-256 hex digest of the raw JWT string (avoids storing the token itself)
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    /// Optional device/user-agent label for "logout all devices" UX
    pub device_label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    /// Device label to revoke (optional — revokes current token if absent)
    pub device_label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionSummary>,
}

#[derive(Debug, Serialize)]
pub struct SessionSummary {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub device_label: Option<String>,
    pub active: bool,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// SHA-256 of the raw JWT string — stored instead of the token itself.
fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(token.as_bytes());
    hex::encode(h.finalize())
}

fn extract_bearer(req: &Request<Body>) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

fn decode_claims(token: &str, secret: &str) -> Option<UserClaims> {
    jsonwebtoken::decode::<UserClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map(|d| d.claims)
    .ok()
}

// ── Service functions ─────────────────────────────────────────────────────────

/// Record a new active session when a user logs in.
pub async fn create_session(
    db: &PgPool,
    user_id: Uuid,
    raw_token: &str,
    device_label: Option<String>,
    expiry: DateTime<Utc>,
) -> Result<Session, ApiError> {
    let token_hash = hash_token(raw_token);
    let session = sqlx::query_as::<_, Session>(
        r#"
        INSERT INTO sessions (id, user_id, token_hash, created_at, expires_at, revoked, device_label)
        VALUES ($1, $2, $3, NOW(), $4, FALSE, $5)
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(&token_hash)
    .bind(expiry)
    .bind(device_label)
    .fetch_one(db)
    .await?;

    Ok(session)
}

/// Revoke a single session by its token hash.
async fn revoke_by_hash(db: &PgPool, token_hash: &str) -> Result<u64, ApiError> {
    let result = sqlx::query(
        r#"
        UPDATE sessions
        SET revoked = TRUE, revoked_at = NOW()
        WHERE token_hash = $1 AND revoked = FALSE
        "#,
    )
    .bind(token_hash)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

// ── HTTP handlers ─────────────────────────────────────────────────────────────

/// `POST /api/v1/auth/logout`
///
/// Revokes the session associated with the current `Authorization` token.
pub async fn logout(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let raw_token = extract_bearer(&req).ok_or_else(|| ApiError::Unauthorized)?;

    let rows = revoke_by_hash(&state.db, &hash_token(&raw_token)).await?;

    if rows == 0 {
        return Err(ApiError::NotFound(
            "Session not found or already revoked".into(),
        ));
    }

    Ok(Json(json!({ "message": "Logged out successfully" })))
}

/// `POST /api/v1/auth/logout-all`
///
/// Revokes **all** active sessions for the authenticated user.
pub async fn logout_all(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let raw_token = extract_bearer(&req).ok_or_else(|| ApiError::Unauthorized)?;

    let claims = decode_claims(&raw_token, &state.config.jwt_secret)
        .ok_or_else(|| ApiError::Unauthorized)?;

    let result = sqlx::query(
        r#"
        UPDATE sessions
        SET revoked = TRUE, revoked_at = NOW()
        WHERE user_id = $1 AND revoked = FALSE
        "#,
    )
    .bind(claims.user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "message": "All sessions revoked",
        "sessions_revoked": result.rows_affected()
    })))
}

/// `GET /api/v1/auth/sessions`
///
/// Returns all sessions for the authenticated user.
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Result<Json<SessionListResponse>, ApiError> {
    let raw_token = extract_bearer(&req).ok_or_else(|| ApiError::Unauthorized)?;

    let claims = decode_claims(&raw_token, &state.config.jwt_secret)
        .ok_or_else(|| ApiError::Unauthorized)?;

    let sessions = sqlx::query_as::<_, Session>(
        r#"
        SELECT * FROM sessions
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(claims.user_id)
    .fetch_all(&state.db)
    .await?;

    let now = Utc::now();
    let summaries = sessions
        .into_iter()
        .map(|s| SessionSummary {
            id: s.id,
            created_at: s.created_at,
            expires_at: s.expires_at,
            device_label: s.device_label,
            active: !s.revoked && s.expires_at > now,
        })
        .collect();

    Ok(Json(SessionListResponse {
        sessions: summaries,
    }))
}

/// `DELETE /api/v1/auth/sessions/:session_id`
///
/// Revoke a specific session by ID (e.g., from the "manage devices" UI).
pub async fn revoke_session(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
    req: Request<Body>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let raw_token = extract_bearer(&req).ok_or_else(|| ApiError::Unauthorized)?;

    let claims = decode_claims(&raw_token, &state.config.jwt_secret)
        .ok_or_else(|| ApiError::Unauthorized)?;

    // Ensure the session belongs to the authenticated user
    let rows = sqlx::query(
        r#"
        UPDATE sessions
        SET revoked = TRUE, revoked_at = NOW()
        WHERE id = $1 AND user_id = $2 AND revoked = FALSE
        "#,
    )
    .bind(session_id)
    .bind(claims.user_id)
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(ApiError::NotFound(
            "Session not found or already revoked".into(),
        ));
    }

    Ok(Json(json!({ "message": "Session revoked" })))
}

// ── Middleware ────────────────────────────────────────────────────────────────

/// Rejects requests whose JWT has been explicitly revoked.
///
/// This middleware runs after the JWT signature check so it only queries the
/// database for structurally valid tokens. Requests without an `Authorization`
/// header are passed through (open endpoints handle their own auth).
pub async fn session_guard_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let raw_token = match extract_bearer(&req) {
        Some(t) => t,
        None => return next.run(req).await,
    };

    let token_hash = hash_token(&raw_token);

    let revoked = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM sessions
            WHERE token_hash = $1 AND revoked = TRUE
        )
        "#,
    )
    .bind(&token_hash)
    .fetch_one(&state.db)
    .await;

    match revoked {
        Ok(true) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Session has been revoked. Please log in again." })),
        )
            .into_response(),
        Ok(false) | Err(_) => next.run(req).await,
    }
}

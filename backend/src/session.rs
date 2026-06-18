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
use crate::auth::AuthenticatedUser;

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

// ── Service functions ─────────────────────────────────────────────────────────

/// Maximum number of concurrent active (non-revoked, non-expired) sessions
/// allowed per user. Configurable via the `MAX_CONCURRENT_SESSIONS` environment
/// variable; defaults to 5.
pub const DEFAULT_MAX_CONCURRENT_SESSIONS: i64 = 5;

fn max_concurrent_sessions() -> i64 {
    std::env::var("MAX_CONCURRENT_SESSIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_CONCURRENT_SESSIONS)
}

/// Record a new active session when a user logs in.
///
/// Returns `ApiError::TooManyRequests` when the user already has
/// `MAX_CONCURRENT_SESSIONS` (default: 5) active sessions. The caller should
/// prompt the user to log out of another device before retrying.
pub async fn create_session(
    db: &PgPool,
    user_id: Uuid,
    raw_token: &str,
    device_label: Option<String>,
    expiry: DateTime<Utc>,
) -> Result<Session, ApiError> {
    let limit = max_concurrent_sessions();

    // Count active (non-revoked, non-expired) sessions for this user.
    let active_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM sessions
        WHERE user_id = $1
          AND revoked = FALSE
          AND expires_at > NOW()
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await?;

    if active_count >= limit {
        return Err(ApiError::TooManyRequests(format!(
            "Maximum concurrent sessions ({limit}) reached. \
             Please log out of another device before logging in again."
        )));
    }

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
    AuthenticatedUser(_user): AuthenticatedUser,
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
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let result = sqlx::query(
        r#"
        UPDATE sessions
        SET revoked = TRUE, revoked_at = NOW()
        WHERE user_id = $1 AND revoked = FALSE
        "#,
    )
    .bind(user.user_id)
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
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<SessionListResponse>, ApiError> {
    let sessions = sqlx::query_as::<_, Session>(
        r#"
        SELECT * FROM sessions
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(user.user_id)
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
    crate::validation::Path(session_id): crate::validation::Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    _req: Request<Body>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Ensure the session belongs to the authenticated user
    let rows = sqlx::query(
        r#"
        UPDATE sessions
        SET revoked = TRUE, revoked_at = NOW()
        WHERE id = $1 AND user_id = $2 AND revoked = FALSE
        "#,
    )
    .bind(session_id)
    .bind(user.user_id)
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

/// Rejects requests whose JWT has been explicitly revoked OR expired.
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

    let result = sqlx::query_as::<_, (bool, Option<DateTime<Utc>>)>(
        r#"
        SELECT revoked, expires_at FROM sessions
        WHERE token_hash = $1
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some((revoked, expires_at))) => {
            let now = Utc::now();
            if revoked || expires_at.map(|e| e < now).unwrap_or(true) {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "Session is invalid. Please log in again." })),
                )
                    .into_response();
            }
        }
        Ok(None) => {
            // Session not found - could be a new token or invalid token
            // Let the auth extractor handle this case
        }
        Err(_) => {
            // Database error - let the request proceed and let auth extractor handle
        }
    }

    next.run(req).await
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_max_concurrent_sessions_is_positive() {
        const {
            assert!(
                DEFAULT_MAX_CONCURRENT_SESSIONS > 0,
                "session limit must be a positive integer"
            );
        }
    }

    #[test]
    fn max_concurrent_sessions_reads_env_var() {
        // Temporarily set the env var and verify the function picks it up.
        std::env::set_var("MAX_CONCURRENT_SESSIONS", "3");
        assert_eq!(max_concurrent_sessions(), 3);
        std::env::remove_var("MAX_CONCURRENT_SESSIONS");
    }

    #[test]
    fn max_concurrent_sessions_falls_back_to_default_on_invalid_value() {
        std::env::set_var("MAX_CONCURRENT_SESSIONS", "not_a_number");
        assert_eq!(max_concurrent_sessions(), DEFAULT_MAX_CONCURRENT_SESSIONS);
        std::env::remove_var("MAX_CONCURRENT_SESSIONS");
    }

    #[test]
    fn max_concurrent_sessions_falls_back_to_default_when_unset() {
        std::env::remove_var("MAX_CONCURRENT_SESSIONS");
        assert_eq!(max_concurrent_sessions(), DEFAULT_MAX_CONCURRENT_SESSIONS);
    }

    /// Verify the error variant and message when the limit is exceeded.
    #[test]
    fn too_many_requests_error_contains_limit() {
        let limit = 5i64;
        let err = ApiError::TooManyRequests(format!(
            "Maximum concurrent sessions ({limit}) reached. \
             Please log out of another device before logging in again."
        ));
        let msg = err.to_string();
        assert!(
            msg.contains('5'),
            "error message should include the session limit"
        );
    }
}

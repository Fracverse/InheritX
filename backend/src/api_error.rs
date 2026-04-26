use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Categorised API errors with specific variants for actionable diagnostics.
///
/// Each variant maps to an HTTP status code and an opaque `error_code` string
/// that clients can rely on without parsing free-form messages.
#[derive(Debug, Error)]
pub enum ApiError {
    // ── 5xx ──────────────────────────────────────────────────────────────────
    /// Catch-all for unexpected internal failures.
    #[error("An unexpected internal error occurred. Please try again later.")]
    Internal(#[from] anyhow::Error),

    /// The database connection pool is unavailable or the connection was refused.
    /// Transient – safe to retry with back-off.
    #[error("The service is temporarily unable to reach the database. Please try again shortly.")]
    DatabaseConnection(String),

    /// A well-formed SQL query produced an unexpected error (constraint violation,
    /// type mismatch, etc.).
    #[error("A database operation failed: {0}")]
    Database(#[from] sqlx::Error),

    /// Database schema migration failed on startup.
    #[error("Database migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// An upstream / external service (price feed, KYC provider, etc.) returned
    /// an error or could not be reached.  Transient – safe to retry.
    #[error("An external service is temporarily unavailable: {0}")]
    ExternalService(String),

    /// A circuit-breaker is open for the named service; requests are being
    /// shed to protect the system.
    #[error("Service '{0}' is temporarily unavailable (circuit open). Please try again later.")]
    CircuitOpen(String),

    /// An operation exceeded its configured timeout.  Transient – safe to retry.
    #[error("The operation timed out. Please try again.")]
    Timeout,

    /// The service is overloaded or under maintenance.
    #[error("The service is temporarily unavailable. Please try again later.")]
    ServiceUnavailable(String),

    // ── 4xx ──────────────────────────────────────────────────────────────────
    /// JWT missing, expired, or signature invalid.
    #[error("Unauthorized")]
    Unauthorized,

    /// Authenticated but lacking the required permission.
    #[error("Access denied: {0}")]
    Forbidden(String),

    /// Requested resource does not exist.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Malformed or semantically incorrect request payload.
    #[error("Invalid request: {0}")]
    BadRequest(String),

    /// One or more fields failed validation rules.
    #[error("Validation failed: {0}")]
    Validation(String),

    /// The resource already exists or a state conflict was detected.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Request body exceeds the permitted size.
    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),

    /// Client has exceeded the configured rate limit for the endpoint.
    #[error("Rate limit exceeded. Please slow down and retry after the indicated period.")]
    TooManyRequests(String),
}

impl ApiError {
    /// Returns `true` if the error is likely transient and the client or an
    /// internal retry loop should attempt the operation again.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::DatabaseConnection(_)
                | Self::ExternalService(_)
                | Self::Timeout
                | Self::ServiceUnavailable(_)
        )
    }

    /// Returns a stable machine-readable code suitable for client-side branching
    /// and for grouping errors in monitoring dashboards.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::DatabaseConnection(_) => "DATABASE_CONNECTION_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Migration(_) => "MIGRATION_ERROR",
            Self::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            Self::CircuitOpen(_) => "CIRCUIT_OPEN",
            Self::Timeout => "TIMEOUT",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Conflict(_) => "CONFLICT",
            Self::PayloadTooLarge(_) => "PAYLOAD_TOO_LARGE",
            Self::TooManyRequests(_) => "RATE_LIMITED",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, user_message) = match &self {
            Self::Internal(e) => {
                tracing::error!(
                    error_code = "INTERNAL_ERROR",
                    error.message = %e,
                    error.debug = ?e,
                    "Internal server error"
                );
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            Self::DatabaseConnection(msg) => {
                tracing::error!(
                    error_code = "DATABASE_CONNECTION_ERROR",
                    error.message = %msg,
                    "Database connection error"
                );
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            Self::Database(e) => {
                tracing::error!(
                    error_code = "DATABASE_ERROR",
                    error.message = %e,
                    error.debug = ?e,
                    "Database operation error"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A database operation failed. Please try again.".to_string(),
                )
            }
            Self::Migration(e) => {
                tracing::error!(
                    error_code = "MIGRATION_ERROR",
                    error.message = %e,
                    "Database migration error"
                );
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            Self::ExternalService(msg) => {
                tracing::warn!(
                    error_code = "EXTERNAL_SERVICE_ERROR",
                    service = %msg,
                    "External service error"
                );
                (StatusCode::BAD_GATEWAY, self.to_string())
            }
            Self::CircuitOpen(service) => {
                tracing::warn!(
                    error_code = "CIRCUIT_OPEN",
                    service = %service,
                    "Circuit breaker open"
                );
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            Self::Timeout => {
                tracing::warn!(error_code = "TIMEOUT", "Request timeout");
                (StatusCode::GATEWAY_TIMEOUT, self.to_string())
            }
            Self::ServiceUnavailable(msg) => {
                tracing::warn!(
                    error_code = "SERVICE_UNAVAILABLE",
                    reason = %msg,
                    "Service unavailable"
                );
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            Self::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
            Self::PayloadTooLarge(_) => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()),
            Self::TooManyRequests(_) => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
        };

        let error_code = self.error_code();
        let body = Json(json!({
            "error": user_message,
            "error_code": error_code,
        }));

        (status, body).into_response()
    }
}

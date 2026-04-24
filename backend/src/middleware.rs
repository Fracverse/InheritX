use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, Request as HttpRequest, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use tower_governor::{errors::GovernorError, key_extractor::KeyExtractor};
use uuid::Uuid;

#[derive(Clone)]
pub struct RateLimitKeyExtractor {
    bypass_tokens: Arc<HashSet<String>>,
}

impl RateLimitKeyExtractor {
    pub fn new(bypass_tokens: Vec<String>) -> Self {
        let bypass_tokens = bypass_tokens
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect::<HashSet<_>>();

        Self {
            bypass_tokens: Arc::new(bypass_tokens),
        }
    }
}

impl KeyExtractor for RateLimitKeyExtractor {
    type Key = String;

    fn extract<T>(&self, req: &HttpRequest<T>) -> Result<Self::Key, GovernorError> {
        let maybe_internal_token = req
            .headers()
            .get("x-internal-token")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

        if let Some(token) = maybe_internal_token {
            if self.bypass_tokens.contains(token) {
                // Bypass by assigning a unique key so quota is effectively never shared.
                // This is intended for low-traffic trusted internal/admin automation.
                return Ok(format!("bypass:{}:{}", token, Uuid::new_v4()));
            }
        }

        let ip_key = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                req.headers()
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
            })
            .map(|s| s.to_string())
            .or_else(|| {
                req.extensions()
                    .get::<std::net::SocketAddr>()
                    .map(|addr| addr.ip().to_string())
            });

        ip_key.ok_or(GovernorError::UnableToExtractKey)
    }
}

pub fn rate_limit_error_response(error: GovernorError) -> Response<Body> {
    match error {
        GovernorError::TooManyRequests { wait_time, headers } => {
            tracing::warn!(
                error_code = "RATE_LIMITED",
                wait_time_seconds = wait_time,
                "Rate limit exceeded"
            );

            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(json!({
                    "error": "Rate limit exceeded. Please retry later.",
                    "error_code": "RATE_LIMITED",
                    "retry_after_seconds": wait_time,
                })),
            )
                .into_response();

            if let Some(extra_headers) = headers {
                response.headers_mut().extend(extra_headers);
            }
            response
        }
        GovernorError::UnableToExtractKey => {
            tracing::warn!("Rate-limit key extraction failed");
            (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({
                    "error": "Unable to determine request identity for rate limiting.",
                    "error_code": "RATE_LIMIT_KEY_ERROR",
                })),
            )
                .into_response()
        }
        GovernorError::Other { code, msg, headers } => {
            let mut response = (
                code,
                axum::Json(json!({
                    "error": msg.unwrap_or_else(|| "Rate limiting error".to_string()),
                    "error_code": "RATE_LIMIT_ERROR",
                })),
            )
                .into_response();

            if let Some(extra_headers) = headers {
                response.headers_mut().extend(extra_headers);
            }
            response
        }
    }
}

pub async fn attach_correlation_id(mut request: Request<Body>, next: Next) -> impl IntoResponse {
    let correlation_id = request
        .headers()
        .get("x-correlation-id")
        .and_then(|h| h.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request.extensions_mut().insert(correlation_id.clone());

    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&correlation_id) {
        response.headers_mut().insert("x-correlation-id", value);
    }
    response
}

pub async fn log_rate_limit_violations(request: Request<Body>, next: Next) -> impl IntoResponse {
    let path = request.uri().path().to_string();
    let method = request.method().clone();
    let correlation_id = request.extensions().get::<String>().cloned();

    let response = next.run(request).await;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        let mut metadata = HeaderMap::new();
        if let Some(value) = response.headers().get("x-ratelimit-after") {
            metadata.insert("x-ratelimit-after", value.clone());
        }
        tracing::warn!(
            error_code = "RATE_LIMITED",
            http.method = %method,
            http.path = %path,
            correlation_id = %correlation_id.unwrap_or_else(|| "n/a".to_string()),
            ratelimit_after = ?metadata.get("x-ratelimit-after"),
            "Request rejected due to rate limit"
        );
    }

    response
}

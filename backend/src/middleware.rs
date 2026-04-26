use axum::{
    body::Body,
    extract::Request,
    http::{
        HeaderMap, HeaderName, HeaderValue, Request as HttpRequest, StatusCode,
    },
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tower_governor::{errors::GovernorError, key_extractor::KeyExtractor};
use uuid::Uuid;

/// Request ID header name.
pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

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
            .map(str::trim)
            .filter(|s| !s.is_empty());

        if let Some(token) = maybe_internal_token {
            if self.bypass_tokens.contains(token) {
                return Ok(format!("bypass:{}:{}", token, Uuid::new_v4()));
            }
        }

        let ip_key = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .or_else(|| {
                req.headers()
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
            })
            .map(str::to_string)
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

/// Injects a unique `x-request-id` into each request and propagates it to the response.
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim()
        .to_string();

    let request_id = if request_id.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        request_id
    };

    req.headers_mut()
        .insert(X_REQUEST_ID.clone(), HeaderValue::from_str(&request_id).unwrap());

    let mut response = next.run(req).await;
    response.headers_mut().insert(
        X_REQUEST_ID.clone(),
        HeaderValue::from_str(&request_id).unwrap(),
    );
    response
}

/// Legacy alias retained for compatibility with existing call sites.
pub async fn attach_correlation_id(req: Request<Body>, next: Next) -> impl IntoResponse {
    request_id_middleware(req, next).await
}

/// Logs each incoming request with its method, URI, and assigned request ID.
pub async fn request_logging_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let request_id = req
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();

    tracing::info!(request_id = %request_id, method = %method, uri = %uri, "incoming request");

    let response = next.run(req).await;

    tracing::info!(
        request_id = %request_id,
        status = %response.status(),
        "request completed"
    );

    response
}

pub async fn log_rate_limit_violations(req: Request<Body>, next: Next) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let request_id = req
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("n/a")
        .to_string();

    let response = next.run(req).await;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        let mut metadata = HeaderMap::new();
        if let Some(value) = response.headers().get("x-ratelimit-after") {
            metadata.insert("x-ratelimit-after", value.clone());
        }
        tracing::warn!(
            error_code = "RATE_LIMITED",
            http.method = %method,
            http.path = %path,
            request_id = %request_id,
            ratelimit_after = ?metadata.get("x-ratelimit-after"),
            "Request rejected due to rate limit"
        );
    }

    response
}

/// Adds security headers to every response.
pub async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'self'"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    response
}

/// Enforces a per-request timeout. Returns 408 if the handler exceeds the limit.
pub async fn request_timeout_middleware(
    req: Request<Body>,
    next: Next,
    duration: Duration,
) -> Response {
    match timeout(duration, next.run(req)).await {
        Ok(response) => response,
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            axum::Json(serde_json::json!({ "error": "Request timed out" })),
        )
            .into_response(),
    }
}

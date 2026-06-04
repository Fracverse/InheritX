use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, Method, Request as HttpRequest, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use serde_json::Value as JsonValue;
use std::collections::{hash_map::DefaultHasher, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tower_governor::{errors::GovernorError, key_extractor::KeyExtractor};
use uuid::Uuid;

/// Request-scoped identifiers for error tracking and observability.
///
/// Middleware and auth extractors populate this automatically from headers and
/// the URI path. Handlers may insert or replace it in request extensions when
/// they have more precise context (e.g. a plan_id resolved from a request body
/// rather than the URL).
#[derive(Clone, Default, Debug)]
pub struct RequestContext {
    /// Authenticated user UUID, when known.
    pub user_id: Option<String>,
    /// Plan UUID being acted upon, when determinable from the request.
    pub plan_id: Option<String>,
}

/// Middleware that enforces a maximum request body size (bytes) and validates
/// JSON string lengths using the validation helpers.
pub async fn enforce_max_request_size(req: Request<Body>, next: Next) -> Response {
    // Respect a client-provided Content-Length header when present.
    if let Some(clv) = req.headers().get(axum::http::header::CONTENT_LENGTH) {
        if let Ok(s) = clv.to_str() {
            if let Ok(n) = s.parse::<usize>() {
                if n > crate::validation::DEFAULT_MAX_BODY_BYTES {
                    return (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        axum::Json(serde_json::json!({
                            "error": "Request body too large",
                            "error_code": "PAYLOAD_TOO_LARGE",
                        })),
                    )
                        .into_response();
                }
            }
        }
    }

    // Read the body up to the configured cap so we can inspect JSON payloads.
    let (parts, body) = req.into_parts();
    let bytes: axum::body::Bytes =
        match axum::body::to_bytes(body, crate::validation::DEFAULT_MAX_BODY_BYTES + 1).await {
            Ok(b) => b,
            Err(_) => {
                // If body couldn't be read, let the inner handler observe the failure.
                let req = Request::from_parts(parts, Body::empty());
                return next.run(req).await;
            }
        };

    if bytes.len() > crate::validation::DEFAULT_MAX_BODY_BYTES {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            axum::Json(serde_json::json!({
                "error": "Request body too large",
                "error_code": "PAYLOAD_TOO_LARGE",
            })),
        )
            .into_response();
    }

    // If JSON, parse and validate string field lengths.
    if let Some(ct) = parts.headers.get(axum::http::header::CONTENT_TYPE) {
        if let Ok(ctv) = ct.to_str() {
            if ctv.starts_with("application/json") {
                if let Ok(json_val) = serde_json::from_slice::<JsonValue>(&bytes) {
                    let mut errors = crate::validation::ValidationErrors::new();
                    crate::validation::validate_json_string_lengths(
                        &mut errors,
                        &json_val,
                        "$",
                        crate::validation::DEFAULT_MAX_FIELD_LENGTH,
                    );
                    if !errors.is_empty() {
                        return (
                            StatusCode::BAD_REQUEST,
                            axum::Json(serde_json::json!({
                                "error": "Validation failed",
                                "fields": errors.fields
                            })),
                        )
                            .into_response();
                    }
                }
            }
        }
    }

    // Reconstruct the request with the original body bytes and call the next
    // handler in the chain.
    let req = Request::from_parts(parts, Body::from(bytes));
    next.run(req).await
}

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

        // Some tests and internal service calls do not populate socket/connect
        // metadata. Falling back avoids converting auth failures into 400s.
        Ok(ip_key.unwrap_or_else(|| "unknown-client".to_string()))
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

    req.headers_mut().insert(
        X_REQUEST_ID.clone(),
        HeaderValue::from_str(&request_id).unwrap(),
    );

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
pub async fn request_logging_middleware(mut req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let request_id = req
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();

    let context = req
        .extensions()
        .get::<RequestContext>()
        .cloned()
        .unwrap_or_default();

    let headers = sanitize_headers(req.headers());
    let start = Instant::now();

    let response = next.run(req).await;
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    let status = response.status();

    let sampling_ratio = log_sampling_ratio();
    let is_high_traffic = is_high_traffic_path(&path);
    let should_emit_full_details =
        !is_high_traffic || should_sample_request(&request_id, sampling_ratio);

    if should_emit_full_details {
        tracing::info!(
            request_id = %request_id,
            http.method = %method,
            http.path = %path,
            http.status_code = %status,
            http.duration_ms = duration_ms,
            user_id = ?context.user_id,
            plan_id = ?context.plan_id,
            http.request_headers = ?headers,
            "request completed"
        );
    } else {
        tracing::info!(
            request_id = %request_id,
            http.method = %method,
            http.path = %path,
            http.status_code = %status,
            http.duration_ms = duration_ms,
            user_id = ?context.user_id,
            plan_id = ?context.plan_id,
            "request completed"
        );
    }

    response
}

fn sanitize_headers(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                sanitize_header_value(name, value),
            )
        })
        .collect()
}

fn sanitize_header_value(name: &HeaderName, value: &HeaderValue) -> String {
    let sensitive_headers = [
        "authorization",
        "cookie",
        "set-cookie",
        "x-api-key",
        "x-csrf-token",
        "x-csrf",
    ];
    let header_name = name.as_str().to_ascii_lowercase();

    if sensitive_headers.contains(&header_name.as_str()) {
        "***".to_string()
    } else {
        value.to_str().unwrap_or("<binary>").to_string()
    }
}

fn is_high_traffic_path(path: &str) -> bool {
    matches!(
        path,
        "/api/metrics" | "/api/health" | "/api/ping" | "/api/status" | "/api/loans/lifecycle"
    )
}

fn log_sampling_ratio() -> f64 {
    static RATIO: OnceLock<f64> = OnceLock::new();
    *RATIO.get_or_init(|| {
        std::env::var("LOG_SAMPLING_PERCENT")
            .ok()
            .and_then(|value| value.parse::<f64>().ok())
            .map(|percent| percent.clamp(0.0, 100.0) / 100.0)
            .unwrap_or(0.1)
    })
}

fn should_sample_request(request_id: &str, ratio: f64) -> bool {
    if ratio <= 0.0 {
        return false;
    }
    if ratio >= 1.0 {
        return true;
    }

    let mut hasher = DefaultHasher::new();
    request_id.hash(&mut hasher);
    let bucket = hasher.finish() % 1000;
    bucket < (ratio * 1000.0).round() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use axum::http::HeaderValue;

    #[test]
    fn sanitize_headers_masks_sensitive_values() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer abc123"));
        headers.insert("cookie", HeaderValue::from_static("session=secret"));
        headers.insert("x-api-key", HeaderValue::from_static("key"));
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        let sanitized = sanitize_headers(&headers);
        assert!(sanitized.contains(&("authorization".to_string(), "***".to_string())));
        assert!(sanitized.contains(&("cookie".to_string(), "***".to_string())));
        assert!(sanitized.contains(&("x-api-key".to_string(), "***".to_string())));
        assert!(sanitized.contains(&("content-type".to_string(), "application/json".to_string())));
    }

    #[test]
    fn should_sample_request_is_deterministic() {
        let request_id = "abc-123";
        let ratio = 0.5;
        let first = should_sample_request(request_id, ratio);
        let second = should_sample_request(request_id, ratio);
        assert_eq!(first, second);
    }
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

/// Ensures write-method responses are never accidentally cached.
///
/// - **GET / HEAD**: passes through untouched — individual handlers set their
///   own `ETag` and `Cache-Control` headers via [`crate::cache`].
/// - **POST / PUT / PATCH / DELETE / OPTIONS**: injects `Cache-Control: no-store`
///   on the response so the client (and any intermediary proxy) cannot cache
///   the result of a mutating operation.
///
/// This middleware sits **after** `security_headers_middleware` in the stack so
/// that it can safely overwrite any generic cache header set upstream without
/// being overwritten itself.
pub async fn cache_headers_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let mut response = next.run(req).await;

    let is_write = matches!(
        method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE | Method::OPTIONS
    );

    if is_write {
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        );
    }

    response
}

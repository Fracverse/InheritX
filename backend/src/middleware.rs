use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Injects standard rate-limit information headers into every response so that
/// API clients can implement adaptive throttling without waiting for a 429.
///
/// The `limit` and `policy` values are informational strings provided by the
/// caller (e.g. "10" and "10;w=1" respectively).  The actual enforcement is
/// done by `tower_governor`; this layer only adds the headers.
pub async fn rate_limit_headers(
    request: Request<Body>,
    next: Next,
    limit: u64,
    policy: &'static str,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    if let Ok(v) = HeaderValue::from_str(&limit.to_string()) {
        headers.insert("X-RateLimit-Limit", v);
    }
    if let Ok(v) = HeaderValue::from_str(policy) {
        headers.insert("X-RateLimit-Policy", v);
    }
    // Inform clients about the rate-limit standard used (draft-ietf-httpapi-ratelimit-headers).
    headers.insert(
        header::VARY,
        HeaderValue::from_static("X-RateLimit-Limit, X-RateLimit-Policy"),
    );
    response
}

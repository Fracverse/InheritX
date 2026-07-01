/// Rate limiting and security-header middleware for InheritX.
use std::{
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{HeaderValue, Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use dashmap::DashMap;

/// Configuration knobs for the rate limiter.
/// Defaults: 100 requests per 60-second window.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u64,
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
        }
    }
}

#[derive(Debug)]
struct RateLimitState {
    count: u64,
    window_start: Instant,
}

/// Thread-safe store of per-IP rate-limit state.
#[derive(Clone, Default)]
pub struct RateLimitStore(Arc<DashMap<IpAddr, RateLimitState>>);

impl RateLimitStore {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }

    /// Returns true when the request is within the allowed rate.
    /// Returns false when the caller should respond with 429.
    pub fn check_and_increment(&self, ip: IpAddr, cfg: &RateLimitConfig) -> bool {
        let now = Instant::now();
        let mut entry = self.0.entry(ip).or_insert_with(|| RateLimitState {
            count: 0,
            window_start: now,
        });

        if now.duration_since(entry.window_start) >= cfg.window {
            entry.count = 0;
            entry.window_start = now;
        }

        entry.count += 1;
        entry.count <= cfg.max_requests
    }
}

/// Axum middleware function for rate limiting.
pub async fn rate_limit_middleware(
    req: Request<Body>,
    next: Next,
    store: RateLimitStore,
    config: Arc<RateLimitConfig>,
) -> Response<Body> {
    let ip = req
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));

    if !store.check_and_increment(ip, &config) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too Many Requests - rate limit exceeded. Please slow down.",
        )
            .into_response();
    }

    next.run(req).await
}

/// HSTS layer: max-age=1 year, includeSubDomains, preload.
pub fn hsts_layer() -> tower_http::set_header::SetResponseHeaderLayer<HeaderValue> {
    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
        axum::http::header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    )
}

/// Content-Security-Policy layer.
pub fn csp_layer() -> tower_http::set_header::SetResponseHeaderLayer<HeaderValue> {
    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
        axum::http::header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"),
    )
}

/// X-Frame-Options: DENY layer.
pub fn x_frame_options_layer() -> tower_http::set_header::SetResponseHeaderLayer<HeaderValue> {
    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
        axum::http::header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    )
}

/// X-Content-Type-Options: nosniff layer.
pub fn x_content_type_options_layer() -> tower_http::set_header::SetResponseHeaderLayer<HeaderValue>
{
    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    )
}

/// Referrer-Policy layer.
pub fn referrer_policy_layer() -> tower_http::set_header::SetResponseHeaderLayer<HeaderValue> {
    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
        axum::http::header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    )
}

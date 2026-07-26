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

#[derive(Debug, Clone)]
pub struct CorsMatcher {
    patterns: Vec<OriginPattern>,
}

#[derive(Debug, Clone)]
struct OriginPattern {
    scheme: String,
    host_pattern: HostPattern,
    port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HostPattern {
    Exact(String),
    SubdomainWildcard(String),
}

impl OriginPattern {
    pub fn parse(pattern: &str) -> Option<Self> {
        let parts: Vec<&str> = pattern.split("://").collect();
        if parts.len() != 2 {
            return None;
        }
        let scheme = parts[0].to_lowercase();
        if scheme != "http" && scheme != "https" {
            return None;
        }

        let host_port = parts[1];
        
        let (host_str, port) = if host_port.starts_with('[') {
            if let Some(bracket_end) = host_port.find(']') {
                let host = &host_port[..=bracket_end];
                let remainder = &host_port[bracket_end + 1..];
                if remainder.starts_with(':') {
                    let port_str = &remainder[1..];
                    let port = port_str.parse::<u16>().ok()?;
                    (host, Some(port))
                } else if remainder.is_empty() {
                    (host, None)
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            let colon_parts: Vec<&str> = host_port.split(':').collect();
            if colon_parts.len() == 1 {
                (host_port, None)
            } else if colon_parts.len() == 2 {
                let port = colon_parts[1].parse::<u16>().ok()?;
                (colon_parts[0], Some(port))
            } else {
                return None;
            }
        };

        if host_str.is_empty() {
            return None;
        }

        // Scheme verification: http is only allowed for loopback origins.
        let is_local = host_str == "localhost" 
            || host_str == "127.0.0.1" 
            || host_str == "[::1]"
            || host_str.ends_with(".localhost")
            || host_str == "*.localhost";

        if scheme == "http" && !is_local {
            return None;
        }

        let host_pattern = if host_str.starts_with("*.") {
            if host_str.len() <= 2 {
                return None;
            }
            HostPattern::SubdomainWildcard(host_str[2..].to_lowercase())
        } else {
            HostPattern::Exact(host_str.to_lowercase())
        };

        Some(OriginPattern {
            scheme,
            host_pattern,
            port,
        })
    }

    pub fn matches(&self, origin: &str) -> bool {
        let parsed = match OriginPattern::parse(origin) {
            Some(o) => o,
            None => return false,
        };

        if self.scheme != parsed.scheme {
            return false;
        }

        if self.port != parsed.port {
            return false;
        }

        match &self.host_pattern {
            HostPattern::Exact(exact_host) => {
                match &parsed.host_pattern {
                    HostPattern::Exact(incoming_host) => exact_host == incoming_host,
                    _ => false,
                }
            }
            HostPattern::SubdomainWildcard(domain) => {
                match &parsed.host_pattern {
                    HostPattern::Exact(incoming_host) => {
                        incoming_host == domain || incoming_host.ends_with(&format!(".{}", domain))
                    }
                    _ => false,
                }
            }
        }
    }
}

impl CorsMatcher {
    pub fn new(allowed_origins: &[String]) -> Self {
        let mut patterns = Vec::new();
        for origin in allowed_origins {
            if let Some(pattern) = OriginPattern::parse(origin) {
                patterns.push(pattern);
            }
        }
        CorsMatcher { patterns }
    }

    pub fn is_allowed(&self, origin: &HeaderValue) -> bool {
        let origin_str = match origin.to_str() {
            Ok(s) => s,
            Err(_) => return false,
        };
        
        for pattern in &self.patterns {
            if pattern.matches(origin_str) {
                return true;
            }
        }
        false
    }
}

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use inheritx_backend::middleware::{rate_limit_middleware, RateLimitConfig, RateLimitStore};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

fn build_rate_limited_app(max_requests: u64, window_secs: u64) -> Router {
    let store = RateLimitStore::new();
    let config = Arc::new(RateLimitConfig {
        max_requests,
        window: Duration::from_secs(window_secs),
    });

    Router::new()
        .route("/test", get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(move |req, next| {
            rate_limit_middleware(req, next, store.clone(), config.clone())
        }))
}

#[tokio::test]
async fn test_requests_within_limit_succeed() {
    let app = build_rate_limited_app(5, 60);

    for _ in 0..5 {
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_request_exceeding_limit_returns_429() {
    let app = build_rate_limited_app(3, 60);

    for _ in 0..3 {
        app.clone()
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();
    }

    // 4th request should be rate limited
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_rate_limit_window_resets() {
    let store = RateLimitStore::new();
    let config = RateLimitConfig {
        max_requests: 2,
        window: Duration::from_millis(100),
    };

    let ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();

    // Use up the limit
    assert!(store.check_and_increment(ip, &config));
    assert!(store.check_and_increment(ip, &config));
    // 3rd should fail
    assert!(!store.check_and_increment(ip, &config));

    // Wait for window to expire
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should be allowed again after window reset
    assert!(store.check_and_increment(ip, &config));
}

#[tokio::test]
async fn test_heavy_mock_traffic_triggers_rate_limit() {
    let app = build_rate_limited_app(10, 60);
    let mut limited_count = 0;

    for _ in 0..30 {
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            limited_count += 1;
        }
    }

    // At least 20 requests should have been rate limited
    assert!(
        limited_count >= 20,
        "Expected at least 20 limited, got {limited_count}"
    );
}

#[tokio::test]
async fn test_different_ips_have_independent_limits() {
    let store = RateLimitStore::new();
    let config = RateLimitConfig {
        max_requests: 1,
        window: Duration::from_secs(60),
    };

    let ip1: std::net::IpAddr = "192.168.1.1".parse().unwrap();
    let ip2: std::net::IpAddr = "192.168.1.2".parse().unwrap();

    // IP1 uses its limit
    assert!(store.check_and_increment(ip1, &config));
    assert!(!store.check_and_increment(ip1, &config));

    // IP2 should still be allowed independently
    assert!(store.check_and_increment(ip2, &config));
}

#[cfg(test)]
mod cors_tests {
    use axum::http::HeaderValue;
    use inheritx_backend::middleware::CorsMatcher;

    #[test]
    fn test_cors_exact_matching() {
        let allowed = vec!["https://inheritx.vercel.app".to_string()];
        let matcher = CorsMatcher::new(&allowed);

        assert!(matcher.is_allowed(&HeaderValue::from_static("https://inheritx.vercel.app")));
        assert!(!matcher.is_allowed(&HeaderValue::from_static("http://inheritx.vercel.app")));
        assert!(!matcher.is_allowed(&HeaderValue::from_static("https://other.app")));
    }

    #[test]
    fn test_cors_wildcard_matching() {
        let allowed = vec!["https://*.inheritx.vercel.app".to_string(), "https://*.vercel.app".to_string()];
        let matcher = CorsMatcher::new(&allowed);

        // Subdomain matching
        assert!(matcher.is_allowed(&HeaderValue::from_static("https://sub.inheritx.vercel.app")));
        assert!(matcher.is_allowed(&HeaderValue::from_static("https://test.dev.vercel.app")));
        assert!(matcher.is_allowed(&HeaderValue::from_static("https://vercel.app")));

        // Block suffix bypasses
        assert!(!matcher.is_allowed(&HeaderValue::from_static("https://attacker-vercel.app")));
        assert!(!matcher.is_allowed(&HeaderValue::from_static("https://attackervercel.app")));
    }

    #[test]
    fn test_cors_scheme_and_local_rules() {
        // HTTP disallowed for remote hosts
        let allowed = vec!["http://example.com".to_string(), "http://localhost".to_string(), "http://127.0.0.1".to_string()];
        let matcher = CorsMatcher::new(&allowed);

        assert!(!matcher.is_allowed(&HeaderValue::from_static("http://example.com")));
        assert!(matcher.is_allowed(&HeaderValue::from_static("http://localhost")));
        assert!(matcher.is_allowed(&HeaderValue::from_static("http://127.0.0.1")));
    }

    #[test]
    fn test_cors_ports() {
        let allowed = vec!["http://localhost:3000".to_string(), "https://example.com:8443".to_string()];
        let matcher = CorsMatcher::new(&allowed);

        assert!(matcher.is_allowed(&HeaderValue::from_static("http://localhost:3000")));
        assert!(!matcher.is_allowed(&HeaderValue::from_static("http://localhost:3001")));
        assert!(!matcher.is_allowed(&HeaderValue::from_static("http://localhost")));

        assert!(matcher.is_allowed(&HeaderValue::from_static("https://example.com:8443")));
        assert!(!matcher.is_allowed(&HeaderValue::from_static("https://example.com")));
    }
}

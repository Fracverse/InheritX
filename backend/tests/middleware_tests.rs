use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use inheritx_backend::middleware::{
    geo_restriction_middleware, rate_limit_middleware, CountryResolver, GeoGuardConfig,
    RateLimitConfig, RateLimitStore,
};
use std::net::IpAddr;
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

/// A fixed-answer `CountryResolver` double, standing in for a MaxMind DB
/// lookup so the fallback path can be tested without a real `.mmdb` file.
struct StubResolver(Option<&'static str>);

impl CountryResolver for StubResolver {
    fn lookup_country(&self, _ip: IpAddr) -> Option<String> {
        self.0.map(str::to_string)
    }
}

fn build_geo_guarded_app(blocked: &[&str], resolver: Option<Arc<dyn CountryResolver>>) -> Router {
    let config = Arc::new(GeoGuardConfig::new(blocked.iter().map(|c| c.to_string())));

    Router::new()
        .route("/test", get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(move |req, next| {
            geo_restriction_middleware(req, next, config.clone(), resolver.clone())
        }))
}

async fn request_with_country(app: &Router, country: Option<&str>) -> StatusCode {
    let mut builder = Request::builder().uri("/test");
    if let Some(country) = country {
        builder = builder.header("CF-IPCountry", country);
    }
    app.clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn test_geo_guard_allows_non_sanctioned_country() {
    let app = build_geo_guarded_app(&["CU", "IR", "KP", "SY"], None);
    assert_eq!(request_with_country(&app, Some("US")).await, StatusCode::OK);
}

#[tokio::test]
async fn test_geo_guard_blocks_sanctioned_country_header() {
    let app = build_geo_guarded_app(&["CU", "IR", "KP", "SY"], None);
    assert_eq!(
        request_with_country(&app, Some("IR")).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_geo_guard_header_check_is_case_insensitive() {
    let app = build_geo_guarded_app(&["CU", "IR", "KP", "SY"], None);
    assert_eq!(
        request_with_country(&app, Some("kp")).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_geo_guard_allows_missing_header_with_no_resolver() {
    let app = build_geo_guarded_app(&["CU", "IR", "KP", "SY"], None);
    assert_eq!(request_with_country(&app, None).await, StatusCode::OK);
}

#[tokio::test]
async fn test_geo_guard_treats_unresolved_cf_codes_as_unknown() {
    // "XX" (unknown) and "T1" (Tor) are not real country codes and must not
    // be matched against the deny-list.
    let app = build_geo_guarded_app(&["XX", "T1"], None);
    assert_eq!(request_with_country(&app, Some("XX")).await, StatusCode::OK);
    assert_eq!(request_with_country(&app, Some("T1")).await, StatusCode::OK);
}

#[tokio::test]
async fn test_geo_guard_falls_back_to_resolver_when_header_absent() {
    let resolver: Arc<dyn CountryResolver> = Arc::new(StubResolver(Some("SY")));
    let app = build_geo_guarded_app(&["CU", "IR", "KP", "SY"], Some(resolver));

    assert_eq!(
        request_with_country(&app, None).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_geo_guard_prefers_header_over_resolver() {
    // Resolver says sanctioned, but the CF header (authoritative at the edge)
    // says a clean country — the header should win.
    let resolver: Arc<dyn CountryResolver> = Arc::new(StubResolver(Some("IR")));
    let app = build_geo_guarded_app(&["CU", "IR", "KP", "SY"], Some(resolver));

    assert_eq!(request_with_country(&app, Some("US")).await, StatusCode::OK);
}

#[tokio::test]
async fn test_geo_guard_config_uses_defaults_when_unset() {
    let config = GeoGuardConfig::from_override(None);
    assert!(config.is_blocked("IR"));
    assert!(config.is_blocked("ir"));
    assert!(!config.is_blocked("US"));
}

#[tokio::test]
async fn test_geo_guard_config_respects_override() {
    let config = GeoGuardConfig::from_override(Some("RU, BY".to_string()));
    assert!(config.is_blocked("RU"));
    assert!(config.is_blocked("BY"));
    assert!(!config.is_blocked("IR"));
}

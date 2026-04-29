//! Integration + unit tests for the HTTP caching layer.
//!
//! These tests verify:
//! - ETag computation (determinism, uniqueness)
//! - Conditional-request evaluation (If-None-Match → 304)
//! - Write-endpoint safety (POST/PUT/DELETE → no-store)
//! - Cache-Control header values

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use inheritx_backend::cache;
use serde_json::{json, Value};
use tower::ServiceExt; // for `oneshot`

// ── Helper ────────────────────────────────────────────────────────────────────

/// Extract the first header value as a string.
fn get_header<'a>(
    response: &'a axum::response::Response,
    name: header::HeaderName,
) -> Option<&'a str> {
    response.headers().get(name).and_then(|v| v.to_str().ok())
}

// ── ETag unit tests ───────────────────────────────────────────────────────────

#[test]
fn etag_deterministic_across_calls() {
    let data = json!({ "id": "abc", "value": 42, "active": true });
    let e1 = cache::compute_etag(&data);
    let e2 = cache::compute_etag(&data);
    assert_eq!(e1, e2, "ETag must be stable for the same data");
}

#[test]
fn etag_unique_for_different_data() {
    let a = json!({ "id": 1 });
    let b = json!({ "id": 2 });
    assert_ne!(
        cache::compute_etag(&a),
        cache::compute_etag(&b),
        "Different data must produce different ETags"
    );
}

#[test]
fn etag_strong_format() {
    let data = json!({ "x": 1 });
    let etag = cache::compute_etag(&data);
    assert!(
        etag.starts_with('"') && etag.ends_with('"'),
        "ETag must be wrapped in double quotes (strong ETag): got {etag}"
    );
    // No W/ prefix → strong ETag
    assert!(
        !etag.starts_with("W/"),
        "ETag must not be a weak ETag: got {etag}"
    );
}

#[test]
fn etag_matches_exact_value() {
    let data = json!({ "hello": "world" });
    let etag = cache::compute_etag(&data);
    assert!(cache::etag_matches(&etag, &etag));
}

#[test]
fn etag_matches_wildcard() {
    let etag = cache::compute_etag(&json!({ "x": 99 }));
    assert!(
        cache::etag_matches(&etag, "*"),
        "Wildcard If-None-Match: * must always match"
    );
}

#[test]
fn etag_no_match_on_stale_value() {
    let current = cache::compute_etag(&json!({ "v": 2 }));
    let stale = cache::compute_etag(&json!({ "v": 1 }));
    assert!(
        !cache::etag_matches(&current, &stale),
        "Stale ETag must not match current"
    );
}

#[test]
fn etag_matches_in_multi_value_header() {
    let current = cache::compute_etag(&json!({ "v": 3 }));
    let old1 = cache::compute_etag(&json!({ "v": 1 }));
    let old2 = cache::compute_etag(&json!({ "v": 2 }));
    // Multi-value: "old1, old2, current"
    let header_value = format!("{old1}, {old2}, {current}");
    assert!(
        cache::etag_matches(&current, &header_value),
        "ETag must match when present in a multi-value If-None-Match header"
    );
}

// ── is_not_modified ───────────────────────────────────────────────────────────

#[test]
fn is_not_modified_true_when_etag_matches() {
    use axum::http::{HeaderMap, HeaderValue};

    let data = json!({ "id": 7 });
    let etag = cache::compute_etag(&data);

    let mut headers = HeaderMap::new();
    headers.insert(header::IF_NONE_MATCH, HeaderValue::from_str(&etag).unwrap());

    assert!(cache::is_not_modified(&headers, &etag));
}

#[test]
fn is_not_modified_false_when_no_header() {
    use axum::http::HeaderMap;

    let etag = cache::compute_etag(&json!({ "id": 7 }));
    let headers = HeaderMap::new();
    assert!(!cache::is_not_modified(&headers, &etag));
}

#[test]
fn is_not_modified_false_when_etag_differs() {
    use axum::http::{HeaderMap, HeaderValue};

    let current = cache::compute_etag(&json!({ "v": 2 }));
    let stale = cache::compute_etag(&json!({ "v": 1 }));

    let mut headers = HeaderMap::new();
    headers.insert(
        header::IF_NONE_MATCH,
        HeaderValue::from_str(&stale).unwrap(),
    );

    assert!(!cache::is_not_modified(&headers, &current));
}

// ── Cache-Control builders ────────────────────────────────────────────────────

#[test]
fn cache_control_public_header_value() {
    let hv = cache::cache_control_public(300);
    assert_eq!(hv.to_str().unwrap(), "public, max-age=300, must-revalidate");
}

#[test]
fn cache_control_private_header_value() {
    let hv = cache::cache_control_private(60);
    assert_eq!(hv.to_str().unwrap(), "private, max-age=60, must-revalidate");
}

#[test]
fn cache_control_no_store_header_value() {
    let hv = cache::cache_control_no_store();
    assert_eq!(hv.to_str().unwrap(), "no-store");
}

// ── apply_cache_headers ───────────────────────────────────────────────────────

#[test]
fn apply_cache_headers_injects_etag_cc_vary() {
    use axum::response::IntoResponse;

    let mut response = StatusCode::OK.into_response();
    let etag = cache::compute_etag(&json!({ "test": true }));
    cache::apply_cache_headers(&mut response, &etag, cache::cache_control_private(120));

    let hdrs = response.headers();
    assert_eq!(
        hdrs.get(header::ETAG).unwrap().to_str().unwrap(),
        etag,
        "ETag header must match computed ETag"
    );
    assert_eq!(
        hdrs.get(header::CACHE_CONTROL).unwrap().to_str().unwrap(),
        "private, max-age=120, must-revalidate"
    );
    assert!(
        hdrs.contains_key(header::VARY),
        "Vary header must be present"
    );
}

// ── not_modified_response ─────────────────────────────────────────────────────

#[tokio::test]
async fn not_modified_response_has_304_status() {
    let etag = cache::compute_etag(&json!({ "v": 1 }));
    let response = cache::not_modified_response(&etag);
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
}

#[tokio::test]
async fn not_modified_response_has_etag_header() {
    let etag = cache::compute_etag(&json!({ "v": 1 }));
    let response = cache::not_modified_response(&etag);
    let etag_hdr = response
        .headers()
        .get(header::ETAG)
        .and_then(|v| v.to_str().ok())
        .expect("ETag header must be present in 304 response");
    assert_eq!(etag_hdr, etag);
}

// ── Health endpoint integration tests ─────────────────────────────────────────

/// Build a minimal router with just the health endpoint wired up with caching.
/// (Tests that don't need a database use this lightweight router.)
fn health_router() -> Router {
    use axum::http::HeaderMap;
    use axum::response::IntoResponse;
    use axum::{routing::get, Json};

    Router::new().route(
        "/health",
        get(|headers: HeaderMap| async move {
            let data = json!({ "status": "ok", "message": "App is healthy" });
            let etag = cache::compute_etag(&data);
            if cache::is_not_modified(&headers, &etag) {
                return cache::not_modified_response_with_cc(
                    &etag,
                    cache::cache_control_public(10),
                );
            }
            let mut response = Json(data).into_response();
            cache::apply_cache_headers(&mut response, &etag, cache::cache_control_public(10));
            response
        }),
    )
}

#[tokio::test]
async fn health_returns_etag_and_cache_control() {
    let app = health_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers().contains_key(header::ETAG),
        "GET /health must include ETag header"
    );
    assert!(
        response.headers().contains_key(header::CACHE_CONTROL),
        "GET /health must include Cache-Control header"
    );
    let cc = response
        .headers()
        .get(header::CACHE_CONTROL)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        cc.contains("public") && cc.contains("max-age=10"),
        "Cache-Control must be public with max-age=10, got: {cc}"
    );
}

#[tokio::test]
async fn conditional_get_returns_304_on_matching_etag() {
    let app = health_router();

    // First request to get the ETag
    let req1 = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    let etag = res1
        .headers()
        .get(header::ETAG)
        .expect("First response must have ETag")
        .to_str()
        .unwrap()
        .to_string();

    // Second request with If-None-Match → expect 304
    let req2 = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header(header::IF_NONE_MATCH, &etag)
        .body(Body::empty())
        .unwrap();
    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(
        res2.status(),
        StatusCode::NOT_MODIFIED,
        "Matching ETag must yield 304 Not Modified"
    );
}

#[tokio::test]
async fn stale_etag_returns_200_with_new_etag() {
    let app = health_router();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header(header::IF_NONE_MATCH, "\"stale-etag-value\"")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Stale ETag must yield 200 with fresh response"
    );
    assert!(
        response.headers().contains_key(header::ETAG),
        "200 response must include a fresh ETag"
    );
}

// ── Write-endpoint safety (no-store) ─────────────────────────────────────────

/// Router with a write endpoint and the cache middleware applied.
fn write_router() -> Router {
    use axum::{middleware, routing::post, Json};

    Router::new()
        .route(
            "/api/things",
            post(|| async { Json(json!({ "status": "created" })) }),
        )
        .layer(middleware::from_fn(
            inheritx_backend::middleware::cache_headers_middleware,
        ))
}

#[tokio::test]
async fn post_endpoint_has_no_store_cache_control() {
    let app = write_router();

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/things")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let cc = response
        .headers()
        .get(header::CACHE_CONTROL)
        .expect("POST response must have Cache-Control header")
        .to_str()
        .unwrap();
    assert_eq!(
        cc, "no-store",
        "POST must set Cache-Control: no-store, got: {cc}"
    );
}

#[tokio::test]
async fn delete_endpoint_has_no_store_cache_control() {
    use axum::{middleware, routing::delete, Json};

    let app = Router::new()
        .route(
            "/api/things/:id",
            delete(|| async { Json(json!({ "status": "deleted" })) }),
        )
        .layer(middleware::from_fn(
            inheritx_backend::middleware::cache_headers_middleware,
        ));

    let request = Request::builder()
        .method(Method::DELETE)
        .uri("/api/things/123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let cc = response
        .headers()
        .get(header::CACHE_CONTROL)
        .expect("DELETE response must have Cache-Control header")
        .to_str()
        .unwrap();
    assert_eq!(cc, "no-store");
}

// ── ETag changes when data changes ────────────────────────────────────────────

#[test]
fn etag_changes_when_plan_is_updated() {
    // Simulate: before and after a plan update
    let before = json!({
        "id": "plan-1",
        "status": "active",
        "amount": "1000.00",
        "updated_at": "2026-01-01T00:00:00Z"
    });
    let after = json!({
        "id": "plan-1",
        "status": "claimed",
        "amount": "1000.00",
        "updated_at": "2026-04-29T10:00:00Z"
    });

    let etag_before = cache::compute_etag(&before);
    let etag_after = cache::compute_etag(&after);

    assert_ne!(
        etag_before, etag_after,
        "ETag must change when plan data changes — cache is automatically invalidated"
    );
}

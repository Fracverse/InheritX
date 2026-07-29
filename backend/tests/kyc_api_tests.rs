use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use inheritx_backend::{create_router, AppState, PlanCache};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

fn setup_app() -> axum::Router {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/test".to_string());
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy(&database_url)
        .unwrap();
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        db_pool,
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: PlanCache::disabled(),
        apy_cache: dashmap::DashMap::new(),
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
    });
    create_router(state)
}

#[tokio::test]
async fn test_get_kyc_status_requires_wallet_address() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/kyc/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_kyc_status_with_address_hits_db() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/kyc/status?wallet_address=GDTEST123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_submit_kyc_rejects_empty_body() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/kyc/submit")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_submit_kyc_with_valid_body_hits_db() {
    let app = setup_app();
    let body = json!({
        "wallet_address": "GDTEST123",
        "full_name": "John Doe",
        "email": "john@example.com",
        "date_of_birth": "1990-01-01",
        "nationality": "US",
        "id_type": "international_passport",
        "id_number": "AB123456",
        "expiry_date": "2030-01-01",
        "street_address": "123 Main St",
        "city": "New York",
        "country": "US",
        "postal_code": "10001"
    })
    .to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/kyc/submit")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_upload_kyc_document_returns_ok() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/kyc/upload")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_upload_kyc_document_returns_expected_structure() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/kyc/upload")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body.get("document_id").is_some(), "missing 'document_id'");
    assert!(body.get("url").is_some(), "missing 'url'");
}

#[tokio::test]
async fn test_is_kyc_required_returns_true() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/kyc/required")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["required"], true);
    assert!(body.get("reason").is_some());
}

#[tokio::test]
async fn test_get_kyc_requirements_returns_ok() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/kyc/requirements")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_kyc_requirements_returns_expected_structure() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/kyc/requirements")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body.get("requires_id").is_some());
    assert!(body.get("requires_address_proof").is_some());
    assert!(body.get("supported_id_types").is_some());
    assert!(body.get("supported_countries").is_some());
    let id_types = body["supported_id_types"].as_array().unwrap();
    assert!(!id_types.is_empty());
    let countries = body["supported_countries"].as_array().unwrap();
    assert!(!countries.is_empty());
}

#[tokio::test]
async fn test_get_kyc_status_is_public() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/kyc/status?wallet_address=GDTEST123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_kyc_endpoints_do_not_require_auth() {
    let app = setup_app();
    for (method, uri) in [
        (http::Method::GET, "/api/kyc/status?wallet_address=GDTEST"),
        (http::Method::POST, "/api/kyc/submit"),
        (http::Method::POST, "/api/kyc/upload"),
        (http::Method::GET, "/api/kyc/required"),
        (http::Method::GET, "/api/kyc/requirements"),
    ] {
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header(http::header::CONTENT_TYPE, "application/json");
        let response = app
            .clone()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_ne!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Endpoint {uri} should not require auth"
        );
    }
}

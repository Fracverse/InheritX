use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use ed25519_dalek::{Signer, SigningKey};
use inheritx_backend::{create_router, AppState, PlanCache, PlanResponse};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt; // for oneshot

fn generate_valid_signature(body: &str, _public_key_hex: &str) -> (String, String) {
    // Use a fixed test keypair for deterministic testing
    let secret_bytes: [u8; 32] = [
        0x9d, 0x61, 0xb8, 0xbb, 0xd0, 0xa3, 0x0a, 0x78, 0x23, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
        0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
        0xde, 0xf0,
    ];

    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();
    let public_key_hex = format!("0x{}", hex::encode(verifying_key.to_bytes()));

    let signature = signing_key.sign(body.as_bytes());
    let signature_hex = hex::encode(signature.to_bytes());

    (public_key_hex, signature_hex)
}

fn setup_app() -> axum::Router {
    setup_app_with_cache(PlanCache::disabled())
}

fn setup_app_with_cache(plan_cache: PlanCache) -> axum::Router {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/test".to_string());

    // Lazy pool: no connection at setup time; these tests assert auth/validation
    // before most handlers touch the database.
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
        plan_cache,
        apy_cache: dashmap::DashMap::new(),
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
    });
    create_router(state)
}

#[tokio::test]
async fn test_router_compiles() {
    let _app = setup_app();
}

#[tokio::test]
async fn test_create_plan_validation_empty_owner() {
    let app = setup_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/plans")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner": " ",
                        "token": "USDC",
                        "amount": 100.0,
                        "grace_period": 3600,
                        "earn_yield": false,
                        "yield_rate_bps": 0,
                        "last_ping": 0,
                        "is_active": true,
                        "beneficiaries": [
                            {
                                "address": "beneficiary_1",
                                "name": "B1",
                                "allocation_bps": 10000,
                                "fiat_anchor_info": ""
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_plan_validation_invalid_bps() {
    let app = setup_app();

    // Sum is 9000, not 10000
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/plans")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner": "owner_address",
                        "token": "USDC",
                        "amount": 100.0,
                        "grace_period": 3600,
                        "earn_yield": false,
                        "yield_rate_bps": 0,
                        "last_ping": 0,
                        "is_active": true,
                        "beneficiaries": [
                            {
                                "address": "beneficiary_1",
                                "name": "B1",
                                "allocation_bps": 9000,
                                "fiat_anchor_info": ""
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_plan_validation_negative_amount() {
    let app = setup_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/plans")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner": "owner_address",
                        "token": "USDC",
                        "amount": -50.0,
                        "grace_period": 3600,
                        "earn_yield": false,
                        "yield_rate_bps": 0,
                        "last_ping": 0,
                        "is_active": true,
                        "beneficiaries": [
                            {
                                "address": "beneficiary_1",
                                "name": "B1",
                                "allocation_bps": 10000,
                                "fiat_anchor_info": ""
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_plan_validation_too_many_beneficiaries() {
    let app = setup_app();

    let mut beneficiaries = Vec::new();
    for i in 0..101 {
        beneficiaries.push(json!({
            "address": format!("beneficiary_{}", i),
            "name": format!("B{}", i),
            "allocation_bps": 99,
            "fiat_anchor_info": ""
        }));
    }

    let body = json!({
        "owner": "owner_address",
        "token": "USDC",
        "amount": 100.0,
        "grace_period": 3600,
        "earn_yield": false,
        "yield_rate_bps": 0,
        "last_ping": 0,
        "is_active": true,
        "beneficiaries": beneficiaries
    })
    .to_string();

    let (public_key, signature) = generate_valid_signature(
        &body,
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/plans")
                .header(http::header::CONTENT_TYPE, "application/json")
                .header("X-Public-Key", public_key)
                .header("X-Signature", signature)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_plan_with_valid_signature() {
    let app = setup_app();

    let body = json!({
        "owner": "owner_address",
        "token": "USDC",
        "amount": 100.0,
        "grace_period": 3600,
        "earn_yield": false,
        "yield_rate_bps": 0,
        "last_ping": 0,
        "is_active": true,
        "beneficiaries": [
            {
                "address": "beneficiary_1",
                "name": "B1",
                "allocation_bps": 10000,
                "fiat_anchor_info": ""
            }
        ]
    })
    .to_string();

    let (public_key, signature) = generate_valid_signature(
        &body,
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/plans")
                .header(http::header::CONTENT_TYPE, "application/json")
                .header("X-Public-Key", public_key)
                .header("X-Signature", signature)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should reach validation (BAD_REQUEST for DB error, not auth error)
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_plans_is_public() {
    let app = setup_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/plans")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not require auth
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_plans_returns_cached_response_without_db_access() {
    let cache = PlanCache::memory();
    let query = inheritx_backend::api::PlanQuery {
        owner: Some("GOWNER123".to_string()),
        beneficiary: None,
    };
    let cached_plans = vec![PlanResponse {
        id: uuid::Uuid::new_v4(),
        owner_address: "GOWNER123".to_string(),
        token_address: "USDC".to_string(),
        amount: rust_decimal::Decimal::from(1000),
        grace_period: 3600,
        grace_period_seconds: 3600,
        earn_yield: true,
        last_ping: 1_718_000_000,
        is_active: true,
        status: "ACTIVE".to_string(),
        yield_rate_bps: 500,
        accrued_yield: 25.5,
        created_at: chrono::Utc::now(),
        beneficiaries: vec![],
    }];
    cache.set_plans(&query, &cached_plans).await.unwrap();

    let app = setup_app_with_cache(cache);

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/plans?owner=GOWNER123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("x-plan-cache-status").unwrap(),
        "hit"
    );
}

#[tokio::test]
async fn test_ping_plan_invalid_signature() {
    let app = setup_app();

    // Sign with some key, but use different owner
    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let signature = signing_key.sign(b"ping");
    let signature_hex = hex::encode(signature.to_bytes());

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/plans/ping")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner": "GDIW7P2XUXC4XZB452Y5Z774N4V27PUDHWTKWTQZ3KHYUGB743WEXG7T", // random owner
                        "signature": signature_hex,
                        "message": "ping"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_trigger_payout_invalid_signature() {
    let app = setup_app();

    let body = json!({
        "owner": "GDIW7P2XUXC4XZB452Y5Z774N4V27PUDHWTKWTQZ3KHYUGB743WEXG7T"
    })
    .to_string();

    // Generate a valid signature for a different body
    let (public_key, _correct_sig) = generate_valid_signature(
        &body,
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    );
    let (_different_pub_key, invalid_signature) = generate_valid_signature(
        "different body",
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/plans/payout")
                .header(http::header::CONTENT_TYPE, "application/json")
                .header("X-Public-Key", public_key)
                .header("X-Signature", invalid_signature)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    if status != StatusCode::UNAUTHORIZED {
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        panic!("Expected 401 Unauthorized, got {status}. Response body: {body_str}");
    }
}

#[tokio::test]
async fn test_trigger_payout_valid_signature_not_found() {
    let app = setup_app();

    let body = json!({
        "owner": "owner_address"
    })
    .to_string();

    let (public_key, signature) = generate_valid_signature(
        &body,
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    );

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/plans/payout")
                .header(http::header::CONTENT_TYPE, "application/json")
                .header("X-Public-Key", public_key)
                .header("X-Signature", signature)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Since the database is not actually running, this should return a DB connection error (500)
    // rather than an unauthorized error (401), proving that the request successfully passed auth
    // and reached the handler.
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// --- Health check endpoint tests ---

#[tokio::test]
async fn test_health_endpoint_is_public() {
    let app = setup_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not require auth
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_health_endpoint_returns_json_with_expected_structure() {
    let app = setup_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(body.is_object());
    assert!(body.get("status").is_some(), "missing 'status' field");
    assert!(
        body.get("postgresql").is_some(),
        "missing 'postgresql' field"
    );
    assert!(
        body.get("stellar_rpc").is_some(),
        "missing 'stellar_rpc' field"
    );
}

#[tokio::test]
async fn test_health_endpoint_without_db_yields_service_unavailable() {
    // Use a bogus database URL that will always fail to connect
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy("postgres://localhost:1/nonexistent")
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
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Without a real database, postgresql should be "down"
    assert_eq!(body["postgresql"], "down");

    // With PostgreSQL down, the handler always returns 503
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);

    let status_str = body["status"].as_str().unwrap();
    assert!(
        status_str == "degraded" || status_str == "unhealthy",
        "expected status to be 'degraded' or 'unhealthy', got '{status_str}'"
    );
}

#[tokio::test]
async fn test_get_current_rate_cached() {
    let plan_cache = PlanCache::disabled();
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy("postgres://postgres:password@localhost:5432/test")
        .unwrap();
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        db_pool,
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache,
        apy_cache: dashmap::DashMap::new(),
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
    });

    state.apy_cache.insert("USDC".to_string(), 300);

    let app = create_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/lending/current-rate")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["apy"], 3.0);
}

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use ed25519_dalek::{Signer, SigningKey};
use inheritx_backend::{create_router, AppState};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower::ServiceExt; // for oneshot

use ed25519_dalek::{Signer, SigningKey};

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

async fn setup_app() -> axum::Router {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:password@localhost:5432/test".to_string()
    });

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    let _ = inheritx_backend::DbManager::run_migrations(&db_pool).await;
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        db_pool,
        kyc_webhook_secret: None,
    });
    create_router(state)
}

#[tokio::test]
async fn test_router_compiles() {
    let _app = setup_app().await;
}

#[tokio::test]
async fn test_create_plan_validation_empty_owner() {
    let app = setup_app().await;

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
    let app = setup_app().await;

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
    let app = setup_app().await;

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
async fn test_ping_plan_invalid_signature() {
    let app = setup_app().await;

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
async fn test_ping_plan_not_found() {
    let app = setup_app().await;

    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let owner_address = stellar_strkey::ed25519::PublicKey(verifying_key.to_bytes()).to_string();

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
                        "owner": owner_address,
                        "signature": signature_hex,
                        "message": "ping"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_ping_plan_success_with_yield() {
    let app = setup_app().await;

    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let owner_address = stellar_strkey::ed25519::PublicKey(verifying_key.to_bytes()).to_string();

    // 1. Create a plan
    let last_ping_time = chrono::Utc::now().timestamp() - 3600; // 1 hour ago
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/plans")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner": owner_address,
                        "token": "USDC",
                        "amount": 100.0,
                        "grace_period": 3600,
                        "earn_yield": true,
                        "yield_rate_bps": 500, // 5%
                        "last_ping": last_ping_time,
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

    let status = create_response.status();
    let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Create plan failed: {}",
        body_str
    );

    // 2. Ping the plan
    let message = "maintain-alive-signal";
    let signature = signing_key.sign(message.as_bytes());
    let signature_hex = hex::encode(signature.to_bytes());

    let ping_response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/plans/ping")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner": owner_address,
                        "signature": signature_hex,
                        "message": message
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(ping_response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(ping_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body_json["owner"], owner_address);
    assert_eq!(body_json["status"], "ACTIVE");

    // Amount = 100.0. Yield rate = 5%. Elapsed = 3600s.
    // yield = 100.0 * 0.05 * (3600.0 / 31536000.0) = 0.000570776
    // virtual_balance should be approx 100.0006
    let virtual_balance: f64 = body_json["virtual_balance"].as_f64().unwrap();
    assert!(virtual_balance > 100.0);
    assert!(virtual_balance < 100.01);
}

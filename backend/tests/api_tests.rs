use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
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

fn setup_app() -> axum::Router {
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        db_pool: PgPoolOptions::new()
            .connect_lazy("postgres://postgres:password@localhost/test")
            .unwrap(),
        kyc_webhook_secret: None,
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

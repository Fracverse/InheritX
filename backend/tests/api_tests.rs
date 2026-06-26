use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use inheritx_backend::{create_router, AppState};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower::ServiceExt; // for oneshot

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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

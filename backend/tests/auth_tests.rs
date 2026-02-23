mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn test_get_nonce_returns_unique_nonce() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let wallet = "GABC1234567890UNIQUE";

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/auth/web3/nonce/{}", wallet))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let nonce = body["nonce"].as_str().expect("nonce should be a string");
    assert!(!nonce.is_empty());
}

#[tokio::test]
async fn test_nonce_stored_in_db() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let wallet = "GABC1234567890STORED";

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/auth/web3/nonce/{}", wallet))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let nonce = body["nonce"].as_str().expect("nonce should be a string");

    // Verify in DB
    let stored_nonce: String =
        sqlx::query_scalar("SELECT nonce FROM users WHERE wallet_address = $1")
            .bind(wallet)
            .fetch_one(&test_context.pool)
            .await
            .unwrap();

    assert_eq!(nonce, stored_nonce);
}

#[tokio::test]
async fn test_two_requests_generate_different_nonces() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let wallet = "GABC1234567890DIFFERENT";

    // First request
    let response1 = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/auth/web3/nonce/{}", wallet))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX)
        .await
        .unwrap();
    let body1: Value = serde_json::from_slice(&body1).unwrap();
    let nonce1 = body1["nonce"].as_str().expect("nonce1 should be a string");

    // Second request
    let response2 = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/auth/web3/nonce/{}", wallet))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let body2: Value = serde_json::from_slice(&body2).unwrap();
    let nonce2 = body2["nonce"].as_str().expect("nonce2 should be a string");

    assert_ne!(nonce1, nonce2);
}

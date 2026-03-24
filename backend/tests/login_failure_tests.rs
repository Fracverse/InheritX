mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_login_invalid_signature() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let wallet_address = "GWALLET_INVALID_SIG";

    // 1. Get nonce
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/auth/nonce/{}", wallet_address))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 2. Login with invalid signature
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/wallet-login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "wallet_address": wallet_address,
                        "signature": "invalid_signature"
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
async fn test_login_replayed_nonce() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let wallet_address = "GWALLET_REPLAY";

    // 1. Get nonce
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/auth/nonce/{}", wallet_address))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 2. Login successfully
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/wallet-login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "wallet_address": wallet_address,
                        "signature": "valid_signature"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 3. Attempt to login again with the same "nonce" (which is now cleared)
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/wallet-login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "wallet_address": wallet_address,
                        "signature": "valid_signature"
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
async fn test_login_wrong_wallet_address() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let wallet_address = "GWALLET_NON_EXISTENT";

    // Attempt login without getting a nonce first
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/wallet-login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "wallet_address": wallet_address,
                        "signature": "any_signature"
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
async fn test_login_expired_nonce() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let wallet_address = "GWALLET_EXPIRED";

    // 1. Get nonce
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/auth/nonce/{}", wallet_address))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 2. Manually expire the nonce in DB
    sqlx::query("UPDATE users SET nonce_expires_at = $1 WHERE wallet_address = $2")
        .bind(Utc::now() - Duration::minutes(1))
        .bind(wallet_address)
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 3. Attempt login
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/wallet-login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "wallet_address": wallet_address,
                        "signature": "valid_signature"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

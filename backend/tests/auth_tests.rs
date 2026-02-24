mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::{
    LoginResponse, NonceRequest, NonceResponse, UserClaims, Web3LoginRequest,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use ring::signature::{self, KeyPair};
use serde_json::Value;
use std::convert::TryInto;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceExt;

#[tokio::test]
async fn test_web3_login_success() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // Spawn server
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get addr");
    let app = ctx.app.clone();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("Server failed");
    });

    let client = reqwest::Client::new();
    let base_url = format!("http://{}", addr);

    // 1. Generate a dummy Stellar-like Ed25519 keypair
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();

    // Get public key
    let public_key_bytes = key_pair.public_key().as_ref();
    let wallet_address = stellar_strkey::Strkey::PublicKeyEd25519(
        stellar_strkey::ed25519::PublicKey(public_key_bytes.try_into().unwrap()),
    )
    .to_string()
    .to_string();

    // 2. Request Nonce
    let response = client
        .post(format!("{}/api/auth/nonce", base_url))
        .json(&NonceRequest {
            wallet_address: wallet_address.to_string(),
        })
        .send()
        .await
        .expect("Nonce request failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let nonce_res: NonceResponse = response.json().await.unwrap();
    let nonce = nonce_res.nonce;

    // 3. Sign Nonce
    let signature = key_pair.sign(nonce.as_bytes());
    let signature_hex = hex::encode(signature.as_ref());

    // 4. Web3 Login
    let response = client
        .post(format!("{}/api/auth/web3-login", base_url))
        .json(&Web3LoginRequest {
            wallet_address: wallet_address.to_string(),
            signature: signature_hex,
        })
        .send()
        .await
        .expect("Login request failed");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let login_res: LoginResponse = response.json().await.unwrap();
    let token = login_res.token;

    // 5. Verify JWT
    let decoding_key = DecodingKey::from_secret(b"test-jwt-secret");
    let mut validation = Validation::default();
    validation.validate_exp = false;
    let token_data =
        decode::<UserClaims>(&token, &decoding_key, &validation).expect("JWT decode failed");

    // Find user in DB to check ID
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE wallet_address = $1")
        .bind(wallet_address.as_str())
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(token_data.claims.user_id, user_id);

    // 6. Verify Nonce Invalidated
    let nonce_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM nonces WHERE wallet_address = $1)")
            .bind(wallet_address.as_str())
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

    assert!(!nonce_exists);
}

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
                .uri(format!("/api/auth/nonce/{}", wallet))
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
                .uri(format!("/api/auth/nonce/{}", wallet))
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

    // Verify in DB - nonces table
    let stored_nonce: String =
        sqlx::query_scalar("SELECT nonce FROM nonces WHERE wallet_address = $1")
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
                .uri(format!("/api/auth/nonce/{}", wallet))
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
                .uri(format!("/api/auth/nonce/{}", wallet))
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

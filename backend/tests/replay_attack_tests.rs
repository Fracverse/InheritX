mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::Value;
use tower::ServiceExt;

fn gen_keypair() -> Ed25519KeyPair {
    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("failed to generate keypair");
    Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("failed to parse keypair")
}

#[tokio::test]
async fn wallet_signature_cannot_be_reused() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let keypair = gen_keypair();
    let wallet = hex::encode(keypair.public_key().as_ref());

    let nonce_response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/auth/nonce/{}", wallet))
                .body(Body::empty())
                .expect("failed to build nonce request"),
        )
        .await
        .expect("nonce request failed");

    assert_eq!(nonce_response.status(), StatusCode::OK);

    let nonce_body = axum::body::to_bytes(nonce_response.into_body(), usize::MAX)
        .await
        .expect("failed to read nonce response");
    let nonce_json: Value = serde_json::from_slice(&nonce_body).expect("invalid nonce json");
    let nonce = nonce_json["nonce"]
        .as_str()
        .expect("nonce must be string")
        .to_string();

    let signature = keypair.sign(nonce.as_bytes());
    let signature_hex = hex::encode(signature.as_ref());

    let payload = serde_json::json!({
        "wallet_address": wallet,
        "signature": signature_hex,
    });

    let first_login = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/web3-login")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .expect("failed to build first login request"),
        )
        .await
        .expect("first login request failed");

    assert_eq!(first_login.status(), StatusCode::OK);

    let second_login = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/web3-login")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .expect("failed to build second login request"),
        )
        .await
        .expect("second login request failed");

    assert_eq!(
        second_login.status(),
        StatusCode::UNAUTHORIZED,
        "reusing the same signed nonce must return 401"
    );
}

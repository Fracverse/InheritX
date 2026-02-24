mod helpers;

use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use inheritx_backend::auth::{
    Claims, LoginResponse, NonceRequest, NonceResponse, Web3LoginRequest,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use ring::signature::{self, KeyPair};
use std::convert::TryInto;
use std::net::SocketAddr;
use tokio::net::TcpListener;

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
            wallet_address: wallet_address.clone(),
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
            wallet_address: wallet_address.clone(),
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
        decode::<Claims>(&token, &decoding_key, &validation).expect("JWT decode failed");

    // Find user in DB to check ID
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE wallet_address = $1")
        .bind(wallet_address.as_str())
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(token_data.claims.sub, user_id.to_string());
    assert_eq!(token_data.claims.role, "user");

    // 6. Verify Nonce Invalidated
    let nonce_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM nonces WHERE wallet_address = $1)")
            .bind(wallet_address.as_str())
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

    assert!(!nonce_exists);
}

/// Integration tests: wallet login must reject any signature that does not
/// cryptographically verify against the stored nonce and the wallet's own
/// Ed25519 public key.
///
/// Signing convention (matches `auth.rs`):
///   - wallet_address = hex-encoded Ed25519 public-key bytes
///   - signature      = hex-encoded Ed25519 signature over the UTF-8 nonce
mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::{json, Value};
use tower::ServiceExt;

// ─── helpers ────────────────────────────────────────────────────────────────

/// Generate a fresh Ed25519 key-pair via `ring`.
fn gen_keypair() -> Ed25519KeyPair {
    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap()
}

/// Hex-encode a byte slice.
fn hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Fetch a nonce for `wallet_address` from the API and return the nonce string.
async fn fetch_nonce(app: axum::Router, wallet_address: &str) -> String {
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/auth/nonce/{}", wallet_address))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "expected 200 from nonce endpoint"
    );

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();

    body["nonce"]
        .as_str()
        .expect("nonce field must be a string")
        .to_string()
}

/// Attempt a wallet login and return the HTTP status code.
async fn attempt_login(app: axum::Router, wallet_address: &str, signature: &str) -> StatusCode {
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/wallet-login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "wallet_address": wallet_address,
                        "signature": signature
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    resp.status()
}

// ─── tests ───────────────────────────────────────────────────────────────────

/// Test 1 — Signature produced by a **different** private key is rejected.
///
/// Alice fetches her nonce.  Bob (a different keypair) signs that nonce and
/// tries to log in as Alice.  The backend must reject with 401 because the
/// signature cannot be verified against Alice's public key.
#[tokio::test]
async fn test_signature_from_different_key_is_rejected() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let alice = gen_keypair();
    let bob = gen_keypair();

    let alice_pub = hex(alice.public_key().as_ref());
    let _nonce = fetch_nonce(ctx.app.clone(), &alice_pub).await;

    // Bob signs the nonce – wrong private key for Alice's wallet
    let bad_sig = hex(bob.sign(_nonce.as_bytes()).as_ref());

    let status = attempt_login(ctx.app.clone(), &alice_pub, &bad_sig).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "signature from a different key must be rejected with 401"
    );
}

/// Test 2 — An **empty** signature string is rejected.
///
/// Submitting an empty string as the signature (invalid hex) must result in
/// 401; there is no crypto shortcut an attacker can use.
#[tokio::test]
async fn test_empty_signature_is_rejected() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let keypair = gen_keypair();
    let wallet = hex(keypair.public_key().as_ref());

    let _ = fetch_nonce(ctx.app.clone(), &wallet).await;

    let status = attempt_login(ctx.app.clone(), &wallet, "").await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "empty signature must be rejected with 401"
    );
}

/// Test 3 — A **bit-corrupted** signature (last byte flipped) is rejected.
///
/// A valid signature is produced, then its last byte is changed before
/// submission.  Even a single bit-flip must invalidate the signature.
#[tokio::test]
async fn test_corrupted_signature_is_rejected() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let keypair = gen_keypair();
    let wallet = hex(keypair.public_key().as_ref());

    let nonce = fetch_nonce(ctx.app.clone(), &wallet).await;

    // Produce a real signature …
    let mut sig_bytes: Vec<u8> = keypair.sign(nonce.as_bytes()).as_ref().to_vec();
    // … then corrupt the last byte
    let last = sig_bytes.len() - 1;
    sig_bytes[last] ^= 0xFF;

    let bad_sig = hex(&sig_bytes);
    let status = attempt_login(ctx.app.clone(), &wallet, &bad_sig).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "a bit-flipped signature must be rejected with 401"
    );
}

/// Test 4 — Signing a **different message** (not the stored nonce) is rejected.
///
/// The attacker signs an arbitrary string instead of the nonce the backend
/// stored.  The signature is valid Ed25519, but over the wrong message, so
/// verification against the stored nonce must fail.
#[tokio::test]
async fn test_signature_over_wrong_message_is_rejected() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let keypair = gen_keypair();
    let wallet = hex(keypair.public_key().as_ref());

    // Register the wallet (creates a nonce) but sign a completely different
    // message – simulating a replay of a signature from another session.
    let _ = fetch_nonce(ctx.app.clone(), &wallet).await;

    let forged_sig = hex(keypair.sign(b"this is not the nonce").as_ref());

    let status = attempt_login(ctx.app.clone(), &wallet, &forged_sig).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "signing a different message must be rejected with 401"
    );
}

/// Test 5 — Completely **random bytes** (not a valid Ed25519 signature) are rejected.
///
/// Submitting 64 random bytes hex-encoded as the signature has no possible
/// relationship to the nonce or the public key; the backend must return 401.
#[tokio::test]
async fn test_random_bytes_as_signature_is_rejected() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let keypair = gen_keypair();
    let wallet = hex(keypair.public_key().as_ref());

    let _ = fetch_nonce(ctx.app.clone(), &wallet).await;

    // 64 bytes of predictable "random" data (non-zero pattern).
    let garbage: Vec<u8> = (0u8..64)
        .map(|i| i.wrapping_mul(7).wrapping_add(13))
        .collect();
    let bad_sig = hex(&garbage);

    let status = attempt_login(ctx.app.clone(), &wallet, &bad_sig).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "random bytes submitted as signature must be rejected with 401"
    );
}

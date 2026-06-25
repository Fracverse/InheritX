use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tower::ServiceExt;

type HmacSha256 = Hmac<Sha256>;

fn sign_payload(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

fn valid_payload() -> &'static str {
    r#"{"wallet_address":"GDTEST123","status":"approved","event_type":"kyc.status_update","provider_reference":"ref-001"}"#
}

fn test_state() -> std::sync::Arc<inheritx_backend::AppState> {
    use inheritx_backend::stellar_anchor::AnchorRegistry;
    let pool = sqlx::PgPool::connect_lazy(
        "postgres://postgres:postgres@localhost:5432/inheritx_test"
    ).unwrap();
    let (kyc_tx, _) = tokio::sync::broadcast::channel(16);
    std::sync::Arc::new(inheritx_backend::AppState {
        anchor: std::sync::Arc::new(AnchorRegistry::new()),
        db_pool: pool,
        kyc_tx,
    })
}

#[tokio::test]
async fn test_webhook_rejects_invalid_signature() {
    std::env::set_var("KYC_WEBHOOK_SECRET", "test-secret");
    let app = inheritx_backend::create_router(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/webhook")
                .header("content-type", "application/json")
                .header("x-kyc-signature", "sha256=invalidsignature")
                .body(Body::from(valid_payload()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    std::env::remove_var("KYC_WEBHOOK_SECRET");
}

#[tokio::test]
async fn test_webhook_rejects_invalid_json() {
    // No secret set — signature check skipped, parse check runs
    std::env::remove_var("KYC_WEBHOOK_SECRET");
    let app = inheritx_backend::create_router(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/webhook")
                .header("content-type", "application/json")
                .body(Body::from("not valid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_valid_signature_accepted() {
    let secret = "test-secret-2";
    let body = valid_payload();
    let sig = sign_payload(secret, body.as_bytes());

    std::env::set_var("KYC_WEBHOOK_SECRET", secret);
    let app = inheritx_backend::create_router(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/webhook")
                .header("content-type", "application/json")
                .header("x-kyc-signature", sig)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Signature valid — not 401
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    std::env::remove_var("KYC_WEBHOOK_SECRET");
}

#[tokio::test]
async fn test_webhook_no_secret_skips_signature_check() {
    std::env::remove_var("KYC_WEBHOOK_SECRET");
    let app = inheritx_backend::create_router(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/webhook")
                .header("content-type", "application/json")
                .body(Body::from(valid_payload()))
                .unwrap(),
        )
        .await
        .unwrap();

    // No secret — signature check skipped, not 401
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

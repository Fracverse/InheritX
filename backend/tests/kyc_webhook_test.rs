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

fn test_state(secret: Option<&str>) -> std::sync::Arc<inheritx_backend::AppState> {
    use inheritx_backend::stellar_anchor::AnchorRegistry;

    let (kyc_tx, _) = tokio::sync::broadcast::channel(100);
    let pool =
        sqlx::PgPool::connect_lazy("postgres://postgres:postgres@localhost:5432/inheritx_test")
            .unwrap();

    std::sync::Arc::new(inheritx_backend::AppState {
        anchor: std::sync::Arc::new(AnchorRegistry::new()),
        db_pool: pool,
        kyc_webhook_secret: secret.map(str::to_string),
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: inheritx_backend::PlanCache::disabled(),
        apy_cache: dashmap::DashMap::new(),
        kyc_tx,
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
    })
}
#[tokio::test]
async fn test_webhook_rejects_invalid_signature() {
    let app = inheritx_backend::create_router(test_state(Some("test-secret")));
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
}

#[tokio::test]
async fn test_webhook_rejects_missing_signature_header() {
    let app = inheritx_backend::create_router(test_state(Some("test-secret")));
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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_webhook_rejects_signature_for_a_different_body() {
    // Signature is valid, but for a payload other than the one sent.
    let secret = "test-secret";
    let sig = sign_payload(
        secret,
        br#"{"wallet_address":"GDOTHER","status":"rejected"}"#,
    );

    let app = inheritx_backend::create_router(test_state(Some(secret)));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/webhook")
                .header("content-type", "application/json")
                .header("x-kyc-signature", sig)
                .body(Body::from(valid_payload()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_webhook_rejects_invalid_json() {
    // Correctly signed, so the request gets past auth and fails on parsing.
    let secret = "test-secret";
    let body = "not valid json";
    let sig = sign_payload(secret, body.as_bytes());

    let app = inheritx_backend::create_router(test_state(Some(secret)));
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_valid_signature_accepted() {
    let secret = "test-secret-2";
    let body = valid_payload();
    let sig = sign_payload(secret, body.as_bytes());

    let app = inheritx_backend::create_router(test_state(Some(secret)));
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

    // Signature valid — request is authenticated and reaches the handler body.
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_webhook_fails_closed_when_secret_not_configured() {
    // Without a configured secret nothing can be verified, so the endpoint must
    // reject rather than accept unauthenticated KYC updates.
    let body = valid_payload();
    let app = inheritx_backend::create_router(test_state(None));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/webhook")
                .header("content-type", "application/json")
                .header(
                    "x-kyc-signature",
                    sign_payload("any-secret", body.as_bytes()),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

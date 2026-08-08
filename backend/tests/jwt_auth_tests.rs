use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use inheritx_backend::auth::Claims;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::Duration;
use tower::ServiceExt;

const JWT_SECRET: &str = "test-jwt-secret-for-testing";

fn ensure_jwt_secret() {
    std::env::set_var("JWT_SECRET", JWT_SECRET);
}

fn setup_app() -> axum::Router {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/test".to_string());
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy(&database_url)
        .unwrap();
    let state = std::sync::Arc::new(inheritx_backend::AppState {
        anchor: std::sync::Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        db_pool,
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: inheritx_backend::PlanCache::disabled(),
        apy_cache: dashmap::DashMap::new(),
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
    });
    inheritx_backend::create_router(state)
}

fn plan_report_uri() -> String {
    format!("/api/plans/{}/report", uuid::Uuid::nil())
}

fn generate_token(role: &str, secret: &str) -> String {
    let claims = Claims {
        sub: "test-admin-id".to_string(),
        role: role.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

#[tokio::test]
async fn test_jwt_missing_authorization_header() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&plan_report_uri())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_jwt_invalid_header_format() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&plan_report_uri())
                .header("Authorization", "NotBearer token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_jwt_empty_bearer_token() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&plan_report_uri())
                .header("Authorization", "Bearer ")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_jwt_invalid_token_payload() {
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&plan_report_uri())
                .header("Authorization", "Bearer invalid.jwt.token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_jwt_valid_token_with_non_admin_role() {
    ensure_jwt_secret();
    let token = generate_token("user", JWT_SECRET);
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&plan_report_uri())
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_jwt_valid_admin_token_passes_middleware() {
    ensure_jwt_secret();
    let token = generate_token("admin", JWT_SECRET);
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&plan_report_uri())
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "JWT middleware should have passed for valid admin token"
    );
}

#[tokio::test]
async fn test_jwt_token_signed_with_wrong_secret_rejected() {
    ensure_jwt_secret();
    let token = generate_token("admin", "some-other-secret");
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&plan_report_uri())
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_jwt_expired_token_rejected() {
    ensure_jwt_secret();
    let claims = Claims {
        sub: "test-admin-id".to_string(),
        role: "admin".to_string(),
        exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize,
    };
    let token = encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .unwrap();
    let app = setup_app();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&plan_report_uri())
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

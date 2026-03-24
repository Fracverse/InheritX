mod helpers;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

/// The secret must match what TestContext injects into the Config.
/// helpers::TestContext defaults to "test-jwt-secret" when JWT_SECRET is not set.
const JWT_SECRET: &[u8] = b"test-jwt-secret";

fn user_token(user_id: Uuid) -> String {
    let claims = UserClaims {
        user_id,
        email: format!("user-{}@example.com", user_id),
        // Use a far-future timestamp so the token never expires in tests
        exp: 9_999_999_999,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .expect("token encoding failed")
}

// ---------------------------------------------------------------------------
// Test 1 – Authenticated user can submit KYC and receives a pending record
// ---------------------------------------------------------------------------
#[tokio::test]
async fn submit_kyc_returns_pending_for_authenticated_user() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("kyc-submit-{}@example.com", user_id))
        .bind("hashed_password")
        .execute(&ctx.pool)
        .await
        .expect("failed to seed user");

    let token = user_token(user_id);

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/submit")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body["user_id"], user_id.to_string());
    assert_eq!(body["status"], "pending");
}

// ---------------------------------------------------------------------------
// Test 2 – Unauthenticated request is rejected with 401
// ---------------------------------------------------------------------------
#[tokio::test]
async fn submit_kyc_without_token_returns_unauthorized() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/submit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Test 3 – Malformed / invalid token returns 401
// ---------------------------------------------------------------------------
#[tokio::test]
async fn submit_kyc_with_invalid_token_returns_unauthorized() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/submit")
                .header(header::AUTHORIZATION, "Bearer totally.invalid.token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Test 4 – Submitting KYC a second time is idempotent (returns existing record)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn submit_kyc_is_idempotent_for_same_user() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("kyc-idem-{}@example.com", user_id))
        .bind("hashed_password")
        .execute(&ctx.pool)
        .await
        .expect("failed to seed user");

    let token = user_token(user_id);

    // First submission
    let _ = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/submit")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Second submission – must still return 200 with pending status
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/submit")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body["user_id"], user_id.to_string());
    assert_eq!(body["status"], "pending");
}

// ---------------------------------------------------------------------------
// Test 5 – Response body contains expected KYC fields
// ---------------------------------------------------------------------------
#[tokio::test]
async fn submit_kyc_response_contains_expected_fields() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("kyc-fields-{}@example.com", user_id))
        .bind("hashed_password")
        .execute(&ctx.pool)
        .await
        .expect("failed to seed user");

    let token = user_token(user_id);

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/kyc/submit")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // All KycRecord fields must be present in the response
    assert!(body.get("user_id").is_some(), "missing field: user_id");
    assert!(body.get("status").is_some(), "missing field: status");
    assert!(
        body.get("created_at").is_some(),
        "missing field: created_at"
    );

    // reviewed_by and reviewed_at should be null on a fresh submission
    assert!(
        body["reviewed_by"].is_null(),
        "reviewed_by should be null on submit"
    );
    assert!(
        body["reviewed_at"].is_null(),
        "reviewed_at should be null on submit"
    );
}

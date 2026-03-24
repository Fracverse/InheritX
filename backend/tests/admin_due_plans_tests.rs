mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::{AdminClaims, UserClaims};
use jsonwebtoken::{encode, EncodingKey, Header};
use tower::ServiceExt; // for `oneshot`

pub fn generate_admin_token() -> String {
    let admin_id = uuid::Uuid::new_v4();
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = AdminClaims {
        admin_id,
        email: "admin@inheritx.test".to_string(),
        role: "admin".to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-jwt-secret"),
    )
    .unwrap()
}

pub fn generate_user_token() -> String {
    let user_id = uuid::Uuid::new_v4();
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = UserClaims {
        user_id,
        email: "user@inheritx.test".to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-jwt-secret"),
    )
    .unwrap()
}

#[tokio::test]
async fn admin_can_get_all_due_plans_returns_200() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_admin_token();

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/api/admin/plans/due-for-claim")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/plans/due-for-claim failed");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn user_token_rejected_returns_401_or_403() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_user_token();

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/api/admin/plans/due-for-claim")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/plans/due-for-claim failed");

    let status = response.status();
    assert!(status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN);
}

mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::{AdminClaims, UserClaims};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

fn generate_admin_token(admin_id: Uuid) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = AdminClaims {
        admin_id,
        email: format!("admin-{}@example.com", admin_id),
        role: "admin".to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-jwt-secret"),
    )
    .expect("Failed to generate admin token")
}

fn generate_user_token(user_id: Uuid) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = UserClaims {
        user_id,
        email: format!("user-{}@example.com", user_id),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-jwt-secret"),
    )
    .expect("Failed to generate user token")
}

#[tokio::test]
async fn admin_can_fetch_metrics_overview() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_admin_token(Uuid::new_v4());

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/overview")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert!(json.get("totalRevenue").is_some(), "missing totalRevenue");
    assert!(json.get("totalPlans").is_some(), "missing totalPlans");
    assert!(json.get("totalClaims").is_some(), "missing totalClaims");
    assert!(json.get("activePlans").is_some(), "missing activePlans");
    assert!(json.get("totalUsers").is_some(), "missing totalUsers");
}

#[tokio::test]
async fn user_cannot_fetch_metrics_overview() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_user_token(Uuid::new_v4());

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/overview")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unauthenticated_cannot_fetch_metrics_overview() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/overview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

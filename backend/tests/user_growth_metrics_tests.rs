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

fn build_get_request(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn admin_can_fetch_user_growth_metrics() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_admin_token(Uuid::new_v4());

    let response = ctx
        .app
        .oneshot(build_get_request("/admin/metrics/users", &token))
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert_eq!(json["status"], "success");
    let data = &json["data"];
    assert!(data.get("totalUsers").is_some(), "missing totalUsers");
    assert!(
        data.get("newUsersLast7Days").is_some(),
        "missing newUsersLast7Days"
    );
    assert!(
        data.get("newUsersLast30Days").is_some(),
        "missing newUsersLast30Days"
    );
    assert!(data.get("activeUsers").is_some(), "missing activeUsers");
}

#[tokio::test]
async fn user_growth_metrics_returns_correct_types() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_admin_token(Uuid::new_v4());

    let response = ctx
        .app
        .oneshot(build_get_request("/admin/metrics/users", &token))
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Response is not valid JSON");

    let data = &json["data"];
    assert!(
        data["totalUsers"].is_number(),
        "totalUsers should be a number"
    );
    assert!(
        data["newUsersLast7Days"].is_number(),
        "newUsersLast7Days should be a number"
    );
    assert!(
        data["newUsersLast30Days"].is_number(),
        "newUsersLast30Days should be a number"
    );
    assert!(
        data["activeUsers"].is_number(),
        "activeUsers should be a number"
    );
}

#[tokio::test]
async fn regular_user_cannot_fetch_user_growth_metrics() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_user_token(Uuid::new_v4());

    let response = ctx
        .app
        .oneshot(build_get_request("/admin/metrics/users", &token))
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unauthenticated_cannot_fetch_user_growth_metrics() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

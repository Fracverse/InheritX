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
async fn admin_can_fetch_revenue_metrics() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_admin_token(Uuid::new_v4());

    // Test default (daily)
    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/revenue")
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

    assert_eq!(json.get("range").and_then(|v| v.as_str()), Some("daily"));
    assert!(json.get("data").and_then(|v| v.as_array()).is_some());

    // Test weekly
    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/revenue?range=weekly")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.get("range").and_then(|v| v.as_str()), Some("weekly"));

    // Test monthly
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/revenue?range=monthly")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.get("range").and_then(|v| v.as_str()), Some("monthly"));
}

#[tokio::test]
async fn user_cannot_fetch_revenue_metrics() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_user_token(Uuid::new_v4());

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/revenue")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_metrics_revenue_invalid_range() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_admin_token(Uuid::new_v4());

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/revenue?range=yearly")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

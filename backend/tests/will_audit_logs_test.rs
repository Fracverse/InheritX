/*
//! # Legal Will Audit Logs Tests
//!
//! Comprehensive tests for the audit logging system

mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

// Helper to create a test user and get auth token
async fn create_test_user(pool: &sqlx::PgPool, email: &str) -> (Uuid, String) {
    let user_id = Uuid::new_v4();
    let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(email)
    .bind(&password_hash)
    .execute(pool)
    .await
    .unwrap();

    // Generate JWT token
    let claims = inheritx_backend::auth::UserClaims {
        user_id,
        email: email.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test-jwt-secret".as_bytes()),
    )
    .unwrap();

    (user_id, token)
}

async fn create_test_admin(pool: &sqlx::PgPool, email: &str) -> (Uuid, String) {
    let admin_id = Uuid::new_v4();
    let password_hash = bcrypt::hash("admin123", bcrypt::DEFAULT_COST).unwrap();

    sqlx::query(
        "INSERT INTO admins (id, email, password_hash, role, status) VALUES ($1, $2, $3, 'super_admin', 'active') ON CONFLICT DO NOTHING",
    )
    .bind(admin_id)
    .bind(email)
    .bind(&password_hash)
    .execute(pool)
    .await
    .unwrap();

    // Generate JWT token
    let claims = inheritx_backend::auth::AdminClaims {
        admin_id,
        email: email.to_string(),
        role: "super_admin".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test-jwt-secret".as_bytes()),
    )
    .unwrap();

    (admin_id, token)
}

#[tokio::test]
async fn test_audit_log_created_on_document_generation() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (user_id, token) = create_test_user(&ctx.pool, "owner@test.com").await;

    // Create a plan
    let plan_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO plans (id, user_id, vault_id, asset_code, amount, inactivity_period_days, status)
         VALUES ($1, $2, 'vault-001', 'USDC', 1000, 90, 'active')",
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Generate a will document
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/plans/{}/will/generate", plan_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "owner_name": "Alice Testator",
                "owner_wallet": "GABC1234567890ABCDEF",
                "vault_id": "vault-001",
                "beneficiaries": [{
                    "name": "Bob Beneficiary",
                    "wallet_address": "GBOB1234567890ABCDEF",
                    "allocation_percent": "100.0000"
                }],
                "template": "formal"
            })
            .to_string(),
        ))
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify audit log was created
    let event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM will_event_log WHERE plan_id = $1 AND event_type = 'will_created'",
    )
    .bind(plan_id)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();

    assert_eq!(event_count, 1);
}

#[tokio::test]
async fn test_admin_can_get_audit_logs() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (_admin_id, admin_token) = create_test_admin(&ctx.pool, "admin@test.com").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/will/audit/logs?limit=10")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn test_admin_can_get_audit_statistics() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (_admin_id, admin_token) = create_test_admin(&ctx.pool, "admin@test.com").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/will/audit/statistics")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert!(json["data"]["total_events"].is_number());
    assert!(json["data"]["event_type_distribution"].is_array());
}

#[tokio::test]
async fn test_admin_can_get_event_types() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (_admin_id, admin_token) = create_test_admin(&ctx.pool, "admin@test.com").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/will/audit/event-types")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn test_user_can_get_plan_audit_summary() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (user_id, token) = create_test_user(&ctx.pool, "owner@test.com").await;

    // Create a plan
    let plan_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO plans (id, user_id, vault_id, asset_code, amount, inactivity_period_days, status)
         VALUES ($1, $2, 'vault-001', 'USDC', 1000, 90, 'active')",
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/will/audit/plan/{}/summary", plan_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert!(json["data"]["total_events"].is_number());
}

#[tokio::test]
async fn test_user_can_get_own_activity() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (_user_id, token) = create_test_user(&ctx.pool, "owner@test.com").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/will/audit/my-activity")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert!(json["data"]["total_actions"].is_number());
}

#[tokio::test]
async fn test_audit_logs_require_authentication() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/will/audit/logs")
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_endpoints_require_admin_role() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (_user_id, user_token) = create_test_user(&ctx.pool, "user@test.com").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/will/audit/logs")
        .header("Authorization", format!("Bearer {}", user_token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_audit_log_filters_work() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (_admin_id, admin_token) = create_test_admin(&ctx.pool, "admin@test.com").await;

    // Test with event_type filter
    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/will/audit/logs?event_type=will_created&limit=5")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn test_admin_can_search_audit_logs() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let (_admin_id, admin_token) = create_test_admin(&ctx.pool, "admin@test.com").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/will/audit/search?q=will&limit=10")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert!(json["data"].is_array());
}
*/

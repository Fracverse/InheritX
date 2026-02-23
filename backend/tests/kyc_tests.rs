mod helpers;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use inheritx_backend::auth::{AdminClaims, UserClaims};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &[u8] = b"secret_key_change_in_production";

fn create_token<T: serde::Serialize>(claims: &T) -> String {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .unwrap()
}

#[tokio::test]
async fn admin_can_approve_kyc() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Seed admin
    let admin_id = Uuid::new_v4();
    sqlx::query("INSERT INTO admins (id, email, password_hash, role) VALUES ($1, $2, $3, $4)")
        .bind(admin_id)
        .bind(format!("admin-{}@example.com", Uuid::new_v4()))
        .bind("$2b$12$6/G8N8zT.E1.F0P8X2.y.e6.E1.F0P8X2.y.e6.E1.F0P8X2") // valid-ish hash
        .bind("super_admin")
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 2. Seed user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", Uuid::new_v4()))
        .bind("hash")
        .execute(&test_context.pool)
        .await
        .unwrap();

    let admin_claims = AdminClaims {
        admin_id,
        email: "admin@example.com".to_string(),
        role: "super_admin".to_string(),
        exp: 0,
    };
    let token = create_token(&admin_claims);

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/api/admin/kyc/approve")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({ "user_id": user_id }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify DB state
    // Use kyc_status table as defined in migrations
    let status: String = sqlx::query_scalar("SELECT status FROM kyc_status WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&test_context.pool)
        .await
        .unwrap();
    assert_eq!(status, "approved");
}

#[tokio::test]
async fn user_cannot_approve_kyc() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let user_claims = UserClaims {
        user_id,
        email: "user@example.com".to_string(),
        exp: 0,
    };
    let token = create_token(&user_claims);

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/api/admin/kyc/approve")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({ "user_id": Uuid::new_v4() }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // The AuthenticatedAdmin extractor will fail to decode UserClaims into AdminClaims
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn invalid_uuid_rejected() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let admin_id = Uuid::new_v4();
    let admin_claims = AdminClaims {
        admin_id,
        email: "admin@example.com".to_string(),
        role: "super_admin".to_string(),
        exp: 0,
    };
    let token = create_token(&admin_claims);

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/api/admin/kyc/approve")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({ "user_id": "not-a-uuid" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Axum's Json extractor returns 422 Unprocessable Entity for malformed body content
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

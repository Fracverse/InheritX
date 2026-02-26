mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::{Send2faRequest, Verify2faRequest};
use chrono::{Duration, Utc};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_2fa_full_flow() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Create a test user
    let user_id = Uuid::new_v4();
    let email = format!("test-{}@example.com", user_id);
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&email)
        .bind("dummy-hash")
        .execute(&ctx.pool)
        .await
        .unwrap();

    // 2. Request 2FA
    let response = ctx.app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user/send-2fa")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&Send2faRequest { user_id }).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 3. Manually update OTP to a known one for verification
    let otp = "123456";
    let otp_hash = bcrypt::hash(otp, bcrypt::DEFAULT_COST).unwrap();
    sqlx::query("UPDATE user_2fa SET otp_hash = $1 WHERE user_id = $2")
        .bind(otp_hash)
        .bind(user_id)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // 4. Verify 2FA
    let response = ctx.app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user/verify-2fa")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&Verify2faRequest { 
                    user_id, 
                    otp: otp.to_string() 
                }).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Verify record is deleted
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_2fa WHERE user_id = $1)")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn test_verify_2fa_invalid_otp() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .bind("dummy-hash")
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Send 2FA
    ctx.app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user/send-2fa")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&Send2faRequest { user_id }).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Verify with WRONG OTP
    let response = ctx.app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user/verify-2fa")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&Verify2faRequest { 
                    user_id, 
                    otp: "000000".to_string() 
                }).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Verify attempts incremented
    let attempts: i32 = sqlx::query_scalar("SELECT attempts FROM user_2fa WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(attempts, 1);
}

#[tokio::test]
async fn test_verify_2fa_too_many_attempts() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .bind("dummy-hash")
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Set 3 attempts in DB
    let expires_at = Utc::now() + Duration::minutes(5);
    sqlx::query("INSERT INTO user_2fa (user_id, otp_hash, expires_at, attempts) VALUES ($1, $2, $3, 3)")
        .bind(user_id)
        .bind("some-hash")
        .bind(expires_at)
        .execute(&ctx.pool)
        .await
        .unwrap();

    let response = ctx.app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user/verify-2fa")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&Verify2faRequest { 
                    user_id, 
                    otp: "123456".to_string() 
                }).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert!(body["message"].as_str().unwrap().contains("Too many verification attempts"));
}

#[tokio::test]
async fn test_verify_2fa_expired() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .bind("dummy-hash")
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Set expired OTP in DB
    let expires_at = Utc::now() - Duration::minutes(1);
    sqlx::query("INSERT INTO user_2fa (user_id, otp_hash, expires_at, attempts) VALUES ($1, $2, $3, 0)")
        .bind(user_id)
        .bind("some-hash")
        .bind(expires_at)
        .execute(&ctx.pool)
        .await
        .unwrap();

    let response = ctx.app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user/verify-2fa")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&Verify2faRequest { 
                    user_id, 
                    otp: "123456".to_string() 
                }).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert!(body["message"].as_str().unwrap().contains("expired"));
}

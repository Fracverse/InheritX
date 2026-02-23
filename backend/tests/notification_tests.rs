// This file contains tests for notification functionality.
mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn mark_notification_read_success() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Create a user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .bind("hash")
        .execute(&ctx.pool)
        .await
        .expect("Failed to create user");

    // 2. Create a notification
    let notif_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO notifications (id, user_id, title, message, is_read) VALUES ($1, $2, $3, $4, false)",
    )
    .bind(notif_id)
    .bind(user_id)
    .bind("Test Notif")
    .bind("Hello")
    .execute(&ctx.pool)
    .await
    .expect("Failed to create notification");

    // 3. Generate token
    let claims = UserClaims {
        user_id,
        email: format!("test-{}@example.com", user_id),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate token");

    // 4. Call mark read endpoint
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/notifications/{}/read", notif_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Verify in DB
    let is_read: bool = sqlx::query_scalar("SELECT is_read FROM notifications WHERE id = $1")
        .bind(notif_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to fetch notification");

    assert!(is_read);
}

#[tokio::test]
async fn cannot_mark_another_user_notification() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Create two users
    let user_a_id = Uuid::new_v4();
    let user_b_id = Uuid::new_v4();

    // FIX: destructure with `&id` to avoid double-reference (&&Uuid) from iterating &[...]
    for &id in &[user_a_id, user_b_id] {
        sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(format!("test-{}@example.com", id))
            .bind("hash")
            .execute(&ctx.pool)
            .await
            .expect("Failed to create user");
    }

    // 2. Create a notification for user B
    let notif_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO notifications (id, user_id, title, message, is_read) VALUES ($1, $2, $3, $4, false)",
    )
    .bind(notif_id)
    .bind(user_b_id)
    .bind("User B Notif")
    .bind("Hello B")
    .execute(&ctx.pool)
    .await
    .expect("Failed to create notification");

    // 3. Generate token for user A
    let claims = UserClaims {
        user_id: user_a_id,
        email: format!("test-{}@example.com", user_a_id),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate token");

    // 4. Call mark read endpoint for user B's notification using user A's token
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/notifications/{}/read", notif_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // Should return 404 — service filters by user_id in UPDATE, so no rows match
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 5. Verify notification is still unread in DB
    let is_read: bool = sqlx::query_scalar("SELECT is_read FROM notifications WHERE id = $1")
        .bind(notif_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to fetch notification");

    assert!(!is_read);
}

#[tokio::test]
async fn mark_already_read_notification_safe_handling() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Create a user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .bind("hash")
        .execute(&ctx.pool)
        .await
        .expect("Failed to create user");

    // 2. Create an already-read notification
    let notif_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO notifications (id, user_id, title, message, is_read) VALUES ($1, $2, $3, $4, true)",
    )
    .bind(notif_id)
    .bind(user_id)
    .bind("Already Read Notif")
    .bind("Hello")
    .execute(&ctx.pool)
    .await
    .expect("Failed to create notification");

    // 3. Generate token
    let claims = UserClaims {
        user_id,
        email: format!("test-{}@example.com", user_id),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate token");

    // 4. Call mark read endpoint again — should be idempotent
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/notifications/{}/read", notif_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Verify it's still read
    let is_read: bool = sqlx::query_scalar("SELECT is_read FROM notifications WHERE id = $1")
        .bind(notif_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to fetch notification");

    assert!(is_read);
}

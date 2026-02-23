// This file contains tests for notification functionality.
mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use tower::ServiceExt;
use uuid::Uuid;

/// Generate a JWT token for a test user
fn generate_user_token(user_id: Uuid, email: String) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = UserClaims {
        user_id,
        email,
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate token")
}

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

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    // 3. Generate token
    let claims = UserClaims {
        user_id,
        email: format!("test-{}@example.com", user_id),
        exp: expiration as usize,
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
async fn test_retrieve_notifications_returns_only_users_notifications() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Create two users
    let user_a_id = Uuid::new_v4();
    let user_b_id = Uuid::new_v4();

    for &id in &[user_a_id, user_b_id] {
        sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(format!("test-{}@example.com", id))
            .bind("hash")
            .execute(&ctx.pool)
            .await
            .expect("Failed to create user");
    }

    // 2. Create notifications for user A
    for i in 0..3 {
        let notif_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO notifications (id, user_id, type, message, is_read) VALUES ($1, $2, $3, $4, false)",
        )
        .bind(notif_id)
        .bind(user_a_id)
        .bind("plan_created")
        .bind(format!("User A notification {}", i))
        .execute(&ctx.pool)
        .await
        .expect("Failed to create notification");
    }

    // 3. Create notifications for user B
    for i in 0..2 {
        let notif_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO notifications (id, user_id, type, message, is_read) VALUES ($1, $2, $3, $4, false)",
        )
        .bind(notif_id)
        .bind(user_b_id)
        .bind("plan_created")
        .bind(format!("User B notification {}", i))
        .execute(&ctx.pool)
        .await
        .expect("Failed to create notification");
    }

    // 4. Generate token for user A
    let token = generate_user_token(user_a_id, format!("test-{}@example.com", user_a_id));

    // 5. Call list notifications endpoint
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/notifications")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // 6. Parse response
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    // 7. Verify only user A's notifications are returned
    assert_eq!(json["status"], "success");
    let notifications = json["data"].as_array().expect("data should be an array");
    assert_eq!(
        notifications.len(),
        3,
        "Should return exactly 3 notifications for user A"
    );

    // 8. Verify all notifications belong to user A
    for notif in notifications {
        assert_eq!(
            notif["user_id"].as_str().unwrap(),
            user_a_id.to_string(),
            "All notifications should belong to user A"
        );
    }
}

#[tokio::test]
async fn test_retrieve_notifications_count_matches() {
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

    // 2. Create multiple notifications
    let notification_count = 5;
    for i in 0..notification_count {
        let notif_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO notifications (id, user_id, type, message, is_read) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(notif_id)
        .bind(user_id)
        .bind("plan_created")
        .bind(format!("Notification {}", i))
        .bind(i % 2 == 0) // Some read, some unread
        .execute(&ctx.pool)
        .await
        .expect("Failed to create notification");
    }

    // 3. Generate token
    let token = generate_user_token(user_id, format!("test-{}@example.com", user_id));

    // 4. Call list notifications endpoint
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/notifications")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Parse response
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    // 6. Verify count matches
    assert_eq!(json["status"], "success");
    let notifications = json["data"].as_array().expect("data should be an array");
    let count = json["count"].as_u64().expect("count should be a number");

    assert_eq!(
        notifications.len() as u64,
        count,
        "Count field should match the number of notifications in data array"
    );
    assert_eq!(
        count, notification_count,
        "Should return all {} notifications",
        notification_count
    );
}

#[tokio::test]
async fn test_retrieve_notifications_empty_list() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Create a user with no notifications
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .bind("hash")
        .execute(&ctx.pool)
        .await
        .expect("Failed to create user");

    // 2. Generate token
    let token = generate_user_token(user_id, format!("test-{}@example.com", user_id));

    // 3. Call list notifications endpoint
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/notifications")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // 4. Parse response
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    // 5. Verify empty list
    assert_eq!(json["status"], "success");
    let notifications = json["data"].as_array().expect("data should be an array");
    let count = json["count"].as_u64().expect("count should be a number");

    assert_eq!(notifications.len(), 0, "Should return empty array");
    assert_eq!(count, 0, "Count should be 0");
}

#[tokio::test]
async fn test_retrieve_notifications_without_auth() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // Call list notifications endpoint without auth token
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/notifications")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_retrieve_notifications_ordered_by_created_at() {
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

    // 2. Create notifications with slight delays to ensure different timestamps
    let mut notification_ids = Vec::new();
    for i in 0..3 {
        let notif_id = Uuid::new_v4();
        notification_ids.push(notif_id);
        sqlx::query(
            "INSERT INTO notifications (id, user_id, type, message, is_read) VALUES ($1, $2, $3, $4, false)",
        )
        .bind(notif_id)
        .bind(user_id)
        .bind("plan_created")
        .bind(format!("Notification {}", i))
        .execute(&ctx.pool)
        .await
        .expect("Failed to create notification");

        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // 3. Generate token
    let token = generate_user_token(user_id, format!("test-{}@example.com", user_id));

    // 4. Call list notifications endpoint
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/notifications")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Parse response
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    // 6. Verify notifications are ordered by created_at DESC (newest first)
    let notifications = json["data"].as_array().expect("data should be an array");
    assert_eq!(notifications.len(), 3);

    // The newest notification should be first (last one created)
    assert_eq!(
        notifications[0]["id"].as_str().unwrap(),
        notification_ids[2].to_string(),
        "Newest notification should be first"
    );
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

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    // 3. Generate token for user A
    let claims = UserClaims {
        user_id: user_a_id,
        email: format!("test-{}@example.com", user_a_id),
        exp: expiration as usize,
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

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    // 3. Generate token
    let claims = UserClaims {
        user_id,
        email: format!("test-{}@example.com", user_id),
        exp: expiration as usize,
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

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
    sqlx::query("INSERT INTO notifications (id, user_id, title, message, is_read) VALUES ($1, $2, $3, $4, false)")
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
    let response = ctx.app
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

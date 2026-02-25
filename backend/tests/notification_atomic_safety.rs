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

#[tokio::test]
async fn test_update_kyc_rollback_on_notification_failure() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Setup: Create a user and an admin
    let user_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let user_email = format!("user-{}@example.com", user_id);
    let admin_email = format!("admin-{}@example.com", admin_id);

    // Insert user
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&user_email)
        .bind("hash")
        .execute(&ctx.pool)
        .await
        .expect("Failed to create user");

    // 2. Generate Admin Token (assuming KYC updates require admin auth)

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .unwrap()
        .timestamp() as usize;
    let claims = UserClaims {
        user_id: admin_id,
        email: admin_email,
        exp: expiration,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate token");

    // 3. Prepare Payload with "Malicious" data
    // We force the notification to fail by sending a status that results in a
    // message/type string exceeding the database column limits for the
    // 'notifications' table (e.g., if notifications.message is VARCHAR(255)).
    let oversized_reason = "F".repeat(500);

    let payload = serde_json::json!({
        "status": "rejected",
        "reason": oversized_reason
    });

    // 4. Dispatch Request to the KYC update endpoint
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/users/{}/kyc", user_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // 5. Assert: The endpoint should fail (500) because notification insert failed
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // 6. THE ATOMIC CHECK: Verify the kyc_status record was NOT created/updated
    // If the notification failed, the KYC update should have rolled back.
    let kyc_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM kyc_status WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("Failed to query database");

    assert!(
        !kyc_exists,
        "ATOMIC SAFETY FAILURE: KYC status was updated even though notification failed!"
    );
}

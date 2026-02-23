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
async fn test_create_plan_rollback_on_audit_failure() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Setup: Create a user
    let user_id = Uuid::new_v4();
    let email = format!("safety-{}@example.com", user_id);
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&email)
        .bind("hash")
        .execute(&ctx.pool)
        .await
        .expect("Failed to create user");

    // 2. Generate token
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .unwrap()
        .timestamp() as usize;

    let claims = UserClaims {
        user_id,
        email,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate token");

    // 3. Prepare Payload
    // We trigger a failure by sending a "title" that is valid for the 'plans' table
    // but we will assume your 'action_logs' table has a constraint (e.g. max 50 chars)
    // and we send 500 characters to force a DB error in the Audit Log.
    let malicious_title = "A".repeat(500);

    let payload = serde_json::json!({
        "title": malicious_title,
        "description": "Atomic test description",
        "fee": "100.00",
        "net_amount": "90.00",
        "currency_preference": "USD"
    });

    // 4. Dispatch Request
    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plans") // Adjust to your actual route
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // 5. Assert: The endpoint should fail (500) because the Audit Log failed
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // 6. THE ATOMIC CHECK: Verify the plan was NOT created
    // Even though the Plan INSERT happens BEFORE the AuditLog in the code,
    // the transaction should have rolled it back.
    let plan_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM plans WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("Failed to query database");

    assert!(
        !plan_exists,
        "ATOMIC SAFETY FAILURE: The plan was saved even though the audit log failed!"
    );
}

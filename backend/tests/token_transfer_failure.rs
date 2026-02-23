mod helpers;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn plan_creation_rolls_back_on_transfer_revert() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", user_id))
        .bind("hash")
        .execute(&ctx.pool)
        .await
        .expect("Failed to insert user");

    sqlx::query(
        "INSERT INTO kyc_status (user_id, status, reviewed_by, reviewed_at, created_at) VALUES ($1, 'approved', $2, NOW(), NOW()) ON CONFLICT (user_id) DO UPDATE SET status = 'approved'",
    )
    .bind(user_id)
    .bind(Uuid::new_v4())
    .execute(&ctx.pool)
    .await
    .expect("Failed to set KYC approved");

    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let token = encode(
        &Header::default(),
        &inheritx_backend::auth::UserClaims {
            user_id,
            email: format!("user-{}@example.com", user_id),
            exp,
        },
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate token");

    let body = json!({
        "title": "Atomic Plan",
        "description": "should rollback on revert",
        "fee": "2.00",
        "net_amount": "98.00",
        "beneficiary_name": "Ben",
        "bank_account_number": "000111",
        "bank_name": "TestBank",
        "currency_preference": "USDC"
    });

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plans")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .header("X-Simulate-Revert", "true")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let plan_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plans WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to count plans");
    assert_eq!(plan_count, 0, "No plan should be inserted on revert");

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM action_logs WHERE user_id = $1 AND action = 'plan_created'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count audit logs");
    assert_eq!(audit_count, 0, "No audit log should be inserted on revert");
}

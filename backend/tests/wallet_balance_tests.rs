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
async fn wallet_balance_less_than_required_returns_400_and_no_plan_or_audit_log() {
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
        "INSERT INTO kyc_status (user_id, status, reviewed_by, reviewed_at, created_at) VALUES ($1, 'approved', $2, NOW(), NOW()) ON CONFLICT (user_id) DO UPDATE SET status = 'approved'"
    )
    .bind(user_id)
    .bind(Uuid::new_v4())
    .execute(&ctx.pool)
    .await
    .expect("Failed to set KYC approved");

    let token = encode(
        &Header::default(),
        &inheritx_backend::auth::UserClaims {
            user_id,
            email: format!("user-{}@example.com", user_id),
            exp: 0,
        },
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate token");

    let body = json!({
        "title": "Insufficient Wallet Plan",
        "description": "should fail if wallet balance is less than required",
        "fee": "10.00",
        "net_amount": "490.00",
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
                .header("X-Simulate-Wallet-Balance", "low") // Simulate low wallet balance
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let plan_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plans WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to count plans");
    assert_eq!(
        plan_count, 0,
        "No plan should be inserted if wallet balance is insufficient"
    );

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM action_logs WHERE user_id = $1 AND action = 'plan_created'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count audit logs");
    assert_eq!(
        audit_count, 0,
        "No audit log should be inserted if wallet balance is insufficient"
    );
}

mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

fn generate_user_token(user_id: Uuid, email: &str) -> String {
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = UserClaims {
        user_id,
        email: email.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate token")
}

async fn seed_user_and_kyc(pool: &sqlx::PgPool, user_id: Uuid, email: &str) {
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(email)
        .bind("hashed_password")
        .execute(pool)
        .await
        .expect("Failed to insert test user");

    sqlx::query(
        r#"
        INSERT INTO kyc_status (user_id, status, reviewed_by, reviewed_at, created_at)
        VALUES ($1, 'approved', $2, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE SET status = 'approved'
        "#,
    )
    .bind(user_id)
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .expect("Failed to approve KYC");
}

async fn seed_due_plan(pool: &sqlx::PgPool, user_id: Uuid) -> Uuid {
    let plan_id = Uuid::new_v4();
    let created_in_past = Utc::now().timestamp() - 3600;

    sqlx::query(
        r#"
        INSERT INTO plans (
            id, user_id, title, description, fee, net_amount, status,
            beneficiary_name, bank_account_number, bank_name, currency_preference,
            distribution_method, contract_plan_id, contract_created_at, is_active
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, $8, $9, $10, 'LumpSum', 1, $11, true)
        "#,
    )
    .bind(plan_id)
    .bind(user_id)
    .bind("Double Claim Test Plan")
    .bind("Protect against duplicate claims")
    .bind("10.00")
    .bind("490.00")
    .bind("Beneficiary")
    .bind("1234567890")
    .bind("Test Bank")
    .bind("USDC")
    .bind(created_in_past)
    .execute(pool)
    .await
    .expect("Failed to insert due plan");

    plan_id
}

#[tokio::test]
async fn first_claim_succeeds_second_claim_fails() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let email = format!("double-claim-{}@example.com", user_id);
    seed_user_and_kyc(&ctx.pool, user_id, &email).await;
    let plan_id = seed_due_plan(&ctx.pool, user_id).await;
    let token = generate_user_token(user_id, &email);

    let claim_body = serde_json::json!({
        "beneficiary_email": "beneficiary@example.com"
    });
    let claim_body_str =
        serde_json::to_string(&claim_body).expect("Failed to serialize claim request");

    let first_response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/plans/{}/claim", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(claim_body_str.clone()))
                .expect("Failed to build first claim request"),
        )
        .await
        .expect("first claim request failed");

    assert_eq!(first_response.status(), StatusCode::OK);

    let second_response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/plans/{}/claim", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(claim_body_str))
                .expect("Failed to build second claim request"),
        )
        .await
        .expect("second claim request failed");

    assert_eq!(
        second_response.status(),
        StatusCode::BAD_REQUEST,
        "Second claim should fail"
    );

    let body = axum::body::to_bytes(second_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read second claim response body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse second response JSON");
    let error_message = json["error"].as_str().unwrap_or_default();
    assert!(
        error_message.contains("already been claimed"),
        "Expected duplicate claim error, got: {error_message}"
    );
}

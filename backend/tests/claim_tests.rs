/// Integration tests for claim plan success (issue #115)
///
/// Tests verify the full claim flow:
/// 1. Plan is due for claim
/// 2. KYC is approved
/// 3. Claim is recorded (HTTP 200, "Claim recorded")
/// 4. Audit log is inserted
/// 5. Notification is created
mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

// ── helpers ──────────────────────────────────────────────────────────────────

fn generate_user_token(user_id: Uuid) -> String {
    let claims = UserClaims {
        user_id,
        email: format!("test-{}@example.com", user_id),
        exp: 0,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate user token")
}

/// Insert an approved KYC record for `user_id` directly in the DB.
async fn approve_kyc_direct(pool: &sqlx::PgPool, user_id: Uuid) {
    sqlx::query(
        r#"
        INSERT INTO kyc_status (user_id, status, reviewed_by, reviewed_at, created_at)
        VALUES ($1, 'approved', $2, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE SET status = 'approved'
        "#,
    )
    .bind(user_id)
    .bind(Uuid::new_v4()) // dummy admin id
    .execute(pool)
    .await
    .expect("Failed to approve KYC");
}

/// Insert a plan that is immediately due for claim (distribution_method = LumpSum,
/// contract_created_at set to a past timestamp).
async fn insert_due_plan(pool: &sqlx::PgPool, user_id: Uuid) -> Uuid {
    let plan_id = Uuid::new_v4();
    // contract_created_at is 1 hour in the past so LumpSum is always due
    let past_ts: i64 = chrono::Utc::now().timestamp() - 3600;

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
    .bind("Claim Integration Plan")
    .bind("Test plan for claim integration tests")
    .bind("10.00")
    .bind("490.00")
    .bind("Test Beneficiary")
    .bind("1234567890")
    .bind("Test Bank")
    .bind("USDC")
    .bind(past_ts)
    .execute(pool)
    .await
    .expect("Failed to insert due plan");

    plan_id
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// Verifies that a plan with `distribution_method = LumpSum` and a past
/// `contract_created_at` is considered due for claim by the service logic.
#[tokio::test]
async fn test_claim_plan_is_due() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    approve_kyc_direct(&ctx.pool, user_id).await;
    let plan_id = insert_due_plan(&ctx.pool, user_id).await;

    let body = serde_json::json!({ "beneficiary_email": "beneficiary@example.com" });

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/plans/{}/claim", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    // A 200 response means the plan was due — a 403 would mean not yet due
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Expected 200: plan should be due for claim"
    );

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse JSON");

    assert_eq!(json["status"], "success");
}

/// Verifies that a claim succeeds only when the user's KYC status is approved.
#[tokio::test]
async fn test_claim_requires_kyc_approved() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // KYC is NOT approved — claiming should fail with 403
    let plan_id = insert_due_plan(&ctx.pool, user_id).await;

    let body = serde_json::json!({ "beneficiary_email": "beneficiary@example.com" });

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/plans/{}/claim", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Expected 403: KYC not approved"
    );
}

/// Verifies that a successful claim returns HTTP 200 and persists a claim record
/// in the `claims` table.
#[tokio::test]
async fn test_claim_recorded_on_success() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    approve_kyc_direct(&ctx.pool, user_id).await;
    let plan_id = insert_due_plan(&ctx.pool, user_id).await;

    let body = serde_json::json!({ "beneficiary_email": "claim-record@example.com" });

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/plans/{}/claim", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse JSON");

    // Response indicates the claim was recorded
    assert_eq!(json["status"], "success");
    assert_eq!(json["message"], "Claim recorded");

    // Verify the claim exists in the DB
    let claim_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM claims WHERE plan_id = $1")
        .bind(plan_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to query claims table");

    assert_eq!(claim_count, 1, "Expected exactly one claim record in DB");
}

/// Verifies that a `plan_claimed` action log is written to `action_logs` after
/// a successful claim.
#[tokio::test]
async fn test_claim_audit_log_inserted() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    approve_kyc_direct(&ctx.pool, user_id).await;
    let plan_id = insert_due_plan(&ctx.pool, user_id).await;

    let body = serde_json::json!({ "beneficiary_email": "audit-test@example.com" });

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/plans/{}/claim", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // Verify audit log was inserted
    let log_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM action_logs
        WHERE user_id = $1
          AND action = 'plan_claimed'
          AND entity_id = $2
        "#,
    )
    .bind(user_id)
    .bind(plan_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to query action_logs");

    assert_eq!(
        log_count, 1,
        "Expected exactly one plan_claimed audit log entry"
    );
}

/// Verifies that a `plan_claimed` notification is created for the user after a
/// successful claim.
#[tokio::test]
async fn test_claim_notification_created() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    approve_kyc_direct(&ctx.pool, user_id).await;
    let plan_id = insert_due_plan(&ctx.pool, user_id).await;

    // Count notifications before the claim
    let before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND type = 'plan_claimed'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count notifications before claim");

    let body = serde_json::json!({ "beneficiary_email": "notify-test@example.com" });

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/plans/{}/claim", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // Count notifications after the claim
    let after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND type = 'plan_claimed'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count notifications after claim");

    assert!(
        after > before,
        "Expected a new 'plan_claimed' notification to be created, before={before} after={after}"
    );
}

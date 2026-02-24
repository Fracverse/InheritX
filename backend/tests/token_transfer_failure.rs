mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

// ── Helpers ─────────────────────────────────────────────────────────────────

fn generate_user_token(user_id: Uuid) -> String {
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    encode(
        &Header::default(),
        &UserClaims {
            user_id,
            email: format!("user-{}@example.com", user_id),
            exp,
        },
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate token")
}

async fn setup_user_with_kyc(pool: &sqlx::PgPool) -> Uuid {
    let user_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", user_id))
        .bind("hash")
        .execute(pool)
        .await
        .expect("Failed to insert user");

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
    .expect("Failed to set KYC approved");

    user_id
}

fn plan_request_body() -> Value {
    json!({
        "title": "Atomic Plan",
        "description": "should rollback on revert",
        "fee": "2.00",
        "net_amount": "98.00",
        "beneficiary_name": "Ben",
        "bank_account_number": "000111",
        "bank_name": "TestBank",
        "currency_preference": "USDC"
    })
}

fn build_create_plan_request(token: &str, body: &Value, simulate_revert: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/plans")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .header("X-Simulate-Revert", simulate_revert)
        .body(Body::from(body.to_string()))
        .expect("Failed to build request")
}

// ── Tests ───────────────────────────────────────────────────────────────────

/// Verify that a simulated smart-contract revert (X-Simulate-Revert: true)
/// returns HTTP 500 and rolls back the plan row entirely.
#[tokio::test]
async fn revert_rolls_back_plan_insert() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = setup_user_with_kyc(&ctx.pool).await;
    let token = generate_user_token(user_id);

    let response = ctx
        .app
        .oneshot(build_create_plan_request(
            &token,
            &plan_request_body(),
            "true",
        ))
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let plan_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plans WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to count plans");

    assert_eq!(plan_count, 0, "No plan should be inserted on revert");
}

/// Verify that the in-transaction audit log (action_logs) is also rolled back
/// when the token transfer reverts.
#[tokio::test]
async fn revert_rolls_back_audit_log() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = setup_user_with_kyc(&ctx.pool).await;
    let token = generate_user_token(user_id);

    let response = ctx
        .app
        .oneshot(build_create_plan_request(
            &token,
            &plan_request_body(),
            "true",
        ))
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM action_logs WHERE user_id = $1 AND action = 'plan_created'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count audit logs");

    assert_eq!(audit_count, 0, "No audit log should be inserted on revert");
}

/// Verify that no notification is created for the user when the token transfer
/// reverts (notifications are emitted post-commit, so a rollback must prevent them).
#[tokio::test]
async fn revert_creates_no_notification() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = setup_user_with_kyc(&ctx.pool).await;
    let token = generate_user_token(user_id);

    let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to count notifications before request");

    let response = ctx
        .app
        .oneshot(build_create_plan_request(
            &token,
            &plan_request_body(),
            "true",
        ))
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to count notifications after request");

    assert_eq!(after, before, "No notification should be created on revert");
}

/// Verify that no plan_log entry is created when the token transfer reverts.
/// The plan_logs INSERT happens after tx.commit(), so a rollback must prevent it.
#[tokio::test]
async fn revert_creates_no_plan_log() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = setup_user_with_kyc(&ctx.pool).await;
    let token = generate_user_token(user_id);

    let before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM plan_logs WHERE performed_by = $1 AND action = 'create'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count plan_logs before request");

    let response = ctx
        .app
        .oneshot(build_create_plan_request(
            &token,
            &plan_request_body(),
            "true",
        ))
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM plan_logs WHERE performed_by = $1 AND action = 'create'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count plan_logs after request");

    assert_eq!(after, before, "No plan_log should be created on revert");
}

/// Verify that the error response body contains the expected error message
/// when the token transfer is reverted.
#[tokio::test]
async fn revert_returns_500_with_error_body() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = setup_user_with_kyc(&ctx.pool).await;
    let token = generate_user_token(user_id);

    let response = ctx
        .app
        .oneshot(build_create_plan_request(
            &token,
            &plan_request_body(),
            "true",
        ))
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse response JSON");

    assert!(
        json.get("error").is_some(),
        "Response should contain an 'error' field"
    );
    assert_eq!(
        json["error"], "Internal Server Error",
        "Error message should indicate internal server error"
    );
}

/// Verify that the `X-Simulate-Revert: 1` header value also triggers a
/// rollback (the handler accepts both "true" and "1").
#[tokio::test]
async fn revert_with_header_value_1_triggers_rollback() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = setup_user_with_kyc(&ctx.pool).await;
    let token = generate_user_token(user_id);

    let response = ctx
        .app
        .oneshot(build_create_plan_request(&token, &plan_request_body(), "1"))
        .await
        .expect("request failed");

    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "X-Simulate-Revert: 1 should also trigger rollback"
    );

    let plan_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plans WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to count plans");

    assert_eq!(
        plan_count, 0,
        "No plan should be inserted when revert header is '1'"
    );
}

/// Comprehensive atomic rollback test: a single simulated revert must leave
/// zero side-effects across ALL tables (plans, action_logs, notifications,
/// plan_logs).
#[tokio::test]
async fn atomic_rollback_leaves_zero_side_effects() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = setup_user_with_kyc(&ctx.pool).await;
    let token = generate_user_token(user_id);

    // Snapshot counts before the request
    let plans_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plans WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("count plans before");

    let audit_before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM action_logs WHERE user_id = $1 AND action = 'plan_created'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("count audit before");

    let notif_before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("count notifications before");

    let plan_logs_before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM plan_logs WHERE performed_by = $1 AND action = 'create'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("count plan_logs before");

    // Fire the request with simulated revert
    let response = ctx
        .app
        .oneshot(build_create_plan_request(
            &token,
            &plan_request_body(),
            "true",
        ))
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Verify every table is unchanged
    let plans_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plans WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("count plans after");

    let audit_after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM action_logs WHERE user_id = $1 AND action = 'plan_created'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("count audit after");

    let notif_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("count notifications after");

    let plan_logs_after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM plan_logs WHERE performed_by = $1 AND action = 'create'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("count plan_logs after");

    assert_eq!(plans_after, plans_before, "plans table must be unchanged");
    assert_eq!(
        audit_after, audit_before,
        "action_logs table must be unchanged"
    );
    assert_eq!(
        notif_after, notif_before,
        "notifications table must be unchanged"
    );
    assert_eq!(
        plan_logs_after, plan_logs_before,
        "plan_logs table must be unchanged"
    );
}

/// Control test: plan creation WITHOUT the revert header should succeed and
/// persist data, confirming that the rollback mechanism is the only thing
/// preventing side-effects in the revert tests above.
#[tokio::test]
async fn plan_creation_succeeds_without_revert_header() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = setup_user_with_kyc(&ctx.pool).await;
    let token = generate_user_token(user_id);

    // Build a request WITHOUT the X-Simulate-Revert header
    let request = Request::builder()
        .method("POST")
        .uri("/api/plans")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(plan_request_body().to_string()))
        .expect("Failed to build request");

    let response = ctx.app.oneshot(request).await.expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse JSON");
    assert_eq!(json["status"], "success");

    // Plan should be persisted
    let plan_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plans WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to count plans");
    assert_eq!(plan_count, 1, "Plan should be persisted on success");

    // Audit log should be persisted
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM action_logs WHERE user_id = $1 AND action = 'plan_created'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count audit logs");
    assert_eq!(audit_count, 1, "Audit log should be persisted on success");

    // plan_logs should be persisted
    let plan_log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM plan_logs WHERE performed_by = $1 AND action = 'create'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count plan_logs");
    assert_eq!(plan_log_count, 1, "Plan log should be persisted on success");
}

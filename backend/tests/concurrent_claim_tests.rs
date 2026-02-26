// Integration test: Concurrent Claim Race Condition
// Tests that only one concurrent claim request succeeds and no duplicate payouts occur
mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use serde_json::json;
use tokio::join;
use tower::ServiceExt;
use uuid::Uuid;

async fn insert_due_plan(pool: &sqlx::PgPool, user_id: Uuid) -> Uuid {
    let plan_id = Uuid::new_v4();
    let past_ts = Utc::now().timestamp() - 3600;

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
    .bind("Concurrent Claim Test Plan")
    .bind("Test plan for concurrent claim race condition")
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

#[tokio::test]
async fn test_concurrent_claim_same_email_only_one_succeeds() {
    // Test: Two parallel requests with SAME email - only one should succeed
    let Some(test_context) = helpers::TestContext::from_env().await else {
        println!("SKIPPING TEST: no database connection");
        return;
    };

    let pool = test_context.pool.clone();
    let app = test_context.app;

    let user_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", user_id);

    // Insert user
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&email)
        .bind("hashed_password")
        .execute(&pool)
        .await
        .expect("Failed to insert user");

    // Approve KYC
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'approved')")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Failed to approve KYC");

    // Create a due plan
    let plan_id = insert_due_plan(&pool, user_id).await;

    // Prepare two simultaneous claim requests with the SAME email
    let claim_email = "beneficiary@test.com";
    let claim_req = || {
        Request::builder()
            .method("POST")
            .uri(format!("/api/plans/{}/claim", plan_id))
            .header("Content-Type", "application/json")
            .header("X-User-Id", user_id.to_string())
            .body(Body::from(
                serde_json::to_string(&json!({ "beneficiary_email": claim_email })).unwrap(),
            ))
            .unwrap()
    };

    // Send both claims in parallel
    let (resp1, resp2) = join!(
        app.clone().oneshot(claim_req()),
        app.clone().oneshot(claim_req())
    );

    let status1 = resp1.expect("claim1 request failed").status();
    let status2 = resp2.expect("claim2 request failed").status();

    // Exactly one should succeed
    let success_count = (status1 == StatusCode::OK) as i32 + (status2 == StatusCode::OK) as i32;
    assert_eq!(
        success_count, 1,
        "Exactly one claim should succeed with same email. Got status1: {}, status2: {}",
        status1, status2
    );

    // Verify only one claim record exists in database
    let claim_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM claims WHERE plan_id = $1")
        .bind(plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        claim_count.0, 1,
        "Only one claim record should exist in database"
    );
}

#[tokio::test]
async fn test_concurrent_claim_different_emails_only_one_succeeds() {
    // Test: Two parallel requests with DIFFERENT emails - only one should succeed
    // This specifically tests the race condition where different beneficiaries try to claim
    let Some(test_context) = helpers::TestContext::from_env().await else {
        println!("SKIPPING TEST: no database connection");
        return;
    };

    let pool = test_context.pool.clone();
    let app = test_context.app;

    let user_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", user_id);

    // Insert user
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&email)
        .bind("hashed_password")
        .execute(&pool)
        .await
        .expect("Failed to insert user");

    // Approve KYC
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'approved')")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Failed to approve KYC");

    // Create a due plan
    let plan_id = insert_due_plan(&pool, user_id).await;

    // Prepare two simultaneous claim requests with DIFFERENT emails
    let claim_req_with_email = |email: &str| {
        Request::builder()
            .method("POST")
            .uri(format!("/api/plans/{}/claim", plan_id))
            .header("Content-Type", "application/json")
            .header("X-User-Id", user_id.to_string())
            .body(Body::from(
                serde_json::to_string(&json!({ "beneficiary_email": email })).unwrap(),
            ))
            .unwrap()
    };

    // Send both claims in parallel with different emails
    let (resp1, resp2) = join!(
        app.clone()
            .oneshot(claim_req_with_email("beneficiary1@test.com")),
        app.clone()
            .oneshot(claim_req_with_email("beneficiary2@test.com"))
    );

    let status1 = resp1.expect("claim1 request failed").status();
    let status2 = resp2.expect("claim2 request failed").status();

    // Exactly one should succeed (this tests the race condition fix)
    let success_count = (status1 == StatusCode::OK) as i32 + (status2 == StatusCode::OK) as i32;
    assert_eq!(
        success_count, 1,
        "Exactly one claim should succeed with different emails. Got status1: {}, status2: {}",
        status1, status2
    );

    // Verify only one claim record exists in database (no duplicate payout)
    let claim_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM claims WHERE plan_id = $1")
        .bind(plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        claim_count.0, 1,
        "Only one claim record should exist (no duplicate payout)"
    );
}

#[tokio::test]
async fn test_concurrent_claim_updates_plan_status() {
    // Test: Verify plan status is updated to 'claimed' after successful claim
    let Some(test_context) = helpers::TestContext::from_env().await else {
        println!("SKIPPING TEST: no database connection");
        return;
    };

    let pool = test_context.pool.clone();
    let app = test_context.app;

    let user_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", user_id);

    // Insert user
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&email)
        .bind("hashed_password")
        .execute(&pool)
        .await
        .expect("Failed to insert user");

    // Approve KYC
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'approved')")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Failed to approve KYC");

    // Create a due plan
    let plan_id = insert_due_plan(&pool, user_id).await;

    // Submit a claim request
    let claim_req = Request::builder()
        .method("POST")
        .uri(format!("/api/plans/{}/claim", plan_id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({ "beneficiary_email": "test@beneficiary.com" })).unwrap(),
        ))
        .unwrap();

    let resp = app
        .clone()
        .oneshot(claim_req)
        .await
        .expect("claim request failed");
    assert_eq!(resp.status(), StatusCode::OK, "Claim should succeed");

    // Verify plan status is updated to 'claimed'
    let plan_status: (String,) = sqlx::query_as("SELECT status FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        plan_status.0, "claimed",
        "Plan status should be 'claimed' after successful claim"
    );
}

#[tokio::test]
async fn test_claim_after_concurrent_claims_fails() {
    // Test: After concurrent claims resolve, subsequent claim attempts should fail
    let Some(test_context) = helpers::TestContext::from_env().await else {
        println!("SKIPPING TEST: no database connection");
        return;
    };

    let pool = test_context.pool.clone();
    let app = test_context.app;

    let user_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", user_id);

    // Insert user
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&email)
        .bind("hashed_password")
        .execute(&pool)
        .await
        .expect("Failed to insert user");

    // Approve KYC
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'approved')")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Failed to approve KYC");

    // Create a due plan
    let plan_id = insert_due_plan(&pool, user_id).await;

    // Submit first claim (should succeed)
    let claim_req = Request::builder()
        .method("POST")
        .uri(format!("/api/plans/{}/claim", plan_id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({ "beneficiary_email": "first@beneficiary.com" }))
                .unwrap(),
        ))
        .unwrap();

    let resp1 = app
        .clone()
        .oneshot(claim_req)
        .await
        .expect("claim request failed");
    assert_eq!(resp1.status(), StatusCode::OK, "First claim should succeed");

    // Submit second claim with different email (should fail - already claimed)
    let claim_req2 = Request::builder()
        .method("POST")
        .uri(format!("/api/plans/{}/claim", plan_id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", user_id.to_string())
        .body(Body::from(
            serde_json::to_string(&json!({ "beneficiary_email": "second@beneficiary.com" }))
                .unwrap(),
        ))
        .unwrap();

    let resp2 = app
        .clone()
        .oneshot(claim_req2)
        .await
        .expect("second claim request failed");

    // Should fail with 400 Bad Request since plan is already claimed
    assert_eq!(
        resp2.status(),
        StatusCode::BAD_REQUEST,
        "Second claim should fail because plan is already claimed"
    );
}

#[tokio::test]
async fn test_concurrent_claim_creates_single_audit_log() {
    // Test: Verify only one audit log is created for concurrent claims
    // This ensures no duplicate payout is recorded
    let Some(test_context) = helpers::TestContext::from_env().await else {
        println!("SKIPPING TEST: no database connection");
        return;
    };

    let pool = test_context.pool.clone();
    let app = test_context.app;

    let user_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", user_id);

    // Insert user
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&email)
        .bind("hashed_password")
        .execute(&pool)
        .await
        .expect("Failed to insert user");

    // Approve KYC
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'approved')")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Failed to approve KYC");

    // Create a due plan
    let plan_id = insert_due_plan(&pool, user_id).await;

    // Submit two concurrent claim requests with different emails
    let claim_req_with_email = |email: &str| {
        Request::builder()
            .method("POST")
            .uri(format!("/api/plans/{}/claim", plan_id))
            .header("Content-Type", "application/json")
            .header("X-User-Id", user_id.to_string())
            .body(Body::from(
                serde_json::to_string(&json!({ "beneficiary_email": email })).unwrap(),
            ))
            .unwrap()
    };

    // Send both claims in parallel
    let (resp1, resp2) = join!(
        app.clone()
            .oneshot(claim_req_with_email("beneficiary1@test.com")),
        app.clone()
            .oneshot(claim_req_with_email("beneficiary2@test.com"))
    );

    let status1 = resp1.expect("claim1 request failed").status();
    let status2 = resp2.expect("claim2 request failed").status();

    // Exactly one should succeed
    let success_count = (status1 == StatusCode::OK) as i32 + (status2 == StatusCode::OK) as i32;
    assert_eq!(success_count, 1, "Exactly one claim should succeed");

    // Check only one audit log for 'claim' action exists
    // This verifies no duplicate payout was recorded
    let audit_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM action_logs WHERE entity_type = 'plan' AND action = 'claim' AND entity_id = $1"
    )
        .bind(plan_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        audit_count.0, 1,
        "Only one audit log should exist for claim action (no duplicate payout)"
    );
}

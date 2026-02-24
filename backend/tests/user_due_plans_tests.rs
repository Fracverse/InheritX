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

fn generate_user_token(user_id: Uuid) -> String {
    let exp = (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = UserClaims {
        user_id,
        email: format!("due-test-{}@example.com", user_id),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-jwt-secret"),
    )
    .expect("Failed to generate user token")
}

/// Insert a plan that IS due for claim (LumpSum, created in the past, active, no claim).
async fn insert_due_plan(pool: &sqlx::PgPool, user_id: Uuid, title: &str) -> Uuid {
    let plan_id = Uuid::new_v4();
    let past_ts = Utc::now().timestamp() - 3600; // 1 hour ago

    sqlx::query(
        r#"
        INSERT INTO plans (
            id, user_id, title, description, fee, net_amount, status,
            beneficiary_name, bank_account_number, bank_name, currency_preference,
            distribution_method, contract_plan_id, contract_created_at, is_active
        )
        VALUES ($1, $2, $3, 'Test due plan', '10.00', '490.00', 'pending',
                'Beneficiary', '1234567890', 'Test Bank', 'USDC',
                'LumpSum', 1, $4, true)
        "#,
    )
    .bind(plan_id)
    .bind(user_id)
    .bind(title)
    .bind(past_ts)
    .execute(pool)
    .await
    .expect("Failed to insert due plan");

    plan_id
}

/// Insert a plan that is NOT due for claim (Monthly, just created → not mature yet).
async fn insert_not_due_plan(pool: &sqlx::PgPool, user_id: Uuid, title: &str) -> Uuid {
    let plan_id = Uuid::new_v4();
    let now_ts = Utc::now().timestamp(); // just created, Monthly needs 30 days

    sqlx::query(
        r#"
        INSERT INTO plans (
            id, user_id, title, description, fee, net_amount, status,
            beneficiary_name, bank_account_number, bank_name, currency_preference,
            distribution_method, contract_plan_id, contract_created_at, is_active
        )
        VALUES ($1, $2, $3, 'Test not-due plan', '10.00', '490.00', 'pending',
                'Beneficiary', '1234567890', 'Test Bank', 'USDC',
                'Monthly', 1, $4, true)
        "#,
    )
    .bind(plan_id)
    .bind(user_id)
    .bind(title)
    .bind(now_ts)
    .execute(pool)
    .await
    .expect("Failed to insert not-due plan");

    plan_id
}

/// Insert a deactivated plan (should never appear in due-for-claim results).
async fn insert_deactivated_plan(pool: &sqlx::PgPool, user_id: Uuid, title: &str) -> Uuid {
    let plan_id = Uuid::new_v4();
    let past_ts = Utc::now().timestamp() - 3600;

    sqlx::query(
        r#"
        INSERT INTO plans (
            id, user_id, title, description, fee, net_amount, status,
            beneficiary_name, bank_account_number, bank_name, currency_preference,
            distribution_method, contract_plan_id, contract_created_at, is_active
        )
        VALUES ($1, $2, $3, 'Deactivated plan', '10.00', '490.00', 'deactivated',
                'Beneficiary', '1234567890', 'Test Bank', 'USDC',
                'LumpSum', 1, $4, false)
        "#,
    )
    .bind(plan_id)
    .bind(user_id)
    .bind(title)
    .bind(past_ts)
    .execute(pool)
    .await
    .expect("Failed to insert deactivated plan");

    plan_id
}

// ── Test: no plans → empty array ────────────────────────────────────────────

#[tokio::test]
async fn user_with_no_plans_returns_empty_array() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .uri("/api/plans/due-for-claim")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse JSON");

    assert_eq!(json["status"], "success");
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
    assert_eq!(json["count"], 0);
}

// ── Test: only due plans are returned (not immature / deactivated / claimed) ─

#[tokio::test]
async fn only_due_plans_returned() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Insert one due plan
    let due_id = insert_due_plan(&ctx.pool, user_id, "Due Plan Alpha").await;

    // Insert one NOT-due plan (Monthly, just created)
    let _not_due_id = insert_not_due_plan(&ctx.pool, user_id, "Not Due Plan Beta").await;

    // Insert one deactivated plan (should be excluded)
    let _deactivated_id =
        insert_deactivated_plan(&ctx.pool, user_id, "Deactivated Plan Gamma").await;

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .uri("/api/plans/due-for-claim")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse JSON");

    assert_eq!(json["status"], "success");

    let data = json["data"].as_array().expect("data should be an array");

    // Only the due plan should be present
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["id"], due_id.to_string());
    assert_eq!(data[0]["title"], "Due Plan Alpha");
}

// ── Test: count field matches data array length ─────────────────────────────

#[tokio::test]
async fn count_matches_data_length() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Insert multiple due plans
    insert_due_plan(&ctx.pool, user_id, "Due Plan 1").await;
    insert_due_plan(&ctx.pool, user_id, "Due Plan 2").await;
    insert_due_plan(&ctx.pool, user_id, "Due Plan 3").await;

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .uri("/api/plans/due-for-claim")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse JSON");

    assert_eq!(json["status"], "success");

    let data = json["data"].as_array().expect("data should be an array");
    let count = json["count"].as_u64().expect("count should be a number");

    assert_eq!(data.len() as u64, count);
    assert_eq!(count, 3);
}

// ── Test: already-claimed plans are excluded ────────────────────────────────

#[tokio::test]
async fn claimed_plan_excluded_from_due_plans() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Insert two due plans
    let plan_to_claim = insert_due_plan(&ctx.pool, user_id, "Will Be Claimed").await;
    let plan_still_due = insert_due_plan(&ctx.pool, user_id, "Still Due").await;

    // Simulate a claim on the first plan
    sqlx::query(
        "INSERT INTO claims (plan_id, contract_plan_id, beneficiary_email) VALUES ($1, 1, 'claimer@example.com')",
    )
    .bind(plan_to_claim)
    .execute(&ctx.pool)
    .await
    .expect("Failed to insert claim");

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .uri("/api/plans/due-for-claim")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse JSON");

    let data = json["data"].as_array().expect("data should be an array");

    // Only the unclaimed plan should appear
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["id"], plan_still_due.to_string());
}

// ── Test: unauthenticated request returns 401 ───────────────────────────────

#[tokio::test]
async fn unauthenticated_request_returns_401() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .uri("/api/plans/due-for-claim")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Test: user cannot see another user's due plans ──────────────────────────

#[tokio::test]
async fn user_cannot_see_other_users_due_plans() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_a = Uuid::new_v4();
    let user_b = Uuid::new_v4();

    // Insert a due plan for user_a
    insert_due_plan(&ctx.pool, user_a, "User A Plan").await;

    // Authenticate as user_b
    let token_b = generate_user_token(user_b);

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .uri("/api/plans/due-for-claim")
                .header("Authorization", format!("Bearer {}", token_b))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse JSON");

    let data = json["data"].as_array().expect("data should be an array");

    // user_b should see zero plans
    assert_eq!(data.len(), 0);
    assert_eq!(json["count"], 0);
}

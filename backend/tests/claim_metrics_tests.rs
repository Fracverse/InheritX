mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::{Duration, Utc};
use inheritx_backend::auth::{AdminClaims, UserClaims};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

fn generate_admin_token(admin_id: Uuid) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = AdminClaims {
        admin_id,
        email: format!("admin-{}@example.com", admin_id),
        role: "admin".to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-jwt-secret"),
    )
    .expect("Failed to generate admin token")
}

fn generate_user_token(user_id: Uuid) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = UserClaims {
        user_id,
        email: format!("user-{}@example.com", user_id),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-jwt-secret"),
    )
    .expect("Failed to generate user token")
}

#[derive(Debug)]
struct ClaimMetricsSnapshot {
    total_claims: i64,
    pending_claims: i64,
    approved_claims: i64,
    rejected_claims: i64,
    average_claim_processing_time_seconds: f64,
}

async fn fetch_claim_metrics(app: &Router, token: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/claims")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .expect("failed to build request"),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read response body");
    let json: Value = serde_json::from_slice(&body).expect("response is not valid JSON");
    (status, json)
}

fn parse_claim_metrics(json: &Value) -> ClaimMetricsSnapshot {
    ClaimMetricsSnapshot {
        total_claims: json["data"]["totalClaims"]
            .as_i64()
            .expect("totalClaims should be i64"),
        pending_claims: json["data"]["pendingClaims"]
            .as_i64()
            .expect("pendingClaims should be i64"),
        approved_claims: json["data"]["approvedClaims"]
            .as_i64()
            .expect("approvedClaims should be i64"),
        rejected_claims: json["data"]["rejectedClaims"]
            .as_i64()
            .expect("rejectedClaims should be i64"),
        average_claim_processing_time_seconds: json["data"]["averageClaimProcessingTimeSeconds"]
            .as_f64()
            .expect("averageClaimProcessingTimeSeconds should be f64"),
    }
}

async fn insert_plan_with_status(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    status: &str,
    updated_at: chrono::DateTime<Utc>,
) -> Uuid {
    let plan_id = Uuid::new_v4();
    let created_at = updated_at - Duration::days(1);

    sqlx::query(
        r#"
        INSERT INTO plans (
            id, user_id, title, description, fee, net_amount, status, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5::numeric, $6::numeric, $7, $8, $9)
        "#,
    )
    .bind(plan_id)
    .bind(user_id)
    .bind(format!("Claim metrics plan {status}"))
    .bind("metrics test plan")
    .bind("0.00")
    .bind("100.00")
    .bind(status)
    .bind(created_at)
    .bind(updated_at)
    .execute(pool)
    .await
    .expect("failed to insert plan");

    plan_id
}

async fn insert_claim(
    pool: &sqlx::PgPool,
    plan_id: Uuid,
    contract_plan_id: i64,
    beneficiary_email: &str,
    claimed_at: chrono::DateTime<Utc>,
) {
    sqlx::query(
        r#"
        INSERT INTO claims (plan_id, contract_plan_id, beneficiary_email, claimed_at, created_at)
        VALUES ($1, $2, $3, $4, $4)
        "#,
    )
    .bind(plan_id)
    .bind(contract_plan_id)
    .bind(beneficiary_email)
    .bind(claimed_at)
    .execute(pool)
    .await
    .expect("failed to insert claim");
}

#[tokio::test]
async fn admin_can_fetch_claim_metrics() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_admin_token(Uuid::new_v4());
    let (status, json) = fetch_claim_metrics(&ctx.app, &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "success");
    assert!(json["data"]["totalClaims"].is_number());
    assert!(json["data"]["pendingClaims"].is_number());
    assert!(json["data"]["approvedClaims"].is_number());
    assert!(json["data"]["rejectedClaims"].is_number());
    assert!(json["data"]["averageClaimProcessingTimeSeconds"].is_number());
}

#[tokio::test]
async fn regular_user_cannot_fetch_claim_metrics() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_user_token(Uuid::new_v4());
    let (status, _json) = fetch_claim_metrics(&ctx.app, &token).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unauthenticated_cannot_fetch_claim_metrics() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/metrics/claims")
                .body(Body::empty())
                .expect("failed to build unauthenticated request"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn claim_metrics_count_and_average_change_with_new_rows() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let admin_token = generate_admin_token(Uuid::new_v4());

    let (status_before, json_before) = fetch_claim_metrics(&ctx.app, &admin_token).await;
    assert_eq!(status_before, StatusCode::OK);
    let before = parse_claim_metrics(&json_before);
    let baseline_processed_samples: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM claims c
        INNER JOIN plans p ON p.id = c.plan_id
        WHERE p.status IN ('claimed', 'rejected', 'deactivated')
          AND p.updated_at >= c.claimed_at
        "#,
    )
    .fetch_one(&ctx.pool)
    .await
    .expect("failed to count baseline processed claims");

    let user_id = Uuid::new_v4();
    let email = format!("claim-metrics-{}@example.com", user_id);
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(email)
        .bind("hashed_password")
        .execute(&ctx.pool)
        .await
        .expect("failed to insert test user");

    let now = Utc::now();
    let approved_updated_at = now - Duration::hours(1);
    let rejected_updated_at = now - Duration::hours(1);

    let pending_plan_id = insert_plan_with_status(&ctx.pool, user_id, "pending", now).await;
    let due_plan_id = insert_plan_with_status(&ctx.pool, user_id, "due-for-claim", now).await;
    let approved_plan_id =
        insert_plan_with_status(&ctx.pool, user_id, "claimed", approved_updated_at).await;
    let rejected_plan_id =
        insert_plan_with_status(&ctx.pool, user_id, "deactivated", rejected_updated_at).await;

    insert_claim(
        &ctx.pool,
        pending_plan_id,
        5001,
        "pending-claim@example.com",
        now - Duration::hours(3),
    )
    .await;
    insert_claim(
        &ctx.pool,
        due_plan_id,
        5002,
        "due-claim@example.com",
        now - Duration::hours(2),
    )
    .await;
    insert_claim(
        &ctx.pool,
        approved_plan_id,
        5003,
        "approved-claim@example.com",
        now - Duration::hours(4),
    )
    .await;
    insert_claim(
        &ctx.pool,
        rejected_plan_id,
        5004,
        "rejected-claim@example.com",
        now - Duration::hours(6),
    )
    .await;

    let (status_after, json_after) = fetch_claim_metrics(&ctx.app, &admin_token).await;
    assert_eq!(status_after, StatusCode::OK);
    let after = parse_claim_metrics(&json_after);

    assert_eq!(after.total_claims, before.total_claims + 4);
    assert_eq!(after.pending_claims, before.pending_claims + 2);
    assert_eq!(after.approved_claims, before.approved_claims + 1);
    assert_eq!(after.rejected_claims, before.rejected_claims + 1);

    let inserted_processing_seconds = 10_800.0 + 18_000.0;
    let expected_average = if baseline_processed_samples == 0 {
        inserted_processing_seconds / 2.0
    } else {
        ((before.average_claim_processing_time_seconds * baseline_processed_samples as f64)
            + inserted_processing_seconds)
            / (baseline_processed_samples + 2) as f64
    };

    assert!(
        (after.average_claim_processing_time_seconds - expected_average).abs() < 1.0,
        "averageClaimProcessingTimeSeconds mismatch; expected ~{expected_average}, got {}",
        after.average_claim_processing_time_seconds
    );
}

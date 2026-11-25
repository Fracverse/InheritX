//! End-to-end coverage for the new lending and AI-optimisation API surfaces
//! once a real PostgreSQL is available (issue #1270 and #1271).
//!
//! Set `WATCHDOG_TEST_DATABASE_URL` (or `DATABASE_URL`) to a throwaway
//! PostgreSQL to run these. Without it they skip, like the other DB suites in
//! this crate.

use axum::http::{Request, StatusCode};
use inheritx_backend::{
    create_router, AppState, DbManager, PlanCache,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn pool()
 -> Option<sqlx::PgPool> {
    let url = std::env::var("WATCHDOG_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .ok()?;
    DbManager::run_migrations(&pool)
        .await
        .expect("migrations must apply to the test database");
    Some(pool)
}

async fn seed_plan(
    pool: &sqlx::PgPool,
    owner: &str,
    token_address: &str,
    amount: rust_decimal::Decimal,
    onchain_plan_id: Option<i64>,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO plans
            (id, owner_address, token_address, amount, grace_period, grace_period_seconds,
             last_ping, is_active, status, earn_yield, yield_rate_bps, onchain_plan_id)
        VALUES ($1, $2, $3, $4, 3600, 86400, 0, TRUE, 'ACTIVE', TRUE, 300, $5)
        "#,
    )
    .bind(id)
    .bind(owner)
    .bind(token_address)
    .bind(amount)
    .bind(onchain_plan_id)
    .execute(pool)
    .await
    .expect("plan insert");
    id
}

fn build_token(sub: &str, role: &str) -> String {
    std::env::set_var("JWT_SECRET", "lending-ai-test-secret");
    let claims = inheritx_backend::auth::Claims {
        sub: sub.to_string(),
        role: role.to_string(),
        exp: (chrono::Utc::now().timestamp() + 3_600) as usize,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"lending-ai-test-secret"),
    )
    .unwrap()
}

fn app(pool: sqlx::PgPool) -> axum::Router {
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new(
            "http://localhost:8081".to_string(),
        )),
        db_pool: pool,
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: PlanCache::disabled(),
        plan_statistics_cache_ttl_secs: 60,
        apy_cache: dashmap::DashMap::new(),
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        status_tx: tokio::sync::broadcast::channel(16).0,
        announcement_tx: tokio::sync::broadcast::channel(16).0,
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
        lending: None,
    });
    create_router(state)
}

async fn readable(api: &axum::Router, method: &str, uri: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(axum::http::header::USER_AGENT, "InheritX-Test/1.0")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = api.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes =
        axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap_or_else(|_| json!({}));
    (status, body)
}

async fn with_body(
    api: &axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/json",
        )
        .header(axum::http::header::USER_AGENT, "InheritX-Test/1.0");
    if let Some(token) = token {
        request = request.header(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {token}"),
        );
    }
    let request = request.body(axum::body::Body::from(body.to_string())).unwrap();
    let response = api.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes =
        axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap_or_else(|_| json!({}));
    (status, body)
}

#[tokio::test]
async fn pool_state_without_config_returns_service_unavailable() {
    let pool = pool().await.expect("DATABASE_URL must be reachable for this test");
    let app = app(pool);

    let (status, body) = readable(&app, "GET", "/api/lending/pool-state").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn user_shares_without_config_returns_service_unavailable() {
    let pool = pool().await.expect("DATABASE_URL must be reachable for this test");
    let app = app(pool);

    let (status, body) =
        readable(&app, "GET", "/api/lending/shares/GABCDEFABCDEFABCDEFABCDEFABCDEDG").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn recommendation_rejects_non_owned_plan() {
    let pool = pool().await.expect("DATABASE_URL must be reachable for this test");
    let owner = "GOWNER".to_string();
    let other = "GOTHER".to_string();

    let plan_id = seed_plan(&pool, &owner, "USDC", "10000.00".parse().unwrap(), Some(7)).await;
    let app = app(pool);

    // A different wallet requesting recommendations for another owner's plan
    // must be forbidden.
    let (status, _) = with_body(
        &app,
        "GET",
        &format!("/api/ai/optimize/{plan_id}"),
        json!({}),
        Some(&build_token(&other, "user")),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn recommendation_requires_a_positive_plan_balance() {
    let pool = pool().await.expect("DATABASE_URL must be reachable for this test");
    let owner = "GOWNER".to_string();
    let token = build_token(&owner, "user");

    let plan_id = seed_plan(
        &pool,
        &owner,
        "USDC",
        "0.00".parse().unwrap(),
        Some(9),
    )
    .await;
    let app = app(pool);

    let (status, body) = with_body(
        &app,
        "GET",
        &format!("/api/ai/optimize/{plan_id}"),
        json!({}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn recommendation_can_persist_and_be_answered() {
    let pool = pool().await.expect("DATABASE_URL must be reachable for this test");
    let owner = "GOWNER".to_string();
    let other = "GOTHER".to_string();

    let plan_id = seed_plan(
        &pool,
        &owner,
        "USDC",
        "10000.00".parse().unwrap(),
        Some(11),
    )
    .await;
    let app = app(pool);
    let owner_token = build_token(&owner, "user");

    // Generate a recommendation for the plan.
    let (status, body) = with_body(
        &app,
        "GET",
        &format!("/api/ai/optimize/{plan_id}"),
        json!({}),
        Some(&owner_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let rec_id = body["id"].as_str().expect("recommendation has an id");
    assert!(!rec_id.is_empty());

    // Accepting the recommendation must succeed.
    let (accept_status, accept_body) = with_body(
        &app,
        "POST",
        &format!("/api/ai/recommendations/{rec_id}/respond"),
        json!({ "action": "accept" }),
        Some(&owner_token),
    )
    .await;
    assert_eq!(accept_status, StatusCode::OK);
    assert_eq!(accept_body["status"], "accepted");
    assert!(accept_body["applied_at"].is_string());

    // A second respond must be rejected as already answered.
    let (second_status, _) = with_body(
        &app,
        "POST",
        &format!("/api/ai/recommendations/{rec_id}/respond"),
        json!({ "action": "accept" }),
        Some(&owner_token),
    )
    .await;
    assert_eq!(second_status, StatusCode::CONFLICT);

    // Rejection without a reason must be a validation error for a pending row.
    let (reject_status, _) = with_body(
        &app,
        "POST",
        &format!("/api/ai/recommendations/{rec_id}/respond"),
        json!({ "action": "reject" }),
        Some(&owner_token),
    )
    .await;
    assert_eq!(reject_status, StatusCode::BAD_REQUEST);

    // Verify the recommendation shape is genuinely derived from plan data.
    assert!(body["expected_return"].is_number());
    assert!(body["risk_score"].is_number());
    assert!(body["confidence_score"].is_number());
    assert!(body["recommended_allocations"].is_array());
    assert!(body["recommended_allocations"].as_array().unwrap().len() >= 1);
    assert!(body["generated_at"].is_string());

    // A different wallet must not be able to answer the same recommendation.
    let (other_status, _) = with_body(
        &app,
        "POST",
        &format!("/api/ai/recommendations/{rec_id}/respond"),
        json!({ "action": "reject", "reason": "someone else decided" }),
        Some(&build_token(&other, "user")),
    )
    .await;
    assert_eq!(other_status, StatusCode::FORBIDDEN);

    // A pending row must accept a valid rejection with reason.
    let (pending_status, pending_body) = with_body(
        &app,
        "POST",
        &format!("/api/ai/recommendations/{rec_id}/respond"),
        json!({ "action": "reject", "reason": "changed my mind" }),
        Some(&owner_token),
    )
    .await;
    assert_eq!(pending_status, StatusCode::CONFLICT);
    assert_eq!(pending_body["status"], "rejected");

    // Replaying the same rejection still yields CONFLICT.
    let (again_status, again_body) = with_body(
        &app,
        "POST",
        &format!("/api/ai/recommendations/{rec_id}/respond"),
        json!({ "action": "reject", "reason": "still changed" }),
        Some(&owner_token),
    )
    .await;
    assert_eq!(again_status, StatusCode::CONFLICT);
    assert_eq!(again_body["status"], "rejected");
    assert!(again_body["applied_at"].is_string());

    // A reason-less reject on the now-rejected row is still CONFLICT.
    let (reasonless_status, _) = with_body(
        &app,
        "POST",
        &format!("/api/ai/recommendations/{rec_id}/respond"),
        json!({ "action": "reject" }),
        Some(&owner_token),
    )
    .await;
    assert_eq!(reasonless_status, StatusCode::CONFLICT);
}



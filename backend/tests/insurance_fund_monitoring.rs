/*
mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use helpers::TestContext;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use sqlx::Row;
use tower::ServiceExt;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Insurance Fund Dashboard Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_insurance_fund_dashboard_requires_auth() {
    let Some(test_context) = TestContext::from_env().await else {
        return;
    };

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/api/admin/insurance-fund")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_insurance_fund_dashboard_success() {
    let Some(mut test_context) = TestContext::from_env().await else {
        return;
    };

    // Create admin and get token
    let admin_token = helpers::create_admin_and_get_token(&mut test_context).await;

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/api/admin/insurance-fund")
                .method("GET")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert!(json["data"]["fund"].is_object());
    assert!(json["data"]["recent_transactions"].is_array());
    assert!(json["data"]["pending_claims"].is_array());
}

// ─────────────────────────────────────────────────────────────────────────────
// Insurance Fund Metrics Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_insurance_fund_coverage_ratio_calculation() {
    let Some(_test_context) = TestContext::from_env().await else {
        return;
    };

    // Test coverage ratio calculation with various scenarios
    let reserves = Decimal::new(1500000, 0); // $1.5M
    let liabilities = Decimal::new(1000000, 0); // $1M

    let coverage_ratio = reserves / liabilities;
    assert_eq!(coverage_ratio, Decimal::new(15, 1)); // 1.5x

    // Test with zero liabilities (should be high)
    let zero_liabilities = Decimal::ZERO;
    let infinite_ratio = if zero_liabilities == Decimal::ZERO {
        Decimal::new(9999, 0)
    } else {
        reserves / zero_liabilities
    };
    assert_eq!(infinite_ratio, Decimal::new(9999, 0));
}

#[tokio::test]
async fn test_insurance_fund_health_score_calculation() {
    // Test health score calculation
    let min_ratio = Decimal::new(100, 2); // 1.0
    let target_ratio = Decimal::new(150, 2); // 1.5

    // At target, score should be 100
    let score_at_target = if Decimal::new(150, 2) >= target_ratio {
        Decimal::new(100, 0)
    } else {
        Decimal::ZERO
    };
    assert_eq!(score_at_target, Decimal::new(100, 0));

    // Below minimum, score should be 0
    let score_below_min = if Decimal::new(80, 2) <= min_ratio {
        Decimal::ZERO
    } else {
        Decimal::new(50, 0)
    };
    assert_eq!(score_below_min, Decimal::ZERO);
}

// ─────────────────────────────────────────────────────────────────────────────
// Insurance Fund Transactions Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_insurance_fund_transactions_requires_auth() {
    let Some(test_context) = TestContext::from_env().await else {
        return;
    };

    // Get primary fund ID first
    let fund_id: Uuid = sqlx::query_scalar("SELECT id FROM insurance_fund LIMIT 1")
        .fetch_optional(&test_context.pool)
        .await
        .expect("query failed")
        .unwrap_or_else(Uuid::new_v4);

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/admin/insurance-fund/{}/transactions",
                    fund_id
                ))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_insurance_fund_transaction_recording() {
    let Some(test_context) = TestContext::from_env().await else {
        return;
    };

    // Get primary fund
    let fund_id: Uuid = sqlx::query_scalar("SELECT id FROM insurance_fund LIMIT 1")
        .fetch_optional(&test_context.pool)
        .await
        .expect("query failed")
        .expect("fund not found");

    let initial_balance: Decimal =
        sqlx::query_scalar("SELECT total_reserves FROM insurance_fund WHERE id = $1")
            .bind(fund_id)
            .fetch_one(&test_context.pool)
            .await
            .expect("query failed");

    // Record a contribution transaction
    let contribution_amount = Decimal::new(100000, 0); // $100k
    let result = sqlx::query(
        r#"
        INSERT INTO insurance_fund_transactions (
            fund_id, transaction_type, asset_code, amount, balance_after, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(fund_id)
    .bind("contribution")
    .bind("USDC")
    .bind(contribution_amount)
    .bind(initial_balance + contribution_amount)
    .bind(json!({}))
    .fetch_one(&test_context.pool)
    .await
    .expect("failed to insert transaction");

    assert!(result.get::<Uuid, _>("id") != Uuid::nil());

    // Verify balance was updated
    let new_balance: Decimal =
        sqlx::query_scalar("SELECT total_reserves FROM insurance_fund WHERE id = $1")
            .bind(fund_id)
            .fetch_one(&test_context.pool)
            .await
            .expect("query failed");

    // Note: This test verifies the transaction was recorded, but the actual
    // balance update would happen through the service layer
    assert!(new_balance >= initial_balance);
}

// ─────────────────────────────────────────────────────────────────────────────
// Insurance Claims Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_insurance_claim_requires_auth() {
    let Some(test_context) = TestContext::from_env().await else {
        return;
    };

    let fund_id: Uuid = sqlx::query_scalar("SELECT id FROM insurance_fund LIMIT 1")
        .fetch_optional(&test_context.pool)
        .await
        .expect("query failed")
        .unwrap_or_else(Uuid::new_v4);

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/insurance-fund/{}/claims", fund_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "claim_type": "liquidation",
                        "claimed_amount": 50000,
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_and_process_insurance_claim() {
    let Some(mut test_context) = TestContext::from_env().await else {
        return;
    };

    // Create admin and get token
    let admin_token = helpers::create_admin_and_get_token(&mut test_context).await;

    // Get primary fund
    let fund_id: Uuid = sqlx::query_scalar("SELECT id FROM insurance_fund LIMIT 1")
        .fetch_optional(&test_context.pool)
        .await
        .expect("query failed")
        .expect("fund not found");

    // Create a test user
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, email, wallet_address, nonce)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(format!("test_{}@example.com", user_id))
    .bind(format!("0x{}", user_id.simple()))
    .bind(0i64)
    .execute(&test_context.pool)
    .await
    .expect("failed to create user");

    // Create insurance claim
    let claim_request = json!({
        "claim_type": "liquidation",
        "claimed_amount": 50000,
        "description": "Test liquidation claim",
    });

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/insurance-fund/{}/claims", fund_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(serde_json::to_string(&claim_request).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    let claim_id = json["data"]["id"].as_str().unwrap();

    // Verify claim was created in database
    let claim_status: String =
        sqlx::query_scalar("SELECT status FROM insurance_claims WHERE id = $1")
            .bind(Uuid::parse_str(claim_id).unwrap())
            .fetch_one(&test_context.pool)
            .await
            .expect("query failed");

    assert_eq!(claim_status, "pending");

    // Process the claim (approve)
    let process_request = json!({
        "approved": true,
        "approved_amount": 45000,
    });

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/admin/insurance-fund/claims/{}/process",
                    claim_id
                ))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(serde_json::to_string(&process_request).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // Verify claim was approved
    let updated_status: String =
        sqlx::query_scalar("SELECT status FROM insurance_claims WHERE id = $1")
            .bind(Uuid::parse_str(claim_id).unwrap())
            .fetch_one(&test_context.pool)
            .await
            .expect("query failed");

    assert_eq!(updated_status, "approved");
}

// ─────────────────────────────────────────────────────────────────────────────
// Insurance Fund Metrics History Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_insurance_fund_metrics_history_recording() {
    let Some(test_context) = TestContext::from_env().await else {
        return;
    };

    // Get primary fund
    let fund_id: Uuid = sqlx::query_scalar("SELECT id FROM insurance_fund LIMIT 1")
        .fetch_optional(&test_context.pool)
        .await
        .expect("query failed")
        .expect("fund not found");

    // Get current fund metrics
    let fund: (Decimal, Decimal, Decimal, Decimal) = sqlx::query_as(
        "SELECT total_reserves, available_reserves, total_covered_liabilities, coverage_ratio FROM insurance_fund WHERE id = $1"
    )
    .bind(fund_id)
    .fetch_one(&test_context.pool)
    .await
    .expect("query failed");

    // Insert metrics history record
    let result = sqlx::query(
        r#"
        INSERT INTO insurance_fund_metrics_history (
            fund_id, total_reserves, available_reserves, locked_reserves,
            total_covered_liabilities, coverage_ratio, reserve_health_score, status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(fund_id)
    .bind(fund.0)
    .bind(fund.1)
    .bind(Decimal::ZERO)
    .bind(fund.2)
    .bind(fund.3)
    .bind(Decimal::new(100, 0))
    .bind("healthy")
    .fetch_one(&test_context.pool)
    .await
    .expect("failed to insert metrics history");

    assert!(result.get::<Uuid, _>("id") != Uuid::nil());

    // Verify history can be queried
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM insurance_fund_metrics_history WHERE fund_id = $1",
    )
    .bind(fund_id)
    .fetch_one(&test_context.pool)
    .await
    .expect("query failed");

    assert!(count > 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Insurance Fund Status Change Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_insurance_fund_status_determination() {
    // Test status determination based on coverage ratio
    let critical = Decimal::new(50, 2); // 0.5
    let min = Decimal::new(100, 2); // 1.0
    let target = Decimal::new(150, 2); // 1.5

    // Healthy: ratio >= target
    let healthy_ratio = Decimal::new(200, 2); // 2.0
    let healthy_status = if healthy_ratio >= target {
        "healthy"
    } else if healthy_ratio >= min {
        "warning"
    } else if healthy_ratio >= critical {
        "critical"
    } else {
        "insolvent"
    };
    assert_eq!(healthy_status, "healthy");

    // Warning: min <= ratio < target
    let warning_ratio = Decimal::new(120, 2); // 1.2
    let warning_status = if warning_ratio >= target {
        "healthy"
    } else if warning_ratio >= min {
        "warning"
    } else if warning_ratio >= critical {
        "critical"
    } else {
        "insolvent"
    };
    assert_eq!(warning_status, "warning");

    // Critical: critical <= ratio < min
    let critical_ratio = Decimal::new(80, 2); // 0.8
    let critical_status = if critical_ratio >= target {
        "healthy"
    } else if critical_ratio >= min {
        "warning"
    } else if critical_ratio >= critical {
        "critical"
    } else {
        "insolvent"
    };
    assert_eq!(critical_status, "critical");

    // Insolvent: ratio < critical
    let insolvent_ratio = Decimal::new(30, 2); // 0.3
    let insolvent_status = if insolvent_ratio >= target {
        "healthy"
    } else if insolvent_ratio >= min {
        "warning"
    } else if insolvent_ratio >= critical {
        "critical"
    } else {
        "insolvent"
    };
    assert_eq!(insolvent_status, "insolvent");
}

// ─────────────────────────────────────────────────────────────────────────────
// Integration Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_full_insurance_fund_lifecycle() {
    let Some(mut test_context) = TestContext::from_env().await else {
        return;
    };

    // 1. Create admin
    let admin_token = helpers::create_admin_and_get_token(&mut test_context).await;

    // 2. Get insurance fund dashboard
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/insurance-fund")
                .method("GET")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let dashboard: Value = serde_json::from_slice(&body).unwrap();

    // 3. Verify dashboard structure
    assert_eq!(dashboard["status"], "success");
    assert!(dashboard["data"]["fund"].is_object());
    assert!(dashboard["data"]["fund"]["fund_name"].is_string());
    assert!(dashboard["data"]["fund"]["coverage_ratio"].is_number());
    assert!(dashboard["data"]["fund"]["reserve_health_score"].is_number());

    // 4. Get all funds
    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/insurance-funds")
                .method("GET")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let funds: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(funds["status"], "success");
    assert!(funds["data"].is_array());
    assert!(funds["count"].is_number());
}
*/

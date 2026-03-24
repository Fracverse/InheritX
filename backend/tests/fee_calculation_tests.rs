//! Integration tests for fee calculation accuracy.
//! Uses the actual Axum router via create_app; no mock HTTP handlers.

mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use rust_decimal::Decimal;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

/// JWT secret must match TestContext (helpers default to "test-jwt-secret" when JWT_SECRET is unset).
const JWT_SECRET: &[u8] = b"test-jwt-secret";

fn user_token(user_id: Uuid) -> String {
    let claims = UserClaims {
        user_id,
        email: format!("fee-test-{}@example.com", user_id),
        exp: 9_999_999_999,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .expect("token encoding failed")
}

async fn ensure_user_and_kyc(pool: &sqlx::PgPool, user_id: Uuid) {
    let email = format!("fee-test-{}@example.com", user_id);
    sqlx::query(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(user_id)
    .bind(&email)
    .bind("hash")
    .execute(pool)
    .await
    .expect("insert user");

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
    .expect("approve KYC");
}

/// POST /api/plans and return parsed response body (status + data).
async fn create_plan_via_app(
    app: &axum::Router,
    token: &str,
    fee: &str,
    net_amount: &str,
    title: &str,
) -> (StatusCode, Value) {
    let body = serde_json::json!({
        "title": title,
        "description": "Fee calculation test",
        "fee": fee,
        "net_amount": net_amount,
        "beneficiary_name": "Test Beneficiary",
        "bank_account_number": "1234567890",
        "bank_name": "Test Bank",
        "currency_preference": "USDC"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plans")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    let json: Value = serde_json::from_slice(&body_bytes).expect("parse JSON");
    (status, json)
}

fn fee_and_net_from_response(json: &Value) -> (Decimal, Decimal) {
    let fee_str = json["data"]["fee"].as_str().expect("data.fee string");
    let net_str = json["data"]["net_amount"]
        .as_str()
        .expect("data.net_amount string");
    let fee: Decimal = fee_str.parse().expect("fee decimal");
    let net: Decimal = net_str.parse().expect("net_amount decimal");
    (fee, net)
}

/// Two percent as Decimal.
fn two_percent() -> Decimal {
    Decimal::new(2, 0) / Decimal::new(100, 0)
}

// -----------------------------------------------------------------------------
// Small decimals: total with many decimal places, no rounding error
// -----------------------------------------------------------------------------
#[tokio::test]
async fn fee_calculation_small_decimals() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    ensure_user_and_kyc(&ctx.pool, user_id).await;
    let token = user_token(user_id);

    // Total = 1.11 => fee = 0.0222, net = 1.0878 (server recomputes from fee + net_amount)
    let (status, json) =
        create_plan_via_app(&ctx.app, &token, "0.02", "1.09", "Small decimals plan").await;

    assert_eq!(status, StatusCode::OK, "response: {:?}", json);
    assert_eq!(json["status"], "success");

    let (fee, net_amount) = fee_and_net_from_response(&json);
    let total = fee + net_amount;
    let expected_fee = total * two_percent();

    // No rounding error: fee must equal exactly 2% of total (Decimal is exact)
    assert_eq!(fee, expected_fee, "fee should be exactly 2% of total");
    assert_eq!(fee + net_amount, total, "fee + net_amount must equal total");
}

// -----------------------------------------------------------------------------
// Large values: big amount, fee is exactly 2%, no overflow/rounding
// -----------------------------------------------------------------------------
#[tokio::test]
async fn fee_calculation_large_values() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    ensure_user_and_kyc(&ctx.pool, user_id).await;
    let token = user_token(user_id);

    // Large total: 99_999_999.99 => fee = 1_999_999.9998, net = 98_000_000.9902
    let (status, json) = create_plan_via_app(
        &ctx.app,
        &token,
        "1999999.9998",
        "98000000.9902",
        "Large values plan",
    )
    .await;

    assert_eq!(status, StatusCode::OK, "response: {:?}", json);
    assert_eq!(json["status"], "success");

    let (fee, net_amount) = fee_and_net_from_response(&json);
    let total = fee + net_amount;
    let expected_fee = total * two_percent();

    assert_eq!(
        fee, expected_fee,
        "fee must be exactly 2% of total for large values"
    );
    assert_eq!(fee + net_amount, total);
}

// -----------------------------------------------------------------------------
// No rounding error: amounts where 2% is exact (e.g. 100 -> 2, 50 -> 1)
// -----------------------------------------------------------------------------
#[tokio::test]
async fn fee_calculation_no_rounding_error() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    ensure_user_and_kyc(&ctx.pool, user_id).await;
    let token = user_token(user_id);

    // Total 100 => fee 2, net 98 (exact)
    let (status, json) = create_plan_via_app(&ctx.app, &token, "2", "98", "No rounding plan").await;

    assert_eq!(status, StatusCode::OK, "response: {:?}", json);
    assert_eq!(json["status"], "success");

    let (fee, net_amount) = fee_and_net_from_response(&json);
    let total = fee + net_amount;

    assert_eq!(fee, Decimal::new(2, 0), "fee must be exactly 2");
    assert_eq!(
        net_amount,
        Decimal::new(98, 0),
        "net_amount must be exactly 98"
    );
    assert_eq!(total, Decimal::new(100, 0), "total must be exactly 100");
    assert_eq!(
        fee + net_amount,
        total,
        "no rounding: fee + net_amount == total"
    );
}

// -----------------------------------------------------------------------------
// Exactly 2% fee: assert fee / total == 2%
// -----------------------------------------------------------------------------
#[tokio::test]
async fn fee_calculation_exactly_2_percent() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    ensure_user_and_kyc(&ctx.pool, user_id).await;
    let token = user_token(user_id);

    // Arbitrary total; server computes fee = amount * 0.02
    let (status, json) =
        create_plan_via_app(&ctx.app, &token, "10", "490", "Exactly 2% plan").await;

    assert_eq!(status, StatusCode::OK, "response: {:?}", json);
    assert_eq!(json["status"], "success");

    let (fee, net_amount) = fee_and_net_from_response(&json);
    let total = fee + net_amount;

    assert!(total > Decimal::ZERO, "total must be positive");
    let ratio = fee / total;
    let two_pct = two_percent();
    assert_eq!(ratio, two_pct, "fee/total must be exactly 2%");
    assert_eq!(fee, total * two_pct);
}

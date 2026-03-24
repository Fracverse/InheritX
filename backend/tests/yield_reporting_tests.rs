mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use inheritx_backend::auth::AdminClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

fn generate_admin_token(admin_id: Uuid) -> String {
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
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

async fn seed_user(ctx: &helpers::TestContext) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user_id)
    .bind(format!("{}@example.com", user_id))
    .bind("hashed-password")
    .execute(&ctx.pool)
    .await
    .expect("failed to insert user");

    user_id
}

async fn seed_plan(ctx: &helpers::TestContext, user_id: Uuid, asset_code: &str) -> Uuid {
    let plan_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO plans (id, user_id, title, fee, net_amount, status, asset_code)
        VALUES ($1, $2, $3, $4, $5, 'pending', $6)
        "#,
    )
    .bind(plan_id)
    .bind(user_id)
    .bind(format!("Yield Plan {}", plan_id))
    .bind("10.00")
    .bind("1000.00")
    .bind(asset_code)
    .execute(&ctx.pool)
    .await
    .expect("failed to insert plan");

    plan_id
}

#[allow(clippy::too_many_arguments)]
async fn seed_interest_accrual(
    ctx: &helpers::TestContext,
    user_id: Uuid,
    plan_id: Uuid,
    asset_code: &str,
    amount: &str,
    interest_rate: &str,
    principal_balance: &str,
    event_timestamp: chrono::DateTime<Utc>,
) {
    let metadata = json!({
        "interest_rate": interest_rate,
        "principal_balance": principal_balance,
        "accrued_interest": amount,
        "total_balance": principal_balance,
    });

    sqlx::query(
        r#"
        INSERT INTO lending_events (
            id, event_type, user_id, plan_id, asset_code, amount, metadata, event_timestamp
        )
        VALUES ($1, 'interest_accrual', $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(plan_id)
    .bind(asset_code)
    .bind(amount)
    .bind(metadata)
    .bind(event_timestamp)
    .execute(&ctx.pool)
    .await
    .expect("failed to insert interest accrual event");
}

#[tokio::test]
async fn admin_can_fetch_yield_summary_for_asset_vault() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_admin_token(Uuid::new_v4());
    let asset_code = format!("YT{}", &Uuid::new_v4().simple().to_string()[..8]).to_uppercase();
    let user_one = seed_user(&ctx).await;
    let user_two = seed_user(&ctx).await;
    let plan_one = seed_plan(&ctx, user_one, &asset_code).await;
    let plan_two = seed_plan(&ctx, user_two, &asset_code).await;
    let now = Utc::now();

    seed_interest_accrual(
        &ctx,
        user_one,
        plan_one,
        &asset_code,
        "25.00",
        "0.10",
        "1000.00",
        now,
    )
    .await;
    seed_interest_accrual(
        &ctx,
        user_one,
        plan_one,
        &asset_code,
        "10.00",
        "0.08",
        "900.00",
        now - Duration::days(1),
    )
    .await;
    seed_interest_accrual(
        &ctx,
        user_two,
        plan_two,
        &asset_code,
        "15.00",
        "5.0",
        "500.00",
        now - Duration::hours(2),
    )
    .await;

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/admin/analytics/yield?assetCode={asset_code}"))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("response is not valid JSON");

    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["filters"]["assetCode"], asset_code);
    assert_eq!(json["data"]["totalRealizedYield"], 50.0);
    assert_eq!(json["data"]["vaults"].as_array().map(Vec::len), Some(1));
    assert_eq!(json["data"]["vaults"][0]["assetCode"], json!(asset_code));
    assert_eq!(json["data"]["vaults"][0]["realizedYield"], 50.0);
    assert_eq!(json["data"]["vaults"][0]["totalPrincipalBalance"], 1500.0);
    assert_eq!(json["data"]["vaults"][0]["accrualEventCount"], 3);
    assert_eq!(json["data"]["vaults"][0]["onChainYield"], 0.0);

    let apy = json["data"]["vaults"][0]["apy"]
        .as_f64()
        .expect("apy should be numeric");
    assert!(apy > 8.6 && apy < 8.8, "unexpected APY: {apy}");
}

#[tokio::test]
async fn admin_can_fetch_filtered_earnings_history() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_admin_token(Uuid::new_v4());
    let asset_code = format!("YH{}", &Uuid::new_v4().simple().to_string()[..8]).to_uppercase();
    let other_asset = format!("YO{}", &Uuid::new_v4().simple().to_string()[..8]).to_uppercase();
    let user_id = seed_user(&ctx).await;
    let other_user_id = seed_user(&ctx).await;
    let plan_id = seed_plan(&ctx, user_id, &asset_code).await;
    let other_plan_id = seed_plan(&ctx, other_user_id, &other_asset).await;
    let now = Utc::now();

    seed_interest_accrual(
        &ctx,
        user_id,
        plan_id,
        &asset_code,
        "12.50",
        "0.07",
        "700.00",
        now - Duration::days(3),
    )
    .await;
    seed_interest_accrual(
        &ctx,
        user_id,
        plan_id,
        &asset_code,
        "7.50",
        "0.07",
        "700.00",
        now - Duration::days(1),
    )
    .await;
    seed_interest_accrual(
        &ctx,
        other_user_id,
        other_plan_id,
        &other_asset,
        "99.00",
        "0.20",
        "1000.00",
        now - Duration::days(1),
    )
    .await;

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/admin/analytics/yield/history?range=daily&assetCode={asset_code}&userId={user_id}&planId={plan_id}"
                ))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("response is not valid JSON");

    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["range"], "daily");
    assert_eq!(json["data"]["filters"]["assetCode"], asset_code);
    assert_eq!(json["data"]["filters"]["userId"], user_id.to_string());
    assert_eq!(json["data"]["filters"]["planId"], plan_id.to_string());
    assert_eq!(json["data"]["totalEarnings"], 20.0);

    let history = json["data"]["history"]
        .as_array()
        .expect("history should be an array");
    assert_eq!(history.len(), 2);
    assert!(history
        .iter()
        .all(|point| point["assetCode"] == json!(asset_code)));
    assert!(history
        .iter()
        .all(|point| point["earnings"].as_f64().unwrap() > 0.0));
}

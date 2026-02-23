mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

fn generate_user_token(user_id: Uuid) -> String {
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = UserClaims {
        user_id,
        email: format!("test-{}@example.com", user_id),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .expect("Failed to generate user token")
}

fn generate_test_token(user_id: Uuid, email: &str) -> String {
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
    .expect("Failed to generate test token")
}

async fn approve_kyc_direct(pool: &sqlx::PgPool, user_id: Uuid) {
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

#[tokio::test]
async fn test_claim_before_maturity_returns_400() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        println!("SKIPPING TEST: no database connection");
        return;
    };

    let pool = test_context.pool.clone();
    let app = test_context.app;

    let user_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", user_id);
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&email)
        .bind("hashed_password")
        .execute(&pool)
        .await
        .expect("Failed to insert user");

    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'approved')")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Failed to approve KYC");

    let plan_id = Uuid::new_v4();
    let now_ts = Utc::now().timestamp();
    sqlx::query(
        r#"
        INSERT INTO plans (
            id, user_id, title, description, fee, net_amount, status,
            distribution_method, contract_created_at, currency_preference
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(plan_id)
    .bind(user_id)
    .bind("Immature Plan")
    .bind("Description")
    .bind("0.00")
    .bind("100.00")
    .bind("pending")
    .bind("Monthly")
    .bind(now_ts)
    .bind("USDC")
    .execute(&pool)
    .await
    .expect("Failed to insert immature plan");

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind listener");
    let addr = listener.local_addr().expect("Failed to get listener addr");

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("Server failed");
    });

    let token = generate_test_token(user_id, &email);
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{}/api/plans/{}/claim", addr, plan_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "beneficiary_email": "beneficiary@example.com" }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: Value = response
        .json()
        .await
        .expect("Failed to parse claim response");
    assert_eq!(
        body["error"],
        "Bad Request: Plan is not yet mature for claim"
    );
}

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
                .body(Body::from(
                    serde_json::to_string(&body).expect("Failed to serialize request body"),
                ))
                .expect("Failed to build request"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse JSON");
    assert_eq!(json["status"], "success");
}

#[tokio::test]
async fn test_claim_requires_kyc_approved() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);
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
                .body(Body::from(
                    serde_json::to_string(&body).expect("Failed to serialize request body"),
                ))
                .expect("Failed to build request"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

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
                .body(Body::from(
                    serde_json::to_string(&body).expect("Failed to serialize request body"),
                ))
                .expect("Failed to build request"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse JSON");
    assert_eq!(json["status"], "success");
    assert_eq!(json["message"], "Claim recorded");

    let claim_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM claims WHERE plan_id = $1")
        .bind(plan_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to query claims table");

    assert_eq!(claim_count, 1);
}

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
                .body(Body::from(
                    serde_json::to_string(&body).expect("Failed to serialize request body"),
                ))
                .expect("Failed to build request"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

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

    assert_eq!(log_count, 1);
}

#[tokio::test]
async fn test_claim_notification_created() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    approve_kyc_direct(&ctx.pool, user_id).await;
    let plan_id = insert_due_plan(&ctx.pool, user_id).await;

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
                .body(Body::from(
                    serde_json::to_string(&body).expect("Failed to serialize request body"),
                ))
                .expect("Failed to build request"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND type = 'plan_claimed'",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count notifications after claim");

    assert!(after > before);
}

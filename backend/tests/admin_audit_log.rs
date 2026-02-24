mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::{AdminClaims, UserClaims};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

fn generate_user_token(user_id: Uuid) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = UserClaims {
        user_id,
        email: format!("test-{}@example.com", user_id),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-jwt-secret"),
    )
    .expect("Failed to generate user token")
}

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

#[tokio::test]
async fn admin_can_fetch_logs() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let admin_id = Uuid::new_v4();
    let token = generate_admin_token(admin_id);

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn user_cannot_fetch_logs() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // Since AuthenticatedAdmin expects AdminClaims, a user token will fail to parse and return 401
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn user_cannot_access_kyc_status() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/admin/kyc/{}", user_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // Since AuthenticatedAdmin expects AdminClaims, a user token will fail to parse and return 401
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn user_cannot_approve_kyc() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/kyc/approve")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"user_id": user_id}).to_string()))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // Since AuthenticatedAdmin expects AdminClaims, a user token will fail to parse and return 401
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn user_cannot_reject_kyc() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/kyc/reject")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"user_id": user_id}).to_string()))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // Since AuthenticatedAdmin expects AdminClaims, a user token will fail to parse and return 401
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[allow(dead_code)]
async fn log_inserted_on_plan_create_and_claim() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();

    // Insert user
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .bind("hash")
        .execute(&ctx.pool)
        .await
        .expect("Failed to create user");

    // 1. Approve KYC first (and verify KYC log)
    let admin_token = generate_admin_token(admin_id);
    let kyc_req = json!({ "user_id": user_id });

    let kyc_res = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/kyc/approve")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(kyc_req.to_string()))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(kyc_res.status(), StatusCode::OK);

    // Verify KYC Approved log
    let kyc_log: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM action_logs WHERE user_id = $1 AND action = 'kyc_approved'",
    )
    .bind(admin_id) // admin performed the action
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to query action logs");
    assert_eq!(kyc_log, 1, "Expected 1 kyc_approved log");

    // 2. Create Plan (and verify plan_created log)
    let user_token = generate_user_token(user_id);
    let plan_req = json!({
        "title": "Test Plan",
        "fee": 2.0,
        "net_amount": 98.0,
        "currency_preference": "USDC",
        "beneficiary_name": "Bene Fish",
        "bank_name": "",
        "bank_account_number": ""
    });

    let create_app = ctx.app.clone();
    let create_res = create_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plans")
                .header("Authorization", format!("Bearer {}", user_token))
                .header("Content-Type", "application/json")
                .body(Body::from(plan_req.to_string()))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(create_res.status(), StatusCode::OK);
    let plan_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM plans WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to get plan");

    let plan_log: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM action_logs WHERE user_id = $1 AND action = 'plan_created' AND entity_id = $2")
        .bind(user_id)
        .bind(plan_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to query action logs");
    assert_eq!(plan_log, 1, "Expected 1 plan_created log");

    // 3. Claim Plan (and verify plan_claimed log)
    let claim_req = json!({
        "beneficiary_email": "bene@example.com"
    });

    let claim_res = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/plans/{}/claim", plan_id))
                .header("Authorization", format!("Bearer {}", user_token))
                .header("Content-Type", "application/json")
                .body(Body::from(claim_req.to_string()))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(claim_res.status(), StatusCode::OK);

    let claim_log: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM action_logs WHERE user_id = $1 AND action = 'plan_claimed' AND entity_id = $2")
        .bind(user_id)
        .bind(plan_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to query action logs");
    assert_eq!(claim_log, 1, "Expected 1 plan_claimed log");
}

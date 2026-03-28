mod helpers;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::Utc;
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &[u8] = b"secret_key_change_in_production";

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
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .expect("Failed to generate user token")
}

#[tokio::test]
async fn user_with_pending_kyc_cannot_create_plan() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Seed user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", Uuid::new_v4()))
        .bind("hash")
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 2. Submit KYC (status will be 'pending')
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'pending')")
        .bind(user_id)
        .execute(&test_context.pool)
        .await
        .unwrap();

    let token = generate_user_token(user_id);

    // 3. Attempt to create plan
    let create_plan_req = json!({
        "title": "Test Plan",
        "description": "Test Description",
        "fee": "10.00",
        "net_amount": "490.00",
        "beneficiary_name": "Test Beneficiary",
        "bank_name": "Test Bank",
        "bank_account_number": "1234567890",
        "currency_preference": "FIAT",
        "two_fa_code": "123456"
    });

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/api/plans")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_plan_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("KYC not approved"));
}

#[tokio::test]
async fn user_with_rejected_kyc_cannot_create_plan() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Seed user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", Uuid::new_v4()))
        .bind("hash")
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 2. Set KYC as rejected
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'rejected')")
        .bind(user_id)
        .execute(&test_context.pool)
        .await
        .unwrap();

    let token = generate_user_token(user_id);

    // 3. Attempt to create plan
    let create_plan_req = json!({
        "title": "Test Plan",
        "description": "Test Description",
        "fee": "10.00",
        "net_amount": "490.00",
        "beneficiary_name": "Test Beneficiary",
        "bank_name": "Test Bank",
        "bank_account_number": "1234567890",
        "currency_preference": "FIAT",
        "two_fa_code": "123456"
    });

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/api/plans")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_plan_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("KYC not approved"));
}

#[tokio::test]
async fn user_with_approved_kyc_can_create_plan() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Seed user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", Uuid::new_v4()))
        .bind("hash")
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 2. Set KYC as approved
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'approved')")
        .bind(user_id)
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 3. Setup 2FA
    let otp = "123456";
    let otp_hash = bcrypt::hash(otp, bcrypt::DEFAULT_COST).unwrap();
    let expires_at = Utc::now() + chrono::Duration::minutes(5);
    sqlx::query("INSERT INTO user_2fa (user_id, otp_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(otp_hash)
        .bind(expires_at)
        .execute(&test_context.pool)
        .await
        .unwrap();

    let token = generate_user_token(user_id);

    // 4. Create plan
    let create_plan_req = json!({
        "title": "Test Plan",
        "description": "Test Description",
        "fee": "10.00",
        "net_amount": "490.00",
        "beneficiary_name": "Test Beneficiary",
        "bank_name": "Test Bank",
        "bank_account_number": "1234567890",
        "currency_preference": "FIAT",
        "two_fa_code": "123456"
    });

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri("/api/plans")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_plan_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should succeed
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("Test Plan"));
}

#[tokio::test]
async fn user_with_pending_kyc_cannot_claim_plan() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Seed user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", Uuid::new_v4()))
        .bind("hash")
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 2. Set KYC as pending
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'pending')")
        .bind(user_id)
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 3. Create a mature plan
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
    .bind("Test Plan")
    .bind("Test Description")
    .bind("10.00")
    .bind("490.00")
    .bind("Test Beneficiary")
    .bind("1234567890")
    .bind("Test Bank")
    .bind("USDC")
    .bind(past_ts)
    .execute(&test_context.pool)
    .await
    .unwrap();

    // 4. Setup 2FA
    let otp = "123456";
    let otp_hash = bcrypt::hash(otp, bcrypt::DEFAULT_COST).unwrap();
    let expires_at = Utc::now() + chrono::Duration::minutes(5);
    sqlx::query("INSERT INTO user_2fa (user_id, otp_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(otp_hash)
        .bind(expires_at)
        .execute(&test_context.pool)
        .await
        .unwrap();

    let token = generate_user_token(user_id);

    // 5. Attempt to claim plan
    let claim_req = json!({
        "beneficiary_email": "beneficiary@example.com"
    });

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri(format!("/api/plans/{}/claim", plan_id))
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(claim_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("KYC not approved"));
}

#[tokio::test]
async fn user_with_rejected_kyc_cannot_claim_plan() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Seed user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", Uuid::new_v4()))
        .bind("hash")
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 2. Set KYC as rejected
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'rejected')")
        .bind(user_id)
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 3. Create a mature plan
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
    .bind("Test Plan")
    .bind("Test Description")
    .bind("10.00")
    .bind("490.00")
    .bind("Test Beneficiary")
    .bind("1234567890")
    .bind("Test Bank")
    .bind("USDC")
    .bind(past_ts)
    .execute(&test_context.pool)
    .await
    .unwrap();

    // 4. Setup 2FA
    let otp = "123456";
    let otp_hash = bcrypt::hash(otp, bcrypt::DEFAULT_COST).unwrap();
    let expires_at = Utc::now() + chrono::Duration::minutes(5);
    sqlx::query("INSERT INTO user_2fa (user_id, otp_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(otp_hash)
        .bind(expires_at)
        .execute(&test_context.pool)
        .await
        .unwrap();

    let token = generate_user_token(user_id);

    // 5. Attempt to claim plan
    let claim_req = json!({
        "beneficiary_email": "beneficiary@example.com"
    });

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri(format!("/api/plans/{}/claim", plan_id))
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(claim_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("KYC not approved"));
}

#[tokio::test]
async fn user_with_approved_kyc_can_claim_plan() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // 1. Seed user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", Uuid::new_v4()))
        .bind("hash")
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 2. Set KYC as approved
    sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'approved')")
        .bind(user_id)
        .execute(&test_context.pool)
        .await
        .unwrap();

    // 3. Create a mature plan
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
    .bind("Test Plan")
    .bind("Test Description")
    .bind("10.00")
    .bind("490.00")
    .bind("Test Beneficiary")
    .bind("1234567890")
    .bind("Test Bank")
    .bind("USDC")
    .bind(past_ts)
    .execute(&test_context.pool)
    .await
    .unwrap();

    // 4. Setup 2FA
    let otp = "123456";
    let otp_hash = bcrypt::hash(otp, bcrypt::DEFAULT_COST).unwrap();
    let expires_at = Utc::now() + chrono::Duration::minutes(5);
    sqlx::query("INSERT INTO user_2fa (user_id, otp_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(otp_hash)
        .bind(expires_at)
        .execute(&test_context.pool)
        .await
        .unwrap();

    let token = generate_user_token(user_id);

    // 5. Claim plan
    let claim_req = json!({
        "beneficiary_email": "beneficiary@example.com"
    });

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .uri(format!("/api/plans/{}/claim", plan_id))
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(claim_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should succeed
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("Claim recorded"));
}

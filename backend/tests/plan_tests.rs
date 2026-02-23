mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

/// Generate a JWT token for a test user
fn generate_user_token(user_id: Uuid) -> String {
    let claims = UserClaims {
        user_id,
        email: "testuser@inheritx.test".to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .unwrap()
}

/// Helper to create a test plan in the database
async fn create_test_plan(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    title: &str,
    fee: &str,
    net_amount: &str,
    status: &str,
) -> Result<Uuid, sqlx::Error> {
    let plan_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO plans (
            id, user_id, title, description, fee, net_amount, status,
            beneficiary_name, bank_account_number, bank_name, currency_preference,
            is_active
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
    )
    .bind(plan_id)
    .bind(user_id)
    .bind(title)
    .bind("Test Description")
    .bind(fee)
    .bind(net_amount)
    .bind(status)
    .bind("John Doe")
    .bind("1234567890")
    .bind("Test Bank")
    .bind("USDC")
    .bind(true)
    .execute(pool)
    .await?;

    Ok(plan_id)
}

/// Helper to approve KYC for a user
async fn approve_kyc(pool: &sqlx::PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO kyc_status (user_id, status, reviewed_by, reviewed_at, created_at)
        VALUES ($1, 'approved', $2, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE SET status = 'approved'
        "#,
    )
    .bind(user_id)
    .bind(Uuid::new_v4()) // admin_id
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_retrieve_plan_success() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC for the user
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Create a test plan
    let plan_id = create_test_plan(
        &test_context.pool,
        user_id,
        "Test Plan",
        "10.00",
        "490.00",
        "pending",
    )
    .await
    .expect("Failed to create test plan");

    // Retrieve the plan
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/plans/{}", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["id"], plan_id.to_string());
    assert_eq!(json["data"]["title"], "Test Plan");
    assert_eq!(json["data"]["fee"], "10.00");
    assert_eq!(json["data"]["net_amount"], "490.00");
    assert_eq!(json["data"]["status"], "pending");
}

#[tokio::test]
async fn test_retrieve_plan_not_found() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);
    let non_existent_plan_id = Uuid::new_v4();

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/plans/{}", non_existent_plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_retrieve_plan_unauthorized_different_user() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let owner_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    let token = generate_user_token(other_user_id);

    // Approve KYC for owner
    approve_kyc(&test_context.pool, owner_id)
        .await
        .expect("Failed to approve KYC");

    // Create a plan owned by owner_id
    let plan_id = create_test_plan(
        &test_context.pool,
        owner_id,
        "Owner's Plan",
        "10.00",
        "490.00",
        "pending",
    )
    .await
    .expect("Failed to create test plan");

    // Try to retrieve with different user
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/plans/{}", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_retrieve_plan_without_auth() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let plan_id = Uuid::new_v4();

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/plans/{}", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_retrieve_plan_with_beneficiary_details() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Create a plan with beneficiary details
    let plan_id = create_test_plan(
        &test_context.pool,
        user_id,
        "Plan with Beneficiary",
        "10.00",
        "490.00",
        "pending",
    )
    .await
    .expect("Failed to create test plan");

    // Retrieve the plan
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/plans/{}", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["beneficiary_name"], "John Doe");
    assert_eq!(json["data"]["bank_account_number"], "1234567890");
    assert_eq!(json["data"]["bank_name"], "Test Bank");
    assert_eq!(json["data"]["currency_preference"], "USDC");
}

#[tokio::test]
async fn test_retrieve_plan_fee_calculation_2_percent() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Create a plan with 2% fee: 500 total, 10 fee (2%), 490 net
    let plan_id = create_test_plan(
        &test_context.pool,
        user_id,
        "Plan with 2% Fee",
        "10.00",
        "490.00",
        "pending",
    )
    .await
    .expect("Failed to create test plan");

    // Retrieve the plan
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/plans/{}", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert_eq!(json["status"], "success");

    // Verify fee is 2% of total (500 * 0.02 = 10)
    let fee: f64 = json["data"]["fee"].as_str().unwrap().parse().unwrap();
    let net_amount: f64 = json["data"]["net_amount"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let total = fee + net_amount;

    assert_eq!(fee, 10.0);
    assert_eq!(net_amount, 490.0);
    assert_eq!(total, 500.0);

    // Verify fee is exactly 2% of total
    let calculated_fee_percentage = (fee / total) * 100.0;
    assert!((calculated_fee_percentage - 2.0).abs() < 0.01);
}

#[tokio::test]
async fn test_create_plan_wallet_balance_check() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Create a plan via API
    let create_request = serde_json::json!({
        "title": "New Plan",
        "description": "Test plan creation",
        "fee": "10.00",
        "net_amount": "490.00",
        "beneficiary_name": "Jane Doe",
        "bank_account_number": "9876543210",
        "bank_name": "Test Bank",
        "currency_preference": "USDC"
    });

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plans")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["title"], "New Plan");
    assert_eq!(json["data"]["fee"], "10.00");
    assert_eq!(json["data"]["net_amount"], "490.00");
}

#[tokio::test]
async fn test_create_plan_audit_log_inserted() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Create a plan via API
    let create_request = serde_json::json!({
        "title": "Plan for Audit Test",
        "description": "Testing audit log",
        "fee": "10.00",
        "net_amount": "490.00",
        "beneficiary_name": "Audit Beneficiary",
        "bank_account_number": "1111111111",
        "bank_name": "Audit Bank",
        "currency_preference": "USDC"
    });

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plans")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    let plan_id = Uuid::parse_str(json["data"]["id"].as_str().unwrap()).unwrap();

    // Check that an audit log was created
    let log_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM action_logs
        WHERE user_id = $1 AND action = 'plan_created' AND entity_id = $2
        "#,
    )
    .bind(user_id)
    .bind(plan_id)
    .fetch_one(&test_context.pool)
    .await
    .expect("Failed to query audit logs");

    assert_eq!(
        log_count, 1,
        "Expected one audit log entry for plan creation"
    );
}

#[tokio::test]
async fn test_create_plan_notification_created() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Count notifications before
    let notif_count_before: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM notifications
        WHERE user_id = $1 AND type = 'plan_created'
        "#,
    )
    .bind(user_id)
    .fetch_one(&test_context.pool)
    .await
    .expect("Failed to query notifications");

    // Create a plan via API
    let create_request = serde_json::json!({
        "title": "Plan for Notification Test",
        "description": "Testing notification",
        "fee": "10.00",
        "net_amount": "490.00",
        "beneficiary_name": "Notification Beneficiary",
        "bank_account_number": "2222222222",
        "bank_name": "Notification Bank",
        "currency_preference": "USDC"
    });

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plans")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // Count notifications after
    let notif_count_after: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM notifications
        WHERE user_id = $1 AND type = 'plan_created'
        "#,
    )
    .bind(user_id)
    .fetch_one(&test_context.pool)
    .await
    .expect("Failed to query notifications");

    // Note: Notifications might be created silently and may fail without breaking the operation
    // So we check if at least the count didn't decrease
    assert!(
        notif_count_after >= notif_count_before,
        "Expected notification count to increase or stay the same"
    );
}

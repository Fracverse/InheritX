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
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = UserClaims {
        user_id,
        email: "testuser@inheritx.test".to_string(),
        exp,
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
async fn test_fetch_own_plan() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    let plan_id = create_test_plan(
        &test_context.pool,
        user_id,
        "Own Plan",
        "10.00",
        "490.00",
        "pending",
    )
    .await
    .expect("Failed to create test plan");

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
}

#[tokio::test]
async fn test_fetch_other_user_plan() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let owner_id = Uuid::new_v4();
    let visitor_id = Uuid::new_v4();
    let visitor_token = generate_user_token(visitor_id);

    approve_kyc(&test_context.pool, owner_id)
        .await
        .expect("Failed to approve KYC");

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

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/plans/{}", plan_id))
                .header("Authorization", format!("Bearer {}", visitor_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");
    assert!(json["error"].as_str().unwrap().contains("permission"));
}

#[tokio::test]
async fn test_fetch_nonexistent_plan() {
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
async fn test_fetch_invalid_uuid() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/plans/invalid-uuid-string")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    // Axum's Path extractor returns 400 for invalid UUIDs
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

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
    .bind("Test Plan")
    .bind("Test Description")
    .bind("10.00")
    .bind("490.00")
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
async fn test_cancel_plan_success() {
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
    let plan_id = create_test_plan(&test_context.pool, user_id, "pending")
        .await
        .expect("Failed to create test plan");

    // Cancel the plan
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("DELETE")
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
    assert_eq!(json["data"]["status"], "deactivated");
    assert_eq!(json["data"]["is_active"], false);
}

#[tokio::test]
async fn test_cancel_plan_not_found() {
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
                .method("DELETE")
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
async fn test_cancel_plan_unauthorized_different_user() {
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
    let plan_id = create_test_plan(&test_context.pool, owner_id, "pending")
        .await
        .expect("Failed to create test plan");

    // Try to cancel with different user
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("DELETE")
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
async fn test_cancel_plan_already_deactivated() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Create a plan that's already deactivated
    let plan_id = create_test_plan(&test_context.pool, user_id, "deactivated")
        .await
        .expect("Failed to create test plan");

    // Try to cancel again
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/plans/{}", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("already deactivated"));
}

#[tokio::test]
async fn test_cancel_plan_already_claimed() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Create a plan that's already claimed
    let plan_id = create_test_plan(&test_context.pool, user_id, "claimed")
        .await
        .expect("Failed to create test plan");

    // Try to cancel
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/plans/{}", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON");

    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Cannot cancel a plan that has been claimed"));
}

#[tokio::test]
async fn test_cancel_plan_without_auth() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let plan_id = Uuid::new_v4();

    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/plans/{}", plan_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_cancel_plan_creates_audit_log() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Create a test plan
    let plan_id = create_test_plan(&test_context.pool, user_id, "pending")
        .await
        .expect("Failed to create test plan");

    // Cancel the plan
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/plans/{}", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // Check that an audit log was created
    let log_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM action_logs
        WHERE user_id = $1 AND action = 'plan_deactivated' AND entity_id = $2
        "#,
    )
    .bind(user_id)
    .bind(plan_id)
    .fetch_one(&test_context.pool)
    .await
    .expect("Failed to query audit logs");

    assert_eq!(log_count, 1, "Expected one audit log entry");
}

#[tokio::test]
async fn test_cancel_plan_creates_notification() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Create a test plan
    let plan_id = create_test_plan(&test_context.pool, user_id, "pending")
        .await
        .expect("Failed to create test plan");

    // Cancel the plan
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/plans/{}", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // Check that a notification was created
    let notif_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM notifications
        WHERE user_id = $1 AND type = 'plan_deactivated'
        "#,
    )
    .bind(user_id)
    .fetch_one(&test_context.pool)
    .await
    .expect("Failed to query notifications");

    assert!(notif_count >= 1, "Expected at least one notification");
}

#[tokio::test]
async fn test_cancel_plan_updates_database_correctly() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);

    // Approve KYC
    approve_kyc(&test_context.pool, user_id)
        .await
        .expect("Failed to approve KYC");

    // Create a test plan
    let plan_id = create_test_plan(&test_context.pool, user_id, "pending")
        .await
        .expect("Failed to create test plan");

    // Cancel the plan
    let response = test_context
        .app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/plans/{}", plan_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the plan was updated in the database
    let (status, is_active): (String, Option<bool>) = sqlx::query_as(
        r#"
        SELECT status, is_active
        FROM plans
        WHERE id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_one(&test_context.pool)
    .await
    .expect("Failed to query plan");

    assert_eq!(status, "deactivated");
    assert_eq!(is_active, Some(false));
}

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

fn user_token(user_id: Uuid) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = UserClaims {
        user_id,
        email: format!("user-{}@example.com", user_id),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-jwt-secret"),
    )
    .expect("failed to create user token")
}

fn admin_token(admin_id: Uuid) -> String {
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
    .expect("failed to create admin token")
}

async fn create_user(pool: &sqlx::PgPool, user_id: Uuid) {
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", user_id))
        .bind("hash")
        .execute(pool)
        .await
        .expect("failed to insert user");
}

#[tokio::test]
async fn notifications_endpoint_supports_page_and_limit() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    create_user(&ctx.pool, user_id).await;

    for i in 0..15 {
        sqlx::query(
            "INSERT INTO notifications (id, user_id, type, message, is_read) VALUES ($1, $2, $3, $4, false)",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind("plan_created")
        .bind(format!("notification-{i}"))
        .execute(&ctx.pool)
        .await
        .expect("failed to insert notification");
    }

    let token = user_token(user_id);

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/notifications?page=1&limit=10")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .expect("failed to build request"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read response body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("failed to parse json");

    assert_eq!(json["data"].as_array().expect("data array").len(), 10);
    assert_eq!(json["page"], 1);
    assert_eq!(json["limit"], 10);
    assert_eq!(json["total_count"], 15);
    assert_eq!(json["total_pages"], 2);
}

#[tokio::test]
async fn admin_logs_endpoint_supports_page_and_limit() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    for _ in 0..14 {
        sqlx::query(
            "INSERT INTO action_logs (user_id, action, entity_id, entity_type) VALUES ($1, $2, $3, $4)",
        )
        .bind(Some(Uuid::new_v4()))
        .bind("plan_created")
        .bind(Some(Uuid::new_v4()))
        .bind(Some("plan"))
        .execute(&ctx.pool)
        .await
        .expect("failed to insert audit log");
    }

    let token = admin_token(Uuid::new_v4());

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs?page=1&limit=10")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .expect("failed to build request"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read response body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("failed to parse json");

    assert_eq!(json["data"].as_array().expect("data array").len(), 10);
    assert_eq!(json["page"], 1);
    assert_eq!(json["limit"], 10);
    assert_eq!(json["total_count"], 14);
    assert_eq!(json["total_pages"], 2);
}

#[tokio::test]
async fn due_plans_endpoint_supports_page_and_limit() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    create_user(&ctx.pool, user_id).await;

    let now = chrono::Utc::now().timestamp();
    for i in 0..13 {
        sqlx::query(
            r#"
            INSERT INTO plans (
                id, user_id, title, description, fee, net_amount, status,
                distribution_method, contract_created_at, is_active,
                beneficiary_name, bank_account_number, bank_name, currency_preference
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', 'LumpSum', $7, true, $8, $9, $10, $11)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(format!("plan-{i}"))
        .bind(Some("desc".to_string()))
        .bind("10.00")
        .bind("490.00")
        .bind(now)
        .bind("Jane Doe")
        .bind("123456789")
        .bind("Bank")
        .bind("USDC")
        .execute(&ctx.pool)
        .await
        .expect("failed to insert plan");
    }

    let token = user_token(user_id);

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/plans/due-for-claim?page=1&limit=10")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .expect("failed to build request"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read response body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("failed to parse json");

    assert_eq!(json["data"].as_array().expect("data array").len(), 10);
    assert_eq!(json["page"], 1);
    assert_eq!(json["limit"], 10);
    assert_eq!(json["total_count"], 13);
    assert_eq!(json["total_pages"], 2);
}

#[tokio::test]
async fn create_plan_accepts_query_pagination_params() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    create_user(&ctx.pool, user_id).await;

    sqlx::query(
        r#"
        INSERT INTO kyc_status (user_id, status, reviewed_by, reviewed_at, created_at, updated_at)
        VALUES ($1, 'approved', $2, NOW(), NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE SET status = 'approved', updated_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(Uuid::new_v4())
    .execute(&ctx.pool)
    .await
    .expect("failed to upsert kyc");

    let token = user_token(user_id);
    let body = json!({
        "title": "Paginated create plan",
        "description": "plan",
        "fee": "10.00",
        "net_amount": "490.00",
        "beneficiary_name": "Jane Doe",
        "bank_name": "",
        "bank_account_number": "",
        "currency_preference": "USDC"
    });

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/plans?page=1&limit=10")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("failed to build request"),
        )
        .await
        .expect("request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "create plan should still succeed when page/limit query params are present"
    );
}

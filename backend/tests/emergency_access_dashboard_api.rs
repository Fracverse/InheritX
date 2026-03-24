mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

fn generate_user_token(user_id: Uuid) -> String {
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
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
    .expect("failed to generate user token")
}

async fn insert_user(pool: &sqlx::PgPool, user_id: Uuid) {
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("user-{}@example.com", user_id))
        .bind("hash")
        .execute(pool)
        .await
        .expect("failed to insert user");
}

async fn insert_contact(pool: &sqlx::PgPool, user_id: Uuid, name: &str) -> Uuid {
    sqlx::query_scalar(
        r#"
        INSERT INTO emergency_contacts (user_id, name, relationship, email)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind("Sibling")
    .bind(format!(
        "{}@example.com",
        name.replace(' ', ".").to_lowercase()
    ))
    .fetch_one(pool)
    .await
    .expect("failed to insert emergency contact")
}

async fn insert_grant(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    contact_id: Uuid,
    permissions: Vec<&str>,
    expires_at: chrono::DateTime<Utc>,
    is_active: bool,
) {
    let permissions = permissions
        .into_iter()
        .map(|permission| permission.to_string())
        .collect::<Vec<_>>();

    sqlx::query(
        r#"
        INSERT INTO emergency_access_grants (
            user_id, emergency_contact_id, permissions, expires_at, is_active, revoked_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(user_id)
    .bind(contact_id)
    .bind(permissions)
    .bind(expires_at)
    .bind(is_active)
    .bind(if is_active { None } else { Some(Utc::now()) })
    .execute(pool)
    .await
    .expect("failed to insert emergency access grant");
}

#[tokio::test]
async fn dashboard_reports_active_access_permissions_and_expiration() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    insert_user(&ctx.pool, user_id).await;

    let active_contact = insert_contact(&ctx.pool, user_id, "Active Contact").await;
    let expired_contact = insert_contact(&ctx.pool, user_id, "Expired Contact").await;
    let revoked_contact = insert_contact(&ctx.pool, user_id, "Revoked Contact").await;

    insert_grant(
        &ctx.pool,
        user_id,
        active_contact,
        vec!["view_plan", "download_documents"],
        Utc::now() + Duration::hours(3),
        true,
    )
    .await;
    insert_grant(
        &ctx.pool,
        user_id,
        expired_contact,
        vec!["view_plan"],
        Utc::now() - Duration::hours(1),
        true,
    )
    .await;
    insert_grant(
        &ctx.pool,
        user_id,
        revoked_contact,
        vec!["manage_beneficiaries"],
        Utc::now() + Duration::hours(4),
        false,
    )
    .await;

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/emergency/access/dashboard")
                .header(
                    "Authorization",
                    format!("Bearer {}", generate_user_token(user_id)),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("dashboard request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read dashboard body");
    let json: Value = serde_json::from_slice(&body).expect("invalid dashboard json");

    assert_eq!(json["data"]["active_access_count"], 1);
    assert_eq!(json["data"]["grants"].as_array().unwrap().len(), 3);

    let statuses = json["data"]["grants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|grant| {
            (
                grant["contact_name"].as_str().unwrap().to_string(),
                grant["status"].as_str().unwrap().to_string(),
                grant["active_access"].as_bool().unwrap(),
            )
        })
        .collect::<Vec<_>>();

    assert!(statuses.contains(&("Active Contact".to_string(), "active".to_string(), true)));
    assert!(statuses.contains(&("Expired Contact".to_string(), "expired".to_string(), false)));
    assert!(statuses.contains(&("Revoked Contact".to_string(), "revoked".to_string(), false)));
}

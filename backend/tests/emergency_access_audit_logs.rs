mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
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

async fn insert_contact(pool: &sqlx::PgPool, user_id: Uuid) -> Uuid {
    sqlx::query_scalar(
        r#"
        INSERT INTO emergency_contacts (user_id, name, relationship, email)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind("Ada Support")
    .bind("Sibling")
    .bind("ada@example.com")
    .fetch_one(pool)
    .await
    .expect("failed to insert emergency contact")
}

#[tokio::test]
async fn grant_and_revoke_emergency_access_creates_audit_logs() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    insert_user(&ctx.pool, user_id).await;
    let contact_id = insert_contact(&ctx.pool, user_id).await;
    let token = generate_user_token(user_id);

    let create_response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/emergency/access/grants")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "emergency_contact_id": contact_id,
                        "permissions": ["view_plan", "download_documents"],
                        "expires_at": (Utc::now() + Duration::hours(6)).to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("grant request failed");

    assert_eq!(create_response.status(), StatusCode::OK);

    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("failed to read create response body");
    let create_json: Value = serde_json::from_slice(&create_body).expect("invalid create json");
    let grant_id = create_json["data"]["grant"]["id"]
        .as_str()
        .expect("grant id should be present")
        .to_string();

    let revoke_response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/emergency/access/grants/{}/revoke", grant_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "reason": "Access no longer needed"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("revoke request failed");

    assert_eq!(revoke_response.status(), StatusCode::OK);

    let logs_response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/emergency/access/audit-logs")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("logs request failed");

    assert_eq!(logs_response.status(), StatusCode::OK);

    let logs_body = axum::body::to_bytes(logs_response.into_body(), usize::MAX)
        .await
        .expect("failed to read logs body");
    let logs_json: Value = serde_json::from_slice(&logs_body).expect("invalid logs json");

    assert_eq!(logs_json["count"], 2);
    assert_eq!(logs_json["data"][0]["action"], "emergency_access_revoked");
    assert_eq!(logs_json["data"][1]["action"], "emergency_access_granted");
}

#[tokio::test]
async fn cannot_create_emergency_access_for_another_users_contact() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let owner_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    insert_user(&ctx.pool, owner_id).await;
    insert_user(&ctx.pool, other_user_id).await;
    let contact_id = insert_contact(&ctx.pool, owner_id).await;

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/emergency/access/grants")
                .header(
                    "Authorization",
                    format!("Bearer {}", generate_user_token(other_user_id)),
                )
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "emergency_contact_id": contact_id,
                        "permissions": ["view_plan"],
                        "expires_at": (Utc::now() + Duration::hours(1)).to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

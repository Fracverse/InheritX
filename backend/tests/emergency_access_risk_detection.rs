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

async fn list_alerts(app: axum::Router, token: &str) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/emergency/access/risk-alerts")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("risk alerts request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read alerts body");
    serde_json::from_slice(&body).expect("invalid alerts json")
}

#[tokio::test]
async fn high_privilege_long_lived_access_is_flagged() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    insert_user(&ctx.pool, user_id).await;
    let contact_id = insert_contact(&ctx.pool, user_id, "High Privilege Contact").await;
    let token = generate_user_token(user_id);

    let response = ctx
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
                        "permissions": ["transfer_funds", "view_plan"],
                        "expires_at": (Utc::now() + Duration::days(8)).to_rfc3339()
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("grant request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let alerts = list_alerts(ctx.app.clone(), &token).await;
    assert_eq!(alerts["count"], 1);
    assert_eq!(
        alerts["data"][0]["alert_type"],
        "high_privilege_long_lived_access"
    );
}

#[tokio::test]
async fn repeated_grants_in_short_window_are_flagged() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    insert_user(&ctx.pool, user_id).await;
    let token = generate_user_token(user_id);

    for idx in 0..4 {
        let contact_id = insert_contact(&ctx.pool, user_id, &format!("Contact {}", idx)).await;
        let response = ctx
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
                            "permissions": ["view_plan"],
                            "expires_at": (Utc::now() + Duration::hours(2)).to_rfc3339()
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("grant request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    let alerts = list_alerts(ctx.app.clone(), &token).await;
    assert!(alerts["count"].as_i64().unwrap_or_default() >= 1);
    let alert_types = alerts["data"]
        .as_array()
        .expect("alerts should be an array")
        .iter()
        .map(|item| item["alert_type"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();

    assert!(alert_types.contains(&"high_frequency_grants"));
}

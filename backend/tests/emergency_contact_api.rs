mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
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

#[tokio::test]
async fn user_can_create_list_update_and_delete_emergency_contacts() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);
    insert_user(&ctx.pool, user_id).await;

    let create_response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/emergency/contacts")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Ada Support",
                        "relationship": "Sibling",
                        "email": "ada@example.com",
                        "phone": "+2348000000000",
                        "wallet_address": "GCONTACTWALLET123",
                        "notes": "Primary fallback contact"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(create_response.status(), StatusCode::OK);

    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("failed to read create body");
    let create_json: Value = serde_json::from_slice(&create_body).expect("invalid create json");
    let contact_id = Uuid::parse_str(
        create_json["data"]["id"]
            .as_str()
            .expect("contact id should be present"),
    )
    .expect("contact id should be a valid uuid");

    let list_response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/emergency/contacts")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(list_response.status(), StatusCode::OK);

    let list_body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("failed to read list body");
    let list_json: Value = serde_json::from_slice(&list_body).expect("invalid list json");
    assert_eq!(list_json["count"], 1);
    assert_eq!(list_json["data"][0]["name"], "Ada Support");

    let update_response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/emergency/contacts/{}", contact_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Ada Backup",
                        "relationship": "Sister",
                        "email": "ada.backup@example.com",
                        "phone": "+2348111111111",
                        "wallet_address": "",
                        "notes": "Updated by integration test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(update_response.status(), StatusCode::OK);

    let update_body = axum::body::to_bytes(update_response.into_body(), usize::MAX)
        .await
        .expect("failed to read update body");
    let update_json: Value = serde_json::from_slice(&update_body).expect("invalid update json");
    assert_eq!(update_json["data"]["name"], "Ada Backup");
    assert!(update_json["data"]["wallet_address"].is_null());

    let delete_response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/emergency/contacts/{}", contact_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(delete_response.status(), StatusCode::OK);

    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM emergency_contacts WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("failed to count emergency contacts");

    assert_eq!(remaining, 0);
}

#[tokio::test]
async fn user_cannot_update_another_users_emergency_contact() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let owner_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    insert_user(&ctx.pool, owner_id).await;
    insert_user(&ctx.pool, other_user_id).await;

    let contact_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO emergency_contacts (user_id, name, relationship, email)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(owner_id)
    .bind("Owner Contact")
    .bind("Brother")
    .bind("owner@example.com")
    .fetch_one(&ctx.pool)
    .await
    .expect("failed to create emergency contact");

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/emergency/contacts/{}", contact_id))
                .header(
                    "Authorization",
                    format!("Bearer {}", generate_user_token(other_user_id)),
                )
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Intruder Edit",
                        "relationship": "Friend",
                        "email": "intruder@example.com"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn emergency_contact_requires_at_least_one_contact_channel() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    insert_user(&ctx.pool, user_id).await;

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/emergency/contacts")
                .header(
                    "Authorization",
                    format!("Bearer {}", generate_user_token(user_id)),
                )
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Incomplete Contact",
                        "relationship": "Cousin"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

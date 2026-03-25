mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
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

#[tokio::test]
async fn emergency_access_grant_rate_limiting_works() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let user_id = Uuid::new_v4();
    let token = generate_user_token(user_id);
    let contact_id = Uuid::new_v4(); // Doesn't need to exist for rate limit check (checked before handler)

    // First request - should be OK (or at least NOT 429)
    // Note: It might return 401 or 404 depending on existence, but we care about 429
    let mut responses = Vec::new();

    for _ in 0..5 {
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
            .expect("request failed");
        
        responses.push(response.status());
    }

    // With burst of 2 and slow refill (1/min), at least some should be 429
    let has_429 = responses.iter().any(|s| *s == StatusCode::TOO_MANY_REQUESTS);
    assert!(has_429, "Expected at least one 429 Too Many Requests response, but got: {:?}", responses);
}

mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use tower::ServiceExt; // for `oneshot`
use uuid::Uuid;

/// Matches the hardcoded secret used in `AuthenticatedUser` and `AuthenticatedAdmin`
/// extractors in `src/auth.rs`.
const JWT_SIGNING_SECRET: &[u8] = b"secret_key_change_in_production";

/// Mirror of `UserClaims` with an added `exp` field for encoding expired tokens.
#[derive(serde::Serialize)]
struct ExpiredUserClaims {
    user_id: Uuid,
    email: String,
    exp: i64,
}

/// Mirror of `AdminClaims` with an added `exp` field for encoding expired tokens.
#[derive(serde::Serialize)]
struct ExpiredAdminClaims {
    admin_id: Uuid,
    email: String,
    role: String,
    exp: i64,
}

fn make_expired_user_token() -> String {
    let expired_at = (Utc::now() - Duration::hours(1)).timestamp();

    let claims = ExpiredUserClaims {
        user_id: Uuid::new_v4(),
        email: "expired@example.com".to_string(),
        exp: expired_at,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SIGNING_SECRET),
    )
    .expect("failed to encode expired user token")
}

fn make_expired_admin_token() -> String {
    let expired_at = (Utc::now() - Duration::hours(1)).timestamp();

    let claims = ExpiredAdminClaims {
        admin_id: Uuid::new_v4(),
        email: "expired-admin@example.com".to_string(),
        role: "admin".to_string(),
        exp: expired_at,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SIGNING_SECRET),
    )
    .expect("failed to encode expired admin token")
}

/// An expired user token must be rejected when accessing a user-protected plans endpoint.
///
/// Scenario: attacker replays a stolen token after it has expired.
/// Expected: HTTP 401 Unauthorized — the server must not grant access.
#[tokio::test]
async fn expired_user_token_rejected_on_plans_endpoint() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = make_expired_user_token();

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/plans/due-for-claim")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/plans/due-for-claim failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expired user JWT must be rejected with 401 on /api/plans/due-for-claim"
    );
}

/// An expired user token must be rejected when accessing the notifications endpoint.
///
/// Confirms expiration enforcement is not route-specific.
#[tokio::test]
async fn expired_user_token_rejected_on_notifications_endpoint() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = make_expired_user_token();

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/notifications")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/notifications failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expired user JWT must be rejected with 401 on /api/notifications"
    );
}

/// An expired admin token must be rejected when accessing the admin audit-log endpoint.
///
/// Ensures expiration is enforced on admin-protected routes as well.
#[tokio::test]
async fn expired_admin_token_rejected_on_admin_logs_endpoint() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = make_expired_admin_token();

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/logs failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expired admin JWT must be rejected with 401 on /api/admin/logs"
    );
}

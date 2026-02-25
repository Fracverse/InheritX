mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use tower::ServiceExt; // for `oneshot`
use uuid::Uuid;

/// Returns the JWT signing secret that matches the test app's config.
/// Mirrors the fallback in `TestContext::from_env`.
fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-jwt-secret".to_string())
}

/// Matches `UserClaims` in `src/auth.rs`.
#[derive(Debug, Serialize, Deserialize)]
struct UserTokenClaims {
    user_id: Uuid,
    email: String,
    exp: usize,
}

/// Matches `AdminClaims` in `src/auth.rs`.
#[derive(Debug, Serialize, Deserialize)]
struct AdminTokenClaims {
    admin_id: Uuid,
    email: String,
    role: String,
    exp: usize,
}

fn make_expired_user_token() -> String {
    let expired_at = (Utc::now() - Duration::hours(1)).timestamp() as usize;
    let claims = UserTokenClaims {
        user_id: Uuid::new_v4(),
        email: "expired@example.com".to_string(),
        exp: expired_at,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .expect("failed to encode expired user token")
}

fn make_expired_admin_token() -> String {
    let expired_at = (Utc::now() - Duration::hours(1)).timestamp() as usize;
    let claims = AdminTokenClaims {
        admin_id: Uuid::new_v4(),
        email: "expired-admin@example.com".to_string(),
        role: "admin".to_string(),
        exp: expired_at,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .expect("failed to encode expired admin token")
}

// ── Expired token tests ───────────────────────────────────────────────────────

/// Scenario: attacker replays a stolen token after it has expired.
/// Expected: HTTP 401 — the server must not grant access.
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

// ── Token integrity / edge-case tests ────────────────────────────────────────

/// Attack scenario: attacker re-encodes a token payload with a different secret
/// (simulating a signature tampering / privilege escalation attempt).
#[tokio::test]
async fn test_modified_jwt_signature_rejected_on_admin_route() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let expiration = (Utc::now() + Duration::hours(24)).timestamp() as usize;

    // Attacker re-encodes with a wrong secret after modifying the payload.
    let tampered_claims = AdminTokenClaims {
        admin_id: Uuid::new_v4(),
        email: "attacker@example.com".to_string(),
        role: "admin".to_string(),
        exp: expiration,
    };
    let tampered_token = encode(
        &Header::default(),
        &tampered_claims,
        &EncodingKey::from_secret(b"wrong_secret_key"),
    )
    .expect("failed to encode tampered token");

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .header("Authorization", format!("Bearer {}", tampered_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/logs failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "token signed with wrong secret must be rejected with 401"
    );
}

/// Positive test: a structurally valid, non-expired token with the correct
/// secret must not be rejected by the auth layer (HTTP 401).
#[tokio::test]
async fn test_valid_jwt_signature_accepted_on_admin_route() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let expiration = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = AdminTokenClaims {
        admin_id: Uuid::new_v4(),
        email: "admin@example.com".to_string(),
        role: "admin".to_string(),
        exp: expiration,
    };

    let valid_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .expect("failed to encode valid admin token");

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .header("Authorization", format!("Bearer {}", valid_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/logs failed");

    // Auth layer must pass (not 401). The handler may return 200 or a DB error,
    // but a valid token must never be rejected by the JWT validator itself.
    assert_ne!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "valid JWT with correct signature must not return 401"
    );
}

/// No Authorization header at all must return 401.
#[tokio::test]
async fn test_missing_authorization_header_rejected() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/logs failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "missing Authorization header must return 401"
    );
}

/// A header that does not start with "Bearer " must return 401.
#[tokio::test]
async fn test_invalid_bearer_format_rejected() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .header("Authorization", "InvalidFormat token123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/logs failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "invalid Bearer format must return 401"
    );
}

/// A token with a syntactically invalid payload (bad base64) must return 401.
#[tokio::test]
async fn test_malformed_jwt_rejected() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let malformed_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid_payload.invalid_signature";

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .header("Authorization", format!("Bearer {}", malformed_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/logs failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "malformed JWT must return 401"
    );
}

/// A token signed with HS512 must be rejected because the server expects HS256.
#[tokio::test]
async fn test_jwt_with_different_algorithm_rejected() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let expiration = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = AdminTokenClaims {
        admin_id: Uuid::new_v4(),
        email: "admin@example.com".to_string(),
        role: "admin".to_string(),
        exp: expiration,
    };

    let header = jsonwebtoken::Header {
        alg: jsonwebtoken::Algorithm::HS512,
        ..Default::default()
    };
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .expect("failed to encode HS512 token");

    let response = test_context
        .app
        .clone()
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
        "JWT signed with HS512 (expected HS256) must return 401"
    );
}

/// An empty token string after "Bearer " must return 401.
#[tokio::test]
async fn test_empty_jwt_token_rejected() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .header("Authorization", "Bearer ")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/logs failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "empty JWT token must return 401"
    );
}

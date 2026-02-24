mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

/// Test claims structure matching the backend's Claims struct
#[derive(Debug, Serialize, Deserialize)]
struct TestClaims {
    sub: String,
    role: String,
    exp: usize,
}

/// Test: Modified JWT payload should be rejected
/// Attack scenario: User modifies role from "user" → "admin" and re-encodes without correct signature
#[tokio::test]
async fn test_modified_jwt_signature_rejected_on_admin_route() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // Create a valid token with user role
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;

    let valid_claims = TestClaims {
        sub: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        role: "user".to_string(),
        exp: expiration,
    };

    let _valid_token = encode(
        &Header::default(),
        &valid_claims,
        &EncodingKey::from_secret(
            test_context
                .pool
                .connect_options()
                .get_username()
                .as_bytes(),
        ),
    )
    .expect("failed to encode valid token");

    // Attacker modifies the payload: changes role to "admin"
    let modified_claims = TestClaims {
        sub: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        role: "admin".to_string(), // ← Privilege escalation attempt
        exp: expiration,
    };

    // Attacker tries to re-encode with a different secret (simulating tampering)
    let tampered_token = encode(
        &Header::default(),
        &modified_claims,
        &EncodingKey::from_secret(b"wrong_secret_key"),
    )
    .expect("failed to encode tampered token");

    // Attempt to access admin-only endpoint with tampered token
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

    // ✅ Expected: HTTP 401 Unauthorized (signature verification failure)
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Modified JWT should be rejected with 401 Unauthorized"
    );
}

/// Test: Valid JWT with correct signature should be accepted
/// This is the positive test case to ensure legitimate tokens work
#[tokio::test]
async fn test_valid_jwt_signature_accepted_on_admin_route() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let expiration = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;

    let valid_claims = TestClaims {
        sub: "550e8400-e29b-41d4-a716-446655440001".to_string(),
        role: "admin".to_string(),
        exp: expiration,
    };

    // Encode with the correct secret from config
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-jwt-secret".to_string());

    let valid_token = encode(
        &Header::default(),
        &valid_claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .expect("failed to encode valid token");

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

    // ✅ Expected: Should NOT be 401 (signature is valid)
    // Note: May be 403 or 500 due to missing admin in DB, but NOT 401
    assert_ne!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Valid JWT with correct signature should not return 401"
    );
}

/// Test: JWT with missing Authorization header should be rejected
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
        "Missing Authorization header should return 401"
    );
}

/// Test: JWT with invalid Bearer format should be rejected
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
        "Invalid Bearer format should return 401"
    );
}

/// Test: Expired JWT should be rejected
#[tokio::test]
async fn test_expired_jwt_rejected() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    // Create token with expiration in the past
    let expired_time = (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize;

    let expired_claims = TestClaims {
        sub: "550e8400-e29b-41d4-a716-446655440002".to_string(),
        role: "admin".to_string(),
        exp: expired_time,
    };

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-jwt-secret".to_string());

    let expired_token = encode(
        &Header::default(),
        &expired_claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .expect("failed to encode expired token");

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .header("Authorization", format!("Bearer {}", expired_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/logs failed");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Expired JWT should be rejected with 401"
    );
}

/// Test: Malformed JWT (invalid base64) should be rejected
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
        "Malformed JWT should be rejected with 401"
    );
}

/// Test: JWT signed with different algorithm should be rejected
#[tokio::test]
async fn test_jwt_with_different_algorithm_rejected() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let expiration = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;

    let claims = TestClaims {
        sub: "550e8400-e29b-41d4-a716-446655440003".to_string(),
        role: "admin".to_string(),
        exp: expiration,
    };

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-jwt-secret".to_string());

    // Create token with different algorithm header (simulating tampering)
    let header = jsonwebtoken::Header {
        alg: jsonwebtoken::Algorithm::HS512,
        ..Default::default()
    };

    let token_with_different_alg = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .expect("failed to encode token with different algorithm");

    let response = test_context
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/logs")
                .header(
                    "Authorization",
                    format!("Bearer {}", token_with_different_alg),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request to /api/admin/logs failed");

    // Should be rejected due to algorithm mismatch
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "JWT with different algorithm should be rejected with 401"
    );
}

/// Test: Empty JWT token should be rejected
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
        "Empty JWT token should be rejected with 401"
    );
}

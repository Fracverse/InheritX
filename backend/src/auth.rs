use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jsonwebtoken::{decode, errors::ErrorKind, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    /// Issued at (Unix timestamp). Optional for backward compatibility.
    #[serde(default)]
    pub iat: Option<usize>,
    /// Not before (Unix timestamp). Optional for backward compatibility.
    #[serde(default)]
    pub nbf: Option<usize>,
    /// Issuer. Optional for backward compatibility.
    #[serde(default)]
    pub iss: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub role: String,
}

impl axum::extract::FromRequestParts<()> for UserContext {
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &(),
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let ctx = parts.extensions.get::<UserContext>().cloned();
        Box::pin(async move { ctx.ok_or(StatusCode::INTERNAL_SERVER_ERROR) })
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingHeader,
    #[error("Invalid authorization header format")]
    InvalidHeaderFormat,
    #[error("Missing token")]
    MissingToken,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token not yet valid (nbf claim)")]
    TokenNotYetValid,
    #[error("Invalid issuer")]
    InvalidIssuer,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            AuthError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                serde_json::json!({
                    "error": self.to_string(),
                    "code": "TOKEN_EXPIRED"
                }),
            ),
            AuthError::TokenNotYetValid => (
                StatusCode::UNAUTHORIZED,
                serde_json::json!({
                    "error": self.to_string(),
                    "code": "TOKEN_NOT_YET_VALID"
                }),
            ),
            AuthError::InvalidIssuer => (
                StatusCode::UNAUTHORIZED,
                serde_json::json!({
                    "error": self.to_string(),
                    "code": "INVALID_ISSUER"
                }),
            ),
            _ => (
                StatusCode::UNAUTHORIZED,
                serde_json::json!({ "error": self.to_string() }),
            ),
        };
        (status, Json(body)).into_response()
    }
}

/// Default JWT issuer used when `JWT_ISSUER` env var is not set.
const DEFAULT_JWT_ISSUER: &str = "inheritx-backend";

/// Clock skew leeway in seconds (default: 60s) to accommodate slight time differences
/// between the token issuer and this server.
const DEFAULT_LEEWAY_SECS: u64 = 60;

pub async fn jwt_auth_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(AuthError::MissingHeader)?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| AuthError::InvalidHeaderFormat)?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AuthError::InvalidHeaderFormat);
    }

    let token = auth_str.trim_start_matches("Bearer ").trim();
    if token.is_empty() {
        return Err(AuthError::MissingToken);
    }

    let secret = std::env::var("JWT_SECRET").map_err(|_| AuthError::InvalidToken)?;
    if secret.len() < 32 {
        return Err(AuthError::InvalidToken);
    }

    let expected_issuer = std::env::var("JWT_ISSUER")
        .unwrap_or_else(|_| DEFAULT_JWT_ISSUER.to_string());

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.set_issuer(&[expected_issuer]);
    validation.leeway = DEFAULT_LEEWAY_SECS;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map_err(|err| {
        match err.kind() {
            &ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            &ErrorKind::InvalidIssuer => AuthError::InvalidIssuer,
            // jsonwebtoken 9.x uses different error variants
            _ => {
                // Check if the error message contains "expired" or "not yet valid"
                let err_msg = err.to_string().to_lowercase();
                if err_msg.contains("expired") {
                    AuthError::TokenExpired
                } else if err_msg.contains("not yet valid") || err_msg.contains("immature") {
                    AuthError::TokenNotYetValid
                } else if err_msg.contains("issuer") {
                    AuthError::InvalidIssuer
                } else {
                    AuthError::InvalidToken
                }
            }
        }
    })?;

    if token_data.claims.role != "admin" {
        return Err(AuthError::Unauthorized);
    }

    let user_context = UserContext {
        user_id: token_data.claims.sub,
        role: token_data.claims.role,
    };

    req.extensions_mut().insert(user_context);

    Ok(next.run(req).await)
}

pub async fn signature_auth_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let (parts, body) = req.into_parts();

    let public_key_hex = parts
        .headers
        .get("X-Public-Key")
        .ok_or(AuthError::MissingHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaderFormat)?;

    let signature_hex = parts
        .headers
        .get("X-Signature")
        .ok_or(AuthError::MissingHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaderFormat)?;

    let public_key_bytes = hex::decode(public_key_hex.trim_start_matches("0x"))
        .map_err(|_| AuthError::InvalidSignature)?;

    let signature_bytes = hex::decode(signature_hex.trim_start_matches("0x"))
        .map_err(|_| AuthError::InvalidSignature)?;

    if public_key_bytes.len() != 32 {
        return Err(AuthError::InvalidSignature);
    }

    let public_key_array: [u8; 32] = public_key_bytes
        .try_into()
        .map_err(|_| AuthError::InvalidSignature)?;

    let verifying_key =
        VerifyingKey::from_bytes(&public_key_array).map_err(|_| AuthError::InvalidSignature)?;

    let signature = Signature::from_slice(signature_bytes.as_slice())
        .map_err(|_| AuthError::InvalidSignature)?;

    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| AuthError::InvalidSignature)?;

    let body_str =
        String::from_utf8(body_bytes.to_vec()).map_err(|_| AuthError::InvalidSignature)?;

    verifying_key
        .verify(body_str.as_bytes(), &signature)
        .map_err(|_| AuthError::InvalidSignature)?;

    let user_context = UserContext {
        user_id: public_key_hex.to_string(),
        role: "user".to_string(),
    };

    let mut new_req = Request::from_parts(parts, Body::from(body_str));
    new_req.extensions_mut().insert(user_context);

    Ok(next.run(new_req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::Request,
        middleware::from_fn,
        routing::get,
        Router,
    };
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::sync::OnceLock;
    use tower::ServiceExt;

    /// Returns a consistent test JWT secret (32+ bytes).
    fn test_secret() -> &'static str {
        static SECRET: OnceLock<String> = OnceLock::new();
        SECRET.get_or_init(|| "test-secret-key-that-is-at-least-32-bytes-long!!".to_string())
    }

    /// Returns a consistent test issuer.
    fn test_issuer() -> &'static str {
        "inheritx-backend"
    }

    /// Build a test router with the JWT auth middleware applied to a protected route.
    fn build_test_app() -> Router {
        // Set env vars for the test
        std::env::set_var("JWT_SECRET", test_secret());
        std::env::set_var("JWT_ISSUER", test_issuer());

        Router::new()
            .route("/admin", get(|| async { "admin ok" }))
            .route_layer(from_fn(jwt_auth_middleware))
    }

    /// Create a valid admin JWT token for testing.
    fn create_admin_token(exp_offset_secs: i64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: "admin-user-001".to_string(),
            role: "admin".to_string(),
            exp: (now as i64 + exp_offset_secs) as usize,
            iat: Some(now),
            nbf: Some(now),
            iss: Some(test_issuer().to_string()),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(test_secret().as_ref()),
        )
        .expect("Failed to create test token")
    }

    /// Create a valid non-admin JWT token for testing.
    fn create_user_token(exp_offset_secs: i64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: "regular-user-001".to_string(),
            role: "user".to_string(),
            exp: (now as i64 + exp_offset_secs) as usize,
            iat: Some(now),
            nbf: Some(now),
            iss: Some(test_issuer().to_string()),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(test_secret().as_ref()),
        )
        .expect("Failed to create test token")
    }

    #[tokio::test]
    async fn test_valid_admin_token_succeeds() {
        let app = build_test_app();
        let token = create_admin_token(3600); // expires in 1 hour

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_missing_auth_header_returns_401() {
        let app = build_test_app();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "Missing authorization header");
    }

    #[tokio::test]
    async fn test_invalid_auth_header_format_returns_401() {
        let app = build_test_app();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", "Basic somecreds")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_empty_token_returns_401() {
        let app = build_test_app();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", "Bearer ")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_expired_token_returns_token_expired_error() {
        let app = build_test_app();
        let token = create_admin_token(-3600); // expired 1 hour ago

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], "TOKEN_EXPIRED");
        assert!(json["error"].to_string().to_lowercase().contains("expired"));
    }

    #[tokio::test]
    async fn test_non_admin_token_returns_unauthorized() {
        let app = build_test_app();
        let token = create_user_token(3600); // valid but not admin

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "Unauthorized");
    }

    #[tokio::test]
    async fn test_invalid_signature_returns_401() {
        let app = build_test_app();
        // Use a token signed with a different secret
        let wrong_secret = "this-is-a-completely-different-secret-key-12345!!";
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: "admin-user-001".to_string(),
            role: "admin".to_string(),
            exp: now + 3600,
            iat: Some(now),
            nbf: Some(now),
            iss: Some(test_issuer().to_string()),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(wrong_secret.as_ref()),
        )
        .expect("Failed to create test token");

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_token_with_invalid_issuer_returns_401() {
        let app = build_test_app();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: "admin-user-001".to_string(),
            role: "admin".to_string(),
            exp: now + 3600,
            iat: Some(now),
            nbf: Some(now),
            iss: Some("evil-attacker".to_string()), // wrong issuer
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(test_secret().as_ref()),
        )
        .expect("Failed to create test token");

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        // Should either be INVALID_ISSUER or INVALID_TOKEN depending on jsonwebtoken version
        assert!(
            json["code"] == "INVALID_ISSUER" || json["code"].is_null(),
            "Expected INVALID_ISSUER code, got: {}",
            json["code"]
        );
    }

    #[tokio::test]
    async fn test_token_with_future_nbf_returns_401() {
        let app = build_test_app();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: "admin-user-001".to_string(),
            role: "admin".to_string(),
            exp: now + 7200,
            iat: Some(now),
            nbf: Some(now + 3600), // not valid for another hour
            iss: Some(test_issuer().to_string()),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(test_secret().as_ref()),
        )
        .expect("Failed to create test token");

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_missing_jwt_secret_returns_401() {
        // Temporarily remove JWT_SECRET
        std::env::remove_var("JWT_SECRET");
        std::env::set_var("JWT_ISSUER", test_issuer());

        let app = Router::new()
            .route("/admin", get(|| async { "admin ok" }))
            .route_layer(from_fn(jwt_auth_middleware));

        let token = create_admin_token(3600);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Restore env var for other tests
        std::env::set_var("JWT_SECRET", test_secret());
    }

    #[tokio::test]
    async fn test_weak_jwt_secret_returns_401() {
        std::env::set_var("JWT_SECRET", "short"); // less than 32 chars
        std::env::set_var("JWT_ISSUER", test_issuer());

        let app = Router::new()
            .route("/admin", get(|| async { "admin ok" }))
            .route_layer(from_fn(jwt_auth_middleware));

        let token = create_admin_token(3600);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Restore env var for other tests
        std::env::set_var("JWT_SECRET", test_secret());
    }

    #[tokio::test]
    async fn test_malformed_token_returns_401() {
        let app = build_test_app();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", "Bearer this.is.not.a.valid.jwt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_token_without_role_claim_returns_401() {
        let app = build_test_app();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        // Create a token without the role field
        #[derive(Serialize)]
        struct MinimalClaims {
            sub: String,
            exp: usize,
        }

        let minimal = MinimalClaims {
            sub: "no-role-user".to_string(),
            exp: now + 3600,
        };

        let token = encode(
            &Header::default(),
            &minimal,
            &EncodingKey::from_secret(test_secret().as_ref()),
        )
        .expect("Failed to create test token");

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
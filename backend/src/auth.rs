use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
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
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.to_string() });
        (StatusCode::UNAUTHORIZED, Json(body)).into_response()
    }
}

fn jwt_validation() -> Validation {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation
}

fn map_jwt_decode_error(err: jsonwebtoken::errors::Error) -> AuthError {
    match err.kind() {
        ErrorKind::ExpiredSignature => AuthError::TokenExpired,
        _ => AuthError::InvalidToken,
    }
}

/// Signs an HS256 admin JWT with a fixed role and expiration.
pub fn generate_admin_jwt(
    admin_id: impl Into<String>,
    secret: &str,
    ttl: Duration,
) -> Result<(String, i64), jsonwebtoken::errors::Error> {
    let expires_at = Utc::now() + ttl;
    let claims = Claims {
        sub: admin_id.into(),
        role: "admin".to_string(),
        exp: expires_at.timestamp() as usize,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok((token, expires_at.timestamp()))
}

/// Validates a Bearer JWT for admin routes (signature, expiry, admin role).
pub fn validate_admin_jwt(token: &str, secret: &str) -> Result<Claims, AuthError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &jwt_validation(),
    )
    .map_err(map_jwt_decode_error)?;

    if token_data.claims.role != "admin" {
        return Err(AuthError::Unauthorized);
    }

    Ok(token_data.claims)
}

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
    let claims = validate_admin_jwt(token, &secret)?;

    let user_context = UserContext {
        user_id: claims.sub,
        role: claims.role,
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
    use axum::middleware::from_fn;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    const TEST_SECRET: &str = "inheritx-jwt-test-secret";

    fn encode_token(claims: &Claims) -> String {
        encode(
            &Header::new(Algorithm::HS256),
            claims,
            &EncodingKey::from_secret(TEST_SECRET.as_ref()),
        )
        .expect("test token should encode")
    }

    async fn jwt_middleware_status(auth_header: Option<&str>) -> StatusCode {
        std::env::set_var("JWT_SECRET", TEST_SECRET);

        let app = Router::new()
            .route("/protected", get(|| async { "ok" }))
            .route_layer(from_fn(jwt_auth_middleware));

        let mut builder = Request::builder().uri("/protected");
        if let Some(header) = auth_header {
            builder = builder.header("Authorization", header);
        }

        let response = app
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();

        response.status()
    }

    #[test]
    fn generate_admin_jwt_includes_admin_role_and_expiry() {
        let (token, expires_at) =
            generate_admin_jwt("admin-42", TEST_SECRET, Duration::hours(1)).unwrap();

        let claims = validate_admin_jwt(&token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, "admin-42");
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.exp as i64, expires_at);
    }

    #[test]
    fn validate_admin_jwt_rejects_expired_token() {
        let token = encode_token(&Claims {
            sub: "admin-1".to_string(),
            role: "admin".to_string(),
            exp: 1,
        });

        let err = validate_admin_jwt(&token, TEST_SECRET).unwrap_err();
        assert!(matches!(err, AuthError::TokenExpired));
    }

    #[test]
    fn validate_admin_jwt_rejects_non_admin_role() {
        let exp = (Utc::now() + Duration::hours(1)).timestamp() as usize;
        let token = encode_token(&Claims {
            sub: "user-1".to_string(),
            role: "user".to_string(),
            exp,
        });

        let err = validate_admin_jwt(&token, TEST_SECRET).unwrap_err();
        assert!(matches!(err, AuthError::Unauthorized));
    }

    #[tokio::test]
    async fn jwt_middleware_accepts_valid_admin_bearer() {
        let exp = (Utc::now() + Duration::hours(1)).timestamp() as usize;
        let token = encode_token(&Claims {
            sub: "admin-1".to_string(),
            role: "admin".to_string(),
            exp,
        });

        let status = jwt_middleware_status(Some(&format!("Bearer {token}"))).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn jwt_middleware_rejects_expired_bearer() {
        let token = encode_token(&Claims {
            sub: "admin-1".to_string(),
            role: "admin".to_string(),
            exp: 1,
        });

        let status = jwt_middleware_status(Some(&format!("Bearer {token}"))).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn jwt_middleware_rejects_missing_authorization_header() {
        let status = jwt_middleware_status(None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}

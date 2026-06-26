use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
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
        let status = match self {
            AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
            _ => StatusCode::UNAUTHORIZED,
        };
        let body = serde_json::json!({ "error": self.to_string() });
        (status, Json(body)).into_response()
    }
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

    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map_err(|_| AuthError::InvalidToken)?;

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
mod tests {}

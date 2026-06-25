use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Validation, DecodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminClaims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub async fn admin_auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

    // Check for Bearer token
    if auth_header.starts_with("Bearer ") {
        let token = &auth_header[7..];
        
        match decode::<AdminClaims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(token_data) => {
                if token_data.claims.role != "admin" {
                    return Err(StatusCode::FORBIDDEN);
                }
                Ok(next.run(req).await)
            }
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    }
    // Check for API key as fallback
    else if auth_header.starts_with("ApiKey ") {
        let api_key = &auth_header[7..];
        let admin_api_key = std::env::var("ADMIN_API_KEY")
            .unwrap_or_else(|_| "default-api-key-change-in-production".to_string());
        
        if api_key == admin_api_key {
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

use crate::api_error::ApiError;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub user_id: uuid::Uuid,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminClaims {
    pub admin_id: uuid::Uuid,
    pub email: String,
    pub role: String,
}

pub struct AuthenticatedUser(pub UserClaims);

pub struct AuthenticatedAdmin(pub AdminClaims);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(ApiError::Unauthorized);
        }

        let token = auth_header.strip_prefix("Bearer ").unwrap();

        let claims: UserClaims = jsonwebtoken::decode(
            token,
            &jsonwebtoken::DecodingKey::from_secret(b"secret_key_change_in_production"),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| ApiError::Unauthorized)?
        .claims;

        Ok(AuthenticatedUser(claims))
    }
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedAdmin
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(ApiError::Unauthorized);
        }

        let token = auth_header.strip_prefix("Bearer ").unwrap();

        let claims: AdminClaims = jsonwebtoken::decode(
            token,
            &jsonwebtoken::DecodingKey::from_secret(b"secret_key_change_in_production"),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| ApiError::Unauthorized)?
        .claims;

        Ok(AuthenticatedAdmin(claims))
    }
}

pub async fn verify_user_exists(db: &PgPool, user_id: &uuid::Uuid) -> Result<(), ApiError> {
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(user_id)
        .fetch_one(db)
        .await?;

    if !exists {
        return Err(ApiError::Unauthorized);
    }

    Ok(())
}

pub async fn verify_admin_exists(db: &PgPool, admin_id: &uuid::Uuid) -> Result<(), ApiError> {
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM admins WHERE id = $1)")
        .bind(admin_id)
        .fetch_one(db)
        .await?;

    if !exists {
        return Err(ApiError::Unauthorized);
    }

    Ok(())
}

use crate::api_error::ApiError;
use crate::app::AppState;
use axum::{extract::State, Json};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String, // subject (user id)
    role: String,
    exp: usize,
}

#[derive(Debug, FromRow)]
struct Admin {
    id: uuid::Uuid,
    _email: String,
    password_hash: String,
    role: String,
    status: String,
}

pub async fn login_admin(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let admin = sqlx::query_as::<_, Admin>(
        "SELECT id, email, password_hash, role, status FROM admins WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await?;

    let admin = match admin {
        Some(a) => a,
        None => return Err(ApiError::Unauthorized),
    };

    if admin.status == "locked" {
        return Err(ApiError::Forbidden("Account is locked".to_string()));
    }

    let valid = verify(&payload.password, &admin.password_hash)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    if !valid {
        return Err(ApiError::Unauthorized);
    }

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: admin.id.to_string(),
        role: admin.role,
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    Ok(Json(LoginResponse { token }))
}

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
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

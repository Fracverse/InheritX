use crate::api_error::ApiError;
use crate::app::AppState;
use axum::extract::FromRequestParts;
use axum::http::{header::AUTHORIZATION, request::Parts};
use axum::{extract::State, Json};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

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

#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Copy)]
pub struct AuthAdmin {
    pub admin_id: Uuid,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(user_id) = parse_bearer_uuid(parts) {
            return Ok(Self { user_id });
        }

        let user_id = parts
            .headers
            .get("x-user-id")
            .ok_or(ApiError::Unauthorized)?
            .to_str()
            .map_err(|_| ApiError::Unauthorized)?;

        let user_id = Uuid::parse_str(user_id).map_err(|_| ApiError::Unauthorized)?;

        Ok(Self { user_id })
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthAdmin
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let admin_id = parts
            .headers
            .get("x-admin-id")
            .ok_or(ApiError::Unauthorized)?
            .to_str()
            .map_err(|_| ApiError::Unauthorized)?;

        let admin_id = Uuid::parse_str(admin_id).map_err(|_| ApiError::Unauthorized)?;

        Ok(Self { admin_id })
    }
}

fn parse_bearer_uuid(parts: &Parts) -> Option<Uuid> {
    let header = parts.headers.get(AUTHORIZATION)?;
    let token = header.to_str().ok()?;
    let token = token.strip_prefix("Bearer ")?;

    Uuid::parse_str(token).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::FromRequestParts;
    use axum::http::Request;

    #[tokio::test]
    async fn auth_user_accepts_x_user_id_header() {
        let user_id = Uuid::new_v4();
        let req = Request::builder()
            .uri("/plans")
            .header("x-user-id", user_id.to_string())
            .body(())
            .expect("request should build");
        let (mut parts, _) = req.into_parts();

        let auth_user = AuthUser::from_request_parts(&mut parts, &())
            .await
            .expect("x-user-id should authenticate user");

        assert_eq!(auth_user.user_id, user_id);
    }

    #[tokio::test]
    async fn auth_user_accepts_bearer_uuid_token() {
        let user_id = Uuid::new_v4();
        let req = Request::builder()
            .uri("/plans")
            .header(AUTHORIZATION, format!("Bearer {}", user_id))
            .body(())
            .expect("request should build");
        let (mut parts, _) = req.into_parts();

        let auth_user = AuthUser::from_request_parts(&mut parts, &())
            .await
            .expect("bearer uuid should authenticate user");

        assert_eq!(auth_user.user_id, user_id);
    }

    #[tokio::test]
    async fn auth_user_rejects_missing_headers() {
        let req = Request::builder()
            .uri("/plans")
            .body(())
            .expect("request should build");
        let (mut parts, _) = req.into_parts();

        let result = AuthUser::from_request_parts(&mut parts, &()).await;

        assert!(matches!(result, Err(ApiError::Unauthorized)));
    }

    #[tokio::test]
    async fn auth_admin_accepts_x_admin_id_header() {
        let admin_id = Uuid::new_v4();
        let req = Request::builder()
            .uri("/admin/plans")
            .header("x-admin-id", admin_id.to_string())
            .body(())
            .expect("request should build");
        let (mut parts, _) = req.into_parts();

        let auth_admin = AuthAdmin::from_request_parts(&mut parts, &())
            .await
            .expect("x-admin-id should authenticate admin");

        assert_eq!(auth_admin.admin_id, admin_id);
    }

    #[tokio::test]
    async fn auth_admin_rejects_missing_headers() {
        let req = Request::builder()
            .uri("/admin/plans")
            .body(())
            .expect("request should build");
        let (mut parts, _) = req.into_parts();

        let result = AuthAdmin::from_request_parts(&mut parts, &()).await;

        assert!(matches!(result, Err(ApiError::Unauthorized)));
    }
}

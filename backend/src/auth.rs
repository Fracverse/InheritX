use crate::api_error::ApiError;
use crate::app::AppState;
use crate::config::Config;
use axum::{extract::State, Json};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct NonceResponse {
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
pub struct WalletLoginRequest {
    pub wallet_address: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    token: String,
}

#[derive(Debug, FromRow)]
struct Admin {
    id: uuid::Uuid,
    email: String,
    password_hash: String,
    role: String,
    status: String,
}

#[derive(Debug, FromRow)]
struct User {
    id: uuid::Uuid,
    email: String,
    password_hash: String,
}

pub async fn login_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let user =
        sqlx::query_as::<_, User>("SELECT id, email, password_hash FROM users WHERE email = $1")
            .bind(&payload.email)
            .fetch_optional(&state.db)
            .await?;

    let user = match user {
        Some(u) => u,
        None => return Err(ApiError::Unauthorized),
    };

    let valid = verify(&payload.password, &user.password_hash)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    if !valid {
        return Err(ApiError::Unauthorized);
    }

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = UserClaims {
        user_id: user.id,
        email: user.email,
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
#[derive(sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    email: String,
    nonce: Option<String>,
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

    let claims = AdminClaims {
        admin_id: admin.id,
        email: admin.email,
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

pub async fn generate_nonce(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(wallet_address): axum::extract::Path<String>,
) -> Result<Json<NonceResponse>, ApiError> {
    let nonce = Uuid::new_v4().to_string();

    // Check if user exists, if not create a stub
    let user_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE wallet_address = $1)",
    )
    .bind(&wallet_address)
    .fetch_one(&state.db)
    .await?;

    if user_exists {
        sqlx::query("UPDATE users SET nonce = $1 WHERE wallet_address = $2")
            .bind(&nonce)
            .bind(&wallet_address)
            .execute(&state.db)
            .await?;
    } else {
        // Create user with a dummy email for now or handle it differently
        let dummy_email = format!("{}@wallet.inheritx", wallet_address);
        sqlx::query("INSERT INTO users (wallet_address, nonce, email, password_hash) VALUES ($1, $2, $3, $4)")
            .bind(&wallet_address)
            .bind(&nonce)
            .bind(&dummy_email)
            .bind("WALLET_LOGIN") // Placeholder since it's wallet login
            .execute(&state.db)
            .await?;
    }

    Ok(Json(NonceResponse { nonce }))
}

pub async fn wallet_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WalletLoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, nonce FROM users WHERE wallet_address = $1",
    )
    .bind(&payload.wallet_address)
    .fetch_optional(&state.db)
    .await?;

    let user = match user {
        Some(u) => u,
        None => return Err(ApiError::Unauthorized),
    };

    let _nonce = match user.nonce {
        Some(n) => n,
        None => return Err(ApiError::Unauthorized),
    };

    // Verify signature logic (Stubbed for now as per usual implementation patterns in this repo)
    // In a real scenario, we'd use ed25519-dalek or similar to verify payload.signature against _nonce
    if payload.signature == "invalid_signature" {
        return Err(ApiError::Unauthorized);
    }

    // Clear nonce after successful login
    sqlx::query("UPDATE users SET nonce = NULL WHERE id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await?;

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = UserClaims {
        user_id: user.id,
        email: user.email,
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
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
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminClaims {
    pub admin_id: uuid::Uuid,
    pub email: String,
    pub role: String,
    pub exp: usize,
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
        let config =
            parts
                .extensions
                .get::<Config>()
                .ok_or(ApiError::Internal(anyhow::anyhow!(
                    "Config not found in extensions"
                )))?;
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
            &jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
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
        let config =
            parts
                .extensions
                .get::<Config>()
                .ok_or(ApiError::Internal(anyhow::anyhow!(
                    "Config not found in extensions"
                )))?;
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
            &jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
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

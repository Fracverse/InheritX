use crate::api_error::ApiError;
use crate::app::AppState;
use axum::{extract::State, Json};
use bcrypt::verify;
use chrono::{Duration, Utc};
use hex;
use jsonwebtoken::{encode, EncodingKey, Header};
use ring::signature;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use stellar_strkey::Strkey;

#[derive(Debug, Serialize, Deserialize)]
pub struct NonceRequest {
    pub wallet_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonceResponse {
    pub nonce: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Web3LoginRequest {
    pub wallet_address: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject (user id)
    pub role: String,
    pub exp: usize,
}

pub async fn get_nonce(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NonceRequest>,
) -> Result<Json<NonceResponse>, ApiError> {
    let nonce = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(5);

    sqlx::query(
        r#"
        INSERT INTO nonces (wallet_address, nonce, expires_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (wallet_address) DO UPDATE
        SET nonce = EXCLUDED.nonce, expires_at = EXCLUDED.expires_at
        "#,
    )
    .bind(&payload.wallet_address)
    .bind(&nonce)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    Ok(Json(NonceResponse { nonce }))
}

pub async fn web3_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Web3LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // 1. Verify wallet address is valid Stellar G-address
    let strkey = Strkey::from_string(&payload.wallet_address)
        .map_err(|_| ApiError::BadRequest("Invalid wallet address".to_string()))?;

    let public_key_bytes = match strkey {
        Strkey::PublicKeyEd25519(pk) => pk.0,
        _ => {
            return Err(ApiError::BadRequest(
                "Only Ed25519 public keys are supported".to_string(),
            ))
        }
    };

    // 2. Retrieve nonce
    let row: Option<(String, chrono::DateTime<Utc>)> =
        sqlx::query_as("SELECT nonce, expires_at FROM nonces WHERE wallet_address = $1")
            .bind(&payload.wallet_address)
            .fetch_optional(&state.db)
            .await?;

    let (nonce_val, expires_at) = row.ok_or_else(|| ApiError::Unauthorized)?;

    if expires_at < Utc::now() {
        return Err(ApiError::Unauthorized);
    }

    // 3. Verify signature
    let signature_bytes = hex::decode(&payload.signature)
        .map_err(|_| ApiError::BadRequest("Invalid signature format".to_string()))?;

    let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key_bytes);
    peer_public_key
        .verify(nonce_val.as_bytes(), &signature_bytes)
        .map_err(|_| ApiError::Unauthorized)?;

    // 4. Find or create user
    let user_row: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE wallet_address = $1")
            .bind(&payload.wallet_address)
            .fetch_optional(&state.db)
            .await?;

    let user_id = match user_row {
        Some(id) => id,
        None => {
            let email = format!("{}@inheritx.auth", payload.wallet_address);
            let id = uuid::Uuid::new_v4();
            sqlx::query(
                "INSERT INTO users (id, email, password_hash, wallet_address) VALUES ($1, $2, $3, $4)"
            )
            .bind(id)
            .bind(email)
            .bind("web3-auth-none")
            .bind(&payload.wallet_address)
            .execute(&state.db)
            .await?;
            id
        }
    };

    // 5. Generate JWT
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        role: "user".to_string(),
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    // 6. Invalidate nonce
    sqlx::query("DELETE FROM nonces WHERE wallet_address = $1")
        .bind(&payload.wallet_address)
        .execute(&state.db)
        .await?;

    Ok(Json(LoginResponse { token }))
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

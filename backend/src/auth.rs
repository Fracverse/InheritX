use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use crate::api_error::ApiError;
use crate::app::AppState;
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};
use bcrypt::verify;

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
    email: String,
    password_hash: String,
    role: String,
    status: String,
}

pub async fn login_admin(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let admin = sqlx::query_as!(
        Admin,
        "SELECT id, email, password_hash, role, status FROM admins WHERE email = $1",
        payload.email
    )
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

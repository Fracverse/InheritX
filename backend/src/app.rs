use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;

use crate::api_error::ApiError;
use crate::config::Config;
use crate::two_fa::handlers::{send_2fa_handler, verify_2fa_handler};

pub struct AppState {
    pub db: PgPool,
    pub config: Config,
}

pub async fn create_app(db: PgPool, config: Config) -> Result<Router, ApiError> {
    let state = Arc::new(AppState { db, config });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(db_health_check))
        .route("/user/send-2fa", post(send_2fa_handler))
        .route("/user/verify-2fa", post(verify_2fa_handler))
        .with_state(state);

    Ok(app)
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "message": "App is healthy" }))
}

async fn db_health_check(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    sqlx::query("SELECT 1").execute(&state.db).await?;
    Ok(Json(
        json!({ "status": "ok", "message": "Database is connected" }),
    ))
}

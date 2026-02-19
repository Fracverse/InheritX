use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;

use crate::api_error::ApiError;
use crate::config::Config;

pub struct AppState {
    pub db: PgPool,
    pub config: Config,
}

pub async fn create_app(db: PgPool, config: Config) -> Result<Router, ApiError> {
    let state = Arc::new(AppState { db, config });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(db_health_check))
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

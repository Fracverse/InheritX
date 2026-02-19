use axum::{routing::{get, post}, Json, Router};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;

use crate::api_error::ApiError;
use crate::config::Config;
use crate::email::EmailService;
use crate::handlers;

pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub email_service: EmailService,
}

pub async fn create_app(db: PgPool, config: Config) -> Result<Router, ApiError> {
    let email_service = EmailService::new(config.email.clone());
    
    let state = Arc::new(AppState {
        db,
        config,
        email_service,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(db_health_check))
        .route("/user/send-2fa", post(handlers::two_fa::send_2fa))
        .route("/user/verify-2fa", post(handlers::two_fa::verify_2fa))
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

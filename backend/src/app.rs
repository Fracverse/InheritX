use axum::{extract::Path, routing::get, Json, Router};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::api_error::ApiError;
use crate::auth::{AuthAdmin, AuthUser};
use crate::config::Config;
use crate::service;

pub struct AppState {
    pub db: PgPool,
    pub config: Config,
}

pub async fn create_app(db: PgPool, config: Config) -> Result<Router, ApiError> {
    let state = Arc::new(AppState { db, config });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(db_health_check))
        .route("/plans", get(get_user_plans))
        .route("/plans/pending", get(get_user_pending_plans))
        .route("/plans/:id", get(get_user_plan_by_id))
        .route("/admin/plans", get(get_admin_plans))
        .route("/admin/plans/pending", get(get_admin_pending_plans))
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

async fn get_user_plan_by_id(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(plan_id): Path<Uuid>,
) -> Result<Json<service::InheritancePlan>, ApiError> {
    let plan = service::get_user_plan_by_id(&state.db, auth_user.user_id, plan_id).await?;
    Ok(Json(plan))
}

async fn get_user_plans(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Json<Vec<service::InheritancePlan>>, ApiError> {
    let plans = service::get_all_user_plans(&state.db, auth_user.user_id).await?;
    Ok(Json(plans))
}

async fn get_user_pending_plans(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Json<Vec<service::InheritancePlan>>, ApiError> {
    let plans = service::get_all_user_pending_plans(&state.db, auth_user.user_id).await?;
    Ok(Json(plans))
}

async fn get_admin_plans(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    auth_admin: AuthAdmin,
) -> Result<Json<Vec<service::InheritancePlan>>, ApiError> {
    let _ = auth_admin.admin_id;
    let plans = service::get_all_admin_plans(&state.db).await?;
    Ok(Json(plans))
}

async fn get_admin_pending_plans(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    auth_admin: AuthAdmin,
) -> Result<Json<Vec<service::InheritancePlan>>, ApiError> {
    let _ = auth_admin.admin_id;
    let plans = service::get_all_admin_pending_plans(&state.db).await?;
    Ok(Json(plans))
}

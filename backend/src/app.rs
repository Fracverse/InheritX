use axum::{
    routing::{get, post},
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::api_error::ApiError;
use crate::auth::{AuthenticatedAdmin, AuthenticatedUser};
use crate::config::Config;
use crate::service::PlanService;

pub struct AppState {
    pub db: PgPool,
    pub config: Config,
}

pub async fn create_app(db: PgPool, config: Config) -> Result<Router, ApiError> {
    let state = Arc::new(AppState { db, config });

    // Rate limiting configuration
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(db_health_check))
        .route("/admin/login", post(crate::auth::login_admin))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(GovernorLayer {
                    config: governor_conf,
                }),
            )
        .route(
            "/api/plans/due-for-claim/:plan_id",
            get(get_due_for_claim_plan),
        )
        .route(
            "/api/plans/due-for-claim",
            get(get_all_due_for_claim_plans_user),
        )
        .route(
            "/api/admin/plans/due-for-claim",
            get(get_all_due_for_claim_plans_admin),
        )
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

async fn get_due_for_claim_plan(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let plan = PlanService::get_due_for_claim_plan_by_id(&state.db, plan_id, user.user_id).await?;

    match plan {
        Some(plan) => Ok(Json(json!({
            "status": "success",
            "data": plan
        }))),
        None => Err(ApiError::NotFound(format!(
            "Plan {} not found or not due for claim",
            plan_id
        ))),
    }
}

async fn get_all_due_for_claim_plans_user(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let plans = PlanService::get_all_due_for_claim_plans_for_user(&state.db, user.user_id).await?;

    Ok(Json(json!({
        "status": "success",
        "data": plans,
        "count": plans.len()
    })))
}

async fn get_all_due_for_claim_plans_admin(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let plans = PlanService::get_all_due_for_claim_plans_admin(&state.db).await?;

    Ok(Json(json!({
        "status": "success",
        "data": plans,
        "count": plans.len()
    })))
}

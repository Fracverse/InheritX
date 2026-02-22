use axum::{
    extract::{Path, State},
    routing::{get, patch, post},
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
use crate::notifications::{AuditLogService, NotificationService};
use crate::service::{
    ClaimPlanRequest, CreatePlanRequest, KycRecord, KycService, KycStatus, PlanService,
};

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
            "/api/plans/due-for-claim",
            get(get_all_due_for_claim_plans_user),
        )
        .route(
            "/api/plans/due-for-claim/:plan_id",
            get(get_due_for_claim_plan),
        )
        .route("/api/plans/:plan_id/claim", post(claim_plan))
        .route("/api/plans/:plan_id", get(get_plan))
        .route("/api/plans", post(create_plan))
        .route(
            "/api/admin/plans/due-for-claim",
            get(get_all_due_for_claim_plans_admin),
        )
        .route("/api/admin/kyc/:user_id", get(get_kyc_status))
        .route("/api/admin/kyc/approve", post(approve_kyc))
        .route("/api/admin/kyc/reject", post(reject_kyc))
        // ── Notifications ────────────────────────────────────────────────
        .route("/api/notifications", get(list_notifications))
        .route("/api/notifications/:id/read", patch(mark_notification_read))
        // ── Admin Audit Logs ─────────────────────────────────────────────
        .route("/api/admin/logs", get(list_audit_logs))
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

async fn create_plan(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CreatePlanRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate KYC approved
    let kyc_record = KycService::get_kyc_status(&state.db, user.user_id).await?;
    if kyc_record.status != "approved" {
        return Err(ApiError::Forbidden("KYC not approved".to_string()));
    }

    // Require 2FA verification (stub, replace with actual logic)
    // if !verify_2fa(user.user_id, req.2fa_code) {
    //     return Err(ApiError::Forbidden("2FA verification failed".to_string()));
    // }

    // Deduct 2% fee
    let amount = req.net_amount + req.fee;
    let fee = amount * rust_decimal::Decimal::new(2, 2) / rust_decimal::Decimal::new(100, 0);
    let net_amount = amount - fee;

    let mut req_mut = req.clone();
    req_mut.fee = fee;
    req_mut.net_amount = net_amount;

    let plan = PlanService::create_plan(&state.db, user.user_id, &req_mut).await?;

    // Audit log
    sqlx::query("INSERT INTO plan_logs (plan_id, action, performed_by) VALUES ($1, $2, $3)")
        .bind(plan.id)
        .bind("create")
        .bind(user.user_id)
        .execute(&state.db)
        .await?;

    // Notification (stub)
    // notify_plan_created(user.user_id, plan.id);

    Ok(Json(json!({
        "status": "success",
        "data": plan
    })))
}

async fn get_plan(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let plan = PlanService::get_plan_by_id(&state.db, plan_id, user.user_id).await?;
    match plan {
        Some(p) => Ok(Json(json!({
            "status": "success",
            "data": p
        }))),
        None => Err(ApiError::NotFound(format!("Plan {} not found", plan_id))),
    }
}

async fn claim_plan(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<ClaimPlanRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate KYC approved
    let kyc_record = KycService::get_kyc_status(&state.db, user.user_id).await?;
    if kyc_record.status != "approved" {
        return Err(ApiError::Forbidden("KYC not approved".to_string()));
    }

    // Require 2FA verification (stub, replace with actual logic)
    // if !verify_2fa(user.user_id, req.2fa_code) {
    //     return Err(ApiError::Forbidden("2FA verification failed".to_string()));
    // }

    let plan = PlanService::claim_plan(&state.db, plan_id, user.user_id, &req).await?;

    // Transfer USDC to user wallet (stub)
    // transfer_usdc_to_wallet(user.user_id, plan.net_amount);

    // Audit log
    sqlx::query("INSERT INTO plan_logs (plan_id, action, performed_by) VALUES ($1, $2, $3)")
        .bind(plan.id)
        .bind("claim")
        .bind(user.user_id)
        .execute(&state.db)
        .await?;

    // Notification (stub)
    // notify_plan_claimed(user.user_id, plan.id);

    Ok(Json(json!({
        "status": "success",
        "message": "Claim recorded",
        "data": plan
    })))
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

#[derive(serde::Deserialize)]
pub struct KycUpdateRequest {
    pub user_id: Uuid,
}

async fn get_kyc_status(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Path(user_id): Path<Uuid>,
) -> Result<Json<KycRecord>, ApiError> {
    let status = KycService::get_kyc_status(&state.db, user_id).await?;
    Ok(Json(status))
}

async fn approve_kyc(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(payload): Json<KycUpdateRequest>,
) -> Result<Json<KycRecord>, ApiError> {
    let status = KycService::update_kyc_status(
        &state.db,
        admin.admin_id,
        payload.user_id,
        KycStatus::Approved,
    )
    .await?;
    Ok(Json(status))
}

async fn reject_kyc(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(payload): Json<KycUpdateRequest>,
) -> Result<Json<KycRecord>, ApiError> {
    let status = KycService::update_kyc_status(
        &state.db,
        admin.admin_id,
        payload.user_id,
        KycStatus::Rejected,
    )
    .await?;
    Ok(Json(status))
}

// ── Notification Handlers ─────────────────────────────────────────────────────

async fn list_notifications(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let notifications = NotificationService::list_for_user(&state.db, user.user_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": notifications,
        "count": notifications.len()
    })))
}

async fn mark_notification_read(
    State(state): State<Arc<AppState>>,
    Path(notif_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let notification = NotificationService::mark_read(&state.db, notif_id, user.user_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": notification
    })))
}

// ── Admin Audit Log Handler ───────────────────────────────────────────────────

async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let logs = AuditLogService::list_all(&state.db).await?;
    Ok(Json(json!({
        "status": "success",
        "data": logs,
        "count": logs.len()
    })))
}

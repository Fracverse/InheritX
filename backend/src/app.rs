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
        .route("/login", post(crate::auth::login_user))
        .route("/admin/login", post(crate::auth::login_admin))
        .route(
            "/api/auth/nonce/:wallet_address",
            get(crate::auth::generate_nonce),
        )
        .route("/api/auth/wallet-login", post(crate::auth::wallet_login))
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    |State(state): State<Arc<AppState>>,
                     mut req: axum::http::Request<axum::body::Body>,
                     next: axum::middleware::Next| async move {
                        req.extensions_mut().insert(state.config.clone());
                        next.run(req).await
                    },
                ))
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
        .route("/api/plans/:plan_id", axum::routing::delete(cancel_plan))
        .route("/api/plans", post(create_plan))
        .route("/api/kyc/submit", post(submit_kyc))
        .route(
            "/api/admin/plans/due-for-claim",
            get(get_all_due_for_claim_plans_admin),
        )
        .route("/api/admin/kyc/:user_id", get(get_kyc_status))
        .route("/api/admin/kyc/approve", post(approve_kyc))
        .route("/api/admin/kyc/reject", post(reject_kyc))
        .route("/api/kyc", get(get_user_kyc))
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

async fn submit_kyc(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<KycRecord>, ApiError> {
    let status = KycService::submit_kyc(&state.db, user.user_id).await?;
    Ok(Json(status))
}

async fn create_plan(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    headers: axum::http::HeaderMap,
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

    let mut tx = state.db.begin().await?;
    let beneficiary_name = req_mut
        .beneficiary_name
        .as_deref()
        .map(|s| s.trim().to_string());
    let bank_name = req_mut.bank_name.as_deref().map(|s| s.trim().to_string());
    let bank_account_number = req_mut
        .bank_account_number
        .as_deref()
        .map(|s| s.trim().to_string());
    let currency_preference = Some(req_mut.currency_preference.trim().to_uppercase());

    let inserted_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO plans (
            user_id, title, description, fee, net_amount, status,
            beneficiary_name, bank_account_number, bank_name, currency_preference
        )
        VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7, $8, $9)
        RETURNING id
        "#,
    )
    .bind(user.user_id)
    .bind(&req_mut.title)
    .bind(&req_mut.description)
    .bind(req_mut.fee.to_string())
    .bind(req_mut.net_amount.to_string())
    .bind(&beneficiary_name)
    .bind(&bank_account_number)
    .bind(&bank_name)
    .bind(&currency_preference)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO action_logs (user_id, action, entity_id, entity_type)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(Some(user.user_id))
    .bind(crate::notifications::audit_action::PLAN_CREATED)
    .bind(Some(inserted_id))
    .bind(Some(crate::notifications::entity_type::PLAN))
    .execute(&mut *tx)
    .await?;

    let should_revert = headers
        .get("X-Simulate-Revert")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);
    if should_revert {
        tx.rollback().await?;
        return Err(ApiError::Internal(anyhow::anyhow!(
            "Token transfer reverted"
        )));
    }
    tx.commit().await?;
    let plan = PlanService::get_plan_by_id(&state.db, inserted_id, user.user_id)
        .await?
        .expect("plan must exist after commit");

    // Audit log
    sqlx::query("INSERT INTO plan_logs (plan_id, action, performed_by) VALUES ($1, $2, $3)")
        .bind(inserted_id)
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

async fn cancel_plan(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let plan = PlanService::cancel_plan(&state.db, plan_id, user.user_id).await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Plan cancelled successfully",
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

async fn get_user_kyc(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<KycRecord>, ApiError> {
    let status = KycService::get_kyc_status(&state.db, user.user_id).await?;
    Ok(Json(status))
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

use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put},
    Json, Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::analytics::analytics_router;
use crate::api_error::ApiError;
use crate::auth::{AuthenticatedAdmin, AuthenticatedUser};
use crate::config::Config;
use crate::loan_lifecycle::{CreateLoanRequest, LoanLifecycleService, LoanListFilters};
use crate::service::{
    ClaimPlanRequest, CreateEmergencyAccessGrantRequest, CreateEmergencyContactRequest,
    CreatePlanRequest, EmergencyAccessService, EmergencyAdminService, EmergencyContactService,
    KycRecord, KycService, KycStatus, LoanSimulationRequest, LoanSimulationService,
    PausePlanRequest, PlanService, RevokeEmergencyAccessGrantRequest, RiskOverrideRequest,
    UnpausePlanRequest, UpdateEmergencyContactRequest,
};
use crate::yield_service::{DefaultOnChainYieldService, OnChainYieldService};

pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub yield_service: Arc<dyn OnChainYieldService>,
}

pub async fn create_app(db: PgPool, config: Config) -> Result<Router, ApiError> {
    let state = Arc::new(AppState {
        db,
        config,
        yield_service: Arc::new(DefaultOnChainYieldService::new()),
    });

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
            "/api/emergency/contacts",
            get(list_emergency_contacts).post(create_emergency_contact),
        )
        .route(
            "/api/emergency/contacts/:contact_id",
            put(update_emergency_contact).delete(delete_emergency_contact),
        )
        .route(
            "/api/emergency/access/grants",
            post(create_emergency_access_grant),
        )
        .route(
            "/api/emergency/access/grants/:grant_id/revoke",
            post(revoke_emergency_access_grant),
        )
        .route(
            "/api/emergency/access/audit-logs",
            get(list_emergency_access_audit_logs),
        )
        .route(
            "/api/emergency/access/risk-alerts",
            get(list_emergency_access_risk_alerts),
        )
        .route(
            "/api/emergency/access/dashboard",
            get(get_emergency_access_dashboard),
        )
        // Loan Simulation endpoints
        .route("/api/loans/simulate", post(simulate_loan))
        .route("/api/loans/simulations", get(get_user_simulations))
        .route("/api/loans/simulations/:simulation_id", get(get_simulation))
        .route("/api/reputation", get(get_user_reputation))
        // ── Loan Lifecycle Tracker ─────────────────────────────────────────────
        .route("/api/loans/lifecycle", post(create_lifecycle_loan))
        .route("/api/loans/lifecycle", get(list_lifecycle_loans))
        .route("/api/loans/lifecycle/summary", get(get_lifecycle_summary))
        .route("/api/loans/lifecycle/:id", get(get_lifecycle_loan))
        .route("/api/loans/lifecycle/:id/repay", post(repay_lifecycle_loan))
        .route(
            "/api/admin/loans/lifecycle/:id/liquidate",
            post(liquidate_lifecycle_loan),
        )
        .route(
            "/api/admin/loans/lifecycle/mark-overdue",
            post(mark_overdue_loans),
        )
        .route(
            "/api/admin/plans/due-for-claim",
            get(get_all_due_for_claim_plans_admin),
        )
        .route("/api/admin/kyc/:user_id", get(get_kyc_status))
        .route("/api/admin/kyc/approve", post(approve_kyc))
        .route("/api/admin/kyc/reject", post(reject_kyc))
        // Emergency Admin endpoints (pause/unpause/risk-override)
        .route("/api/admin/emergency/pause", post(pause_plan))
        .route("/api/admin/emergency/unpause", post(unpause_plan))
        .route(
            "/api/admin/emergency/risk-override",
            post(set_risk_override),
        )
        .route("/api/admin/emergency/paused-plans", get(get_paused_plans))
        .route(
            "/api/admin/emergency/risk-override-plans",
            get(get_risk_override_plans),
        )
        // ── Emergency Access (Issue #293) ──────────────────────────────────────
        .route(
            "/api/admin/emergency-access/grant",
            post(grant_emergency_access),
        )
        .route(
            "/api/admin/emergency-access/revoke",
            post(revoke_emergency_access),
        )
        .route(
            "/api/admin/emergency-access/all",
            get(get_all_emergency_access),
        )
        .route(
            "/api/admin/emergency-access/plan/:plan_id",
            get(get_plan_emergency_access),
        )
        .merge(analytics_router())
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
    let plan = PlanService::create_plan(&state.db, user.user_id, &req).await?;
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
    let plan = PlanService::claim_plan(&state.db, plan_id, user.user_id, &req).await?;
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

async fn list_emergency_contacts(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let contacts = EmergencyContactService::list_for_user(&state.db, user.user_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": contacts, "count": contacts.len() }),
    ))
}

async fn create_emergency_contact(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CreateEmergencyContactRequest>,
) -> Result<Json<Value>, ApiError> {
    let contact = EmergencyContactService::create_contact(&state.db, user.user_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": contact })))
}

async fn update_emergency_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<UpdateEmergencyContactRequest>,
) -> Result<Json<Value>, ApiError> {
    let contact =
        EmergencyContactService::update_contact(&state.db, user.user_id, contact_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": contact })))
}

async fn delete_emergency_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let result =
        EmergencyContactService::delete_contact(&state.db, user.user_id, contact_id).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn create_emergency_access_grant(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CreateEmergencyAccessGrantRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = EmergencyAccessService::grant_access(&state.db, user.user_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn revoke_emergency_access_grant(
    State(state): State<Arc<AppState>>,
    Path(grant_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<RevokeEmergencyAccessGrantRequest>,
) -> Result<Json<Value>, ApiError> {
    let result =
        EmergencyAccessService::revoke_access(&state.db, user.user_id, grant_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn list_emergency_access_audit_logs(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let logs = EmergencyAccessService::list_audit_logs(&state.db, user.user_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": logs, "count": logs.len() }),
    ))
}

async fn list_emergency_access_risk_alerts(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let alerts = EmergencyAccessService::list_risk_alerts(&state.db, user.user_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": alerts, "count": alerts.len() }),
    ))
}

async fn get_emergency_access_dashboard(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let dashboard = EmergencyAccessService::get_dashboard(&state.db, user.user_id).await?;
    Ok(Json(json!({ "status": "success", "data": dashboard })))
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

// =============================================================================
// Loan Simulation Endpoints
// =============================================================================

/// Preview loan simulation without saving
async fn simulate_loan(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<LoanSimulationRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = LoanSimulationService::create_simulation(&state.db, user.user_id, &req).await?;
    Ok(Json(json!({
        "status": "success",
        "data": result
    })))
}

/// Get all loan simulations for the current user
async fn get_user_simulations(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let limit = 50; // Default limit
    let simulations =
        LoanSimulationService::get_user_simulations(&state.db, user.user_id, limit).await?;
    Ok(Json(json!({
        "status": "success",
        "data": simulations,
        "count": simulations.len()
    })))
}

/// Get a specific loan simulation by ID
async fn get_simulation(
    State(state): State<Arc<AppState>>,
    Path(simulation_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let simulation =
        LoanSimulationService::get_simulation_by_id(&state.db, simulation_id, user.user_id).await?;
    match simulation {
        Some(sim) => Ok(Json(json!({
            "status": "success",
            "data": sim
        }))),
        None => Err(ApiError::NotFound(format!(
            "Simulation {} not found",
            simulation_id
        ))),
    }
}

// =============================================================================
// Reputation Endpoints
// =============================================================================

/// Get the current user's borrower reputation
async fn get_user_reputation(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let reputation =
        crate::reputation::ReputationService::get_reputation(&state.db, user.user_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": reputation
    })))
}

// =============================================================================
// Loan Lifecycle Endpoints
// =============================================================================

/// Open a new loan in the `active` state.
///
/// `POST /api/loans/lifecycle`
async fn create_lifecycle_loan(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(mut req): Json<CreateLoanRequest>,
) -> Result<Json<Value>, ApiError> {
    // Override user_id from the authenticated token to prevent impersonation.
    req.user_id = user.user_id;
    let record = LoanLifecycleService::create_loan(&state.db, &req).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// List loans, optionally filtered by status.
///
/// `GET /api/loans/lifecycle[?status=active|repaid|overdue|liquidated]`
async fn list_lifecycle_loans(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(mut filters): Query<LoanListFilters>,
) -> Result<Json<Value>, ApiError> {
    // Users may only see their own loans.
    filters.user_id = Some(user.user_id);
    let loans = LoanLifecycleService::list_loans(&state.db, &filters).await?;
    Ok(Json(json!({
        "status": "success",
        "data": loans,
        "count": loans.len()
    })))
}

/// Aggregate counts by lifecycle status for the authenticated user.
///
/// `GET /api/loans/lifecycle/summary`
async fn get_lifecycle_summary(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let summary =
        LoanLifecycleService::get_lifecycle_summary(&state.db, Some(user.user_id)).await?;
    Ok(Json(json!({ "status": "success", "data": summary })))
}

/// Fetch a single loan by its UUID.
///
/// `GET /api/loans/lifecycle/:id`
async fn get_lifecycle_loan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let record = LoanLifecycleService::get_loan(&state.db, id).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// Apply a repayment to a loan.  When cumulative repayments reach or exceed
/// the principal the loan transitions to `repaid`.
///
/// `POST /api/loans/lifecycle/:id/repay`
#[derive(serde::Deserialize)]
struct RepayRequest {
    amount: rust_decimal::Decimal,
}

async fn repay_lifecycle_loan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<RepayRequest>,
) -> Result<Json<Value>, ApiError> {
    let record = LoanLifecycleService::repay_loan(&state.db, id, user.user_id, req.amount).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// Admin: forcefully liquidate a loan.
///
/// `POST /api/admin/loans/lifecycle/:id/liquidate`
async fn liquidate_lifecycle_loan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let record = LoanLifecycleService::liquidate_loan(&state.db, id, admin.admin_id).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// Admin: sweep all active loans whose due_date has passed and mark them
/// `overdue`.  Designed to be triggered by a cron / background job.
///
/// `POST /api/admin/loans/lifecycle/mark-overdue`
async fn mark_overdue_loans(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let marked_ids = LoanLifecycleService::mark_overdue_loans(&state.db).await?;
    Ok(Json(json!({
        "status": "success",
        "marked_overdue": marked_ids.len(),
        "loan_ids": marked_ids
    })))
}

// =============================================================================
// Emergency Access Endpoints (Issue #293)
// =============================================================================

use crate::emergency_access::{
    EmergencyAccessService as LegacyEmergencyAccessService, GrantEmergencyAccessRequest,
    RevokeEmergencyAccessRequest,
};

/// Admin: Grant emergency access to a plan
///
/// `POST /api/admin/emergency-access/grant`
async fn grant_emergency_access(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<GrantEmergencyAccessRequest>,
) -> Result<Json<Value>, ApiError> {
    let response =
        LegacyEmergencyAccessService::grant_access(&state.db, admin.admin_id, &req).await?;
    Ok(Json(json!({
        "status": "success",
        "data": response
    })))
}

/// Admin: Revoke emergency access
///
/// `POST /api/admin/emergency-access/revoke`
async fn revoke_emergency_access(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<RevokeEmergencyAccessRequest>,
) -> Result<Json<Value>, ApiError> {
    let response =
        LegacyEmergencyAccessService::revoke_access(&state.db, admin.admin_id, &req).await?;
    Ok(Json(json!({
        "status": "success",
        "data": response
    })))
}

/// Admin: Get all emergency access records
///
/// `GET /api/admin/emergency-access/all`
async fn get_all_emergency_access(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let access_records = LegacyEmergencyAccessService::get_all_access(&state.db).await?;
    Ok(Json(json!({
        "status": "success",
        "data": access_records,
        "count": access_records.len()
    })))
}

/// Admin: Get emergency access records for a specific plan
///
/// `GET /api/admin/emergency-access/plan/:plan_id`
async fn get_plan_emergency_access(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let access_records =
        LegacyEmergencyAccessService::get_active_access_for_plan(&state.db, plan_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": access_records,
        "count": access_records.len()
    })))
}

// Emergency Admin Endpoints
// =============================================================================

async fn pause_plan(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<PausePlanRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = EmergencyAdminService::pause_plan(&state.db, admin.admin_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn unpause_plan(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<UnpausePlanRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = EmergencyAdminService::unpause_plan(&state.db, admin.admin_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn set_risk_override(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<RiskOverrideRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = EmergencyAdminService::set_risk_override(&state.db, admin.admin_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn get_paused_plans(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let plans = EmergencyAdminService::get_paused_plans(&state.db).await?;
    Ok(Json(
        json!({ "status": "success", "data": plans, "count": plans.len() }),
    ))
}

async fn get_risk_override_plans(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let plans = EmergencyAdminService::get_risk_override_plans(&state.db).await?;
    Ok(Json(
        json!({ "status": "success", "data": plans, "count": plans.len() }),
    ))
}

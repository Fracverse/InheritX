//! Admin dashboard and management endpoints (issue #1268).
//!
//! Every handler here is registered behind [`crate::auth::jwt_auth_middleware`],
//! so an admin JWT is required. State-changing operations validate their input,
//! return an explicit 404 for missing records and append an `audit_logs` entry.
//! Responses use the stable `{ "status": "ok", "data": ... }` envelope the
//! frontend `AdminAPI` expects.

use crate::api::AppState;
use crate::auth::UserContext;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

fn internal_error(action: &str, err: sqlx::Error) -> Response {
    tracing::error!(error = %err, "Failed to {action}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": format!("Failed to {action}") })),
    )
        .into_response()
}

async fn record_audit(
    state: &Arc<AppState>,
    admin_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    metadata: Value,
) {
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO audit_logs (admin_id, action, entity_id, entity_type, metadata)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(admin_id)
    .bind(action)
    .bind(entity_id)
    .bind(entity_type)
    .bind(metadata)
    .execute(&state.db_pool)
    .await
    {
        tracing::error!(error = %e, action, entity_id, "Failed to write audit log");
    }
}

// ─── Metrics ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AdminMetrics {
    pub total_revenue: f64,
    pub total_plans: i64,
    pub total_claims: i64,
    pub active_plans: i64,
    pub total_users: i64,
}

/// GET /api/admin/metrics
pub async fn get_metrics(State(state): State<Arc<AppState>>) -> Response {
    let revenue: Option<Decimal> = match sqlx::query_scalar::<_, Option<Decimal>>(
        "SELECT COALESCE(SUM(amount), 0) FROM payouts WHERE status = 'completed'",
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(value) => value,
        Err(e) => return internal_error("load revenue metrics", e),
    };

    let total_plans: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM plans")
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(value) => value,
        Err(e) => return internal_error("count plans", e),
    };

    let active_plans: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM plans WHERE is_active = TRUE AND is_paused = FALSE",
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(value) => value,
        Err(e) => return internal_error("count active plans", e),
    };

    let total_claims: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM payouts")
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(value) => value,
        Err(e) => return internal_error("count claims", e),
    };

    let total_users: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(value) => value,
        Err(e) => return internal_error("count users", e),
    };

    let metrics = AdminMetrics {
        total_revenue: revenue
            .unwrap_or(Decimal::ZERO)
            .to_string()
            .parse()
            .unwrap_or(0.0),
        total_plans,
        total_claims,
        active_plans,
        total_users,
    };

    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "data": metrics })),
    )
        .into_response()
}

// ─── Audit log ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditLogRow {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub admin_id: Option<String>,
    pub action: String,
    pub entity_id: Option<String>,
    pub entity_type: Option<String>,
    pub metadata: Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

/// GET /api/admin/logs
pub async fn get_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PaginationQuery>,
) -> Response {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let total: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs")
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(value) => value,
        Err(e) => return internal_error("count audit logs", e),
    };

    let rows = sqlx::query_as::<_, AuditLogRow>(
        r#"
        SELECT id, user_id, admin_id, action, entity_id, entity_type,
               metadata, created_at AS timestamp
        FROM audit_logs
        ORDER BY created_at DESC, id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db_pool)
    .await;

    let rows = match rows {
        Ok(rows) => rows,
        Err(e) => return internal_error("load audit logs", e),
    };

    let total_pages = ((total as f64) / (limit as f64)).ceil() as i64;

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "data": rows,
            "page": page,
            "limit": limit,
            "total_count": total,
            "total_pages": total_pages,
            "has_next": page < total_pages,
            "has_prev": page > 1,
        })),
    )
        .into_response()
}

// ─── KYC moderation ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AdminKycStatus {
    pub user_id: String,
    pub status: String,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct KycStatusRow {
    wallet_address: String,
    kyc_status: String,
    created_at: DateTime<Utc>,
    reviewed_by: Option<String>,
    reviewed_at: Option<DateTime<Utc>>,
    rejection_reason: Option<String>,
}

async fn load_kyc_status(
    state: &Arc<AppState>,
    user_id: &str,
) -> Result<Option<AdminKycStatus>, sqlx::Error> {
    let row = sqlx::query_as::<_, KycStatusRow>(
        r#"
        SELECT u.wallet_address,
               u.kyc_status::text AS kyc_status,
               u.created_at,
               k.reviewed_by,
               k.reviewed_at,
               k.rejection_reason
        FROM users u
        LEFT JOIN kyc_records k ON k.wallet_address = u.wallet_address
        WHERE u.wallet_address = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await?;

    Ok(row.map(|row| AdminKycStatus {
        user_id: row.wallet_address,
        status: row.kyc_status,
        reviewed_by: row.reviewed_by,
        reviewed_at: row.reviewed_at,
        rejection_reason: row.rejection_reason,
        created_at: row.created_at,
    }))
}

/// GET /api/admin/kyc/{user_id}
pub async fn get_kyc_status(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Response {
    if user_id.trim().is_empty() {
        return error(StatusCode::BAD_REQUEST, "user_id is required");
    }

    match load_kyc_status(&state, &user_id).await {
        Ok(Some(status)) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "data": status })),
        )
            .into_response(),
        Ok(None) => error(StatusCode::NOT_FOUND, "KYC record not found"),
        Err(e) => internal_error("load KYC status", e),
    }
}

#[derive(Debug, Deserialize)]
pub struct KycDecisionRequest {
    pub user_id: String,
    pub reason: Option<String>,
}

async fn decide_kyc(
    state: &Arc<AppState>,
    admin: &UserContext,
    payload: KycDecisionRequest,
    approve: bool,
) -> Response {
    let user_id = payload.user_id.trim().to_string();
    if user_id.is_empty() {
        return error(StatusCode::BAD_REQUEST, "user_id is required");
    }

    let reason = payload
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty());

    if !approve && reason.is_none() {
        return error(
            StatusCode::BAD_REQUEST,
            "reason is required when rejecting KYC",
        );
    }

    if let Some(reason) = reason {
        if reason.len() > 500 {
            return error(
                StatusCode::BAD_REQUEST,
                "reason must be 500 characters or fewer",
            );
        }
    }

    let status = if approve { "approved" } else { "rejected" };

    let mut tx = match state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_error("begin KYC decision transaction", e),
    };

    let updated = sqlx::query(
        r#"
        UPDATE users SET kyc_status = $1::kyc_status WHERE wallet_address = $2
        "#,
    )
    .bind(status)
    .bind(&user_id)
    .execute(&mut *tx)
    .await;

    match updated {
        Ok(result) if result.rows_affected() == 0 => {
            return error(StatusCode::NOT_FOUND, "User not found");
        }
        Ok(_) => {}
        Err(e) => return internal_error("update KYC status", e),
    }

    if let Err(e) = sqlx::query(
        r#"
        UPDATE kyc_records
        SET status = $1,
            reviewed_by = $2,
            reviewed_at = NOW(),
            rejection_reason = $3
        WHERE wallet_address = $4
        "#,
    )
    .bind(status)
    .bind(&admin.user_id)
    .bind(if approve { None } else { reason })
    .bind(&user_id)
    .execute(&mut *tx)
    .await
    {
        return internal_error("update KYC review metadata", e);
    }

    if let Err(e) = tx.commit().await {
        return internal_error("commit KYC decision", e);
    }

    record_audit(
        state,
        &admin.user_id,
        if approve { "kyc.approve" } else { "kyc.reject" },
        "kyc",
        &user_id,
        json!({ "status": status, "reason": reason }),
    )
    .await;

    match load_kyc_status(state, &user_id).await {
        Ok(Some(status)) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "data": status })),
        )
            .into_response(),
        Ok(None) => error(StatusCode::NOT_FOUND, "KYC record not found"),
        Err(e) => internal_error("load KYC status", e),
    }
}

/// POST /api/admin/kyc/approve
pub async fn approve_kyc(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<UserContext>,
    Json(payload): Json<KycDecisionRequest>,
) -> Response {
    decide_kyc(&state, &admin, payload, true).await
}

/// POST /api/admin/kyc/reject
pub async fn reject_kyc(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<UserContext>,
    Json(payload): Json<KycDecisionRequest>,
) -> Response {
    decide_kyc(&state, &admin, payload, false).await
}

// ─── Insurance fund ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct InsuranceFundRow {
    pub id: Uuid,
    pub name: String,
    pub asset_code: String,
    pub balance: Decimal,
    pub total_deposits: Decimal,
    pub total_withdrawals: Decimal,
    pub total_claims_paid: Decimal,
    pub reserve_ratio: Decimal,
    pub health_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct InsuranceFundMetrics {
    pub current_balance: Decimal,
    pub total_claims: i64,
    pub pending_claims: i64,
    pub approved_claims: i64,
    pub rejected_claims: i64,
    pub total_claims_amount: Decimal,
    pub reserve_ratio: Decimal,
    pub health_status: String,
}

#[derive(Debug, Serialize)]
pub struct InsuranceFundDashboard {
    pub fund: InsuranceFundRow,
    pub metrics: InsuranceFundMetrics,
    pub recent_transactions: Vec<Value>,
}

#[derive(Debug, sqlx::FromRow)]
struct ClaimCounts {
    total: i64,
    pending: i64,
    approved: i64,
    rejected: i64,
    amount: Option<Decimal>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RecentPayout {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub beneficiary_address: String,
    pub amount: Decimal,
    pub payout_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

const FUND_COLUMNS: &str = "id, name, asset_code, balance, total_deposits, total_withdrawals, \
                            total_claims_paid, reserve_ratio, health_status, created_at, updated_at";

async fn load_dashboard(
    state: &Arc<AppState>,
) -> Result<Option<InsuranceFundDashboard>, sqlx::Error> {
    let fund = sqlx::query_as::<_, InsuranceFundRow>(&format!(
        "SELECT {FUND_COLUMNS} FROM insurance_funds ORDER BY created_at ASC LIMIT 1"
    ))
    .fetch_optional(&state.db_pool)
    .await?;

    let Some(fund) = fund else {
        return Ok(None);
    };

    let counts = sqlx::query_as::<_, ClaimCounts>(
        r#"
        SELECT COUNT(*) AS total,
               COUNT(*) FILTER (WHERE status = 'pending') AS pending,
               COUNT(*) FILTER (WHERE status = 'completed') AS approved,
               COUNT(*) FILTER (WHERE status = 'failed') AS rejected,
               COALESCE(SUM(amount) FILTER (WHERE status = 'completed'), 0) AS amount
        FROM payouts
        "#,
    )
    .fetch_one(&state.db_pool)
    .await?;

    let recent = sqlx::query_as::<_, RecentPayout>(
        r#"
        SELECT id, plan_id, beneficiary_address, amount, payout_type::text, status::text, created_at
        FROM payouts
        ORDER BY created_at DESC
        LIMIT 10
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    let recent_transactions = recent
        .into_iter()
        .map(|row| serde_json::to_value(row).unwrap_or_else(|_| json!({})))
        .collect();

    Ok(Some(InsuranceFundDashboard {
        metrics: InsuranceFundMetrics {
            current_balance: fund.balance,
            total_claims: counts.total,
            pending_claims: counts.pending,
            approved_claims: counts.approved,
            rejected_claims: counts.rejected,
            total_claims_amount: counts.amount.unwrap_or(Decimal::ZERO),
            reserve_ratio: fund.reserve_ratio,
            health_status: fund.health_status.clone(),
        },
        fund,
        recent_transactions,
    }))
}

/// GET /api/admin/insurance-fund
pub async fn get_insurance_fund_dashboard(State(state): State<Arc<AppState>>) -> Response {
    match load_dashboard(&state).await {
        Ok(Some(dashboard)) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "data": dashboard })),
        )
            .into_response(),
        Ok(None) => error(StatusCode::NOT_FOUND, "Insurance fund not found"),
        Err(e) => internal_error("load insurance fund", e),
    }
}

/// GET /api/admin/insurance-funds
pub async fn get_all_insurance_funds(State(state): State<Arc<AppState>>) -> Response {
    let funds = sqlx::query_as::<_, InsuranceFundRow>(&format!(
        "SELECT {FUND_COLUMNS} FROM insurance_funds ORDER BY created_at ASC"
    ))
    .fetch_all(&state.db_pool)
    .await;

    match funds {
        Ok(funds) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "data": funds })),
        )
            .into_response(),
        Err(e) => internal_error("load insurance funds", e),
    }
}

// ─── Emergency controls ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct EmergencyPauseRequest {
    pub plan_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct EmergencyUnpauseRequest {
    pub plan_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RiskOverrideRequest {
    pub plan_id: Uuid,
    pub enabled: bool,
    pub reason: String,
}

fn validate_reason(reason: &str) -> Result<String, (StatusCode, &'static str)> {
    let reason = reason.trim();
    if reason.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "reason is required"));
    }
    if reason.len() > 500 {
        return Err((
            StatusCode::BAD_REQUEST,
            "reason must be 500 characters or fewer",
        ));
    }
    Ok(reason.to_string())
}

async fn plan_exists(state: &Arc<AppState>, plan_id: Uuid) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count > 0)
}

/// POST /api/admin/emergency/pause
pub async fn pause_plan(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<UserContext>,
    Json(payload): Json<EmergencyPauseRequest>,
) -> Response {
    let reason = match validate_reason(&payload.reason) {
        Ok(reason) => reason,
        Err((status, message)) => return error(status, message),
    };

    let result = sqlx::query(
        r#"
        UPDATE plans
        SET is_paused = TRUE, paused_reason = $1, paused_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(&reason)
    .bind(payload.plan_id)
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(result) if result.rows_affected() == 0 => error(StatusCode::NOT_FOUND, "Plan not found"),
        Ok(_) => {
            record_audit(
                &state,
                &admin.user_id,
                "emergency.pause",
                "plan",
                &payload.plan_id.to_string(),
                json!({ "reason": reason }),
            )
            .await;

            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "data": { "plan_id": payload.plan_id, "is_paused": true, "reason": reason }
                })),
            )
                .into_response()
        }
        Err(e) => internal_error("pause plan", e),
    }
}

/// POST /api/admin/emergency/unpause
pub async fn unpause_plan(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<UserContext>,
    Json(payload): Json<EmergencyUnpauseRequest>,
) -> Response {
    let result = sqlx::query(
        r#"
        UPDATE plans
        SET is_paused = FALSE, paused_reason = NULL, paused_at = NULL
        WHERE id = $1
        "#,
    )
    .bind(payload.plan_id)
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(result) if result.rows_affected() == 0 => error(StatusCode::NOT_FOUND, "Plan not found"),
        Ok(_) => {
            record_audit(
                &state,
                &admin.user_id,
                "emergency.unpause",
                "plan",
                &payload.plan_id.to_string(),
                json!({}),
            )
            .await;

            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "data": { "plan_id": payload.plan_id, "is_paused": false }
                })),
            )
                .into_response()
        }
        Err(e) => internal_error("unpause plan", e),
    }
}

/// POST /api/admin/emergency/risk-override
pub async fn set_risk_override(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<UserContext>,
    Json(payload): Json<RiskOverrideRequest>,
) -> Response {
    let reason = match validate_reason(&payload.reason) {
        Ok(reason) => reason,
        Err((status, message)) => return error(status, message),
    };

    if !matches!(plan_exists(&state, payload.plan_id).await, Ok(true)) {
        return error(StatusCode::NOT_FOUND, "Plan not found");
    }

    let result = sqlx::query(
        r#"
        UPDATE plans
        SET risk_override_enabled = $1,
            risk_override_reason = $2,
            risk_override_at = NOW()
        WHERE id = $3
        "#,
    )
    .bind(payload.enabled)
    .bind(&reason)
    .bind(payload.plan_id)
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => {
            record_audit(
                &state,
                &admin.user_id,
                "emergency.risk_override",
                "plan",
                &payload.plan_id.to_string(),
                json!({ "enabled": payload.enabled, "reason": reason }),
            )
            .await;

            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "data": {
                        "plan_id": payload.plan_id,
                        "risk_override_enabled": payload.enabled,
                        "reason": reason
                    }
                })),
            )
                .into_response()
        }
        Err(e) => internal_error("set risk override", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_validation_rejects_blank_and_oversized_values() {
        assert!(validate_reason("  ").is_err());
        assert!(validate_reason(&"x".repeat(501)).is_err());
        assert_eq!(validate_reason("  maintenance ").unwrap(), "maintenance");
    }

    #[test]
    fn metrics_serialize_with_frontend_field_names() {
        let metrics = AdminMetrics {
            total_revenue: 1.5,
            total_plans: 2,
            total_claims: 3,
            active_plans: 4,
            total_users: 5,
        };
        let value = serde_json::to_value(metrics).unwrap();
        for key in [
            "total_revenue",
            "total_plans",
            "total_claims",
            "active_plans",
            "total_users",
        ] {
            assert!(value.get(key).is_some(), "missing {key}");
        }
    }
}

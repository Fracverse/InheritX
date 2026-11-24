//! Plan-client route alignment (issue #1275).
//!
//! `PlansAPI` targets `/api/plans/{id}/trigger`, `/keep-alive` and
//! `/inactivity-status`; these handlers back those canonical paths. All three
//! are registered under the authenticated router and enforce that the caller is
//! the plan owner or an admin.

use crate::api::{load_beneficiaries, plan_row_to_response, AppState, PlanResponse, PlanRow};
use crate::auth::UserContext;
use crate::will_documents::caller_can_access;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

const PLAN_COLUMNS: &str = "id, owner_address, token_address, amount, grace_period, \
                            grace_period_seconds, earn_yield, last_ping, is_active, status, \
                            yield_rate_bps, accrued_yield, created_at, onchain_plan_id";

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

async fn fetch_plan(state: &Arc<AppState>, plan_id: Uuid) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as::<_, PlanRow>(&format!("SELECT {PLAN_COLUMNS} FROM plans WHERE id = $1"))
        .bind(plan_id)
        .fetch_optional(&state.db_pool)
        .await
}

async fn plan_response(
    state: &Arc<AppState>,
    plan_id: Uuid,
) -> Result<Option<PlanResponse>, sqlx::Error> {
    let Some(row) = fetch_plan(state, plan_id).await? else {
        return Ok(None);
    };
    let beneficiaries = load_beneficiaries(&state.db_pool, row.id).await?;
    Ok(Some(plan_row_to_response(row, beneficiaries)))
}

async fn authorize(
    state: &Arc<AppState>,
    ctx: &UserContext,
    plan_id: Uuid,
) -> Result<PlanRow, Response> {
    match fetch_plan(state, plan_id).await {
        Ok(Some(plan)) if caller_can_access(ctx, &plan.owner_address) => Ok(plan),
        Ok(Some(_)) => Err(error(
            StatusCode::FORBIDDEN,
            "You do not have access to this plan",
        )),
        Ok(None) => Err(error(StatusCode::NOT_FOUND, "Plan not found")),
        Err(e) => Err(internal_error("load plan", e)),
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct KeepAliveRequest {
    pub signed_transaction: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct InactivityRow {
    last_ping: i64,
    grace_period_seconds: i64,
    status: String,
    is_active: bool,
}

/// GET /api/plans/{id}/inactivity-status
pub async fn get_inactivity_status(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(plan_id): Path<Uuid>,
) -> Response {
    let row = match sqlx::query_as::<_, InactivityRow>(
        "SELECT last_ping, grace_period_seconds, status, is_active FROM plans WHERE id = $1",
    )
    .bind(plan_id)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return error(StatusCode::NOT_FOUND, "Plan not found"),
        Err(e) => return internal_error("load plan inactivity status", e),
    };

    let owner: Option<String> =
        match sqlx::query_scalar("SELECT owner_address FROM plans WHERE id = $1")
            .bind(plan_id)
            .fetch_optional(&state.db_pool)
            .await
        {
            Ok(owner) => owner,
            Err(e) => return internal_error("load plan owner", e),
        };

    if let Some(owner) = owner {
        if !caller_can_access(&ctx, &owner) {
            return error(StatusCode::FORBIDDEN, "You do not have access to this plan");
        }
    }

    let now = Utc::now().timestamp();
    let deadline = row.last_ping.saturating_add(row.grace_period_seconds);
    let remaining_seconds = deadline.saturating_sub(now);
    let inactivity_period_days = row.grace_period_seconds / 86_400;
    let days_until_claimable = if remaining_seconds > 0 {
        (remaining_seconds as f64 / 86_400.0).ceil() as i64
    } else {
        0
    };
    let is_claimable = remaining_seconds <= 0
        || matches!(row.status.as_str(), "TRIGGERED" | "CLAIMABLE")
        || !row.is_active;

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "data": {
                // The client computes deadlines in milliseconds.
                "last_ping_timestamp": row.last_ping.saturating_mul(1000),
                "inactivity_period_days": inactivity_period_days,
                "days_until_claimable": days_until_claimable,
                "is_claimable": is_claimable,
            }
        })),
    )
        .into_response()
}

/// POST /api/plans/{id}/keep-alive
pub async fn keep_alive(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(plan_id): Path<Uuid>,
    Json(payload): Json<KeepAliveRequest>,
) -> Response {
    let plan = match authorize(&state, &ctx, plan_id).await {
        Ok(plan) => plan,
        Err(response) => return response,
    };

    if !plan.is_active || !matches!(plan.status.as_str(), "ACTIVE" | "PAUSED") {
        return error(
            StatusCode::CONFLICT,
            "Plan is not active and cannot receive a keep-alive ping",
        );
    }

    // Optional on-chain proof-of-life is accepted for clients that sign one;
    // the database timer is reset regardless once the caller is authorised.
    let _signed_transaction = payload
        .signed_transaction
        .filter(|tx| !tx.trim().is_empty());

    let now = Utc::now().timestamp();
    let elapsed = (now - plan.last_ping).max(0) as u64;
    let mut accrued_yield = plan.accrued_yield;

    if plan.earn_yield && elapsed > 0 {
        let amount = plan.amount.to_string().parse::<f64>().unwrap_or(0.0);
        let accrued =
            crate::yield_calculator::calculate_yield(amount, plan.yield_rate_bps as u32, elapsed);
        if let Some(value) = Decimal::from_f64_retain(accrued) {
            accrued_yield += value;
        }
    }

    let mut tx = match state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_error("begin keep-alive transaction", e),
    };

    if let Err(e) = sqlx::query("UPDATE plans SET last_ping = $1, accrued_yield = $2 WHERE id = $3")
        .bind(now)
        .bind(accrued_yield)
        .bind(plan_id)
        .execute(&mut *tx)
        .await
    {
        return internal_error("update plan keep-alive", e);
    }

    if let Err(e) = sqlx::query(
        "INSERT INTO ping_logs (plan_id, pinged_at, accrued_yield_snapshot) VALUES ($1, NOW(), $2)",
    )
    .bind(plan_id)
    .bind(accrued_yield)
    .execute(&mut *tx)
    .await
    {
        return internal_error("record keep-alive ping", e);
    }

    if let Err(e) = tx.commit().await {
        return internal_error("commit keep-alive ping", e);
    }

    match plan_response(&state, plan_id).await {
        Ok(Some(plan)) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "data": plan })),
        )
            .into_response(),
        Ok(None) => error(StatusCode::NOT_FOUND, "Plan not found"),
        Err(e) => internal_error("reload plan", e),
    }
}

/// POST /api/plans/{id}/trigger
pub async fn trigger_plan(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(plan_id): Path<Uuid>,
) -> Response {
    let plan = match authorize(&state, &ctx, plan_id).await {
        Ok(plan) => plan,
        Err(response) => return response,
    };

    if plan.status != "ACTIVE" {
        return error(
            StatusCode::CONFLICT,
            &format!(
                "Plan cannot be triggered from status {}; only ACTIVE plans can be triggered",
                plan.status
            ),
        );
    }

    let is_paused = match sqlx::query_scalar::<_, bool>("SELECT is_paused FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_optional(&state.db_pool)
        .await
    {
        Ok(Some(is_paused)) => is_paused,
        Ok(None) => return error(StatusCode::NOT_FOUND, "Plan not found"),
        Err(e) => return internal_error("load plan pause state", e),
    };

    if is_paused {
        return error(
            StatusCode::CONFLICT,
            "Plan is paused by an administrator; unpause it before triggering",
        );
    }

    let mut tx_hash: Option<String> = None;
    let mut last_error: Option<String> = None;

    if let Some(onchain_plan_id) = plan.onchain_plan_id {
        if state.stellar_submit.soroban_enabled() {
            let Ok(onchain_plan_id) = u64::try_from(onchain_plan_id) else {
                return error(StatusCode::CONFLICT, "Plan has an invalid on-chain id");
            };

            match state
                .stellar_submit
                .trigger_inheritance(onchain_plan_id)
                .await
            {
                Ok(outcome) => tx_hash = Some(outcome.tx_hash),
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }
        }
    }

    if let Some(message) = last_error {
        // Leave the plan ACTIVE so the operation can be retried, and record the
        // failure for the audit trail.
        let _ = sqlx::query(
            "UPDATE plans SET trigger_attempts = trigger_attempts + 1, last_trigger_error = $1 WHERE id = $2",
        )
        .bind(&message)
        .bind(plan_id)
        .execute(&state.db_pool)
        .await;

        return error(
            StatusCode::BAD_GATEWAY,
            &format!("On-chain inheritance trigger failed: {message}"),
        );
    }

    let updated = sqlx::query(
        r#"
        UPDATE plans
        SET status = 'TRIGGERED',
            trigger_started_at = NOW(),
            trigger_attempts = trigger_attempts + 1,
            trigger_tx_hash = $1,
            last_trigger_error = NULL
        WHERE id = $2
        "#,
    )
    .bind(tx_hash.as_deref())
    .bind(plan_id)
    .execute(&state.db_pool)
    .await;

    if let Err(e) = updated {
        return internal_error("mark plan triggered", e);
    }

    match plan_response(&state, plan_id).await {
        Ok(Some(plan)) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "message": "Inheritance triggered successfully",
                "data": plan,
            })),
        )
            .into_response(),
        Ok(None) => error(StatusCode::NOT_FOUND, "Plan not found"),
        Err(e) => internal_error("reload plan", e),
    }
}

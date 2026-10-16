#![allow(clippy::result_large_err)]

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use tracing::{error, warn};
use uuid::Uuid;

use crate::api::AppState;
use crate::stellar_submit::{event_u64_field, find_event, InvocationOutcome, StellarSubmitError};
use crate::ws::PlanStatusEvent;

const FREEZE_EVENT_TOPICS: [&str; 2] = ["LOAN", "FREEZE"];
const RECALL_EVENT_TOPICS: [&str; 2] = ["LOAN", "RECALL"];
const LIQUIDATE_EVENT_TOPICS: [&str; 2] = ["LOAN", "LIQUIDAT"];

const ALLOWED_STATUSES: [&str; 2] = ["TRIGGERED", "CLAIMABLE"];

#[derive(Debug, Default, Deserialize)]
pub struct LoanLifecycleRequest {
    pub recall_amount: Option<u64>,
}

#[derive(Debug, Clone, FromRow)]
struct PlanRef {
    id: Uuid,
    owner_address: String,
    status: String,
    is_active: bool,
    onchain_plan_id: Option<i64>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
struct LoanLifecycleRow {
    plan_id: Uuid,
    freeze_status: String,
    recall_progress: i32,
    settlement_status: String,
    outstanding_loaned: i64,
    last_tx_hash: Option<String>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct OutstandingLoan {
    pool: String,
    amount: String,
    status: String,
}

pub async fn freeze_loans(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    Json(_payload): Json<LoanLifecycleRequest>,
) -> Response {
    let plan = match load_plan(&state, plan_id).await {
        Ok(plan) => plan,
        Err(response) => return response,
    };
    if let Err(response) = require_triggered(&plan) {
        return response;
    }

    let mut tx_hash = None;
    let mut remaining = None;

    if state.stellar_submit.soroban_enabled() {
        let onchain_id = match require_onchain_id(&plan) {
            Ok(id) => id,
            Err(response) => return response,
        };
        match state.stellar_submit.freeze_loans(onchain_id).await {
            Ok(outcome) => {
                if let Err(response) = verify_loan_event(
                    &state,
                    &outcome,
                    onchain_id,
                    &FREEZE_EVENT_TOPICS,
                    "LOAN/FREEZE",
                ) {
                    return response;
                }
                tx_hash = Some(outcome.tx_hash);
            }
            Err(error) => return stellar_error_response(&error, "freeze loans"),
        }
        remaining = outstanding_after(&state, onchain_id).await;
    }

    let remaining_loaned = remaining.unwrap_or(0);
    if let Err(response) = upsert_lifecycle(
        &state,
        plan.id,
        Some("FROZEN"),
        None,
        None,
        remaining_loaned,
        tx_hash.as_deref(),
    )
    .await
    {
        return response;
    }

    invalidate_cache(&state, &plan).await;
    emit_status(
        &state,
        PlanStatusEvent {
            event_type: "plan.loans_frozen".into(),
            plan_id: plan.id,
            status: "FROZEN".into(),
            message: "Loans frozen successfully".into(),
            tx_hash: tx_hash.clone(),
            freeze_status: Some("FROZEN".into()),
            recall_progress: None,
            settlement_status: None,
            remaining_loaned: Some(remaining_loaned),
        },
        "plan.loans_frozen",
        &plan,
        tx_hash.as_deref(),
    )
    .await;

    ok_response(
        "Loans frozen successfully",
        serde_json::json!({
            "plan_id": plan.id,
            "tx_hash": tx_hash,
            "on_chain": state.stellar_submit.soroban_enabled(),
            "freeze_status": "FROZEN",
            "remaining_loaned": remaining_loaned,
        }),
    )
}

pub async fn recall_loans(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    Json(payload): Json<LoanLifecycleRequest>,
) -> Response {
    let plan = match load_plan(&state, plan_id).await {
        Ok(plan) => plan,
        Err(response) => return response,
    };
    if let Err(response) = require_triggered(&plan) {
        return response;
    }

    let mut tx_hash = None;
    let mut remaining_loaned = 0u64;
    let mut recalled_amount = 0u64;

    if state.stellar_submit.soroban_enabled() {
        let onchain_id = match require_onchain_id(&plan) {
            Ok(id) => id,
            Err(response) => return response,
        };

        let outstanding =
            match resolve_recall_amount(&state, onchain_id, payload.recall_amount).await {
                Ok(amount) => amount,
                Err(response) => return response,
            };

        if outstanding == 0 {
            remaining_loaned = 0;
        } else {
            match state
                .stellar_submit
                .recall_loan(onchain_id, outstanding)
                .await
            {
                Ok(outcome) => {
                    if let Err(response) = verify_loan_event(
                        &state,
                        &outcome,
                        onchain_id,
                        &RECALL_EVENT_TOPICS,
                        "LOAN/RECALL",
                    ) {
                        return response;
                    }
                    if let Some(contract) = state.stellar_submit.contract() {
                        if let Some(event) =
                            find_event(&outcome.events, contract, &RECALL_EVENT_TOPICS)
                        {
                            remaining_loaned =
                                event_u64_field(event, "remaining_loaned").unwrap_or(0);
                            recalled_amount =
                                event_u64_field(event, "recalled_amount").unwrap_or(outstanding);
                        }
                    }
                    tx_hash = Some(outcome.tx_hash);
                }
                Err(error) => return stellar_error_response(&error, "recall loans"),
            }
        }
    } else if let Some(amount) = payload.recall_amount {
        recalled_amount = amount;
    }

    let recall_progress = if remaining_loaned == 0 { 100 } else { 50 };

    if let Err(response) = upsert_lifecycle(
        &state,
        plan.id,
        None,
        Some(recall_progress),
        None,
        remaining_loaned,
        tx_hash.as_deref(),
    )
    .await
    {
        return response;
    }

    invalidate_cache(&state, &plan).await;
    emit_status(
        &state,
        PlanStatusEvent {
            event_type: "plan.loans_recalled".into(),
            plan_id: plan.id,
            status: if remaining_loaned == 0 {
                "RECALLED".into()
            } else {
                "PARTIAL".into()
            },
            message: "Loans recalled successfully".into(),
            tx_hash: tx_hash.clone(),
            freeze_status: None,
            recall_progress: Some(recall_progress),
            settlement_status: None,
            remaining_loaned: Some(remaining_loaned),
        },
        "plan.loans_recalled",
        &plan,
        tx_hash.as_deref(),
    )
    .await;

    ok_response(
        "Loans recalled successfully",
        serde_json::json!({
            "plan_id": plan.id,
            "tx_hash": tx_hash,
            "on_chain": state.stellar_submit.soroban_enabled(),
            "recalled_amount": recalled_amount,
            "recall_progress": recall_progress,
            "remaining_loaned": remaining_loaned,
        }),
    )
}

pub async fn liquidate_and_settle(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    Json(_payload): Json<LoanLifecycleRequest>,
) -> Response {
    let plan = match load_plan(&state, plan_id).await {
        Ok(plan) => plan,
        Err(response) => return response,
    };
    if let Err(response) = require_triggered(&plan) {
        return response;
    }

    let mut tx_hash = None;
    let mut remaining_loaned = 0u64;
    let mut settled_amount = 0u64;

    if state.stellar_submit.soroban_enabled() {
        let onchain_id = match require_onchain_id(&plan) {
            Ok(id) => id,
            Err(response) => return response,
        };

        let outstanding = match state.stellar_submit.outstanding_loaned(onchain_id).await {
            Ok(value) => value.unwrap_or(0),
            Err(error) => return stellar_error_response(&error, "read outstanding loans"),
        };

        if outstanding > 0 {
            match state.stellar_submit.liquidation_fallback(onchain_id).await {
                Ok(outcome) => {
                    if let Err(response) = verify_loan_event(
                        &state,
                        &outcome,
                        onchain_id,
                        &LIQUIDATE_EVENT_TOPICS,
                        "LOAN/LIQUIDAT",
                    ) {
                        return response;
                    }
                    if let Some(contract) = state.stellar_submit.contract() {
                        if let Some(event) =
                            find_event(&outcome.events, contract, &LIQUIDATE_EVENT_TOPICS)
                        {
                            settled_amount =
                                event_u64_field(event, "settled_amount").unwrap_or(outstanding);
                        }
                    }
                    tx_hash = Some(outcome.tx_hash);
                }
                Err(error) => return stellar_error_response(&error, "liquidate and settle"),
            }
        }
        remaining_loaned = 0;
    }

    if let Err(response) = upsert_lifecycle(
        &state,
        plan.id,
        None,
        Some(100),
        Some("SETTLED"),
        remaining_loaned,
        tx_hash.as_deref(),
    )
    .await
    {
        return response;
    }

    if let Err(e) =
        sqlx::query("UPDATE plans SET status = 'CLAIMABLE' WHERE id = $1 AND status = 'TRIGGERED'")
            .bind(plan.id)
            .execute(&state.db_pool)
            .await
    {
        error!(plan_id = %plan.id, error = %e, "Failed to mark plan claimable after settlement");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to update plan status: {e}") })),
        )
            .into_response();
    }

    invalidate_cache(&state, &plan).await;
    emit_status(
        &state,
        PlanStatusEvent {
            event_type: "plan.liquidated".into(),
            plan_id: plan.id,
            status: "SETTLED".into(),
            message: "Collateral liquidated and plan settled successfully".into(),
            tx_hash: tx_hash.clone(),
            freeze_status: None,
            recall_progress: Some(100),
            settlement_status: Some("SETTLED".into()),
            remaining_loaned: Some(remaining_loaned),
        },
        "plan.settled",
        &plan,
        tx_hash.as_deref(),
    )
    .await;

    ok_response(
        "Collateral liquidated and plan settled successfully",
        serde_json::json!({
            "plan_id": plan.id,
            "tx_hash": tx_hash,
            "on_chain": state.stellar_submit.soroban_enabled(),
            "settled_amount": settled_amount,
            "settlement_status": "SETTLED",
            "remaining_loaned": remaining_loaned,
        }),
    )
}

pub async fn get_trigger_info(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
) -> Response {
    match load_plan(&state, plan_id).await {
        Ok(_) => {}
        Err(response) => return response,
    }

    let row = match sqlx::query_as::<_, LoanLifecycleRow>(
        r#"
        SELECT plan_id, freeze_status, recall_progress, settlement_status,
               outstanding_loaned, last_tx_hash, updated_at
        FROM plan_loan_lifecycle
        WHERE plan_id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            error!(plan_id = %plan_id, error = %e, "Failed to load loan lifecycle");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Database query failed: {e}") })),
            )
                .into_response();
        }
    };

    let (freeze_status, recall_progress, settlement_status, outstanding, timestamp) = match row {
        Some(row) => (
            row.freeze_status,
            row.recall_progress,
            row.settlement_status,
            row.outstanding_loaned,
            Some(row.updated_at.to_rfc3339()),
        ),
        None => ("PENDING".to_string(), 0, "PENDING".to_string(), 0, None),
    };

    let loan_status = match freeze_status.as_str() {
        "FROZEN" if recall_progress >= 100 => "Recalled",
        "FROZEN" => "Frozen",
        _ => "Active",
    };

    let outstanding_loans = if outstanding > 0 {
        vec![OutstandingLoan {
            pool: "Soroban inheritance vault".into(),
            amount: outstanding.to_string(),
            status: loan_status.into(),
        }]
    } else {
        Vec::new()
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "data": {
                "timestamp": timestamp,
                "freeze_status": freeze_status,
                "recall_progress": recall_progress,
                "settlement_status": settlement_status,
                "outstanding_loans": outstanding_loans,
            }
        })),
    )
        .into_response()
}

async fn load_plan(state: &AppState, plan_id: Uuid) -> Result<PlanRef, Response> {
    match sqlx::query_as::<_, PlanRef>(
        r#"
        SELECT id, owner_address, status, is_active, onchain_plan_id
        FROM plans
        WHERE id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(row)) => Ok(row),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Plan not found" })),
        )
            .into_response()),
        Err(e) => {
            error!(plan_id = %plan_id, error = %e, "Failed to fetch plan for loan lifecycle");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Database query failed: {e}") })),
            )
                .into_response())
        }
    }
}

fn require_triggered(plan: &PlanRef) -> Result<(), Response> {
    if !plan.is_active {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Plan is no longer active" })),
        )
            .into_response());
    }
    if !ALLOWED_STATUSES.contains(&plan.status.as_str()) {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": format!(
                    "Plan must be in the triggered state. Current status: {}",
                    plan.status
                )
            })),
        )
            .into_response());
    }
    Ok(())
}

fn require_onchain_id(plan: &PlanRef) -> Result<u64, Response> {
    match plan.onchain_plan_id {
        Some(id) if id >= 0 => Ok(id as u64),
        _ => Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Plan has no on-chain identifier; it cannot invoke the lending vault"
            })),
        )
            .into_response()),
    }
}

async fn resolve_recall_amount(
    state: &AppState,
    onchain_id: u64,
    requested: Option<u64>,
) -> Result<u64, Response> {
    if let Some(amount) = requested {
        if amount == 0 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "recall_amount must be greater than zero" })),
            )
                .into_response());
        }
        return Ok(amount);
    }

    match state.stellar_submit.outstanding_loaned(onchain_id).await {
        Ok(Some(amount)) => Ok(amount),
        Ok(None) => Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Inheritance has not been triggered on-chain"
            })),
        )
            .into_response()),
        Err(error) => Err(stellar_error_response(&error, "read outstanding loans")),
    }
}

async fn outstanding_after(state: &AppState, onchain_id: u64) -> Option<u64> {
    match state.stellar_submit.outstanding_loaned(onchain_id).await {
        Ok(value) => value,
        Err(error) => {
            warn!(error = %error, "Failed to read outstanding loans after freeze");
            None
        }
    }
}

fn verify_loan_event(
    state: &AppState,
    outcome: &InvocationOutcome,
    expected_plan_id: u64,
    topics: &[&str],
    label: &str,
) -> Result<(), Response> {
    let Some(contract) = state.stellar_submit.contract() else {
        return Ok(());
    };
    let Some(event) = find_event(&outcome.events, contract, topics) else {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!(
                    "transaction {} succeeded but emitted no {label} event",
                    outcome.tx_hash
                )
            })),
        )
            .into_response());
    };
    match event_u64_field(event, "plan_id") {
        Some(plan_id) if plan_id == expected_plan_id => Ok(()),
        Some(plan_id) => Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!(
                    "transaction {} targeted plan {plan_id}, expected {expected_plan_id}",
                    outcome.tx_hash
                )
            })),
        )
            .into_response()),
        None => Ok(()),
    }
}

async fn upsert_lifecycle(
    state: &AppState,
    plan_id: Uuid,
    freeze_status: Option<&str>,
    recall_progress: Option<i32>,
    settlement_status: Option<&str>,
    outstanding_loaned: u64,
    tx_hash: Option<&str>,
) -> Result<(), Response> {
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO plan_loan_lifecycle (
            plan_id, freeze_status, recall_progress, settlement_status,
            outstanding_loaned, last_tx_hash, updated_at
        )
        VALUES (
            $1,
            COALESCE($2, 'PENDING'),
            COALESCE($3, 0),
            COALESCE($4, 'PENDING'),
            $5, $6, NOW()
        )
        ON CONFLICT (plan_id) DO UPDATE SET
            freeze_status = COALESCE($2, plan_loan_lifecycle.freeze_status),
            recall_progress = COALESCE($3, plan_loan_lifecycle.recall_progress),
            settlement_status = COALESCE($4, plan_loan_lifecycle.settlement_status),
            outstanding_loaned = $5,
            last_tx_hash = COALESCE($6, plan_loan_lifecycle.last_tx_hash),
            updated_at = NOW()
        "#,
    )
    .bind(plan_id)
    .bind(freeze_status)
    .bind(recall_progress)
    .bind(settlement_status)
    .bind(outstanding_loaned as i64)
    .bind(tx_hash)
    .execute(&state.db_pool)
    .await
    {
        error!(plan_id = %plan_id, error = %e, "Failed to persist loan lifecycle");
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to persist loan status: {e}") })),
        )
            .into_response());
    }
    Ok(())
}

async fn invalidate_cache(state: &AppState, plan: &PlanRef) {
    let addresses: Vec<String> =
        sqlx::query_scalar("SELECT wallet_address FROM beneficiaries WHERE plan_id = $1")
            .bind(plan.id)
            .fetch_all(&state.db_pool)
            .await
            .unwrap_or_default();
    if let Err(err) = state
        .plan_cache
        .invalidate_queries(&plan.owner_address, &addresses)
        .await
    {
        warn!(
            plan_id = %plan.id,
            error = %err,
            "Failed to invalidate plan cache after loan lifecycle update"
        );
    }
}

async fn emit_status(
    state: &AppState,
    event: PlanStatusEvent,
    webhook_type: &str,
    plan: &PlanRef,
    tx_hash: Option<&str>,
) {
    if let Err(e) = state.status_tx.send(event.clone()) {
        tracing::debug!("No WebSocket subscribers for plan status: {e}");
    }

    let payload = serde_json::json!({
        "plan_id": plan.id,
        "owner_address": plan.owner_address,
        "onchain_plan_id": plan.onchain_plan_id,
        "status": event.status,
        "tx_hash": tx_hash,
        "remaining_loaned": event.remaining_loaned,
    });
    if let Err(e) =
        crate::WebhookDispatcherService::enqueue_event(&state.db_pool, webhook_type, &payload).await
    {
        warn!("Failed to enqueue webhook for {webhook_type}: {e:?}");
    }
}

fn stellar_error_response(error: &StellarSubmitError, action: &str) -> Response {
    let (status, message) = match error {
        StellarSubmitError::NotConfigured => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Soroban contract invocation is not configured".to_string(),
        ),
        StellarSubmitError::Simulation(detail) => {
            let conflict = detail.to_lowercase();
            if conflict.contains("nottriggered")
                || conflict.contains("not triggered")
                || conflict.contains("#43")
            {
                (
                    StatusCode::CONFLICT,
                    format!("Inheritance has not been triggered: {detail}"),
                )
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    format!("On-chain {action} simulation failed: {detail}"),
                )
            }
        }
        StellarSubmitError::Network(_)
        | StellarSubmitError::Rpc(_)
        | StellarSubmitError::Timeout { .. }
        | StellarSubmitError::TransactionFailed { .. }
        | StellarSubmitError::Rejected(_) => (
            StatusCode::BAD_GATEWAY,
            format!("On-chain {action} failed: {error}"),
        ),
        StellarSubmitError::Config(_) | StellarSubmitError::Xdr(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("On-chain {action} failed: {error}"),
        ),
    };
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}

fn ok_response(message: &str, data: serde_json::Value) -> Response {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "message": message,
            "data": data,
        })),
    )
        .into_response()
}

// ─── Automated Loan Liquidation Watchdog Task (#1119) ─────────

pub const DEFAULT_LIQUIDATION_LTV_THRESHOLD_BPS: u32 = 8000; // 80% LTV threshold
pub const DEFAULT_LIQUIDATION_WATCHDOG_INTERVAL_SECS: u64 = 60;
pub const LOAN_WATCHDOG_LOCK_KEY: i64 = 822;

/// Configuration for the Automated Loan Liquidation Watchdog.
#[derive(Debug, Clone, Copy)]
pub struct LoanLiquidationWatchdogConfig {
    pub interval: std::time::Duration,
    pub liquidation_ltv_threshold_bps: u32,
    pub batch_size: i64,
}

impl LoanLiquidationWatchdogConfig {
    pub fn from_env() -> Self {
        let interval_secs = std::env::var("LOAN_LIQUIDATION_WATCHDOG_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DEFAULT_LIQUIDATION_WATCHDOG_INTERVAL_SECS);
        let threshold_bps = std::env::var("LOAN_LIQUIDATION_LTV_THRESHOLD_BPS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(DEFAULT_LIQUIDATION_LTV_THRESHOLD_BPS);
        Self {
            interval: std::time::Duration::from_secs(interval_secs.max(1)),
            liquidation_ltv_threshold_bps: threshold_bps,
            batch_size: 100,
        }
    }
}

/// Price Oracle trait for fetching market prices of collateral assets.
pub trait CollateralPriceOracle: Send + Sync {
    /// Returns the price ratio of collateral / debt asset.
    /// E.g. 1.0 means 1 unit of collateral = 1 unit of debt currency.
    fn get_collateral_price(&self, collateral_asset: &str) -> f64;
}

#[derive(Debug, Clone, Default)]
pub struct EnvPriceOracle;

impl CollateralPriceOracle for EnvPriceOracle {
    fn get_collateral_price(&self, _collateral_asset: &str) -> f64 {
        std::env::var("COLLATERAL_MARKET_PRICE")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0)
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct UndercollateralizedLoanCandidate {
    pub plan_id: Uuid,
    pub owner_address: String,
    pub onchain_plan_id: Option<i64>,
    pub outstanding_loaned: i64,
    pub plan_amount: rust_decimal::Decimal,
}

/// Computes the Loan-to-Value (LTV) ratio in basis points.
/// LTV = (debt / (collateral_amount * price)) * 10000
pub fn calculate_ltv_bps(debt: u64, collateral_amount: f64, collateral_price: f64) -> u32 {
    let collateral_value = collateral_amount * collateral_price;
    if collateral_value <= 0.0 {
        return 100_000; // 1000% LTV (insolvent)
    }
    let ltv = (debt as f64 / collateral_value) * 10_000.0;
    ltv.min(100_000.0) as u32
}

pub struct LoanLiquidationWatchdogService;

impl LoanLiquidationWatchdogService {
    /// Executes one sweep over active loans, evaluating current market LTV.
    /// Triggers on-chain liquidation whenever LTV > liquidation threshold (default 80%).
    pub async fn sweep_candidates(
        state: &AppState,
        oracle: &dyn CollateralPriceOracle,
        config: &LoanLiquidationWatchdogConfig,
    ) -> Result<usize, String> {
        let candidates = match sqlx::query_as::<_, UndercollateralizedLoanCandidate>(
            r#"
            SELECT l.plan_id, p.owner_address, p.onchain_plan_id,
                   l.outstanding_loaned, p.amount as plan_amount
            FROM plan_loan_lifecycle l
            JOIN plans p ON p.id = l.plan_id
            WHERE l.settlement_status != 'SETTLED'
              AND l.outstanding_loaned > 0
              AND p.is_active = true
            LIMIT $1
            "#,
        )
        .bind(config.batch_size)
        .fetch_all(&state.db_pool)
        .await
        {
            Ok(c) => c,
            Err(e) => return Err(format!("Failed to query loan candidates: {e}")),
        };

        let mut liquidated_count = 0;

        for candidate in candidates {
            let collateral_price = oracle.get_collateral_price("COLLATERAL");
            let collateral_amount =
                rust_decimal::prelude::ToPrimitive::to_f64(&candidate.plan_amount).unwrap_or(0.0);
            let ltv_bps = calculate_ltv_bps(
                candidate.outstanding_loaned as u64,
                collateral_amount,
                collateral_price,
            );

            if ltv_bps > config.liquidation_ltv_threshold_bps {
                tracing::warn!(
                    plan_id = %candidate.plan_id,
                    ltv_bps = ltv_bps,
                    threshold_bps = config.liquidation_ltv_threshold_bps,
                    "Loan LTV exceeded liquidation threshold (> 80%). Triggering automated liquidation."
                );

                let mut tx_hash = None;
                if state.stellar_submit.soroban_enabled() {
                    if let Some(onchain_id) = candidate.onchain_plan_id {
                        if onchain_id >= 0 {
                            match state
                                .stellar_submit
                                .liquidate_loan(
                                    onchain_id as u64,
                                    candidate.outstanding_loaned as u64,
                                )
                                .await
                            {
                                Ok(outcome) => {
                                    tx_hash = Some(outcome.tx_hash);
                                }
                                Err(err) => {
                                    tracing::error!(
                                        plan_id = %candidate.plan_id,
                                        error = %err,
                                        "Automated liquidation on-chain submission failed"
                                    );
                                    continue;
                                }
                            }
                        }
                    }
                }

                // Update lifecycle to SETTLED
                let _ = upsert_lifecycle(
                    state,
                    candidate.plan_id,
                    Some("LIQUIDATED"),
                    Some(100),
                    Some("SETTLED"),
                    0,
                    tx_hash.as_deref(),
                )
                .await;

                let _ = sqlx::query("UPDATE plans SET status = 'CLAIMABLE' WHERE id = $1")
                    .bind(candidate.plan_id)
                    .execute(&state.db_pool)
                    .await;

                let plan_ref = PlanRef {
                    id: candidate.plan_id,
                    owner_address: candidate.owner_address.clone(),
                    status: "CLAIMABLE".into(),
                    is_active: true,
                    onchain_plan_id: candidate.onchain_plan_id,
                };

                invalidate_cache(state, &plan_ref).await;
                emit_status(
                    state,
                    PlanStatusEvent {
                        event_type: "plan.liquidated".into(),
                        plan_id: candidate.plan_id,
                        status: "SETTLED".into(),
                        message: "Automated liquidation triggered by loan watchdog (LTV > 80%)"
                            .into(),
                        tx_hash: tx_hash.clone(),
                        freeze_status: Some("LIQUIDATED".into()),
                        recall_progress: Some(100),
                        settlement_status: Some("SETTLED".into()),
                        remaining_loaned: Some(0),
                    },
                    "plan.liquidated",
                    &plan_ref,
                    tx_hash.as_deref(),
                )
                .await;

                liquidated_count += 1;
            }
        }

        Ok(liquidated_count)
    }

    /// Spawns the background watchdog worker.
    pub fn spawn(
        state: Arc<AppState>,
        config: LoanLiquidationWatchdogConfig,
        mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(config.interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let oracle = EnvPriceOracle;

            tracing::info!(
                interval = ?config.interval,
                threshold_bps = config.liquidation_ltv_threshold_bps,
                "Automated loan liquidation watchdog service started"
            );

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let mut tx = match state.db_pool.begin().await {
                            Ok(tx) => tx,
                            Err(e) => {
                                tracing::warn!("Failed to start transaction for loan watchdog lock: {e}");
                                continue;
                            }
                        };

                        let lock_result = sqlx::query_scalar::<_, bool>(
                            "SELECT pg_try_advisory_xact_lock($1)",
                        )
                        .bind(LOAN_WATCHDOG_LOCK_KEY)
                        .fetch_one(&mut *tx)
                        .await;

                        match lock_result {
                            Ok(true) => {
                                match Self::sweep_candidates(&state, &oracle, &config).await {
                                    Ok(count) => {
                                        if count > 0 {
                                            tracing::info!("Loan watchdog liquidated {count} underwater loans");
                                        }
                                    }
                                    Err(err) => {
                                        tracing::error!("Loan watchdog sweep failed: {err}");
                                    }
                                }
                                let _ = tx.commit().await;
                            }
                            Ok(false) => {
                                tracing::debug!("Another worker holds loan watchdog advisory lock; skipping tick");
                                let _ = tx.rollback().await;
                            }
                            Err(e) => {
                                tracing::warn!("Failed to acquire advisory lock for loan watchdog: {e}");
                                let _ = tx.rollback().await;
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        tracing::info!("Loan liquidation watchdog service received shutdown signal");
                        break;
                    }
                }
            }
        })
    }
}

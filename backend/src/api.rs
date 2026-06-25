use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::stellar_anchor::{AnchorPayout, AnchorRegistry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanBeneficiary {
    pub address: String,
    pub name: String,
    pub allocation_bps: u32,
    pub fiat_anchor_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub owner: String,
    pub token: String,
    pub amount: f64,
    pub beneficiaries: Vec<PlanBeneficiary>,
    pub last_ping: i64,
    pub grace_period: u64,
    pub earn_yield: bool,
    pub yield_rate_bps: u32,
    pub is_active: bool,
}

pub struct AppState {
    pub anchor: Arc<AnchorRegistry>,
    pub db_pool: sqlx::PgPool,
}

#[derive(Deserialize)]
pub struct PlanQuery {
    pub owner: Option<String>,
}

#[derive(Deserialize)]
pub struct PingRequest {
    pub owner: String,
}

#[derive(Deserialize)]
pub struct PayoutRequest {
    pub owner: String,
}

#[derive(Deserialize)]
pub struct AnchorQuery {
    pub beneficiary_address: Option<String>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/plans", post(create_plan).get(get_plans))
        .route("/api/plans/ping", post(ping_plan))
        .route("/api/plans/payout", post(trigger_payout))
        .route("/api/anchor/payout-status", get(get_anchor_payouts))
        .layer(cors)
        .with_state(state)
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PlanRow {
    pub id: uuid::Uuid,
    pub owner_address: String,
    pub token_address: String,
    pub amount: rust_decimal::Decimal,
    pub grace_period: i64,
    pub grace_period_seconds: i64,
    pub earn_yield: bool,
    pub last_ping: i64,
    pub is_active: bool,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct BeneficiaryRow {
    pub id: uuid::Uuid,
    pub plan_id: uuid::Uuid,
    pub wallet_address: String,
    pub allocation_bps: i32,
    pub fiat_anchor_info: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanResponse {
    pub id: uuid::Uuid,
    pub owner_address: String,
    pub token_address: String,
    pub amount: rust_decimal::Decimal,
    pub grace_period: i64,
    pub grace_period_seconds: i64,
    pub earn_yield: bool,
    pub last_ping: i64,
    pub is_active: bool,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub beneficiaries: Vec<BeneficiaryResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BeneficiaryResponse {
    pub id: uuid::Uuid,
    pub plan_id: uuid::Uuid,
    pub wallet_address: String,
    pub allocation_bps: i32,
    pub fiat_anchor_info: String,
}

// Handler: Create Plan
// Contributors: Implement saving plan to database, set default fields, and run in a transaction
async fn create_plan(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Plan>,
) -> impl IntoResponse {
    // 1. Validation
    if payload.owner.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Owner address cannot be empty" })),
        )
            .into_response();
    }
    if payload.token.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Token address cannot be empty" })),
        )
            .into_response();
    }
    if payload.amount < 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Amount must be non-negative" })),
        )
            .into_response();
    }
    if payload.grace_period == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Grace period must be greater than zero" })),
        )
            .into_response();
    }
    if payload.beneficiaries.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Plan must have at least one beneficiary" })),
        )
            .into_response();
    }
    let mut total_bps = 0;
    for b in &payload.beneficiaries {
        if b.address.trim().is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Beneficiary address cannot be empty" })),
            )
                .into_response();
        }
        if b.allocation_bps > 10000 {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Beneficiary allocation_bps cannot exceed 10000" })),
            ).into_response();
        }
        total_bps += b.allocation_bps;
    }
    if total_bps != 10000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Total allocation_bps must be exactly 10000 (100%), got {}", total_bps)
            })),
        ).into_response();
    }

    // Convert amount to rust_decimal::Decimal
    let amount_dec = match rust_decimal::Decimal::from_f64_retain(payload.amount) {
        Some(d) => d.normalize(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid amount representation" })),
            )
                .into_response()
        }
    };

    // 2. Transaction Execution
    let mut tx = match state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to begin database transaction: {}", e) })),
        ).into_response(),
    };

    let plan_row = match sqlx::query_as::<_, PlanRow>(
        r#"
        INSERT INTO plans (
            owner_address,
            token_address,
            amount,
            grace_period,
            grace_period_seconds,
            earn_yield,
            last_ping,
            is_active,
            status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, owner_address, token_address, amount, grace_period, grace_period_seconds, earn_yield, last_ping, is_active, status, created_at
        "#
    )
    .bind(&payload.owner)
    .bind(&payload.token)
    .bind(amount_dec)
    .bind(payload.grace_period as i64)
    .bind(payload.grace_period as i64)
    .bind(payload.earn_yield)
    .bind(payload.last_ping)
    .bind(payload.is_active)
    .bind("ACTIVE")
    .fetch_one(&mut *tx)
    .await {
        Ok(row) => row,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to save plan: {}", e) })),
            ).into_response();
        }
    };

    let mut inserted_beneficiaries = Vec::new();
    for b in &payload.beneficiaries {
        let beneficiary_row = match sqlx::query_as::<_, BeneficiaryRow>(
            r#"
            INSERT INTO beneficiaries (
                plan_id,
                wallet_address,
                allocation_bps,
                fiat_anchor_info
            ) VALUES ($1, $2, $3, $4)
            RETURNING id, plan_id, wallet_address, allocation_bps, fiat_anchor_info
            "#,
        )
        .bind(plan_row.id)
        .bind(&b.address)
        .bind(b.allocation_bps as i32)
        .bind(&b.fiat_anchor_info)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(row) => row,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to save beneficiary: {}", e) })),
                ).into_response();
            }
        };

        inserted_beneficiaries.push(BeneficiaryResponse {
            id: beneficiary_row.id,
            plan_id: beneficiary_row.plan_id,
            wallet_address: beneficiary_row.wallet_address,
            allocation_bps: beneficiary_row.allocation_bps,
            fiat_anchor_info: beneficiary_row.fiat_anchor_info,
        });
    }

    if let Err(e) = tx.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to commit database transaction: {}", e) })),
        ).into_response();
    }

    let response = PlanResponse {
        id: plan_row.id,
        owner_address: plan_row.owner_address,
        token_address: plan_row.token_address,
        amount: plan_row.amount,
        grace_period: plan_row.grace_period,
        grace_period_seconds: plan_row.grace_period_seconds,
        earn_yield: plan_row.earn_yield,
        last_ping: plan_row.last_ping,
        is_active: plan_row.is_active,
        status: plan_row.status,
        created_at: plan_row.created_at,
        beneficiaries: inserted_beneficiaries,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

// Handler: Get Plans
// Contributors: Implement plan retrieval, filtering by owner, and apply on-the-fly yield accumulation
async fn get_plans(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<PlanQuery>,
) -> impl IntoResponse {
    let empty_list: Vec<Plan> = Vec::new();
    (StatusCode::OK, Json(empty_list))
}

// Handler: Ping Plan
// Contributors: Implement resetting last_ping timestamp and calculating accrued yield up to the ping time
async fn ping_plan(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<PingRequest>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Ping logic not implemented")
}

// Handler: Trigger Payout
// Contributors: Implement calculating final payout with yield, parsing fiat payout details,
// submitting fiat payouts to AnchorRegistry, and marking the plan inactive
async fn trigger_payout(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<PayoutRequest>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Payout trigger logic not implemented",
    )
}

// Handler: Get Anchor Payouts
// Contributors: List payouts from AnchorRegistry
async fn get_anchor_payouts(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<AnchorQuery>,
) -> impl IntoResponse {
    let empty_list: Vec<AnchorPayout> = Vec::new();
    (StatusCode::OK, Json(empty_list))
}

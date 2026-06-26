use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::error;
use uuid::Uuid;

use crate::kyc_webhook::kyc_webhook_handler;
use crate::stellar_anchor::AnchorRegistry;
use crate::ws::{ws_handler, KycUpdateEvent};

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
    pub kyc_tx: tokio::sync::broadcast::Sender<KycUpdateEvent>,
    pub kyc_webhook_secret: Option<String>,
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
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PayoutRow {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub beneficiary_address: String,
    pub amount: String,
    pub payout_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct PayoutStatusResponse {
    pub data: Vec<PayoutRow>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
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
        .route("/api/kyc/webhook", post(kyc_webhook_handler))
        .route("/ws/kyc", get(ws_handler))
        .layer(cors)
        .with_state(state)
}

// Handler: Create Plan
// Contributors: Implement saving plan to in-memory state, set default fields
async fn create_plan(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<Plan>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(payload))
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
// Queries the payouts table filtered by beneficiary_address with pagination.
async fn get_anchor_payouts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnchorQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;
    let address = query.beneficiary_address.as_deref();

    let total: i64 = match sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM payouts WHERE ($1::text IS NULL OR beneficiary_address = $1)"#,
    )
    .bind(address)
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(count) => count,
        Err(e) => {
            error!(error = %e, "Failed to count payouts");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Database query failed".to_string(),
                }),
            )
                .into_response();
        }
    };

    let rows: Vec<PayoutRow> = match sqlx::query_as::<_, PayoutRow>(
        r#"
        SELECT
            id,
            plan_id,
            beneficiary_address,
            amount::text      AS amount,
            payout_type::text AS payout_type,
            status::text      AS status,
            created_at
        FROM payouts
        WHERE ($1::text IS NULL OR beneficiary_address = $1)
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(address)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!(error = %e, "Failed to query payouts");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Database query failed".to_string(),
                }),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(PayoutStatusResponse {
            data: rows,
            page,
            page_size,
            total,
        }),
    )
        .into_response()
}

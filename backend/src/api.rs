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

use crate::kyc_webhook::kyc_webhook_handler;
use crate::stellar_anchor::{AnchorPayout, AnchorRegistry};
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
// Contributors: List payouts from AnchorRegistry
async fn get_anchor_payouts(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<AnchorQuery>,
) -> impl IntoResponse {
    let empty_list: Vec<AnchorPayout> = Vec::new();
    (StatusCode::OK, Json(empty_list))
}

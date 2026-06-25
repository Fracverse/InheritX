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
use axum::middleware;

use crate::stellar_anchor::{AnchorPayout, AnchorRegistry};
use crate::admin_auth::admin_auth_middleware;

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

#[derive(Debug, Serialize)]
pub struct AdminMetrics {
    pub total_value_locked: i64,
    pub active_users: i64,
    pub active_plans: i64,
    pub accumulated_yield_paid: i64,
    pub generated_at: String,
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
        .route("/api/admin/metrics", get(get_admin_metrics))
        .route_layer(middleware::from_fn(admin_auth_middleware))
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

// Handler: Get Admin Metrics
// Aggregates platform statistics including TVL, active users, active plans, and accumulated yield
async fn get_admin_metrics(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Execute all queries in parallel for better performance
    let (tvl_result, active_users_result, active_plans_result, yield_result) = tokio::try_join!(
        // TVL: Sum of all active plan amounts
        sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(SUM(amount), 0) FROM plans WHERE is_active = TRUE"
        )
        .fetch_one(&state.db_pool),
        
        // Active users: Count of unique owners with active plans
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT owner_address) FROM plans WHERE is_active = TRUE"
        )
        .fetch_one(&state.db_pool),
        
        // Active plans: Count of active plans
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM plans WHERE is_active = TRUE"
        )
        .fetch_one(&state.db_pool),
        
        // Accumulated yield: Sum of completed payouts
        sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(SUM(amount), 0) FROM payouts WHERE status = 'completed'"
        )
        .fetch_one(&state.db_pool)
    );

    match (tvl_result, active_users_result, active_plans_result, yield_result) {
        (Ok(tvl), Ok(active_users), Ok(active_plans), Ok(yield)) => {
            let metrics = AdminMetrics {
                total_value_locked: tvl,
                active_users,
                active_plans,
                accumulated_yield_paid: yield,
                generated_at: chrono::Utc::now().to_rfc3339(),
            };
            (StatusCode::OK, Json(metrics))
        }
        Err(e) => {
            tracing::error!("Failed to fetch admin metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch metrics")
        }
    }
}

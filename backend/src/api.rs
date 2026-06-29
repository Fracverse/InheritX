use axum::{
    extract::{Query, State},
    http::StatusCode,
    http::{header::HeaderName, HeaderValue},
    middleware::from_fn,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::error;
use uuid::Uuid;

use crate::auth::signature_auth_middleware;
use crate::cache::PlanCache;
use crate::kyc_webhook::kyc_webhook_handler;
use crate::stellar_anchor::AnchorRegistry;
use crate::ws::{ws_handler, KycUpdateEvent};
use crate::yield_calculator;

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
    pub apy_config: yield_calculator::ApyConfig,
    pub plan_cache: PlanCache,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlanQuery {
    pub owner: Option<String>,
    pub beneficiary: Option<String>,
}

#[derive(Deserialize)]
pub struct PingRequest {
    pub owner: String,
    pub signature: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct PingResponse {
    pub owner: String,
    pub status: String,
    pub virtual_balance: rust_decimal::Decimal,
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

    // User routes requiring signature verification
    let user_routes = Router::new()
        .route("/api/plans", post(create_plan))
        .route("/api/plans/ping", post(ping_plan))
        .route("/api/plans/payout", post(trigger_payout))
        .route_layer(from_fn(signature_auth_middleware));

    // Public or admin routes
    let public_routes = Router::new()
        .route("/api/plans", get(get_plans))
        .route("/api/anchor/payout-status", get(get_anchor_payouts))
        .route("/api/kyc/webhook", post(kyc_webhook_handler))
        .route("/api/kyc/status", get(get_kyc_status))
        .route("/api/kyc/submit", post(submit_kyc))
        .route("/api/kyc/upload", post(upload_kyc_document))
        .route("/api/kyc/required", get(is_kyc_required))
        .route("/api/kyc/requirements", get(get_kyc_requirements))
        .route("/ws/kyc", get(ws_handler));

    Router::new()
        .merge(user_routes)
        .merge(public_routes)
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
    pub yield_rate_bps: i32,
    pub accrued_yield: rust_decimal::Decimal,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub yield_rate_bps: i32,
    pub accrued_yield: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub beneficiaries: Vec<BeneficiaryResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeneficiaryResponse {
    pub id: uuid::Uuid,
    pub plan_id: uuid::Uuid,
    pub wallet_address: String,
    pub allocation_bps: i32,
    pub fiat_anchor_info: String,
}

/// Compute the accrued yield for a plan based on elapsed time since last_ping.
fn compute_accrued_yield(amount: &Decimal, yield_rate_bps: i32, last_ping: i64) -> f64 {
    if yield_rate_bps == 0 || last_ping == 0 {
        return 0.0;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let elapsed_secs = (now - last_ping).max(0) as u64;
    let amount_f64 = amount.to_string().parse::<f64>().unwrap_or(0.0);

    yield_calculator::calculate_yield(amount_f64, yield_rate_bps as u32, elapsed_secs)
}

fn compute_projected_accrued_yield(row: &PlanRow) -> f64 {
    let persisted = row.accrued_yield.to_string().parse::<f64>().unwrap_or(0.0);

    if !row.earn_yield {
        return persisted;
    }

    persisted + compute_accrued_yield(&row.amount, row.yield_rate_bps, row.last_ping)
}

/// Load beneficiaries for a given plan.
async fn load_beneficiaries(
    pool: &sqlx::PgPool,
    plan_id: uuid::Uuid,
) -> Result<Vec<BeneficiaryResponse>, sqlx::Error> {
    let rows = sqlx::query_as::<_, BeneficiaryRow>(
        r#"
        SELECT id, plan_id, wallet_address, allocation_bps, fiat_anchor_info
        FROM beneficiaries
        WHERE plan_id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| BeneficiaryResponse {
            id: r.id,
            plan_id: r.plan_id,
            wallet_address: r.wallet_address,
            allocation_bps: r.allocation_bps,
            fiat_anchor_info: r.fiat_anchor_info,
        })
        .collect())
}

// Helper: convert PlanRow + beneficiaries into PlanResponse with yield
fn plan_row_to_response(row: PlanRow, beneficiaries: Vec<BeneficiaryResponse>) -> PlanResponse {
    let accrued_yield = compute_projected_accrued_yield(&row);

    PlanResponse {
        id: row.id,
        owner_address: row.owner_address,
        token_address: row.token_address,
        amount: row.amount,
        grace_period: row.grace_period,
        grace_period_seconds: row.grace_period_seconds,
        earn_yield: row.earn_yield,
        last_ping: row.last_ping,
        is_active: row.is_active,
        status: row.status,
        yield_rate_bps: row.yield_rate_bps,
        accrued_yield,
        created_at: row.created_at,
        beneficiaries,
    }
}

async fn load_beneficiary_addresses(
    pool: &sqlx::PgPool,
    plan_id: uuid::Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT wallet_address
        FROM beneficiaries
        WHERE plan_id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_all(pool)
    .await
}

#[derive(Clone, Copy)]
enum PlanCacheStatus {
    Hit,
    Miss,
    Bypass,
    ErrorFallback,
}

impl PlanCacheStatus {
    fn as_header_value(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Bypass => "bypass",
            Self::ErrorFallback => "error-fallback",
        }
    }
}

fn add_plan_cache_headers(
    response: &mut axum::response::Response,
    cache_status: PlanCacheStatus,
    cache_lookup_ms: u128,
    db_query_ms: u128,
    total_ms: u128,
) {
    response.headers_mut().insert(
        HeaderName::from_static("x-plan-cache-status"),
        HeaderValue::from_static(cache_status.as_header_value()),
    );

    for (name, value) in [
        ("x-plan-cache-lookup-ms", cache_lookup_ms),
        ("x-plan-db-query-ms", db_query_ms),
        ("x-plan-total-latency-ms", total_ms),
    ] {
        if let Ok(value) = HeaderValue::from_str(&value.to_string()) {
            response
                .headers_mut()
                .insert(HeaderName::from_static(name), value);
        }
    }
}

async fn invalidate_plan_cache(
    cache: &PlanCache,
    owner_address: &str,
    beneficiary_addresses: &[String],
) {
    if let Err(err) = cache
        .invalidate_queries(owner_address, beneficiary_addresses)
        .await
    {
        error!(
            owner_address = %owner_address,
            error = %err,
            "Failed to invalidate plan cache"
        );
    }
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
            yield_rate_bps,
            accrued_yield,
            last_ping,
            is_active,
            status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, owner_address, token_address, amount, grace_period, grace_period_seconds, earn_yield, last_ping, is_active, status, yield_rate_bps, accrued_yield, created_at
        "#
    )
    .bind(&payload.owner)
    .bind(&payload.token)
    .bind(amount_dec)
    .bind(payload.grace_period as i64)
    .bind(payload.grace_period as i64)
    .bind(payload.earn_yield)
    .bind(payload.yield_rate_bps as i32)
    .bind(rust_decimal::Decimal::ZERO)
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

    let beneficiary_addresses: Vec<String> = inserted_beneficiaries
        .iter()
        .map(|beneficiary| beneficiary.wallet_address.clone())
        .collect();
    invalidate_plan_cache(&state.plan_cache, &payload.owner, &beneficiary_addresses).await;

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
        yield_rate_bps: plan_row.yield_rate_bps,
        accrued_yield: 0.0, // No yield accrued at creation
        created_at: plan_row.created_at,
        beneficiaries: inserted_beneficiaries,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

// Handler: Get Plans
// Contributors: Implement plan retrieval, filtering by owner, and apply on-the-fly yield accumulation
async fn get_plans(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PlanQuery>,
) -> impl IntoResponse {
    let total_started = std::time::Instant::now();
    let cache_lookup_started = std::time::Instant::now();
    let mut cache_status = if state.plan_cache.is_enabled() {
        PlanCacheStatus::Miss
    } else {
        PlanCacheStatus::Bypass
    };

    if state.plan_cache.is_enabled() {
        match state.plan_cache.get_plans(&query).await {
            Ok(Some(plans)) => {
                let mut response = (StatusCode::OK, Json(plans)).into_response();
                add_plan_cache_headers(
                    &mut response,
                    PlanCacheStatus::Hit,
                    cache_lookup_started.elapsed().as_millis(),
                    0,
                    total_started.elapsed().as_millis(),
                );
                return response;
            }
            Ok(None) => {}
            Err(err) => {
                error!(error = %err, "Plan cache lookup failed, falling back to PostgreSQL");
                cache_status = PlanCacheStatus::ErrorFallback;
            }
        }
    }

    let cache_lookup_ms = cache_lookup_started.elapsed().as_millis();
    let db_query_started = std::time::Instant::now();

    // Build the query dynamically based on filters
    let rows: Vec<PlanRow> = match (&query.owner, &query.beneficiary) {
        (Some(owner), None) => {
            // Filter by owner only
            match sqlx::query_as::<_, PlanRow>(
                r#"
                SELECT id, owner_address, token_address, amount, grace_period,
                       grace_period_seconds, earn_yield, last_ping, is_active,
                       status, yield_rate_bps, accrued_yield, created_at
                FROM plans
                WHERE owner_address = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(owner)
            .fetch_all(&state.db_pool)
            .await
            {
                Ok(rows) => rows,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            serde_json::json!({ "error": format!("Database query failed: {}", e) }),
                        ),
                    )
                        .into_response();
                }
            }
        }
        (None, Some(beneficiary)) => {
            // Filter by beneficiary: plans where beneficiary address is listed
            match sqlx::query_as::<_, PlanRow>(
                r#"
                SELECT DISTINCT p.id, p.owner_address, p.token_address, p.amount,
                       p.grace_period, p.grace_period_seconds, p.earn_yield,
                       p.last_ping, p.is_active, p.status, p.yield_rate_bps, p.accrued_yield, p.created_at
                FROM plans p
                INNER JOIN beneficiaries b ON b.plan_id = p.id
                WHERE b.wallet_address = $1
                ORDER BY p.created_at DESC
                "#,
            )
            .bind(beneficiary)
            .fetch_all(&state.db_pool)
            .await
            {
                Ok(rows) => rows,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            serde_json::json!({ "error": format!("Database query failed: {}", e) }),
                        ),
                    )
                        .into_response();
                }
            }
        }
        (Some(owner), Some(beneficiary)) => {
            // Filter by both owner and beneficiary
            match sqlx::query_as::<_, PlanRow>(
                r#"
                SELECT DISTINCT p.id, p.owner_address, p.token_address, p.amount,
                       p.grace_period, p.grace_period_seconds, p.earn_yield,
                       p.last_ping, p.is_active, p.status, p.yield_rate_bps, p.accrued_yield, p.created_at
                FROM plans p
                INNER JOIN beneficiaries b ON b.plan_id = p.id
                WHERE p.owner_address = $1 AND b.wallet_address = $2
                ORDER BY p.created_at DESC
                "#,
            )
            .bind(owner)
            .bind(beneficiary)
            .fetch_all(&state.db_pool)
            .await
            {
                Ok(rows) => rows,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            serde_json::json!({ "error": format!("Database query failed: {}", e) }),
                        ),
                    )
                        .into_response();
                }
            }
        }
        (None, None) => {
            // No filters: return all plans
            match sqlx::query_as::<_, PlanRow>(
                r#"
                SELECT id, owner_address, token_address, amount, grace_period,
                       grace_period_seconds, earn_yield, last_ping, is_active,
                       status, yield_rate_bps, accrued_yield, created_at
                FROM plans
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&state.db_pool)
            .await
            {
                Ok(rows) => rows,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            serde_json::json!({ "error": format!("Database query failed: {}", e) }),
                        ),
                    )
                        .into_response();
                }
            }
        }
    };

    // Convert each plan row to a response with beneficiaries and yield
    let mut responses = Vec::with_capacity(rows.len());
    for row in rows {
        let beneficiaries = match load_beneficiaries(&state.db_pool, row.id).await {
            Ok(b) => b,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to load beneficiaries: {}", e) })),
                )
                    .into_response();
            }
        };

        responses.push(plan_row_to_response(row, beneficiaries));
    }

    let db_query_ms = db_query_started.elapsed().as_millis();

    if state.plan_cache.is_enabled() && responses.iter().any(|plan| plan.is_active) {
        if let Err(err) = state.plan_cache.set_plans(&query, &responses).await {
            error!(error = %err, "Failed to populate plan cache");
        }
    }

    let mut response = (StatusCode::OK, Json(responses)).into_response();
    add_plan_cache_headers(
        &mut response,
        cache_status,
        cache_lookup_ms,
        db_query_ms,
        total_started.elapsed().as_millis(),
    );
    response
}

/// Verify the ping signature using ed25519.
/// In a production environment this would verify a cryptographic signature;
/// for now we accept any non-empty signature.
fn verify_ping_signature(_owner: &str, signature: &str, _message: &str) -> bool {
    !signature.is_empty()
}

// Handler: Ping Plan
// Contributors: Implement resetting last_ping timestamp and calculating accrued yield up to the ping time
async fn ping_plan(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PingRequest>,
) -> impl IntoResponse {
    // 1. Verify signature
    if !verify_ping_signature(&payload.owner, &payload.signature, &payload.message) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid signature" })),
        )
            .into_response();
    }

    // 2. Fetch the active plan from DB
    let plan = match sqlx::query_as::<_, PlanRow>(
        "SELECT * FROM plans WHERE owner_address = $1 AND is_active = true",
    )
    .bind(&payload.owner)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Active plan not found" })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Database error: {}", e) })),
            )
                .into_response();
        }
    };

    // 3. Calculate accumulated yield
    let current_time = chrono::Utc::now().timestamp();
    let elapsed = if current_time > plan.last_ping {
        (current_time - plan.last_ping) as u64
    } else {
        0
    };

    let mut new_accrued_yield: rust_decimal::Decimal = plan.accrued_yield;
    if plan.earn_yield && elapsed > 0 {
        let amount_f64 = plan.amount.to_string().parse::<f64>().unwrap_or(0.0);
        let yield_val =
            yield_calculator::calculate_yield(amount_f64, plan.yield_rate_bps as u32, elapsed);
        if let Some(yield_dec) = rust_decimal::Decimal::from_f64_retain(yield_val) {
            new_accrued_yield += yield_dec;
        }
    }

    // 4. Update plans in PostgreSQL
    if let Err(e) = sqlx::query("UPDATE plans SET last_ping = $1, accrued_yield = $2 WHERE id = $3")
        .bind(current_time)
        .bind(new_accrued_yield)
        .bind(plan.id)
        .execute(&state.db_pool)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to update plan: {}", e) })),
        )
            .into_response();
    }

    let beneficiary_addresses = match load_beneficiary_addresses(&state.db_pool, plan.id).await {
        Ok(addresses) => addresses,
        Err(err) => {
            error!(
                plan_id = %plan.id,
                error = %err,
                "Failed to load beneficiaries for plan cache invalidation"
            );
            Vec::new()
        }
    };
    invalidate_plan_cache(
        &state.plan_cache,
        &plan.owner_address,
        &beneficiary_addresses,
    )
    .await;

    // 5. Return updated plan status and virtual balance
    let virtual_balance = plan.amount + new_accrued_yield;
    (
        StatusCode::OK,
        Json(PingResponse {
            owner: plan.owner_address,
            status: plan.status,
            virtual_balance,
        }),
    )
        .into_response()
}
// Handler: Trigger Payout
// Contributors: Implement calculating final payout with yield, parsing fiat payout details,
// submitting fiat payouts to AnchorRegistry, and marking the plan inactive
async fn trigger_payout(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PayoutRequest>,
) -> impl IntoResponse {
    // 1. Begin database transaction
    let mut tx = match state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(error = %e, "Failed to begin database transaction");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to begin database transaction: {}", e) })),
            ).into_response();
        }
    };

    // 2. Fetch the active plan for the owner
    let plan = match sqlx::query_as::<_, PlanRow>(
        "SELECT id, owner_address, token_address, amount, grace_period, grace_period_seconds, earn_yield, last_ping, is_active, status, yield_rate_bps, accrued_yield, created_at FROM plans WHERE owner_address = $1 AND is_active = true FOR UPDATE",
    )
    .bind(&payload.owner)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "No active plan found for this owner" })),
            ).into_response();
        }
        Err(e) => {
            error!(owner = %payload.owner, error = %e, "Database error fetching plan");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Database error: {}", e) })),
            ).into_response();
        }
    };

    // 3. Verify if the grace period has elapsed
    let now = chrono::Utc::now().timestamp();
    let deadline = plan.last_ping + plan.grace_period_seconds;
    if now < deadline {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Grace period has not elapsed" })),
        )
            .into_response();
    }

    // 4. Compute final locked amount + yield
    let accrued_yield_f64 = compute_projected_accrued_yield(&plan);
    let accrued_yield_dec = match Decimal::from_f64_retain(accrued_yield_f64) {
        Some(d) => d.normalize(),
        None => Decimal::ZERO,
    };
    let total_payout_dec = plan.amount + accrued_yield_dec;

    // 5. Load beneficiaries for the plan
    let beneficiaries_rows = match sqlx::query_as::<_, BeneficiaryRow>(
        r#"
        SELECT id, plan_id, wallet_address, allocation_bps, fiat_anchor_info
        FROM beneficiaries
        WHERE plan_id = $1
        "#,
    )
    .bind(plan.id)
    .fetch_all(&mut *tx)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!(plan_id = %plan.id, error = %e, "Failed to load beneficiaries");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": format!("Failed to load beneficiaries: {}", e) }),
                ),
            )
                .into_response();
        }
    };

    let n = beneficiaries_rows.len();
    if n == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Plan has no beneficiaries" })),
        )
            .into_response();
    }

    // 6. Iterate over beneficiaries and insert payout records
    let mut remaining = total_payout_dec;
    let mut payout_rows = Vec::with_capacity(n);

    for (i, b) in beneficiaries_rows.iter().enumerate() {
        let share = if i == n - 1 {
            remaining
        } else {
            let amount =
                (total_payout_dec * Decimal::from(b.allocation_bps)) / Decimal::from(10000);
            let amount = amount.floor();
            remaining -= amount;
            amount
        };

        if share <= Decimal::ZERO {
            continue;
        }

        let is_fiat = !b.fiat_anchor_info.trim().is_empty();
        let payout_type_str = if is_fiat { "fiat" } else { "crypto" };
        let payout_status_str = "processing";

        let payout_row = match sqlx::query_as::<_, PayoutRow>(
            r#"
            INSERT INTO payouts (plan_id, beneficiary_address, amount, payout_type, status)
            VALUES ($1, $2, $3, $4::payout_type, $5::payout_status)
            RETURNING id, plan_id, beneficiary_address, amount::text, payout_type::text, status::text, created_at
            "#,
        )
        .bind(plan.id)
        .bind(&b.wallet_address)
        .bind(share)
        .bind(payout_type_str)
        .bind(payout_status_str)
        .fetch_one(&mut *tx)
        .await {
            Ok(row) => row,
            Err(e) => {
                error!(plan_id = %plan.id, beneficiary = %b.wallet_address, error = %e, "Failed to insert payout record");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to insert payout record: {}", e) })),
                ).into_response();
            }
        };

        // Initiate payout distribution
        if is_fiat {
            let (beneficiary_name, fiat_currency, bank_name, account_number) =
                parse_fiat_anchor_info(&b.fiat_anchor_info, &b.wallet_address);
            let token_amount_f64 = share.to_string().parse::<f64>().unwrap_or(0.0);
            let req = crate::stellar_anchor::AnchorPayoutRequest {
                beneficiary_address: b.wallet_address.clone(),
                beneficiary_name,
                token: plan.token_address.clone(),
                token_amount: token_amount_f64,
                fiat_currency,
                bank_name,
                account_number,
            };
            state.anchor.create_payout(req);
        } else {
            tracing::info!(
                plan_id = %plan.id,
                beneficiary = %b.wallet_address,
                amount = %share,
                "Initiated on-chain crypto distribution"
            );
        }

        payout_rows.push(payout_row);
    }

    // 7. Mark the plan as inactive
    if let Err(e) = sqlx::query(
        "UPDATE plans SET is_active = false, status = 'PAID_OUT', accrued_yield = $1, last_ping = $2 WHERE id = $3"
    )
    .bind(accrued_yield_dec)
    .bind(now)
    .bind(plan.id)
    .execute(&mut *tx)
    .await {
        error!(plan_id = %plan.id, error = %e, "Failed to mark plan as inactive");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to mark plan as inactive: {}", e) })),
        ).into_response();
    }

    // 8. Commit transaction
    if let Err(e) = tx.commit().await {
        error!(error = %e, "Failed to commit database transaction");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to commit database transaction: {}", e) })),
        ).into_response();
    }

    // 9. Invalidate cache
    let beneficiary_addresses: Vec<String> = beneficiaries_rows
        .iter()
        .map(|b| b.wallet_address.clone())
        .collect();
    invalidate_plan_cache(
        &state.plan_cache,
        &plan.owner_address,
        &beneficiary_addresses,
    )
    .await;

    (StatusCode::OK, Json(payout_rows)).into_response()
}

fn parse_fiat_anchor_info(info: &str, wallet_address: &str) -> (String, String, String, String) {
    #[derive(Deserialize)]
    struct LocalAnchorInfo {
        name: Option<String>,
        currency: Option<String>,
        bank: Option<String>,
        account: Option<String>,
    }

    if let Ok(parsed) = serde_json::from_str::<LocalAnchorInfo>(info) {
        return (
            parsed.name.unwrap_or_else(|| "Beneficiary".to_string()),
            parsed.currency.unwrap_or_else(|| "USD".to_string()),
            parsed
                .bank
                .unwrap_or_else(|| "Stellar Anchor Bank".to_string()),
            parsed.account.unwrap_or_else(|| {
                format!("ACC-{}", &wallet_address[..8.min(wallet_address.len())])
            }),
        );
    }

    let parts: Vec<&str> = if info.contains(';') {
        info.split(';').map(|s| s.trim()).collect()
    } else if info.contains(',') {
        info.split(',').map(|s| s.trim()).collect()
    } else {
        vec![info]
    };

    if parts.len() >= 3 {
        return (
            "Beneficiary".to_string(),
            parts[2].to_string(),
            parts[0].to_string(),
            parts[1].to_string(),
        );
    }

    let info_upper = info.to_uppercase();
    let fiat_currency = if info_upper.contains("NGN") {
        "NGN"
    } else if info_upper.contains("KES") {
        "KES"
    } else if info_upper.contains("BRL") {
        "BRL"
    } else if info_upper.contains("PHP") {
        "PHP"
    } else if info_upper.contains("EUR") {
        "EUR"
    } else {
        "USD"
    };

    let bank_name = if info.trim().is_empty() {
        "Stellar Anchor Bank".to_string()
    } else {
        info.to_string()
    };

    let account_number = format!("ACC-{}", &wallet_address[..8.min(wallet_address.len())]);

    (
        "Beneficiary".to_string(),
        fiat_currency.to_string(),
        bank_name,
        account_number,
    )
}
//
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

// --- KYC Endpoints ---

#[derive(Debug, Serialize, Deserialize)]
pub struct KYCStatusResponse {
    pub wallet_address: String,
    pub kyc_status: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub provider_reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KYCSubmitRequest {
    pub full_name: String,
    pub email: String,
    pub date_of_birth: String,
    pub nationality: String,
    pub id_type: String,
    pub id_number: String,
    pub expiry_date: String,
    pub street_address: String,
    pub city: String,
    pub country: String,
    pub postal_code: String,
    pub document_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct KYCDocumentResponse {
    pub document_id: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct KYCRequirementsResponse {
    pub requires_id: bool,
    pub requires_address_proof: bool,
    pub requires_selfie: bool,
    pub supported_id_types: Vec<String>,
    pub supported_countries: Vec<String>,
}

// Get user's KYC status
async fn get_kyc_status() -> impl IntoResponse {
    // In a real implementation, this would get the user from authentication context
    // For now, return a mock response
    let response = KYCStatusResponse {
        wallet_address: "GDTEST123".to_string(),
        kyc_status: "pending".to_string(),
        submitted_at: None,
        approved_at: None,
        rejected_at: None,
        rejection_reason: None,
        provider_reference: None,
    };

    (StatusCode::OK, Json(response))
}

// Submit KYC verification data
async fn submit_kyc(Json(_payload): Json<KYCSubmitRequest>) -> impl IntoResponse {
    // In a real implementation, this would:
    // 1. Validate the request
    // 2. Submit to third-party KYC provider
    // 3. Store in database
    // 4. Return reference ID

    let response = KYCStatusResponse {
        wallet_address: "GDTEST123".to_string(),
        kyc_status: "submitted".to_string(),
        submitted_at: Some(Utc::now()),
        approved_at: None,
        rejected_at: None,
        rejection_reason: None,
        provider_reference: Some("ref-001".to_string()),
    };

    (StatusCode::OK, Json(response))
}

// Upload KYC document
async fn upload_kyc_document() -> impl IntoResponse {
    // In a real implementation, this would:
    // 1. Receive multipart form data with file and document_type
    // 2. Validate file (size, type)
    // 3. Upload to cloud storage (S3, etc.)
    // 4. Store metadata in database
    // 5. Return document_id and URL

    let response = KYCDocumentResponse {
        document_id: Uuid::new_v4().to_string(),
        url: "https://example.com/documents/doc-001".to_string(),
    };

    (StatusCode::OK, Json(response))
}

// Check if KYC is required
async fn is_kyc_required() -> impl IntoResponse {
    #[derive(Debug, Serialize)]
    struct RequiredResponse {
        required: bool,
        reason: Option<String>,
    }

    let response = RequiredResponse {
        required: true,
        reason: Some("All users must complete KYC to create plans".to_string()),
    };

    (StatusCode::OK, Json(response))
}

// Get KYC requirements
async fn get_kyc_requirements() -> impl IntoResponse {
    let response = KYCRequirementsResponse {
        requires_id: true,
        requires_address_proof: true,
        requires_selfie: false,
        supported_id_types: vec![
            "international_passport".to_string(),
            "national_id".to_string(),
            "drivers_license".to_string(),
        ],
        supported_countries: vec![
            "US".to_string(),
            "UK".to_string(),
            "DE".to_string(),
            "FR".to_string(),
            "CA".to_string(),
            "AU".to_string(),
            "JP".to_string(),
            "SG".to_string(),
        ],
    };

    (StatusCode::OK, Json(response))
}

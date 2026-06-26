use axum::{
    extract::{Query, State},
    http::StatusCode,
    middleware::from_fn,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::auth::signature_auth_middleware;
use crate::kyc_webhook::kyc_webhook_handler;
use crate::stellar_anchor::{AnchorPayout, AnchorRegistry};
use crate::ws::{ws_handler, KycUpdateEvent};
use crate::yield_calculator;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use stellar_strkey::ed25519::PublicKey as StellarPublicKey;

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
}

#[derive(Deserialize)]
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
    pub yield_rate_bps: i32,
    pub accrued_yield: rust_decimal::Decimal,
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

/// Compute the accrued yield for a plan based on elapsed time since last_ping.
fn compute_accrued_yield(
    amount: &Decimal,
    yield_rate_bps: i32,
    last_ping: i64,
    config: &yield_calculator::ApyConfig,
) -> Decimal {
    if yield_rate_bps == 0 || last_ping == 0 {
        return Decimal::ZERO;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let elapsed_secs = (now - last_ping).max(0) as u64;

    yield_calculator::calculate_yield(*amount, yield_rate_bps as u32, elapsed_secs, config)
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
fn plan_row_to_response(
    row: PlanRow,
    beneficiaries: Vec<BeneficiaryResponse>,
    config: &yield_calculator::ApyConfig,
) -> PlanResponse {
    let _accrued_yield =
        compute_accrued_yield(&row.amount, row.yield_rate_bps, row.last_ping, config);

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
        accrued_yield: row.accrued_yield,
        created_at: row.created_at,
        beneficiaries,
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
        RETURNING id, owner_address, token_address, amount, grace_period, grace_period_seconds, earn_yield, yield_rate_bps, accrued_yield, last_ping, is_active, status, created_at
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
    .bind(payload.yield_rate_bps as i32)
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
        yield_rate_bps: plan_row.yield_rate_bps,
        accrued_yield: plan_row.accrued_yield,
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
    // Build the query dynamically based on filters
    let rows: Vec<PlanRow> = match (&query.owner, &query.beneficiary) {
        (Some(owner), None) => {
            // Filter by owner only
            match sqlx::query_as::<_, PlanRow>(
                r#"
                SELECT id, owner_address, token_address, amount, grace_period,
                       grace_period_seconds, earn_yield, last_ping, is_active,
                       status, yield_rate_bps, created_at
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
                       p.last_ping, p.is_active, p.status, p.yield_rate_bps, p.created_at
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
                       p.last_ping, p.is_active, p.status, p.yield_rate_bps, p.created_at
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
                       status, yield_rate_bps, created_at
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

        responses.push(plan_row_to_response(row, beneficiaries, &state.apy_config));
    }

    (StatusCode::OK, Json(responses)).into_response()
}

fn verify_ping_signature(owner: &str, signature: &str, message: &str) -> bool {
    let public_key_bytes = match StellarPublicKey::from_string(owner) {
        Ok(pk) => pk.0,
        Err(_) => return false,
    };
    let verifying_key = match VerifyingKey::from_bytes(&public_key_bytes) {
        Ok(key) => key,
        Err(_) => return false,
    };
    let sig_bytes = match hex::decode(signature) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let sig = match Signature::from_slice(&sig_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };
    verifying_key.verify(message.as_bytes(), &sig).is_ok()
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

    let mut new_accrued_yield = plan.accrued_yield;
    if plan.earn_yield && elapsed > 0 {
        let yield_val = yield_calculator::calculate_yield(
            plan.amount,
            plan.yield_rate_bps as u32,
            elapsed,
            &state.apy_config,
        );
        new_accrued_yield += yield_val.normalize();
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

//! AI plan optimisation endpoints (issue #1271).
//!
//! `frontend/app/lib/api/aiOptimization.ts` targets recommendation, custom
//! projection and recommendation-response routes. Recommendations are computed
//! deterministically from plan data and the authoritative APY configuration in
//! PostgreSQL — no random or fixed mock values. When the inputs needed for an
//! analysis are missing the handlers return an explicit error instead of a
//! success-shaped placeholder.

use crate::api::AppState;
use crate::auth::UserContext;
use crate::will_documents::caller_can_access;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Risk-free rate assumed by the Sharpe-ratio calculation, in percent.
const RISK_FREE_RATE: f64 = 4.0;
/// One-year 95% Value-at-Risk z-score.
const VAR_95_Z: f64 = 1.645;
/// Universe cap: recommendations stay focused on a handful of pools.
const MAX_RECOMMENDED_ALLOCATIONS: usize = 3;
/// Request cap for caller-supplied projections.
const MAX_CUSTOM_ALLOCATIONS: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetAllocation {
    pub asset_symbol: String,
    pub chain: String,
    pub current_percentage: f64,
    pub recommended_percentage: f64,
    pub adjustment_reason: String,
    pub expected_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskMetrics {
    /// Annualised volatility, in percent.
    pub volatility: f64,
    pub sharpe_ratio: f64,
    /// Expected maximum drawdown, in percent.
    pub max_drawdown: f64,
    /// One-year 95% Value-at-Risk, in percent.
    pub value_at_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectedOutcomes {
    pub estimated_value1_year: f64,
    pub estimated_value5_year: f64,
    pub estimated_value10_year: f64,
    pub risk_metrics: RiskMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationRecommendation {
    pub id: String,
    pub plan_id: i64,
    pub recommended_allocations: Vec<AssetAllocation>,
    /// 0.0–1.0, based on how many pools had authoritative rate data.
    pub confidence_score: f64,
    /// Weighted annual return, in percent.
    pub expected_return: f64,
    /// Weighted annual volatility, in percent.
    pub risk_score: f64,
    pub reasoning: String,
    pub generated_at: DateTime<Utc>,
    pub projected_outcomes: ProjectedOutcomes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomProjection {
    pub allocations: Vec<AssetAllocation>,
    pub projected_outcomes: ProjectedOutcomes,
    pub expected_return: f64,
    pub risk_score: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationResponse {
    pub status: String,
    pub reason: Option<String>,
    pub applied_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct AiPlanRow {
    id: Uuid,
    owner_address: String,
    token_address: String,
    amount: Decimal,
    earn_yield: bool,
    yield_rate_bps: i32,
    onchain_plan_id: Option<i64>,
}

impl AiPlanRow {
    fn amount_f64(&self) -> f64 {
        self.amount.to_string().parse::<f64>().unwrap_or(0.0)
    }

    /// Numeric id exposed to the frontend, which keys plans by their on-chain
    /// id. Plans that were never written on-chain report 0.
    fn public_plan_id(&self) -> i64 {
        self.onchain_plan_id.unwrap_or(0)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ApyRow {
    token_address: String,
    rate_bps: i32,
}

fn bad_request(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": message.into() })),
    )
        .into_response()
}

fn unprocessable(message: impl Into<String>) -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({ "error": message.into() })),
    )
        .into_response()
}

fn db_error(action: &str, error: sqlx::Error) -> Response {
    tracing::error!(error = %error, "Failed to {action}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": format!("Failed to {action}") })),
    )
        .into_response()
}

fn unavailable(message: impl Into<String>) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": message.into() })),
    )
        .into_response()
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Annualised volatility assumption per asset class. Documented here because
/// it is the model behind every risk field in the API; it is not derived from
/// live market data, which the backend does not collect.
fn volatility_for(symbol: &str) -> f64 {
    match symbol.to_ascii_uppercase().as_str() {
        "USDC" | "USDT" | "DAI" | "EURC" | "PYUSD" => 2.0,
        "BTC" | "WBTC" => 15.0,
        "ETH" | "WETH" => 20.0,
        "XLM" => 35.0,
        _ => 25.0,
    }
}

fn chain_for(symbol: &str) -> &'static str {
    match symbol.to_ascii_uppercase().as_str() {
        "BTC" | "WBTC" => "bitcoin",
        "ETH" | "WETH" => "ethereum",
        _ => "stellar",
    }
}

fn risk_metrics(expected_return: f64, volatility: f64) -> RiskMetrics {
    let sharpe = if volatility > 0.0 {
        (expected_return - RISK_FREE_RATE) / volatility
    } else {
        0.0
    };
    RiskMetrics {
        volatility: round2(volatility),
        sharpe_ratio: round2(sharpe),
        max_drawdown: round2(2.0 * volatility),
        value_at_risk: round2(VAR_95_Z * volatility),
    }
}

/// Compound `amount` at `annual_return` percent for `years`, using annual
/// compounding.
fn project_value(amount: f64, annual_return: f64, years: u32) -> f64 {
    round2(amount * (1.0 + annual_return / 100.0).powf(years as f64))
}

fn projected_outcomes(amount: f64, expected_return: f64, volatility: f64) -> ProjectedOutcomes {
    ProjectedOutcomes {
        estimated_value1_year: project_value(amount, expected_return, 1),
        estimated_value5_year: project_value(amount, expected_return, 5),
        estimated_value10_year: project_value(amount, expected_return, 10),
        risk_metrics: risk_metrics(expected_return, volatility),
    }
}

/// Normalise positive weights so they sum to exactly 100.00 percent.
pub(crate) fn normalise_percentages(weights: &mut [f64]) {
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        let even = 100.0 / weights.len() as f64;
        weights.fill(round2(even));
    } else {
        for weight in weights.iter_mut() {
            *weight = round2(*weight / total * 100.0);
        }
    }
    let sum: f64 = weights.iter().sum();
    if let Some(last) = weights.last_mut() {
        *last = round2(*last + (100.0 - sum));
    }
}

/// Build an allocation set from the plan's current asset plus the highest
/// yielding configured pools. Returned percentages always sum to 100.
fn build_allocations(
    plan: &AiPlanRow,
    rates: &HashMap<String, i32>,
) -> (Vec<AssetAllocation>, f64, f64) {
    let plan_symbol = canonical_symbol(&plan.token_address);
    let mut universe: Vec<(String, f64)> = Vec::new();

    fn add(universe: &mut Vec<(String, f64)>, symbol: &str, apy: f64) {
        if !symbol.is_empty()
            && !universe
                .iter()
                .any(|(existing, _)| existing.eq_ignore_ascii_case(symbol))
        {
            universe.push((symbol.to_string(), apy));
        }
    }

    add(
        &mut universe,
        &plan_symbol,
        rates
            .get(&plan_symbol.to_ascii_lowercase())
            .copied()
            .unwrap_or(plan.yield_rate_bps)
            .max(0) as f64
            / 100.0,
    );

    let mut configured: Vec<(&String, &i32)> = rates.iter().collect();
    configured.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    for (symbol, rate) in configured {
        if universe.len() >= MAX_RECOMMENDED_ALLOCATIONS {
            break;
        }
        if *rate > 0 {
            add(&mut universe, symbol, *rate as f64 / 100.0);
        }
    }

    let mut scored: Vec<(String, f64, f64)> = universe
        .into_iter()
        .map(|(symbol, apy)| {
            let volatility = volatility_for(&symbol);
            // Yield per unit of risk; the +1 keeps zero-yield pools eligible.
            let score = (1.0 + apy) / volatility;
            (symbol, apy, score)
        })
        .collect();
    scored.sort_by(|a, b| {
        b.2.partial_cmp(&a.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    let mut weights: Vec<f64> = scored.iter().map(|(_, _, score)| *score).collect();
    normalise_percentages(&mut weights);

    let mut expected_return = 0.0;
    let mut volatility = 0.0;
    let allocations = scored
        .into_iter()
        .zip(weights)
        .map(|((symbol, apy, _), weight)| {
            expected_return += apy * weight / 100.0;
            volatility += volatility_for(&symbol) * weight / 100.0;
            let is_current = symbol.eq_ignore_ascii_case(&plan_symbol);
            let current = if is_current { 100.0 } else { 0.0 };
            AssetAllocation {
                asset_symbol: symbol.clone(),
                chain: chain_for(&symbol).to_string(),
                current_percentage: current,
                recommended_percentage: weight,
                adjustment_reason: if is_current {
                    "Plan's existing asset, retained as part of the diversified allocation".to_string()
                } else {
                    format!(
                        "Higher configured yield per unit of modelled risk ({} bps APY)",
                        (apy * 100.0).round() as i64
                    )
                },
                expected_impact: format!(
                    "Expected annual return contribution {:.2}% at {}% weight",
                    apy * weight / 100.0,
                    weight
                ),
            }
        })
        .collect();

    (allocations, round2(expected_return), round2(volatility))
}

/// APY configuration stores symbols for the built-in assets (`USDC`, `XLM`, …)
/// and full SAC addresses for others. Derive a display symbol for the latter.
fn canonical_symbol(token_address: &str) -> String {
    let trimmed = token_address.trim();
    if trimmed.len() <= 12 {
        return trimmed.to_ascii_uppercase();
    }
    // Soroban SAC addresses are `C…`; the last four characters are not a
    // meaningful ticker, so label the plan's own pool explicitly.
    if trimmed.starts_with('C') {
        "CUSTOM".to_string()
    } else {
        trimmed.to_ascii_uppercase()
    }
}

async fn resolve_plan(state: &Arc<AppState>, raw: &str) -> Result<AiPlanRow, Response> {
    const COLUMNS: &str = "id, owner_address, token_address, amount, earn_yield, \
                           yield_rate_bps, onchain_plan_id";

    if let Ok(id) = Uuid::parse_str(raw) {
        return match sqlx::query_as::<_, AiPlanRow>(&format!(
            "SELECT {COLUMNS} FROM plans WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&state.db_pool)
        .await
        {
            Ok(Some(plan)) => Ok(plan),
            Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({ "error": "Plan not found" }))).into_response()),
            Err(error) => Err(db_error("load plan", error)),
        };
    }

    let Ok(onchain_id) = raw.parse::<i64>() else {
        return Err(bad_request(
            "plan id must be a UUID or a numeric on-chain plan id",
        ));
    };
    match sqlx::query_as::<_, AiPlanRow>(&format!(
        "SELECT {COLUMNS} FROM plans WHERE onchain_plan_id = $1 ORDER BY created_at DESC LIMIT 1"
    ))
    .bind(onchain_id)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(plan)) => Ok(plan),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({ "error": "Plan not found" }))).into_response()),
        Err(error) => Err(db_error("load plan", error)),
    }
}

fn authorize_plan(ctx: &UserContext, plan: &AiPlanRow) -> Result<(), Response> {
    if caller_can_access(ctx, &plan.owner_address) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You do not have access to this plan" })),
        )
            .into_response())
    }
}

async fn load_rates(state: &Arc<AppState>) -> Result<HashMap<String, i32>, Response> {
    let rows = match sqlx::query_as::<_, ApyRow>(
        "SELECT token_address, rate_bps FROM apy_configurations ORDER BY rate_bps DESC",
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(rows) => rows,
        Err(error) => return Err(db_error("load APY configuration", error)),
    };

    Ok(rows
        .into_iter()
        .map(|row| (canonical_symbol(&row.token_address).to_ascii_lowercase(), row.rate_bps))
        .collect())
}

/// GET /api/ai/optimize/{planId}
pub async fn get_optimization(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(plan_id): Path<String>,
) -> Response {
    let plan = match resolve_plan(&state, &plan_id).await {
        Ok(plan) => plan,
        Err(response) => return response,
    };
    if let Err(response) = authorize_plan(&ctx, &plan) {
        return response;
    }
    if plan.amount_f64() <= 0.0 {
        return unprocessable("plan has no positive balance to analyse");
    }

    // Reuse an outstanding recommendation so accept/reject always refers to the
    // allocation set the client was shown.
    let existing: Option<serde_json::Value> = match sqlx::query_scalar(
        r#"
        SELECT payload FROM ai_recommendations
        WHERE plan_id = $1 AND status = 'pending'
        ORDER BY created_at DESC LIMIT 1
        "#,
    )
    .bind(plan.id)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(value) => value,
        Err(error) => return db_error("load recommendation", error),
    };
    if let Some(payload) = existing {
        return Json(payload).into_response();
    }

    let rates = match load_rates(&state).await {
        Ok(rates) => rates,
        Err(response) => return response,
    };
    if rates.is_empty() {
        return unavailable("no APY configuration is available for analysis");
    }

    let (allocations, expected_return, risk_score) = build_allocations(&plan, &rates);
    let amount = plan.amount_f64();
    let data_points = allocations.len();
    let confidence = round2((0.4 + 0.15 * data_points as f64).min(0.95));
    let reasoning = format!(
        "Allocation derived from the plan's {:.2} {} balance and {} configured pool rate(s). \
         Weights are proportional to configured yield per unit of modelled risk; \
         projected values compound the weighted return annually.",
        amount,
        canonical_symbol(&plan.token_address),
        rates.len()
    );

    let id = Uuid::new_v4();
    let recommendation = OptimizationRecommendation {
        id: id.to_string(),
        plan_id: plan.public_plan_id(),
        recommended_allocations: allocations,
        confidence_score: confidence,
        expected_return,
        risk_score,
        reasoning,
        generated_at: Utc::now(),
        projected_outcomes: projected_outcomes(amount, expected_return, risk_score),
    };

    let payload = match serde_json::to_value(&recommendation) {
        Ok(payload) => payload,
        Err(error) => {
            tracing::error!(error = %error, "Failed to serialise recommendation");
            return db_error("serialise recommendation", sqlx::Error::Protocol(error.to_string()));
        }
    };

    if let Err(error) = sqlx::query(
        r#"
        INSERT INTO ai_recommendations (id, plan_id, owner_address, payload)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(id)
    .bind(plan.id)
    .bind(&plan.owner_address)
    .bind(&payload)
    .execute(&state.db_pool)
    .await
    {
        return db_error("persist recommendation", error);
    }

    Json(recommendation).into_response()
}

#[derive(Debug, Deserialize)]
pub struct CustomProjectionRequest {
    pub allocations: Vec<AssetAllocation>,
}

/// POST /api/ai/optimize/{planId}/custom
pub async fn get_custom_projection(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(plan_id): Path<String>,
    Json(request): Json<CustomProjectionRequest>,
) -> Response {
    let plan = match resolve_plan(&state, &plan_id).await {
        Ok(plan) => plan,
        Err(response) => return response,
    };
    if let Err(response) = authorize_plan(&ctx, &plan) {
        return response;
    }

    if request.allocations.is_empty() {
        return bad_request("allocations must not be empty");
    }
    if request.allocations.len() > MAX_CUSTOM_ALLOCATIONS {
        return bad_request(format!(
            "at most {MAX_CUSTOM_ALLOCATIONS} allocations are supported"
        ));
    }
    for allocation in &request.allocations {
        if allocation.asset_symbol.trim().is_empty() {
            return bad_request("every allocation must name an asset");
        }
        if !allocation.recommended_percentage.is_finite()
            || !(0.0..=100.0).contains(&allocation.recommended_percentage)
        {
            return bad_request("allocation percentages must be between 0 and 100");
        }
    }
    let total: f64 = request
        .allocations
        .iter()
        .map(|allocation| allocation.recommended_percentage)
        .sum();
    if (total - 100.0).abs() > 0.01 {
        return bad_request("allocation percentages must sum to 100");
    }

    let rates = match load_rates(&state).await {
        Ok(rates) => rates,
        Err(response) => return response,
    };
    if rates.is_empty() {
        return unavailable("no APY configuration is available for analysis");
    }

    let mut expected_return = 0.0;
    let mut volatility = 0.0;
    for allocation in &request.allocations {
        let key = allocation.asset_symbol.trim().to_ascii_lowercase();
        let Some(rate_bps) = rates.get(&key) else {
            return unprocessable(format!(
                "asset {} is not supported for projections",
                allocation.asset_symbol
            ));
        };
        let weight = allocation.recommended_percentage / 100.0;
        expected_return += (*rate_bps as f64 / 100.0) * weight;
        volatility += volatility_for(&allocation.asset_symbol) * weight;
    }

    let expected_return = round2(expected_return);
    let risk_score = round2(volatility);

    Json(CustomProjection {
        allocations: request.allocations,
        projected_outcomes: projected_outcomes(plan.amount_f64(), expected_return, risk_score),
        expected_return,
        risk_score,
    })
    .into_response()
}

#[derive(Debug, Deserialize)]
pub struct RespondRequest {
    pub action: String,
    pub reason: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct RecommendationRow {
    id: Uuid,
    owner_address: String,
    status: String,
}

/// POST /api/ai/recommendations/{id}/respond
pub async fn respond_to_recommendation(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(recommendation_id): Path<String>,
    Json(request): Json<RespondRequest>,
) -> Response {
    let Ok(recommendation_id) = Uuid::parse_str(&recommendation_id) else {
        return bad_request("recommendation id must be a UUID");
    };

    let action = request.action.trim().to_ascii_lowercase();
    if action != "accept" && action != "reject" {
        return bad_request("action must be `accept` or `reject`");
    }
    let reason = request
        .reason
        .map(|reason| reason.trim().to_string())
        .filter(|reason| !reason.is_empty());
    if action == "reject" && reason.is_none() {
        return bad_request("a reason is required when rejecting a recommendation");
    }
    if reason.as_ref().is_some_and(|reason| reason.len() > 500) {
        return bad_request("reason must be 500 characters or fewer");
    }

    let row = match sqlx::query_as::<_, RecommendationRow>(
        "SELECT id, owner_address, status FROM ai_recommendations WHERE id = $1",
    )
    .bind(recommendation_id)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Recommendation not found" })),
            )
                .into_response()
        }
        Err(error) => return db_error("load recommendation", error),
    };

    if !caller_can_access(&ctx, &row.owner_address) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You do not have access to this recommendation" })),
        )
            .into_response();
    }
    if row.status != "pending" {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "Recommendation has already been answered" })),
        )
            .into_response();
    }

    let status = if action == "accept" { "accepted" } else { "rejected" };
    let applied_at = Utc::now();
    let updated = sqlx::query(
        r#"
        UPDATE ai_recommendations
        SET status = $1, response_reason = $2, responded_by = $3, responded_at = $4
        WHERE id = $5 AND status = 'pending'
        "#,
    )
    .bind(status)
    .bind(&reason)
    .bind(&ctx.user_id)
    .bind(applied_at)
    .bind(row.id)
    .execute(&state.db_pool)
    .await;

    match updated {
        Ok(result) if result.rows_affected() == 0 => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "Recommendation has already been answered" })),
        )
            .into_response(),
        Ok(_) => Json(RecommendationResponse {
            status: status.to_string(),
            reason,
            applied_at,
        })
        .into_response(),
        Err(error) => db_error("update recommendation", error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(token: &str, amount: Decimal) -> AiPlanRow {
        AiPlanRow {
            id: Uuid::new_v4(),
            owner_address: "GOWNER".to_string(),
            token_address: token.to_string(),
            amount,
            earn_yield: true,
            yield_rate_bps: 300,
            onchain_plan_id: Some(7),
        }
    }

    #[test]
    fn allocations_are_normalised_and_include_the_plan_asset() {
        let rates = HashMap::from([
            ("usdc".to_string(), 300),
            ("xlm".to_string(), 200),
            ("custom".to_string(), 100),
        ]);
        let (allocations, expected_return, risk) =
            build_allocations(&plan("USDC", Decimal::new(100_000, 2)), &rates);

        let total: f64 = allocations
            .iter()
            .map(|allocation| allocation.recommended_percentage)
            .sum();
        assert!((total - 100.0).abs() < 0.01, "weights sum to {total}");
        assert!(allocations
            .iter()
            .any(|allocation| allocation.asset_symbol == "USDC"));
        assert!(expected_return > 0.0);
        assert!(risk > 0.0);
    }

    #[test]
    fn stablecoin_allocations_have_lower_volatility_than_xlm() {
        assert!(volatility_for("USDC") < volatility_for("XLM"));
    }

    #[test]
    fn projection_compounds_annually() {
        assert_eq!(project_value(1_000.0, 10.0, 1), 1_100.0);
        assert_eq!(project_value(1_000.0, 10.0, 2), 1_210.0);
    }

    #[test]
    fn full_sac_addresses_are_labelled_custom() {
        assert_eq!(
            canonical_symbol("CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4"),
            "CUSTOM"
        );
        assert_eq!(canonical_symbol("usdc"), "USDC");
    }

    #[test]
    fn risk_metrics_are_finite_for_zero_volatility() {
        let metrics = risk_metrics(3.0, 0.0);
        assert_eq!(metrics.sharpe_ratio, 0.0);
        assert_eq!(metrics.max_drawdown, 0.0);
        assert_eq!(metrics.value_at_risk, 0.0);
    }

    #[test]
    fn public_plan_id_falls_back_to_zero() {
        let mut row = plan("USDC", Decimal::new(100, 0));
        row.onchain_plan_id = None;
        assert_eq!(row.public_plan_id(), 0);
    }
}

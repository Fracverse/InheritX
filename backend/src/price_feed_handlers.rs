use crate::api_error::ApiError;
use crate::auth::AuthenticatedAdmin;
use crate::price_feed::{PriceFeedService, PriceFeedSource};
use axum::extract::{Path, State};
use axum::Json;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use tracing;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPriceResponse {
    pub asset_code: String,
    pub price: Decimal,
    pub timestamp: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceHistoryResponse {
    pub asset_code: String,
    pub prices: Vec<PricePoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricePoint {
    pub price: Decimal,
    pub timestamp: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterFeedRequest {
    pub asset_code: String,
    pub source: String,
    pub feed_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePriceRequest {
    pub price: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValuationResponse {
    pub asset_code: String,
    pub amount: Decimal,
    pub current_price: Decimal,
    pub valuation_usd: Decimal,
    pub collateral_ratio: Decimal,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanValuationResponse {
    pub plan_id: Uuid,
    pub asset_code: String,
    pub amount: Decimal,
    pub current_price: Decimal,
    pub valuation_usd: Decimal,
    pub collateral_ratio: Decimal,
    pub last_updated: String,
}

/// Get current price for an asset
pub async fn get_price(
    State((_db, price_service)): State<(PgPool, Arc<dyn PriceFeedService>)>,
    Path(asset_code): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let asset_price = price_service.get_price(&asset_code).await?;

    Ok(Json(json!({
        "status": "success",
        "data": GetPriceResponse {
            asset_code: asset_price.asset_code,
            price: asset_price.price,
            timestamp: asset_price.timestamp.to_rfc3339(),
            source: asset_price.source,
        }
    })))
}

/// Get price history for an asset
pub async fn get_price_history(
    State((_db, price_service)): State<(PgPool, Arc<dyn PriceFeedService>)>,
    Path(asset_code): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let history = price_service.get_price_history(&asset_code, 100).await?;

    let prices = history
        .into_iter()
        .map(|p| PricePoint {
            price: p.price,
            timestamp: p.timestamp.to_rfc3339(),
            source: p.source,
        })
        .collect();

    Ok(Json(json!({
        "status": "success",
        "data": PriceHistoryResponse {
            asset_code,
            prices,
        }
    })))
}

/// Register a new price feed (admin only)
pub async fn register_price_feed(
    State((_db, price_service)): State<(PgPool, Arc<dyn PriceFeedService>)>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Json(req): Json<RegisterFeedRequest>,
) -> Result<Json<Value>, ApiError> {
    let source = match req.source.to_lowercase().as_str() {
        "pyth" => PriceFeedSource::Pyth,
        "chainlink" => PriceFeedSource::Chainlink,
        "custom" => PriceFeedSource::Custom,
        _ => {
            return Err(ApiError::BadRequest(
                "Invalid source. Must be 'pyth', 'chainlink', or 'custom'".to_string(),
            ))
        }
    };

    let config = price_service
        .register_feed(&req.asset_code, source, &req.feed_id)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "message": format!("Price feed registered for {}", req.asset_code),
        "data": {
            "id": config.id,
            "asset_code": config.asset_code,
            "source": config.source,
            "feed_id": config.feed_id,
            "is_active": config.is_active,
        }
    })))
}

/// Update price for an asset (admin only)
pub async fn update_price(
    State((_db, price_service)): State<(PgPool, Arc<dyn PriceFeedService>)>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Path(asset_code): Path<String>,
    Json(req): Json<UpdatePriceRequest>,
) -> Result<Json<Value>, ApiError> {
    if req.price <= Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "Price must be greater than zero".to_string(),
        ));
    }

    let asset_price = price_service.update_price(&asset_code, req.price).await?;

    Ok(Json(json!({
        "status": "success",
        "message": format!("Price updated for {}", asset_code),
        "data": GetPriceResponse {
            asset_code: asset_price.asset_code,
            price: asset_price.price,
            timestamp: asset_price.timestamp.to_rfc3339(),
            source: asset_price.source,
        }
    })))
}

/// Calculate collateral valuation for an amount
pub async fn calculate_valuation(
    State((_db, price_service)): State<(PgPool, Arc<dyn PriceFeedService>)>,
    Path((asset_code, amount_str)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let amount: Decimal = amount_str
        .parse()
        .map_err(|_| ApiError::BadRequest("Invalid amount format".to_string()))?;

    if amount <= Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "Amount must be greater than zero".to_string(),
        ));
    }

    let valuation = price_service
        .calculate_valuation(&asset_code, amount)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "data": ValuationResponse {
            asset_code: valuation.asset_code,
            amount: valuation.amount,
            current_price: valuation.current_price,
            valuation_usd: valuation.valuation_usd,
            collateral_ratio: valuation.collateral_ratio,
            last_updated: valuation.last_updated.to_rfc3339(),
        }
    })))
}

/// Get plan valuation
pub async fn get_plan_valuation(
    State((db, price_service)): State<(PgPool, Arc<dyn PriceFeedService>)>,
    Path(plan_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    // Fetch plan from database
    let plan = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT asset_code, net_amount::text
        FROM plans
        WHERE id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_optional(&db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch plan: {}", e);
        ApiError::Internal(anyhow::anyhow!("Database error"))
    })?
    .ok_or_else(|| ApiError::NotFound(format!("Plan {} not found", plan_id)))?;

    let asset_code = plan.0;
    let amount = Decimal::from_str(&plan.1).map_err(|e| {
        tracing::error!("Failed to parse amount: {}", e);
        ApiError::Internal(anyhow::anyhow!("Invalid amount format"))
    })?;

    let mut valuation = price_service
        .calculate_valuation(&asset_code, amount)
        .await?;

    valuation.plan_id = plan_id;

    Ok(Json(json!({
        "status": "success",
        "data": PlanValuationResponse {
            plan_id: valuation.plan_id,
            asset_code: valuation.asset_code,
            amount: valuation.amount,
            current_price: valuation.current_price,
            valuation_usd: valuation.valuation_usd,
            collateral_ratio: valuation.collateral_ratio,
            last_updated: valuation.last_updated.to_rfc3339(),
        }
    })))
}

/// Get all active price feeds (admin only)
pub async fn get_active_feeds(
    State((_db, price_service)): State<(PgPool, Arc<dyn PriceFeedService>)>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let feeds = price_service.get_active_feeds().await?;

    Ok(Json(json!({
        "status": "success",
        "data": feeds,
        "count": feeds.len()
    })))
}

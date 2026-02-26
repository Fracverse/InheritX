use crate::api_error::ApiError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Price feed source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriceFeedSource {
    Pyth,
    Chainlink,
    Custom,
}

impl PriceFeedSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            PriceFeedSource::Pyth => "pyth",
            PriceFeedSource::Chainlink => "chainlink",
            PriceFeedSource::Custom => "custom",
        }
    }
}

/// Asset price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPrice {
    pub asset_code: String,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

/// Price feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeedConfig {
    pub id: Uuid,
    pub asset_code: String,
    pub source: String,
    pub feed_id: String,
    pub is_active: bool,
    pub last_updated: Option<DateTime<Utc>>,
}

/// Collateral valuation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollateralValuation {
    pub plan_id: Uuid,
    pub asset_code: String,
    pub amount: Decimal,
    pub current_price: Decimal,
    pub valuation_usd: Decimal,
    pub collateral_ratio: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Price feed service trait
#[async_trait]
pub trait PriceFeedService: Send + Sync {
    /// Get current price for an asset
    async fn get_price(&self, asset_code: &str) -> Result<AssetPrice, ApiError>;

    /// Get price history for an asset
    async fn get_price_history(
        &self,
        asset_code: &str,
        limit: i64,
    ) -> Result<Vec<AssetPrice>, ApiError>;

    /// Register a new price feed
    async fn register_feed(
        &self,
        asset_code: &str,
        source: PriceFeedSource,
        feed_id: &str,
    ) -> Result<PriceFeedConfig, ApiError>;

    /// Update price for an asset
    async fn update_price(&self, asset_code: &str, price: Decimal) -> Result<AssetPrice, ApiError>;

    /// Calculate collateral valuation
    async fn calculate_valuation(
        &self,
        asset_code: &str,
        amount: Decimal,
    ) -> Result<CollateralValuation, ApiError>;

    /// Get all active price feeds
    async fn get_active_feeds(&self) -> Result<Vec<PriceFeedConfig>, ApiError>;
}

/// In-memory price cache with database persistence
pub struct DefaultPriceFeedService {
    db: PgPool,
    price_cache: Arc<RwLock<HashMap<String, AssetPrice>>>,
    cache_ttl_secs: u64,
}

impl DefaultPriceFeedService {
    pub fn new(db: PgPool, cache_ttl_secs: u64) -> Self {
        Self {
            db,
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_secs,
        }
    }

    /// Check if cached price is still valid
    fn is_cache_valid(&self, timestamp: DateTime<Utc>) -> bool {
        let age = Utc::now().signed_duration_since(timestamp).num_seconds() as u64;
        age < self.cache_ttl_secs
    }

    /// Initialize default price feeds (USDC)
    pub async fn initialize_defaults(&self) -> Result<(), ApiError> {
        // Check if USDC feed already exists
        let existing = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM price_feeds WHERE asset_code = 'USDC')",
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to check existing price feeds: {}", e);
            ApiError::Internal(anyhow::anyhow!("Database error"))
        })?;

        if !existing {
            sqlx::query(
                r#"
                INSERT INTO price_feeds (asset_code, source, feed_id, is_active)
                VALUES ('USDC', 'custom', 'usdc-usd', true)
                "#,
            )
            .execute(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to initialize default price feeds: {}", e);
                ApiError::Internal(anyhow::anyhow!("Database error"))
            })?;

            info!("Initialized default USDC price feed");
        }

        Ok(())
    }
}

#[async_trait]
impl PriceFeedService for DefaultPriceFeedService {
    async fn get_price(&self, asset_code: &str) -> Result<AssetPrice, ApiError> {
        // Check cache first
        {
            let cache = self.price_cache.read().await;
            if let Some(cached_price) = cache.get(asset_code) {
                if self.is_cache_valid(cached_price.timestamp) {
                    return Ok(cached_price.clone());
                }
            }
        }

        // Fetch from database
        let price_record = sqlx::query_as::<_, (String, String)>(
            r#"
            SELECT price::text, price_timestamp::text
            FROM asset_price_history
            WHERE asset_code = $1
            ORDER BY price_timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(asset_code)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch price from database: {}", e);
            ApiError::Internal(anyhow::anyhow!("Database error"))
        })?
        .ok_or_else(|| {
            warn!("No price found for asset: {}", asset_code);
            ApiError::NotFound(format!("Price not found for asset: {}", asset_code))
        })?;

        let price = Decimal::from_str(&price_record.0).map_err(|e| {
            error!("Failed to parse price: {}", e);
            ApiError::Internal(anyhow::anyhow!("Invalid price format"))
        })?;

        let timestamp = chrono::DateTime::parse_from_rfc3339(&price_record.1)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| {
                error!("Failed to parse timestamp: {}", e);
                ApiError::Internal(anyhow::anyhow!("Invalid timestamp format"))
            })?;

        let asset_price = AssetPrice {
            asset_code: asset_code.to_string(),
            price,
            timestamp,
            source: "custom".to_string(),
        };

        // Update cache
        {
            let mut cache = self.price_cache.write().await;
            cache.insert(asset_code.to_string(), asset_price.clone());
        }

        Ok(asset_price)
    }

    async fn get_price_history(
        &self,
        asset_code: &str,
        limit: i64,
    ) -> Result<Vec<AssetPrice>, ApiError> {
        let records = sqlx::query_as::<_, (String, String, String)>(
            r#"
            SELECT price::text, price_timestamp::text, source
            FROM asset_price_history
            WHERE asset_code = $1
            ORDER BY price_timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(asset_code)
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch price history: {}", e);
            ApiError::Internal(anyhow::anyhow!("Database error"))
        })?;

        let mut prices = Vec::new();
        for (price_str, timestamp_str, source) in records {
            let price = Decimal::from_str(&price_str).map_err(|e| {
                error!("Failed to parse price: {}", e);
                ApiError::Internal(anyhow::anyhow!("Invalid price format"))
            })?;

            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| {
                    error!("Failed to parse timestamp: {}", e);
                    ApiError::Internal(anyhow::anyhow!("Invalid timestamp format"))
                })?;

            prices.push(AssetPrice {
                asset_code: asset_code.to_string(),
                price,
                timestamp,
                source,
            });
        }

        Ok(prices)
    }

    async fn register_feed(
        &self,
        asset_code: &str,
        source: PriceFeedSource,
        feed_id: &str,
    ) -> Result<PriceFeedConfig, ApiError> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO price_feeds (id, asset_code, source, feed_id, is_active)
            VALUES ($1, $2, $3, $4, true)
            ON CONFLICT (asset_code) DO UPDATE
            SET source = $3, feed_id = $4, is_active = true, updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(id)
        .bind(asset_code)
        .bind(source.as_str())
        .bind(feed_id)
        .execute(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to register price feed: {}", e);
            ApiError::Internal(anyhow::anyhow!("Database error"))
        })?;

        info!(
            "Registered price feed for {} from {}",
            asset_code,
            source.as_str()
        );

        Ok(PriceFeedConfig {
            id,
            asset_code: asset_code.to_string(),
            source: source.as_str().to_string(),
            feed_id: feed_id.to_string(),
            is_active: true,
            last_updated: None,
        })
    }

    async fn update_price(&self, asset_code: &str, price: Decimal) -> Result<AssetPrice, ApiError> {
        // Validate price feed exists
        let _feed = sqlx::query_scalar::<_, String>(
            "SELECT source FROM price_feeds WHERE asset_code = $1 AND is_active = true",
        )
        .bind(asset_code)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to check price feed: {}", e);
            ApiError::Internal(anyhow::anyhow!("Database error"))
        })?
        .ok_or_else(|| {
            ApiError::BadRequest(format!("Price feed not found for asset: {}", asset_code))
        })?;

        let now = Utc::now();

        // Insert price history
        sqlx::query(
            r#"
            INSERT INTO asset_price_history (asset_code, price, price_timestamp, source)
            VALUES ($1, $2, $3, 'custom')
            "#,
        )
        .bind(asset_code)
        .bind(price.to_string())
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to update price: {}", e);
            ApiError::Internal(anyhow::anyhow!("Database error"))
        })?;

        // Update last_updated in price_feeds
        sqlx::query("UPDATE price_feeds SET last_updated = $1 WHERE asset_code = $2")
            .bind(now)
            .bind(asset_code)
            .execute(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to update feed timestamp: {}", e);
                ApiError::Internal(anyhow::anyhow!("Database error"))
            })?;

        let asset_price = AssetPrice {
            asset_code: asset_code.to_string(),
            price,
            timestamp: now,
            source: "custom".to_string(),
        };

        // Update cache
        {
            let mut cache = self.price_cache.write().await;
            cache.insert(asset_code.to_string(), asset_price.clone());
        }

        info!("Updated price for {}: {}", asset_code, price);

        Ok(asset_price)
    }

    async fn calculate_valuation(
        &self,
        asset_code: &str,
        amount: Decimal,
    ) -> Result<CollateralValuation, ApiError> {
        let asset_price = self.get_price(asset_code).await?;

        let valuation_usd = amount * asset_price.price;
        let collateral_ratio = Decimal::from(100); // 100% for now, can be adjusted

        Ok(CollateralValuation {
            plan_id: Uuid::nil(), // Will be set by caller
            asset_code: asset_code.to_string(),
            amount,
            current_price: asset_price.price,
            valuation_usd,
            collateral_ratio,
            last_updated: asset_price.timestamp,
        })
    }

    async fn get_active_feeds(&self) -> Result<Vec<PriceFeedConfig>, ApiError> {
        let feeds = sqlx::query_as::<_, (Uuid, String, String, String, Option<String>)>(
            r#"
            SELECT id, asset_code, source, feed_id, last_updated::text
            FROM price_feeds
            WHERE is_active = true
            ORDER BY asset_code
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch active feeds: {}", e);
            ApiError::Internal(anyhow::anyhow!("Database error"))
        })?;

        let mut result = Vec::new();
        for (id, asset_code, source, feed_id, last_updated_str) in feeds {
            let last_updated = last_updated_str.and_then(|ts| {
                chrono::DateTime::parse_from_rfc3339(&ts)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            });

            result.push(PriceFeedConfig {
                id,
                asset_code,
                source,
                feed_id,
                is_active: true,
                last_updated,
            });
        }

        Ok(result)
    }
}

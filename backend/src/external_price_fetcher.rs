use crate::api_error::ApiError;
use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tracing::{error, info, warn};

/// External price source result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPrice {
    pub asset_code: String,
    pub price: Decimal,
    pub source: String,
    pub timestamp_seconds: i64,
}

/// Trait for external price providers
#[async_trait]
pub trait ExternalPriceProvider: Send + Sync {
    /// Fetch price for an asset
    async fn fetch_price(&self, asset_code: &str) -> Result<ExternalPrice, ApiError>;

    /// Get provider name
    fn name(&self) -> &'static str;

    /// Check if provider is available (optional health check)
    async fn is_available(&self) -> bool {
        true
    }
}

/// CoinGecko price provider
pub struct CoinGeckoProvider {
    client: Client,
    base_url: String,
}

impl Default for CoinGeckoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CoinGeckoProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.coingecko.com/api/v3".to_string(),
        }
    }

    /// Map asset codes to CoinGecko IDs
    fn get_coingecko_id(asset_code: &str) -> Option<&'static str> {
        match asset_code.to_uppercase().as_str() {
            "ETH" => Some("ethereum"),
            "BTC" => Some("bitcoin"),
            "USDC" => Some("usd-coin"),
            "USDT" => Some("tether"),
            "SOL" => Some("solana"),
            "XRP" => Some("ripple"),
            "ADA" => Some("cardano"),
            "MATIC" => Some("matic-network"),
            "DAI" => Some("dai"),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CoinGeckoResponse {
    #[serde(rename(deserialize = "usd"))]
    price: f64,
}

#[async_trait]
impl ExternalPriceProvider for CoinGeckoProvider {
    async fn fetch_price(&self, asset_code: &str) -> Result<ExternalPrice, ApiError> {
        let coin_id = Self::get_coingecko_id(asset_code).ok_or_else(|| {
            ApiError::BadRequest(format!("Asset {} not supported by CoinGecko", asset_code))
        })?;

        let url = format!(
            "{}/simple/price?ids={}&vs_currencies=usd",
            self.base_url, coin_id
        );

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                error!("CoinGecko request failed: {}", e);
                ApiError::Internal(anyhow::anyhow!("CoinGecko fetch failed"))
            })?;

        if !response.status().is_success() {
            return Err(ApiError::Internal(anyhow::anyhow!(
                "CoinGecko returned status: {}",
                response.status()
            )));
        }

        let data: HashMap<String, CoinGeckoResponse> = response.json().await.map_err(|e| {
            error!("Failed to parse CoinGecko response: {}", e);
            ApiError::Internal(anyhow::anyhow!("Invalid CoinGecko response"))
        })?;

        let coin_data = data.get(coin_id).ok_or_else(|| {
            ApiError::Internal(anyhow::anyhow!("Coin data not found in response"))
        })?;

        let price = Decimal::from_f64_retain(coin_data.price)
            .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("Invalid price value")))?;

        Ok(ExternalPrice {
            asset_code: asset_code.to_uppercase(),
            price,
            source: "coingecko".to_string(),
            timestamp_seconds: chrono::Utc::now().timestamp(),
        })
    }

    fn name(&self) -> &'static str {
        "CoinGecko"
    }
}

/// Binance price provider
pub struct BinanceProvider {
    client: Client,
    base_url: String,
}

impl Default for BinanceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl BinanceProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.binance.com/api/v3".to_string(),
        }
    }

    /// Map asset codes to Binance symbols
    fn get_binance_symbol(asset_code: &str) -> Option<&'static str> {
        match asset_code.to_uppercase().as_str() {
            "ETH" => Some("ETHUSDT"),
            "BTC" => Some("BTCUSDT"),
            "USDC" => Some("USDCUSDT"),
            "USDT" => Some("USDTUSD"), // USDT is 1:1 with USD
            "SOL" => Some("SOLUSDT"),
            "XRP" => Some("XRPUSDT"),
            "ADA" => Some("ADAUSDT"),
            "MATIC" => Some("MATICUSDT"),
            "DAI" => Some("DAIUSDT"),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct BinancePriceResponse {
    #[serde(rename(deserialize = "price"))]
    price: String,
}

#[async_trait]
impl ExternalPriceProvider for BinanceProvider {
    async fn fetch_price(&self, asset_code: &str) -> Result<ExternalPrice, ApiError> {
        let symbol = Self::get_binance_symbol(asset_code).ok_or_else(|| {
            ApiError::BadRequest(format!("Asset {} not supported by Binance", asset_code))
        })?;

        let url = format!("{}/ticker/price?symbol={}", self.base_url, symbol);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                error!("Binance request failed: {}", e);
                ApiError::Internal(anyhow::anyhow!("Binance fetch failed"))
            })?;

        if !response.status().is_success() {
            return Err(ApiError::Internal(anyhow::anyhow!(
                "Binance returned status: {}",
                response.status()
            )));
        }

        let data: BinancePriceResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Binance response: {}", e);
            ApiError::Internal(anyhow::anyhow!("Invalid Binance response"))
        })?;

        let price = Decimal::from_str(&data.price).map_err(|e| {
            error!("Failed to parse price as Decimal: {}", e);
            ApiError::Internal(anyhow::anyhow!("Invalid price format"))
        })?;

        Ok(ExternalPrice {
            asset_code: asset_code.to_uppercase(),
            price,
            source: "binance".to_string(),
            timestamp_seconds: chrono::Utc::now().timestamp(),
        })
    }

    fn name(&self) -> &'static str {
        "Binance"
    }
}

/// Redundant price fetcher with fallback logic
pub struct RedundantPriceFetcher {
    providers: Vec<Box<dyn ExternalPriceProvider>>,
}

impl Default for RedundantPriceFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl RedundantPriceFetcher {
    pub fn new() -> Self {
        Self {
            providers: vec![
                Box::new(BinanceProvider::new()),
                Box::new(CoinGeckoProvider::new()),
            ],
        }
    }

    /// Fetch price with fallback to next provider on failure
    pub async fn fetch_price(&self, asset_code: &str) -> Result<ExternalPrice, ApiError> {
        let mut last_error = None;

        for (idx, provider) in self.providers.iter().enumerate() {
            info!(
                "Attempting to fetch {} from {} ({}/{})",
                asset_code,
                provider.name(),
                idx + 1,
                self.providers.len()
            );

            match provider.fetch_price(asset_code).await {
                Ok(price) => {
                    info!(
                        "Successfully fetched {} price {} from {}",
                        asset_code,
                        price.price,
                        provider.name()
                    );
                    return Ok(price);
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch {} from {}: {}. Trying next provider...",
                        asset_code,
                        provider.name(),
                        e
                    );
                    last_error = Some(e);
                }
            }
        }

        error!("All providers failed to fetch price for {}", asset_code);
        Err(last_error
            .unwrap_or_else(|| ApiError::Internal(anyhow::anyhow!("No price providers available"))))
    }

    /// Add a custom provider (useful for testing or custom sources)
    pub fn add_provider(&mut self, provider: Box<dyn ExternalPriceProvider>) {
        self.providers.push(provider);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coingecko_id_mapping() {
        assert_eq!(CoinGeckoProvider::get_coingecko_id("ETH"), Some("ethereum"));
        assert_eq!(CoinGeckoProvider::get_coingecko_id("BTC"), Some("bitcoin"));
        assert_eq!(CoinGeckoProvider::get_coingecko_id("UNKNOWN"), None);
    }

    #[test]
    fn test_binance_symbol_mapping() {
        assert_eq!(BinanceProvider::get_binance_symbol("ETH"), Some("ETHUSDT"));
        assert_eq!(BinanceProvider::get_binance_symbol("BTC"), Some("BTCUSDT"));
        assert_eq!(BinanceProvider::get_binance_symbol("UNKNOWN"), None);
    }
}

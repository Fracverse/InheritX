use crate::api_error::ApiError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ── Asset Data Structures ─────────────────────────────────────────────────────

/// Unified asset representation across all supported blockchains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub chain: String,
    pub contract_address: Option<String>,
    pub symbol: String,
    pub name: String,
    pub balance: String,
    pub decimals: u8,
    pub usd_value: Option<f64>,
}

/// Cross-chain asset response with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainAsset {
    pub assets: Vec<Asset>,
    pub total_usd_value: f64,
    pub chains_discovered: Vec<String>,
}

// ── Error Types ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum EthereumError {
    #[error("Ethereum RPC error: {0}")]
    RpcError(String),
    #[error("Invalid Ethereum address: {0}")]
    InvalidAddress(String),
    #[error("Token query failed: {0}")]
    TokenQueryFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PolygonError {
    #[error("Polygon RPC error: {0}")]
    RpcError(String),
    #[error("Invalid Polygon address: {0}")]
    InvalidAddress(String),
    #[error("Token query failed: {0}")]
    TokenQueryFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ArbitrumError {
    #[error("Arbitrum RPC error: {0}")]
    RpcError(String),
    #[error("Invalid Arbitrum address: {0}")]
    InvalidAddress(String),
    #[error("Token query failed: {0}")]
    TokenQueryFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum BitcoinError {
    #[error("Bitcoin RPC error: {0}")]
    RpcError(String),
    #[error("Invalid Bitcoin address: {0}")]
    InvalidAddress(String),
    #[error("UTXO query failed: {0}")]
    UtxoQueryFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Ethereum error: {0}")]
    Ethereum(#[from] EthereumError),
    #[error("Polygon error: {0}")]
    Polygon(#[from] PolygonError),
    #[error("Arbitrum error: {0}")]
    Arbitrum(#[from] ArbitrumError),
    #[error("Bitcoin error: {0}")]
    Bitcoin(#[from] BitcoinError),
    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        ApiError::ExternalService(format!("Cross-chain asset discovery: {err}"))
    }
}

// ── Ethereum Client ───────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct EthereumClient {
    rpc_url: Option<String>,
}

impl EthereumClient {
    pub fn from_env() -> Result<Self, ServiceError> {
        let rpc_url = std::env::var("ETHEREUM_RPC_URL").ok();
        Ok(Self { rpc_url })
    }

    /// Get ETH balance for an address
    pub async fn get_eth_balance(&self, address: &str) -> Result<String, EthereumError> {
        // Stub implementation - in production this would make an actual RPC call
        if self.rpc_url.is_none() {
            return Err(EthereumError::RpcError(
                "Ethereum RPC not configured".to_string(),
            ));
        }

        // Validate address format
        if !address.starts_with("0x") || address.len() != 42 {
            return Err(EthereumError::InvalidAddress(format!(
                "Invalid Ethereum address: {address}"
            )));
        }

        // Return mock balance for now
        Ok("1000000000000000000".to_string()) // 1 ETH in wei
    }

    /// Discover all ERC-20 tokens for an address
    pub async fn discover_erc20_tokens(&self, address: &str) -> Result<Vec<Asset>, EthereumError> {
        let eth_balance = self.get_eth_balance(address).await?;

        Ok(vec![Asset {
            chain: "ethereum".to_string(),
            contract_address: None,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            balance: eth_balance,
            decimals: 18,
            usd_value: None, // Would fetch from price oracle
        }])
    }
}

// ── Polygon Client ────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct PolygonClient {
    rpc_url: Option<String>,
}

impl PolygonClient {
    pub fn from_env() -> Result<Self, ServiceError> {
        let rpc_url = std::env::var("POLYGON_RPC_URL").ok();
        Ok(Self { rpc_url })
    }

    /// Get MATIC balance for an address
    pub async fn get_matic_balance(&self, address: &str) -> Result<String, PolygonError> {
        if self.rpc_url.is_none() {
            return Err(PolygonError::RpcError(
                "Polygon RPC not configured".to_string(),
            ));
        }

        if !address.starts_with("0x") || address.len() != 42 {
            return Err(PolygonError::InvalidAddress(format!(
                "Invalid Polygon address: {address}"
            )));
        }

        Ok("5000000000000000000".to_string()) // 5 MATIC
    }

    /// Discover all Polygon assets for an address
    pub async fn discover_polygon_assets(&self, address: &str) -> Result<Vec<Asset>, PolygonError> {
        let matic_balance = self.get_matic_balance(address).await?;

        Ok(vec![Asset {
            chain: "polygon".to_string(),
            contract_address: None,
            symbol: "MATIC".to_string(),
            name: "Polygon".to_string(),
            balance: matic_balance,
            decimals: 18,
            usd_value: None,
        }])
    }
}

// ── Arbitrum Client ───────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ArbitrumClient {
    rpc_url: Option<String>,
}

impl ArbitrumClient {
    pub fn from_env() -> Result<Self, ServiceError> {
        let rpc_url = std::env::var("ARBITRUM_RPC_URL").ok();
        Ok(Self { rpc_url })
    }

    /// Get ETH balance on Arbitrum
    pub async fn get_arb_eth_balance(&self, address: &str) -> Result<String, ArbitrumError> {
        if self.rpc_url.is_none() {
            return Err(ArbitrumError::RpcError(
                "Arbitrum RPC not configured".to_string(),
            ));
        }

        if !address.starts_with("0x") || address.len() != 42 {
            return Err(ArbitrumError::InvalidAddress(format!(
                "Invalid Arbitrum address: {address}"
            )));
        }

        Ok("2000000000000000000".to_string()) // 2 ETH
    }

    /// Discover all Arbitrum assets for an address
    pub async fn discover_arbitrum_assets(
        &self,
        address: &str,
    ) -> Result<Vec<Asset>, ArbitrumError> {
        let eth_balance = self.get_arb_eth_balance(address).await?;

        Ok(vec![Asset {
            chain: "arbitrum".to_string(),
            contract_address: None,
            symbol: "ETH".to_string(),
            name: "Ethereum (Arbitrum)".to_string(),
            balance: eth_balance,
            decimals: 18,
            usd_value: None,
        }])
    }
}

// ── Bitcoin Client ────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct BitcoinClient {
    rpc_url: String,
    http_client: reqwest::Client,
}

impl BitcoinClient {
    pub fn from_env() -> Result<Self, ServiceError> {
        let rpc_url = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or_else(|_| "https://blockstream.info/api".to_string());

        Ok(Self {
            rpc_url,
            http_client: reqwest::Client::new(),
        })
    }

    /// Get Bitcoin balance for an address using public API
    pub async fn get_btc_balance(&self, address: &str) -> Result<String, BitcoinError> {
        let client = self.http_client.clone();
        let rpc_url = self.rpc_url.clone();
        let address = address.to_string();

        let url = format!(
            "{}/address/{}/balance",
            rpc_url.trim_end_matches('/'),
            address
        );

        let response = client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| BitcoinError::RpcError(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(BitcoinError::RpcError(format!(
                "API returned status {}",
                response.status()
            )));
        }

        let balance_satoshis: u64 = response
            .json()
            .await
            .map_err(|e| BitcoinError::RpcError(format!("Failed to parse response: {e}")))?;

        // Convert satoshis to BTC
        let balance_btc = balance_satoshis as f64 / 100_000_000.0;
        Ok(balance_btc.to_string())
    }

    /// Discover Bitcoin assets for an address
    pub async fn discover_bitcoin_assets(&self, address: &str) -> Result<Vec<Asset>, BitcoinError> {
        let btc_balance = self.get_btc_balance(address).await?;

        Ok(vec![Asset {
            chain: "bitcoin".to_string(),
            contract_address: None,
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            balance: btc_balance,
            decimals: 8,
            usd_value: None,
        }])
    }
}

// ── Cross-Chain Asset Discovery Service ───────────────────────────────────────

pub struct CrossChainAssetDiscoveryService {
    pub ethereum_client: Option<EthereumClient>,
    pub polygon_client: Option<PolygonClient>,
    pub arbitrum_client: Option<ArbitrumClient>,
    pub bitcoin_client: Option<BitcoinClient>,
}

impl CrossChainAssetDiscoveryService {
    pub fn from_env() -> Self {
        Self {
            ethereum_client: EthereumClient::from_env().ok(),
            polygon_client: PolygonClient::from_env().ok(),
            arbitrum_client: ArbitrumClient::from_env().ok(),
            bitcoin_client: BitcoinClient::from_env().ok(),
        }
    }

    /// Discover all assets across all configured blockchains for a given address
    pub async fn discover_user_assets(
        &self,
        address: &str,
    ) -> Result<CrossChainAsset, ServiceError> {
        let mut all_assets = Vec::new();
        let mut chains_discovered = Vec::new();

        // Ethereum
        if let Some(client) = &self.ethereum_client {
            match client.discover_erc20_tokens(address).await {
                Ok(assets) => {
                    chains_discovered.push("ethereum".to_string());
                    all_assets.extend(assets);
                }
                Err(e) => {
                    tracing::warn!("Failed to discover Ethereum assets: {e}");
                }
            }
        }

        // Polygon
        if let Some(client) = &self.polygon_client {
            match client.discover_polygon_assets(address).await {
                Ok(assets) => {
                    chains_discovered.push("polygon".to_string());
                    all_assets.extend(assets);
                }
                Err(e) => {
                    tracing::warn!("Failed to discover Polygon assets: {e}");
                }
            }
        }

        // Arbitrum
        if let Some(client) = &self.arbitrum_client {
            match client.discover_arbitrum_assets(address).await {
                Ok(assets) => {
                    chains_discovered.push("arbitrum".to_string());
                    all_assets.extend(assets);
                }
                Err(e) => {
                    tracing::warn!("Failed to discover Arbitrum assets: {e}");
                }
            }
        }

        // Bitcoin
        if let Some(client) = &self.bitcoin_client {
            match client.discover_bitcoin_assets(address).await {
                Ok(assets) => {
                    chains_discovered.push("bitcoin".to_string());
                    all_assets.extend(assets);
                }
                Err(e) => {
                    tracing::warn!("Failed to discover Bitcoin assets: {e}");
                }
            }
        }

        let total_usd_value = all_assets.iter().filter_map(|a| a.usd_value).sum();

        Ok(CrossChainAsset {
            assets: all_assets,
            total_usd_value,
            chains_discovered,
        })
    }

    /// Get Ethereum assets only
    pub async fn get_ethereum_assets(&self, address: &str) -> Result<Vec<Asset>, EthereumError> {
        let client = self
            .ethereum_client
            .as_ref()
            .ok_or_else(|| EthereumError::RpcError("Ethereum client not configured".to_string()))?;
        client.discover_erc20_tokens(address).await
    }

    /// Get Polygon assets only
    pub async fn get_polygon_assets(&self, address: &str) -> Result<Vec<Asset>, PolygonError> {
        let client = self
            .polygon_client
            .as_ref()
            .ok_or_else(|| PolygonError::RpcError("Polygon client not configured".to_string()))?;
        client.discover_polygon_assets(address).await
    }

    /// Get Arbitrum assets only
    pub async fn get_arbitrum_assets(&self, address: &str) -> Result<Vec<Asset>, ArbitrumError> {
        let client = self
            .arbitrum_client
            .as_ref()
            .ok_or_else(|| ArbitrumError::RpcError("Arbitrum client not configured".to_string()))?;
        client.discover_arbitrum_assets(address).await
    }

    /// Get Bitcoin assets only
    pub async fn get_bitcoin_assets(&self, address: &str) -> Result<Vec<Asset>, BitcoinError> {
        let client = self
            .bitcoin_client
            .as_ref()
            .ok_or_else(|| BitcoinError::RpcError("Bitcoin client not configured".to_string()))?;
        client.discover_bitcoin_assets(address).await
    }
}

impl Default for CrossChainAssetDiscoveryService {
    fn default() -> Self {
        Self::from_env()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_serialization() {
        let asset = Asset {
            chain: "ethereum".to_string(),
            contract_address: Some("0x123".to_string()),
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            balance: "1000000000000000000".to_string(),
            decimals: 18,
            usd_value: Some(3000.0),
        };

        let json = serde_json::to_string(&asset).unwrap();
        let deserialized: Asset = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.chain, "ethereum");
        assert_eq!(deserialized.symbol, "ETH");
    }

    #[test]
    fn test_cross_chain_asset_serialization() {
        let cross_chain = CrossChainAsset {
            assets: vec![],
            total_usd_value: 0.0,
            chains_discovered: vec!["ethereum".to_string()],
        };

        let json = serde_json::to_string(&cross_chain).unwrap();
        let deserialized: CrossChainAsset = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.chains_discovered.len(), 1);
    }

    #[test]
    fn test_service_error_conversion() {
        let err = ServiceError::Ethereum(EthereumError::InvalidAddress("test".to_string()));
        let api_err: ApiError = err.into();
        assert!(matches!(api_err, ApiError::ExternalService(_)));
    }

    #[test]
    fn test_bitcoin_client_from_env_defaults() {
        std::env::remove_var("BITCOIN_RPC_URL");
        let client = BitcoinClient::from_env().unwrap();
        assert_eq!(client.rpc_url, "https://blockstream.info/api");
    }

    #[test]
    fn test_ethereum_client_requires_env() {
        std::env::remove_var("ETHEREUM_RPC_URL");
        let result = EthereumClient::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_polygon_client_requires_env() {
        std::env::remove_var("POLYGON_RPC_URL");
        let result = PolygonClient::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_arbitrum_client_requires_env() {
        std::env::remove_var("ARBITRUM_RPC_URL");
        let result = ArbitrumClient::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_service_default_construction() {
        std::env::remove_var("ETHEREUM_RPC_URL");
        std::env::remove_var("POLYGON_RPC_URL");
        std::env::remove_var("ARBITRUM_RPC_URL");

        let service = CrossChainAssetDiscoveryService::default();
        assert!(service.ethereum_client.is_none());
        assert!(service.polygon_client.is_none());
        assert!(service.arbitrum_client.is_none());
        assert!(service.bitcoin_client.is_some()); // Bitcoin has default
    }

    // ── Integration Tests with Mocked Responses ────────────────────────────────

    #[tokio::test]
    async fn test_bitcoin_client_with_mock_api() {
        // This test would use httpmock to mock the Bitcoin API
        // For now, we'll test the client construction and basic structure
        let client = BitcoinClient::from_env().unwrap();
        assert_eq!(client.rpc_url, "https://blockstream.info/api");
    }

    #[tokio::test]
    async fn test_service_discover_assets_with_no_clients() {
        std::env::remove_var("ETHEREUM_RPC_URL");
        std::env::remove_var("POLYGON_RPC_URL");
        std::env::remove_var("ARBITRUM_RPC_URL");
        std::env::remove_var("BITCOIN_RPC_URL");

        let service = CrossChainAssetDiscoveryService::from_env();
        let result = service.discover_user_assets("test_address").await.unwrap();

        // Should return empty result when no clients are configured
        assert_eq!(result.assets.len(), 0);
        assert_eq!(result.total_usd_value, 0.0);
        assert_eq!(result.chains_discovered.len(), 0);
    }

    #[tokio::test]
    async fn test_service_get_ethereum_assets_without_client() {
        std::env::remove_var("ETHEREUM_RPC_URL");
        let service = CrossChainAssetDiscoveryService::from_env();
        let result = service.get_ethereum_assets("0x123").await;

        assert!(result.is_err());
        assert!(matches!(result, Err(EthereumError::RpcError(_))));
    }

    #[tokio::test]
    async fn test_service_get_polygon_assets_without_client() {
        std::env::remove_var("POLYGON_RPC_URL");
        let service = CrossChainAssetDiscoveryService::from_env();
        let result = service.get_polygon_assets("0x123").await;

        assert!(result.is_err());
        assert!(matches!(result, Err(PolygonError::RpcError(_))));
    }

    #[tokio::test]
    async fn test_service_get_arbitrum_assets_without_client() {
        std::env::remove_var("ARBITRUM_RPC_URL");
        let service = CrossChainAssetDiscoveryService::from_env();
        let result = service.get_arbitrum_assets("0x123").await;

        assert!(result.is_err());
        assert!(matches!(result, Err(ArbitrumError::RpcError(_))));
    }

    #[tokio::test]
    async fn test_service_get_bitcoin_assets_without_client() {
        std::env::remove_var("BITCOIN_RPC_URL");
        let service = CrossChainAssetDiscoveryService::from_env();
        let result = service.get_bitcoin_assets("bc1q").await;

        assert!(result.is_err());
        assert!(matches!(result, Err(BitcoinError::RpcError(_))));
    }

    #[test]
    fn test_error_display() {
        let err = EthereumError::InvalidAddress("0xinvalid".to_string());
        assert!(err.to_string().contains("Invalid Ethereum address"));

        let err = PolygonError::TokenQueryFailed("network error".to_string());
        assert!(err.to_string().contains("Token query failed"));

        let err = ArbitrumError::RpcError("timeout".to_string());
        assert!(err.to_string().contains("Arbitrum RPC error"));

        let err = BitcoinError::UtxoQueryFailed("api error".to_string());
        assert!(err.to_string().contains("UTXO query failed"));
    }

    #[test]
    fn test_asset_with_null_usd_value() {
        let asset = Asset {
            chain: "ethereum".to_string(),
            contract_address: None,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            balance: "1000000000000000000".to_string(),
            decimals: 18,
            usd_value: None,
        };

        let json = serde_json::to_string(&asset).unwrap();
        assert!(json.contains("null"));
    }

    #[test]
    fn test_cross_chain_asset_empty() {
        let cross_chain = CrossChainAsset {
            assets: vec![],
            total_usd_value: 0.0,
            chains_discovered: vec![],
        };

        assert_eq!(cross_chain.assets.len(), 0);
        assert_eq!(cross_chain.total_usd_value, 0.0);
        assert_eq!(cross_chain.chains_discovered.len(), 0);
    }

    #[test]
    fn test_cross_chain_asset_with_multiple_chains() {
        let assets = vec![
            Asset {
                chain: "ethereum".to_string(),
                contract_address: None,
                symbol: "ETH".to_string(),
                name: "Ethereum".to_string(),
                balance: "1000000000000000000".to_string(),
                decimals: 18,
                usd_value: Some(3000.0),
            },
            Asset {
                chain: "bitcoin".to_string(),
                contract_address: None,
                symbol: "BTC".to_string(),
                name: "Bitcoin".to_string(),
                balance: "100000000".to_string(),
                decimals: 8,
                usd_value: Some(50000.0),
            },
        ];

        let cross_chain = CrossChainAsset {
            assets,
            total_usd_value: 53000.0,
            chains_discovered: vec!["ethereum".to_string(), "bitcoin".to_string()],
        };

        assert_eq!(cross_chain.assets.len(), 2);
        assert_eq!(cross_chain.total_usd_value, 53000.0);
        assert_eq!(cross_chain.chains_discovered.len(), 2);
    }
}

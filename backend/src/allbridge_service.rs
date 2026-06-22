use serde::{Deserialize, Serialize};
use std::time::Duration;

// ── Error Types ───────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum AllbridgeError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
    #[error("Unsupported token: {0}")]
    UnsupportedToken(String),
    #[error("Insufficient liquidity: available {available}, required {required}")]
    InsufficientLiquidity { available: String, required: String },
    #[error("Invalid transfer request: {0}")]
    InvalidRequest(String),
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    #[error("Bridge transfer failed: {0}")]
    TransferFailed(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
}

// ── Chain Configuration ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub name: String,
    pub rpc_url: Option<String>,
    pub allbridge_chain_symbol: String,
}

impl ChainConfig {
    pub fn ethereum() -> Self {
        Self {
            chain_id: "1".to_string(),
            name: "Ethereum".to_string(),
            rpc_url: std::env::var("ETHEREUM_RPC_URL").ok(),
            allbridge_chain_symbol: "ETH".to_string(),
        }
    }

    pub fn polygon() -> Self {
        Self {
            chain_id: "137".to_string(),
            name: "Polygon".to_string(),
            rpc_url: std::env::var("POLYGON_RPC_URL").ok(),
            allbridge_chain_symbol: "POL".to_string(),
        }
    }

    pub fn arbitrum() -> Self {
        Self {
            chain_id: "42161".to_string(),
            name: "Arbitrum".to_string(),
            rpc_url: std::env::var("ARBITRUM_RPC_URL").ok(),
            allbridge_chain_symbol: "ARB".to_string(),
        }
    }

    pub fn stellar() -> Self {
        Self {
            chain_id: "stellar".to_string(),
            name: "Stellar".to_string(),
            rpc_url: std::env::var("STELLAR_RPC_URL").ok(),
            allbridge_chain_symbol: "STLR".to_string(),
        }
    }
}

// ── Data Structures ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransferRequest {
    pub source_chain: String,
    pub destination_chain: String,
    pub token_address: String,
    pub amount: String,
    pub recipient: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransfer {
    pub request: BridgeTransferRequest,
    pub sender: String,
    pub nonce: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeFeeEstimate {
    pub bridge_fee: String,
    pub gas_fee: String,
    pub total_cost: String,
    pub estimated_time: u32,
    pub source_chain: String,
    pub destination_chain: String,
    pub token_symbol: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BridgeStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
    Unknown,
}

impl std::fmt::Display for BridgeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeStatus::Pending => write!(f, "pending"),
            BridgeStatus::InProgress => write!(f, "in_progress"),
            BridgeStatus::Completed => write!(f, "completed"),
            BridgeStatus::Failed(reason) => write!(f, "failed: {reason}"),
            BridgeStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub name: String,
    pub address: String,
    pub chain: String,
    pub decimals: u8,
    pub is_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub pool_address: String,
    pub token_symbol: String,
    pub chain: String,
    pub liquidity: String,
    pub apr: Option<f64>,
}

// ── Allbridge Core Client ─────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AllbridgeCoreClient {
    http_client: reqwest::Client,
    api_base_url: String,
}

impl AllbridgeCoreClient {
    pub fn new(api_base_url: impl Into<String>) -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            api_base_url: api_base_url.into(),
        }
    }

    /// GET /tokens - Fetch all supported tokens from Allbridge Core API
    pub async fn fetch_tokens(&self) -> Result<Vec<serde_json::Value>, AllbridgeError> {
        let url = format!("{}/tokens", self.api_base_url.trim_end_matches('/'));
        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AllbridgeError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AllbridgeError::ApiError(format!(
                "Tokens endpoint returned status {}",
                response.status()
            )));
        }

        response
            .json::<Vec<serde_json::Value>>()
            .await
            .map_err(|e| AllbridgeError::ApiError(format!("Failed to parse tokens response: {e}")))
    }

    /// GET /pools - Fetch liquidity pool information
    pub async fn fetch_pools(&self) -> Result<Vec<serde_json::Value>, AllbridgeError> {
        let url = format!("{}/pools", self.api_base_url.trim_end_matches('/'));
        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AllbridgeError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AllbridgeError::ApiError(format!(
                "Pools endpoint returned status {}",
                response.status()
            )));
        }

        response
            .json::<Vec<serde_json::Value>>()
            .await
            .map_err(|e| AllbridgeError::ApiError(format!("Failed to parse pools response: {e}")))
    }

    /// POST /send - Initiate a bridge transfer
    pub async fn post_send(
        &self,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, AllbridgeError> {
        let url = format!("{}/send", self.api_base_url.trim_end_matches('/'));
        let response = self
            .http_client
            .post(&url)
            .json(payload)
            .send()
            .await
            .map_err(|e| AllbridgeError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AllbridgeError::TransferFailed(format!(
                "Send endpoint returned status {status}: {body}"
            )));
        }

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AllbridgeError::ApiError(format!("Failed to parse send response: {e}")))
    }

    /// GET /tx/{hash} - Check transaction status
    pub async fn get_tx_status(&self, tx_hash: &str) -> Result<serde_json::Value, AllbridgeError> {
        let url = format!(
            "{}/tx/{}",
            self.api_base_url.trim_end_matches('/'),
            tx_hash
        );
        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AllbridgeError::HttpError(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(AllbridgeError::TransactionNotFound(tx_hash.to_string()));
        }

        if !response.status().is_success() {
            return Err(AllbridgeError::ApiError(format!(
                "TX status endpoint returned status {}",
                response.status()
            )));
        }

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AllbridgeError::ApiError(format!("Failed to parse tx response: {e}")))
    }
}

// ── Allbridge Service ─────────────────────────────────────────────────────────

pub struct AllbridgeService {
    pub core_client: AllbridgeCoreClient,
    pub supported_chains: Vec<ChainConfig>,
    pub api_base_url: String,
}

impl AllbridgeService {
    pub fn new(api_base_url: impl Into<String>) -> Self {
        let api_base_url = api_base_url.into();
        Self {
            core_client: AllbridgeCoreClient::new(api_base_url.clone()),
            supported_chains: vec![
                ChainConfig::ethereum(),
                ChainConfig::polygon(),
                ChainConfig::arbitrum(),
                ChainConfig::stellar(),
            ],
            api_base_url,
        }
    }

    pub fn from_env() -> Result<Self, AllbridgeError> {
        let api_base_url = std::env::var("ALLBRIDGE_API_URL")
            .unwrap_or_else(|_| "https://core.api.allbridgeapp.com".to_string());
        Ok(Self::new(api_base_url))
    }

    fn validate_chain(&self, chain: &str) -> Result<&ChainConfig, AllbridgeError> {
        self.supported_chains
            .iter()
            .find(|c| c.chain_id == chain || c.allbridge_chain_symbol.to_lowercase() == chain.to_lowercase())
            .ok_or_else(|| AllbridgeError::UnsupportedChain(chain.to_string()))
    }

    fn is_supported_stablecoin(token_address: &str) -> bool {
        // USDC and USDT are the primary supported stablecoins
        let lower = token_address.to_lowercase();
        lower.contains("usdc") || lower.contains("usdt")
            || lower == "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" // USDC on Ethereum
            || lower == "0xdac17f958d2ee523a2206206994597c13d831ec7" // USDT on Ethereum
            || lower == "0x2791bca1f2de4661ed88a30c99a7a9449aa84174" // USDC on Polygon
            || lower == "0xc2132d05d31c914a87c6611c10748aeb04b58e8f" // USDT on Polygon
            || lower == "0xff970a61a04b1ca14834a43f5de4533ebddb5cc8" // USDC on Arbitrum
            || lower == "0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9" // USDT on Arbitrum
    }

    /// Estimate bridging fees for a transfer request
    pub async fn estimate_bridge_fee(
        &self,
        transfer: BridgeTransferRequest,
    ) -> Result<BridgeFeeEstimate, AllbridgeError> {
        // Validate chains
        let source = self.validate_chain(&transfer.source_chain)?;
        let _ = self.validate_chain(&transfer.destination_chain)?;

        // Validate token
        if !Self::is_supported_stablecoin(&transfer.token_address) {
            return Err(AllbridgeError::UnsupportedToken(
                transfer.token_address.clone(),
            ));
        }

        // Validate amount is non-zero numeric string
        let amount_val: f64 = transfer
            .amount
            .parse()
            .map_err(|_| AllbridgeError::InvalidRequest("Amount must be a valid number".to_string()))?;

        if amount_val <= 0.0 {
            return Err(AllbridgeError::InvalidRequest(
                "Amount must be greater than zero".to_string(),
            ));
        }

        // Determine bridge fee (0.3% of amount) and gas fee based on source chain
        let bridge_fee_rate = 0.003_f64;
        let bridge_fee = amount_val * bridge_fee_rate;

        let gas_fee = match source.allbridge_chain_symbol.as_str() {
            "ETH" => 5.0,
            "POL" => 0.5,
            "ARB" => 1.0,
            "STLR" => 0.01,
            _ => 2.0,
        };

        let total_cost = bridge_fee + gas_fee;

        // Estimated time in minutes based on route
        let estimated_time = match (
            source.allbridge_chain_symbol.as_str(),
            transfer.destination_chain.as_str(),
        ) {
            ("ETH", _) | (_, "ETH") => 15,
            ("STLR", _) | (_, "STLR") => 10,
            _ => 5,
        };

        let token_symbol = if transfer.token_address.to_lowercase().contains("usdc") {
            "USDC"
        } else {
            "USDT"
        };

        Ok(BridgeFeeEstimate {
            bridge_fee: format!("{bridge_fee:.6}"),
            gas_fee: format!("{gas_fee:.6}"),
            total_cost: format!("{total_cost:.6}"),
            estimated_time,
            source_chain: transfer.source_chain,
            destination_chain: transfer.destination_chain,
            token_symbol: token_symbol.to_string(),
            amount: transfer.amount,
        })
    }

    /// Execute a bridge transfer
    pub async fn execute_bridge_transfer(
        &self,
        transfer: BridgeTransfer,
    ) -> Result<String, AllbridgeError> {
        // Validate request fields
        let _ = self.validate_chain(&transfer.request.source_chain)?;
        let _ = self.validate_chain(&transfer.request.destination_chain)?;

        if !Self::is_supported_stablecoin(&transfer.request.token_address) {
            return Err(AllbridgeError::UnsupportedToken(
                transfer.request.token_address.clone(),
            ));
        }

        if transfer.sender.is_empty() {
            return Err(AllbridgeError::InvalidRequest(
                "Sender address is required".to_string(),
            ));
        }

        if transfer.request.recipient.is_empty() {
            return Err(AllbridgeError::InvalidRequest(
                "Recipient address is required".to_string(),
            ));
        }

        let amount_val: f64 = transfer
            .request
            .amount
            .parse()
            .map_err(|_| AllbridgeError::InvalidRequest("Amount must be a valid number".to_string()))?;

        if amount_val <= 0.0 {
            return Err(AllbridgeError::InvalidRequest(
                "Amount must be greater than zero".to_string(),
            ));
        }

        // Check liquidity before sending
        self.check_liquidity(
            &transfer.request.destination_chain,
            &transfer.request.token_address,
            &transfer.request.amount,
        )
        .await?;

        let payload = serde_json::json!({
            "sourceChain": transfer.request.source_chain,
            "destinationChain": transfer.request.destination_chain,
            "tokenAddress": transfer.request.token_address,
            "amount": transfer.request.amount,
            "recipient": transfer.request.recipient,
            "sender": transfer.sender,
            "nonce": transfer.nonce,
        });

        let response = self.core_client.post_send(&payload).await?;

        let tx_hash = response
            .get("txHash")
            .or_else(|| response.get("transactionHash"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AllbridgeError::TransferFailed("No transaction hash in response".to_string())
            })?;

        Ok(tx_hash.to_string())
    }

    /// Get the current status of a bridge transfer by transaction hash
    pub async fn get_transfer_status(
        &self,
        tx_hash: &str,
    ) -> Result<BridgeStatus, AllbridgeError> {
        if tx_hash.is_empty() {
            return Err(AllbridgeError::InvalidRequest(
                "Transaction hash is required".to_string(),
            ));
        }

        let response = self.core_client.get_tx_status(tx_hash).await?;

        let status_str = response
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let status = match status_str.to_lowercase().as_str() {
            "pending" | "initiated" => BridgeStatus::Pending,
            "in_progress" | "processing" | "confirming" => BridgeStatus::InProgress,
            "completed" | "success" | "confirmed" => BridgeStatus::Completed,
            "failed" | "error" | "reverted" => {
                let reason = response
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string();
                BridgeStatus::Failed(reason)
            }
            _ => BridgeStatus::Unknown,
        };

        Ok(status)
    }

    /// Get all supported tokens for a specific chain
    pub async fn get_supported_tokens(
        &self,
        chain: &str,
    ) -> Result<Vec<TokenInfo>, AllbridgeError> {
        let chain_config = self.validate_chain(chain)?;
        let chain_symbol = chain_config.allbridge_chain_symbol.clone();

        // Attempt to fetch from live API, fall back to built-in supported list
        match self.core_client.fetch_tokens().await {
            Ok(raw_tokens) => {
                let tokens = raw_tokens
                    .into_iter()
                    .filter_map(|t| {
                        let token_chain = t.get("chainSymbol")?.as_str()?.to_string();
                        if token_chain != chain_symbol {
                            return None;
                        }
                        Some(TokenInfo {
                            symbol: t.get("symbol")?.as_str()?.to_string(),
                            name: t.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            address: t.get("tokenAddress").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            chain: chain.to_string(),
                            decimals: t.get("decimals").and_then(|v| v.as_u64()).unwrap_or(6) as u8,
                            is_supported: true,
                        })
                    })
                    .collect();
                Ok(tokens)
            }
            Err(_) => Ok(self.get_default_tokens_for_chain(&chain_symbol, chain)),
        }
    }

    fn get_default_tokens_for_chain(&self, chain_symbol: &str, chain: &str) -> Vec<TokenInfo> {
        match chain_symbol {
            "ETH" => vec![
                TokenInfo {
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                    chain: chain.to_string(),
                    decimals: 6,
                    is_supported: true,
                },
                TokenInfo {
                    symbol: "USDT".to_string(),
                    name: "Tether USD".to_string(),
                    address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
                    chain: chain.to_string(),
                    decimals: 6,
                    is_supported: true,
                },
            ],
            "POL" => vec![
                TokenInfo {
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    address: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(),
                    chain: chain.to_string(),
                    decimals: 6,
                    is_supported: true,
                },
                TokenInfo {
                    symbol: "USDT".to_string(),
                    name: "Tether USD".to_string(),
                    address: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F".to_string(),
                    chain: chain.to_string(),
                    decimals: 6,
                    is_supported: true,
                },
            ],
            "ARB" => vec![
                TokenInfo {
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    address: "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8".to_string(),
                    chain: chain.to_string(),
                    decimals: 6,
                    is_supported: true,
                },
                TokenInfo {
                    symbol: "USDT".to_string(),
                    name: "Tether USD".to_string(),
                    address: "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9".to_string(),
                    chain: chain.to_string(),
                    decimals: 6,
                    is_supported: true,
                },
            ],
            "STLR" => vec![TokenInfo {
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                address: "USDC".to_string(),
                chain: chain.to_string(),
                decimals: 7,
                is_supported: true,
            }],
            _ => vec![],
        }
    }

    /// Verify that the destination chain pool has sufficient liquidity
    pub async fn check_liquidity(
        &self,
        destination_chain: &str,
        token_address: &str,
        required_amount: &str,
    ) -> Result<bool, AllbridgeError> {
        let chain_config = self.validate_chain(destination_chain)?;

        let required: f64 = required_amount
            .parse()
            .map_err(|_| AllbridgeError::InvalidRequest("Invalid amount format".to_string()))?;

        match self.core_client.fetch_pools().await {
            Ok(pools) => {
                let chain_symbol = &chain_config.allbridge_chain_symbol;
                let token_symbol = if token_address.to_lowercase().contains("usdc") {
                    "USDC"
                } else {
                    "USDT"
                };

                let available_liquidity = pools
                    .iter()
                    .find(|p| {
                        p.get("chainSymbol").and_then(|v| v.as_str()) == Some(chain_symbol)
                            && p.get("tokenSymbol").and_then(|v| v.as_str()) == Some(token_symbol)
                    })
                    .and_then(|p| p.get("liquidity").and_then(|v| v.as_str()))
                    .and_then(|l| l.parse::<f64>().ok())
                    .unwrap_or(f64::MAX); // if pool not found, assume sufficient

                if available_liquidity < required {
                    return Err(AllbridgeError::InsufficientLiquidity {
                        available: format!("{available_liquidity:.6}"),
                        required: required_amount.to_string(),
                    });
                }

                Ok(true)
            }
            // If pools API is unavailable, allow transfer to proceed
            Err(_) => Ok(true),
        }
    }

    /// Return all supported bridge routes
    pub fn get_supported_routes(&self) -> Vec<(String, String, Vec<String>)> {
        vec![
            ("ethereum".to_string(), "polygon".to_string(), vec!["USDC".to_string(), "USDT".to_string()]),
            ("polygon".to_string(), "ethereum".to_string(), vec!["USDC".to_string(), "USDT".to_string()]),
            ("ethereum".to_string(), "arbitrum".to_string(), vec!["USDC".to_string(), "USDT".to_string()]),
            ("arbitrum".to_string(), "ethereum".to_string(), vec!["USDC".to_string(), "USDT".to_string()]),
            ("polygon".to_string(), "arbitrum".to_string(), vec!["USDC".to_string(), "USDT".to_string()]),
            ("arbitrum".to_string(), "polygon".to_string(), vec!["USDC".to_string(), "USDT".to_string()]),
            ("stellar".to_string(), "ethereum".to_string(), vec!["USDC".to_string()]),
            ("ethereum".to_string(), "stellar".to_string(), vec!["USDC".to_string()]),
        ]
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service() -> AllbridgeService {
        AllbridgeService::new("https://core.api.allbridgeapp.com")
    }

    // ── ChainConfig tests ─────────────────────────────────────────────────────

    #[test]
    fn test_chain_config_ethereum() {
        let config = ChainConfig::ethereum();
        assert_eq!(config.chain_id, "1");
        assert_eq!(config.name, "Ethereum");
        assert_eq!(config.allbridge_chain_symbol, "ETH");
    }

    #[test]
    fn test_chain_config_polygon() {
        let config = ChainConfig::polygon();
        assert_eq!(config.chain_id, "137");
        assert_eq!(config.allbridge_chain_symbol, "POL");
    }

    #[test]
    fn test_chain_config_arbitrum() {
        let config = ChainConfig::arbitrum();
        assert_eq!(config.chain_id, "42161");
        assert_eq!(config.allbridge_chain_symbol, "ARB");
    }

    #[test]
    fn test_chain_config_stellar() {
        let config = ChainConfig::stellar();
        assert_eq!(config.chain_id, "stellar");
        assert_eq!(config.allbridge_chain_symbol, "STLR");
    }

    // ── validate_chain tests ──────────────────────────────────────────────────

    #[test]
    fn test_validate_chain_by_id() {
        let svc = make_service();
        assert!(svc.validate_chain("1").is_ok());
        assert!(svc.validate_chain("137").is_ok());
        assert!(svc.validate_chain("42161").is_ok());
        assert!(svc.validate_chain("stellar").is_ok());
    }

    #[test]
    fn test_validate_chain_by_symbol() {
        let svc = make_service();
        assert!(svc.validate_chain("ETH").is_ok());
        assert!(svc.validate_chain("eth").is_ok());
        assert!(svc.validate_chain("POL").is_ok());
        assert!(svc.validate_chain("ARB").is_ok());
        assert!(svc.validate_chain("STLR").is_ok());
    }

    #[test]
    fn test_validate_chain_unsupported() {
        let svc = make_service();
        let err = svc.validate_chain("bsc").unwrap_err();
        assert!(matches!(err, AllbridgeError::UnsupportedChain(_)));
    }

    // ── is_supported_stablecoin tests ─────────────────────────────────────────

    #[test]
    fn test_supported_stablecoin_usdc_keyword() {
        assert!(AllbridgeService::is_supported_stablecoin("usdc"));
        assert!(AllbridgeService::is_supported_stablecoin("USDC"));
    }

    #[test]
    fn test_supported_stablecoin_usdt_keyword() {
        assert!(AllbridgeService::is_supported_stablecoin("usdt"));
        assert!(AllbridgeService::is_supported_stablecoin("USDT"));
    }

    #[test]
    fn test_supported_stablecoin_eth_usdc_address() {
        assert!(AllbridgeService::is_supported_stablecoin(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        ));
    }

    #[test]
    fn test_supported_stablecoin_polygon_usdc_address() {
        assert!(AllbridgeService::is_supported_stablecoin(
            "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"
        ));
    }

    #[test]
    fn test_unsupported_token() {
        assert!(!AllbridgeService::is_supported_stablecoin(
            "0xSomeRandomToken"
        ));
        assert!(!AllbridgeService::is_supported_stablecoin("DAI"));
    }

    // ── fee estimation tests ──────────────────────────────────────────────────

    #[tokio::test]
    async fn test_estimate_bridge_fee_eth_to_polygon_usdc() {
        let svc = make_service();
        let req = BridgeTransferRequest {
            source_chain: "ETH".to_string(),
            destination_chain: "POL".to_string(),
            token_address: "usdc".to_string(),
            amount: "1000.0".to_string(),
            recipient: "0xRecipient".to_string(),
        };

        let estimate = svc.estimate_bridge_fee(req).await.unwrap();
        assert_eq!(estimate.token_symbol, "USDC");
        assert_eq!(estimate.source_chain, "ETH");
        assert_eq!(estimate.destination_chain, "POL");
        // bridge fee = 1000 * 0.003 = 3.0, gas fee = 5.0 (ETH source), total = 8.0
        let total: f64 = estimate.total_cost.parse().unwrap();
        assert!((total - 8.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_estimate_bridge_fee_polygon_to_arbitrum_usdt() {
        let svc = make_service();
        let req = BridgeTransferRequest {
            source_chain: "POL".to_string(),
            destination_chain: "ARB".to_string(),
            token_address: "usdt".to_string(),
            amount: "500.0".to_string(),
            recipient: "0xRecipient".to_string(),
        };

        let estimate = svc.estimate_bridge_fee(req).await.unwrap();
        assert_eq!(estimate.token_symbol, "USDT");
        // bridge fee = 500 * 0.003 = 1.5, gas fee = 0.5 (POL source), total = 2.0
        let total: f64 = estimate.total_cost.parse().unwrap();
        assert!((total - 2.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_estimate_bridge_fee_stellar_to_eth_estimated_time() {
        let svc = make_service();
        let req = BridgeTransferRequest {
            source_chain: "STLR".to_string(),
            destination_chain: "ETH".to_string(),
            token_address: "usdc".to_string(),
            amount: "100.0".to_string(),
            recipient: "0xRecipient".to_string(),
        };

        let estimate = svc.estimate_bridge_fee(req).await.unwrap();
        // Stellar <-> ETH route uses 10 minutes for Stellar source
        assert_eq!(estimate.estimated_time, 10);
    }

    #[tokio::test]
    async fn test_estimate_bridge_fee_unsupported_chain() {
        let svc = make_service();
        let req = BridgeTransferRequest {
            source_chain: "BSC".to_string(),
            destination_chain: "ETH".to_string(),
            token_address: "usdc".to_string(),
            amount: "100.0".to_string(),
            recipient: "0xRecipient".to_string(),
        };

        let err = svc.estimate_bridge_fee(req).await.unwrap_err();
        assert!(matches!(err, AllbridgeError::UnsupportedChain(_)));
    }

    #[tokio::test]
    async fn test_estimate_bridge_fee_unsupported_token() {
        let svc = make_service();
        let req = BridgeTransferRequest {
            source_chain: "ETH".to_string(),
            destination_chain: "POL".to_string(),
            token_address: "DAI".to_string(),
            amount: "100.0".to_string(),
            recipient: "0xRecipient".to_string(),
        };

        let err = svc.estimate_bridge_fee(req).await.unwrap_err();
        assert!(matches!(err, AllbridgeError::UnsupportedToken(_)));
    }

    #[tokio::test]
    async fn test_estimate_bridge_fee_zero_amount() {
        let svc = make_service();
        let req = BridgeTransferRequest {
            source_chain: "ETH".to_string(),
            destination_chain: "POL".to_string(),
            token_address: "usdc".to_string(),
            amount: "0".to_string(),
            recipient: "0xRecipient".to_string(),
        };

        let err = svc.estimate_bridge_fee(req).await.unwrap_err();
        assert!(matches!(err, AllbridgeError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_estimate_bridge_fee_invalid_amount() {
        let svc = make_service();
        let req = BridgeTransferRequest {
            source_chain: "ETH".to_string(),
            destination_chain: "POL".to_string(),
            token_address: "usdc".to_string(),
            amount: "not_a_number".to_string(),
            recipient: "0xRecipient".to_string(),
        };

        let err = svc.estimate_bridge_fee(req).await.unwrap_err();
        assert!(matches!(err, AllbridgeError::InvalidRequest(_)));
    }

    // ── execute_bridge_transfer validation tests ──────────────────────────────

    #[tokio::test]
    async fn test_execute_bridge_transfer_empty_sender() {
        let svc = make_service();
        let transfer = BridgeTransfer {
            request: BridgeTransferRequest {
                source_chain: "ETH".to_string(),
                destination_chain: "POL".to_string(),
                token_address: "usdc".to_string(),
                amount: "100.0".to_string(),
                recipient: "0xRecipient".to_string(),
            },
            sender: "".to_string(),
            nonce: None,
        };

        let err = svc.execute_bridge_transfer(transfer).await.unwrap_err();
        assert!(matches!(err, AllbridgeError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_execute_bridge_transfer_empty_recipient() {
        let svc = make_service();
        let transfer = BridgeTransfer {
            request: BridgeTransferRequest {
                source_chain: "ETH".to_string(),
                destination_chain: "POL".to_string(),
                token_address: "usdc".to_string(),
                amount: "100.0".to_string(),
                recipient: "".to_string(),
            },
            sender: "0xSender".to_string(),
            nonce: None,
        };

        let err = svc.execute_bridge_transfer(transfer).await.unwrap_err();
        assert!(matches!(err, AllbridgeError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_execute_bridge_transfer_unsupported_chain() {
        let svc = make_service();
        let transfer = BridgeTransfer {
            request: BridgeTransferRequest {
                source_chain: "BSC".to_string(),
                destination_chain: "ETH".to_string(),
                token_address: "usdc".to_string(),
                amount: "100.0".to_string(),
                recipient: "0xRecipient".to_string(),
            },
            sender: "0xSender".to_string(),
            nonce: None,
        };

        let err = svc.execute_bridge_transfer(transfer).await.unwrap_err();
        assert!(matches!(err, AllbridgeError::UnsupportedChain(_)));
    }

    #[tokio::test]
    async fn test_execute_bridge_transfer_unsupported_token() {
        let svc = make_service();
        let transfer = BridgeTransfer {
            request: BridgeTransferRequest {
                source_chain: "ETH".to_string(),
                destination_chain: "POL".to_string(),
                token_address: "DAI".to_string(),
                amount: "100.0".to_string(),
                recipient: "0xRecipient".to_string(),
            },
            sender: "0xSender".to_string(),
            nonce: None,
        };

        let err = svc.execute_bridge_transfer(transfer).await.unwrap_err();
        assert!(matches!(err, AllbridgeError::UnsupportedToken(_)));
    }

    // ── get_transfer_status tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_transfer_status_empty_hash() {
        let svc = make_service();
        let err = svc.get_transfer_status("").await.unwrap_err();
        assert!(matches!(err, AllbridgeError::InvalidRequest(_)));
    }

    // ── get_supported_tokens tests ────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_supported_tokens_ethereum_fallback() {
        let svc = make_service();
        // API will fail in test env, should return default list
        let tokens = svc.get_supported_tokens("ETH").await.unwrap();
        assert_eq!(tokens.len(), 2);
        let symbols: Vec<&str> = tokens.iter().map(|t| t.symbol.as_str()).collect();
        assert!(symbols.contains(&"USDC"));
        assert!(symbols.contains(&"USDT"));
    }

    #[tokio::test]
    async fn test_get_supported_tokens_polygon_fallback() {
        let svc = make_service();
        let tokens = svc.get_supported_tokens("POL").await.unwrap();
        assert_eq!(tokens.len(), 2);
    }

    #[tokio::test]
    async fn test_get_supported_tokens_arbitrum_fallback() {
        let svc = make_service();
        let tokens = svc.get_supported_tokens("ARB").await.unwrap();
        assert_eq!(tokens.len(), 2);
    }

    #[tokio::test]
    async fn test_get_supported_tokens_stellar_fallback() {
        let svc = make_service();
        // Stellar only supports USDC
        let tokens = svc.get_supported_tokens("STLR").await.unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].symbol, "USDC");
    }

    #[tokio::test]
    async fn test_get_supported_tokens_unsupported_chain() {
        let svc = make_service();
        let err = svc.get_supported_tokens("BSC").await.unwrap_err();
        assert!(matches!(err, AllbridgeError::UnsupportedChain(_)));
    }

    // ── get_supported_routes tests ────────────────────────────────────────────

    #[test]
    fn test_get_supported_routes_count() {
        let svc = make_service();
        let routes = svc.get_supported_routes();
        // 8 routes defined: ETH<->POL, ETH<->ARB, POL<->ARB, ETH<->STLR
        assert_eq!(routes.len(), 8);
    }

    #[test]
    fn test_get_supported_routes_includes_stellar() {
        let svc = make_service();
        let routes = svc.get_supported_routes();
        let stellar_routes: Vec<_> = routes
            .iter()
            .filter(|(src, dst, _)| src == "stellar" || dst == "stellar")
            .collect();
        assert_eq!(stellar_routes.len(), 2);
        // Stellar only supports USDC
        for (_, _, tokens) in &stellar_routes {
            assert!(tokens.contains(&"USDC".to_string()));
            assert!(!tokens.contains(&"USDT".to_string()));
        }
    }

    #[test]
    fn test_bridge_status_display() {
        assert_eq!(BridgeStatus::Pending.to_string(), "pending");
        assert_eq!(BridgeStatus::InProgress.to_string(), "in_progress");
        assert_eq!(BridgeStatus::Completed.to_string(), "completed");
        assert_eq!(
            BridgeStatus::Failed("timeout".to_string()).to_string(),
            "failed: timeout"
        );
        assert_eq!(BridgeStatus::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_bridge_fee_estimate_fields() {
        // Ensure BridgeFeeEstimate serializes cleanly
        let estimate = BridgeFeeEstimate {
            bridge_fee: "3.000000".to_string(),
            gas_fee: "5.000000".to_string(),
            total_cost: "8.000000".to_string(),
            estimated_time: 15,
            source_chain: "ETH".to_string(),
            destination_chain: "POL".to_string(),
            token_symbol: "USDC".to_string(),
            amount: "1000.0".to_string(),
        };
        let json = serde_json::to_string(&estimate).unwrap();
        assert!(json.contains("bridge_fee"));
        assert!(json.contains("total_cost"));
        assert!(json.contains("estimated_time"));
    }
}

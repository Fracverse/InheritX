use crate::api_error::ApiError;
use axum::async_trait;
use rust_decimal::Decimal;

#[async_trait]
pub trait OnChainYieldService: Send + Sync {
    async fn get_total_on_chain_yield_amount(&self, asset_code: &str) -> Result<Decimal, ApiError>;
    async fn get_total_on_chain_balance(&self, asset_code: &str) -> Result<Decimal, ApiError>;
}

#[derive(Default)]
pub struct DefaultOnChainYieldService;

impl DefaultOnChainYieldService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl OnChainYieldService for DefaultOnChainYieldService {
    async fn get_total_on_chain_yield_amount(&self, asset_code: &str) -> Result<Decimal, ApiError> {
        use rust_decimal_macros::dec;
        // Mock implementation for on-chain total yield amount
        // In a real application, this would query a smart contract or indexer
        match asset_code.to_uppercase().as_str() {
            "USDC" => Ok(dec!(25.50)), // Return a mock amount
            "XLM" => Ok(dec!(100.0)),
            _ => Ok(dec!(0.0)),
        }
    }

    async fn get_total_on_chain_balance(&self, asset_code: &str) -> Result<Decimal, ApiError> {
        use rust_decimal_macros::dec;
        // Mock implementation for on-chain total balance
        match asset_code.to_uppercase().as_str() {
            "USDC" => Ok(dec!(100025.50)),
            "XLM" => Ok(dec!(500100.0)),
            _ => Ok(dec!(0.0)),
        }
    }
}

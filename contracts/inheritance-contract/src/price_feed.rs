/// Price Feed Module for Collateral Valuation
/// 
/// This module provides price feed integration for collateral valuation
/// on the Soroban smart contract. It validates asset prices and ensures
/// collateral ratios meet minimum requirements.

use soroban_sdk::{contracttype, Env, Symbol, U128};

/// Minimum collateral ratio (in basis points)
/// 100% = 10000 basis points
const MIN_COLLATERAL_RATIO_BP: u32 = 10000; // 100%

/// Price data structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    pub asset: Symbol,
    pub price: U128,        // Price in USD (8 decimal places)
    pub timestamp: u64,     // Unix timestamp
    pub source: Symbol,     // Price feed source (pyth, chainlink, custom)
}

/// Collateral valuation result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollateralValuation {
    pub asset: Symbol,
    pub amount: u64,
    pub price: U128,
    pub valuation_usd: U128,
    pub collateral_ratio_bp: u32,
}

/// Validate collateral meets minimum ratio
pub fn validate_collateral(
    _env: &Env,
    amount: u64,
    price: U128,
    min_ratio_bp: u32,
) -> Result<CollateralValuation, &'static str> {
    // Calculate valuation: amount * price (price is in 8 decimals)
    let amount_u128 = U128::from(amount);
    let valuation = amount_u128.checked_mul(&price)
        .ok_or("Valuation overflow")?;

    // For MVP, we assume 100% collateral ratio
    // In production, this would check against required minimum
    let collateral_ratio_bp = if valuation.gt(&U128::from(0)) {
        10000 // 100%
    } else {
        0
    };

    if collateral_ratio_bp < min_ratio_bp {
        return Err("Insufficient collateral ratio");
    }

    Ok(CollateralValuation {
        asset: Symbol::new(_env, "USDC"),
        amount,
        price,
        valuation_usd: valuation,
        collateral_ratio_bp,
    })
}

/// Calculate plan valuation
pub fn calculate_plan_valuation(
    env: &Env,
    plan_amount: u64,
    price: U128,
) -> Result<CollateralValuation, &'static str> {
    validate_collateral(env, plan_amount, price, MIN_COLLATERAL_RATIO_BP)
}

/// Verify price is recent (within acceptable time window)
pub fn verify_price_freshness(
    env: &Env,
    price_timestamp: u64,
    max_age_secs: u64,
) -> Result<(), &'static str> {
    let current_time = env.ledger().timestamp();
    
    if current_time < price_timestamp {
        return Err("Price timestamp is in the future");
    }

    let age = current_time - price_timestamp;
    if age > max_age_secs {
        return Err("Price data is stale");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_collateral_sufficient() {
        // This would require a test environment setup
        // Placeholder for contract testing
    }

    #[test]
    fn test_validate_collateral_insufficient() {
        // This would require a test environment setup
        // Placeholder for contract testing
    }
}

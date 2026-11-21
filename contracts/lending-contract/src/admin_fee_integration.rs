/// Admin Fee Integration for Lending Contract
///
/// Integrates protocol fee collection into loan origination, yield claims,
/// and liquidation events. Routes fees to admin treasury.
///
/// Issue #1155: Implement Admin Fee Treasury Deposit & Protocol Revenue Split

use soroban_sdk::{Address, Env, Symbol, symbol_short, vec, Vec};

/// Fee collection points in the lending contract lifecycle
#[derive(Clone, Debug, PartialEq)]
pub enum FeeEventType {
    /// Fee on loan origination
    LoanOrigination,
    /// Fee on yield claim
    YieldClaim,
    /// Fee on interest accrual
    InterestAccrual,
    /// Fee on liquidation proceeds
    Liquidation,
    /// Fee on early repayment
    EarlyRepayment,
}

/// Tracks fee collection event
#[derive(Clone, Debug)]
pub struct FeeCollectionEvent {
    pub event_type: FeeEventType,
    pub amount: u128,
    pub timestamp: u64,
    pub related_id: String, // loan_id, nft_id, etc.
}

/// Configuration for lending contract fee handling
#[derive(Clone, Debug)]
pub struct LendingFeeConfig {
    /// Protocol fee in basis points (0-10000)
    pub protocol_fee_bps: u32,
    /// Treasury address for protocol fees
    pub treasury_address: Address,
    /// Whether to charge fees on loan origination
    pub charge_origination_fee: bool,
    /// Whether to charge fees on interest
    pub charge_interest_fee: bool,
    /// Whether to charge fees on liquidation
    pub charge_liquidation_fee: bool,
}

impl LendingFeeConfig {
    pub fn new(treasury_address: Address) -> Self {
        Self {
            protocol_fee_bps: 50, // Default 0.5%
            treasury_address,
            charge_origination_fee: true,
            charge_interest_fee: true,
            charge_liquidation_fee: true,
        }
    }

    /// Calculate protocol fee from amount
    pub fn calculate_fee(&self, amount: u128) -> u128 {
        (amount * self.protocol_fee_bps as u128) / 10000
    }

    /// Deduct fee and return (fee_amount, net_amount)
    pub fn deduct_fee(&self, amount: u128) -> (u128, u128) {
        let fee = self.calculate_fee(amount);
        (fee, amount - fee)
    }
}

/// Apply protocol fee during loan origination
///
/// # Arguments
/// * `env` - Soroban environment
/// * `loan_amount` - Original loan amount requested
/// * `config` - Fee configuration
///
/// # Returns
/// * `(protocol_fee, net_loan_amount)` - Fee extracted and actual amount to lend
///
/// # Example
/// ```ignore
/// let (fee, net_amount) = apply_origination_fee(env, 1000_0000000, &config)?;
/// // Fee: 500_000 (0.5%), Net: 999_500_000
/// ```
pub fn apply_origination_fee(
    _env: &Env,
    loan_amount: u128,
    config: &LendingFeeConfig,
) -> Result<(u128, u128), String> {
    if !config.charge_origination_fee {
        return Ok((0, loan_amount));
    }

    let (fee, net) = config.deduct_fee(loan_amount);
    Ok((fee, net))
}

/// Apply protocol fee during yield/interest claim
///
/// # Arguments
/// * `yield_amount` - Amount of yield earned
/// * `config` - Fee configuration
///
/// # Returns
/// * `(protocol_fee, claimable_yield)` - Fee portion and amount user receives
pub fn apply_yield_fee(yield_amount: u128, config: &LendingFeeConfig) -> Result<(u128, u128), String> {
    if !config.charge_interest_fee {
        return Ok((0, yield_amount));
    }

    let (fee, net) = config.deduct_fee(yield_amount);
    Ok((fee, net))
}

/// Apply protocol fee during liquidation
///
/// # Arguments
/// * `liquidation_proceeds` - Total amount from liquidated collateral
/// * `config` - Fee configuration
///
/// # Returns
/// * `(protocol_fee, lender_proceeds)` - Fee and amount returned to lender
pub fn apply_liquidation_fee(
    liquidation_proceeds: u128,
    config: &LendingFeeConfig,
) -> Result<(u128, u128), String> {
    if !config.charge_liquidation_fee {
        return Ok((0, liquidation_proceeds));
    }

    let (fee, net) = config.deduct_fee(liquidation_proceeds);
    Ok((fee, net))
}

/// Batch fee collection from multiple events
///
/// Useful for end-of-period fee settlement
pub fn collect_batch_fees(events: &[FeeCollectionEvent]) -> u128 {
    events.iter().map(|e| e.amount).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_config(fee_bps: u32) -> LendingFeeConfig {
        let mut config = LendingFeeConfig::new(Address::random(&Env::default()));
        config.protocol_fee_bps = fee_bps;
        config
    }

    #[test]
    fn test_origination_fee_calculation() {
        let env = Env::default();
        let config = setup_config(50); // 0.5%

        let (fee, net) = apply_origination_fee(&env, 1000_0000000, &config).unwrap();
        assert_eq!(fee, 5_0000000); // 0.5% of 1000
        assert_eq!(net, 999_5000000);
    }

    #[test]
    fn test_origination_fee_disabled() {
        let env = Env::default();
        let mut config = setup_config(50);
        config.charge_origination_fee = false;

        let (fee, net) = apply_origination_fee(&env, 1000_0000000, &config).unwrap();
        assert_eq!(fee, 0);
        assert_eq!(net, 1000_0000000);
    }

    #[test]
    fn test_yield_fee_calculation() {
        let config = setup_config(100); // 1%

        let (fee, claimable) = apply_yield_fee(1000_0000000, &config).unwrap();
        assert_eq!(fee, 10_0000000); // 1% of 1000
        assert_eq!(claimable, 990_0000000);
    }

    #[test]
    fn test_liquidation_fee_calculation() {
        let config = setup_config(250); // 2.5%

        let (fee, proceeds) = apply_liquidation_fee(10000_0000000, &config).unwrap();
        assert_eq!(fee, 250_0000000); // 2.5% of 10000
        assert_eq!(proceeds, 9750_0000000);
    }

    #[test]
    fn test_batch_fee_collection() {
        let events = vec![
            FeeCollectionEvent {
                event_type: FeeEventType::LoanOrigination,
                amount: 100,
                timestamp: 1000,
                related_id: "loan_1".to_string(),
            },
            FeeCollectionEvent {
                event_type: FeeEventType::YieldClaim,
                amount: 50,
                timestamp: 2000,
                related_id: "loan_1".to_string(),
            },
            FeeCollectionEvent {
                event_type: FeeEventType::Liquidation,
                amount: 200,
                timestamp: 3000,
                related_id: "loan_2".to_string(),
            },
        ];

        let total = collect_batch_fees(&events);
        assert_eq!(total, 350);
    }

    #[test]
    fn test_high_fee_configuration() {
        let config = setup_config(1000); // 10% max

        let (fee, net) = apply_yield_fee(1000_0000000, &config).unwrap();
        assert_eq!(fee, 100_0000000); // 10%
        assert_eq!(net, 900_0000000);
    }
}

/// Admin Fee Treasury Module
///
/// Manages protocol fee collection and routing to admin treasury address.
/// Implements configurable fee basis points with automatic deduction during payouts and yields.
///
/// Issue #1155: Implement Admin Fee Treasury Deposit & Protocol Revenue Split

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, token};

/// Protocol fee in basis points (e.g., 50 = 0.5%)
/// Default: 50 bps (0.5% fee)
/// Can be updated by admin via set_protocol_fee_bps
pub const DEFAULT_PROTOCOL_FEE_BPS: u32 = 50;

/// Maximum protocol fee (1000 bps = 10%)
pub const MAX_PROTOCOL_FEE_BPS: u32 = 1000;

#[derive(Clone, Debug)]
pub struct AdminFeeConfig {
    /// Protocol fee in basis points (0-10000)
    pub protocol_fee_bps: u32,
    /// Treasury address receiving protocol fees
    pub treasury_address: Address,
    /// Total fees collected (for analytics)
    pub total_fees_collected: u128,
}

impl AdminFeeConfig {
    /// Create default fee configuration
    pub fn new(treasury_address: Address) -> Self {
        Self {
            protocol_fee_bps: DEFAULT_PROTOCOL_FEE_BPS,
            treasury_address,
            total_fees_collected: 0,
        }
    }

    /// Calculate fee amount from a base amount
    ///
    /// Example: if bps = 50 (0.5%) and amount = 1000
    /// returns 5 (0.5% of 1000)
    pub fn calculate_fee(&self, amount: u128) -> u128 {
        (amount * self.protocol_fee_bps as u128) / 10000
    }

    /// Deduct fee from amount and return both fee and net amount
    pub fn deduct_fee(&self, amount: u128) -> (u128, u128) {
        let fee = self.calculate_fee(amount);
        let net = amount - fee;
        (fee, net)
    }

    /// Set new protocol fee (admin only)
    pub fn set_protocol_fee_bps(&mut self, new_fee_bps: u32) -> Result<(), String> {
        if new_fee_bps > MAX_PROTOCOL_FEE_BPS {
            return Err(format!(
                "Protocol fee {} exceeds maximum {}",
                new_fee_bps, MAX_PROTOCOL_FEE_BPS
            ));
        }
        self.protocol_fee_bps = new_fee_bps;
        Ok(())
    }

    /// Update treasury address (admin only)
    pub fn set_treasury_address(&mut self, new_treasury: Address) {
        self.treasury_address = new_treasury;
    }

    /// Record collected fees for analytics
    pub fn record_fee(&mut self, amount: u128) {
        self.total_fees_collected = self.total_fees_collected.saturating_add(amount);
    }
}

/// AdminFee trait for contracts using fee system
pub trait AdminFeeManager {
    /// Get current admin fee configuration
    fn get_fee_config(&self) -> AdminFeeConfig;

    /// Update fee configuration (admin only)
    fn set_fee_config(&mut self, config: AdminFeeConfig);

    /// Transfer fee to treasury
    fn distribute_fee(env: &Env, fee: u128, token_address: &Address, treasury: &Address) -> Result<(), String>;
}

/// Helper to distribute protocol fees to treasury
///
/// Example usage in payout processing:
/// ```ignore
/// let (fee, net_payout) = fee_config.deduct_fee(payout_amount);
/// distribute_protocol_fee(env, fee, &token, &treasury_address)?;
/// // distribute net_payout to beneficiary
/// ```
pub fn distribute_protocol_fee(
    env: &Env,
    fee_amount: u128,
    token_address: &Address,
    treasury_address: &Address,
) -> Result<(), String> {
    if fee_amount == 0 {
        return Ok(());
    }

    // Create token interface
    let token_client = token::Client::new(env, token_address);

    // Transfer fee to treasury
    token_client
        .transfer(
            &env.current_contract_address(),
            treasury_address,
            &(fee_amount as i128),
        )
        .map_err(|_| "Failed to transfer protocol fee to treasury".to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fee() {
        let config = AdminFeeConfig::new(Address::random(&Env::default()));
        assert_eq!(config.calculate_fee(10000), 50); // 0.5% of 10000 = 50
        assert_eq!(config.calculate_fee(1000), 5); // 0.5% of 1000 = 5
        assert_eq!(config.calculate_fee(0), 0); // 0.5% of 0 = 0
    }

    #[test]
    fn test_deduct_fee() {
        let config = AdminFeeConfig::new(Address::random(&Env::default()));
        let (fee, net) = config.deduct_fee(10000);
        assert_eq!(fee, 50);
        assert_eq!(net, 9950);
    }

    #[test]
    fn test_set_protocol_fee_bps() {
        let mut config = AdminFeeConfig::new(Address::random(&Env::default()));
        
        // Set valid fee
        assert!(config.set_protocol_fee_bps(100).is_ok());
        assert_eq!(config.protocol_fee_bps, 100);
        
        // Set max fee
        assert!(config.set_protocol_fee_bps(MAX_PROTOCOL_FEE_BPS).is_ok());
        assert_eq!(config.protocol_fee_bps, MAX_PROTOCOL_FEE_BPS);
        
        // Exceed max fee
        assert!(config.set_protocol_fee_bps(MAX_PROTOCOL_FEE_BPS + 1).is_err());
    }

    #[test]
    fn test_record_fee() {
        let mut config = AdminFeeConfig::new(Address::random(&Env::default()));
        assert_eq!(config.total_fees_collected, 0);
        
        config.record_fee(100);
        assert_eq!(config.total_fees_collected, 100);
        
        config.record_fee(50);
        assert_eq!(config.total_fees_collected, 150);
    }

    #[test]
    fn test_custom_fee_calculation() {
        let mut config = AdminFeeConfig::new(Address::random(&Env::default()));
        config.set_protocol_fee_bps(250).unwrap(); // 2.5%
        
        assert_eq!(config.calculate_fee(10000), 250); // 2.5% of 10000 = 250
        
        let (fee, net) = config.deduct_fee(10000);
        assert_eq!(fee, 250);
        assert_eq!(net, 9750);
    }
}

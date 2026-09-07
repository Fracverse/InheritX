use soroban_sdk::{contracttype, Address};

/// Configuration and isolated tracking for a single collateral/borrow reserve
/// (e.g. XLM, USDC, EURC). Each token gets its own reserve so multi-collateral
/// lending can price risk per asset (LTV, liquidation threshold) and track how
/// much is borrowed against it and how much collateral is locked.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveConfig {
    pub token: Address,
    /// Maximum loan-to-value ratio in basis points (e.g. 5000 = 50%).
    pub ltv_bps: u32,
    /// Liquidation threshold in basis points (e.g. 8000 = 80%).
    pub liquidation_threshold_bps: u32,
    /// Total principal currently borrowed against this reserve's collateral.
    pub total_borrowed: u64,
    /// Total collateral of this token currently locked in the protocol.
    pub total_collateral: u64,
    /// Enabled flag; disallowing integration until a reserve is added.
    pub enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveAddedEvent {
    pub token: Address,
    pub ltv_bps: u32,
    pub liquidation_threshold_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveWithdrawnEvent {
    pub amount: u64,
    pub withdrawn_by: Address,
    pub withdrawn_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveAllocatedEvent {
    pub amount: u64,
    pub allocated_to: Address,
    pub allocated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveFactorUpdatedEvent {
    pub new_reserve_factor_bps: u32,
    pub updated_at: u64,
}

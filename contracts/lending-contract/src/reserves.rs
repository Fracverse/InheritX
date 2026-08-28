use soroban_sdk::{contracttype, Address};

/// Utilization kink used by the default reserve interest-rate curve.
pub const DEFAULT_KINK_UTILIZATION_BPS: u32 = 8_000;
/// Default second-stage rate slope, in basis points from kink to 100% utilization.
pub const DEFAULT_SLOPE2_BPS: u32 = 30_000;

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

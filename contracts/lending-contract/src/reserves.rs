use soroban_sdk::{contracttype, Address};

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

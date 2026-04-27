use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Filed = 0,
    UnderReview = 1,
    Resolved = 2,
    Rejected = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRecord {
    pub dispute_id: u64,
    pub plan_id: u64,
    pub disputer: Address,
    pub reason: soroban_sdk::String,
    pub status: DisputeStatus,
    pub filed_at: u64,
    pub resolved_at: u64,
    pub resolution_notes: soroban_sdk::String,
    pub arbitrator: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeFiledEvent {
    pub dispute_id: u64,
    pub plan_id: u64,
    pub disputer: Address,
    pub reason: soroban_sdk::String,
    pub filed_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeResolvedEvent {
    pub dispute_id: u64,
    pub plan_id: u64,
    pub status: DisputeStatus,
    pub arbitrator: Address,
    pub resolved_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanFrozenEvent {
    pub plan_id: u64,
    pub dispute_id: u64,
    pub frozen_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanUnfrozenEvent {
    pub plan_id: u64,
    pub dispute_id: u64,
    pub unfrozen_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitratorAddedEvent {
    pub arbitrator: Address,
    pub added_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitratorRemovedEvent {
    pub arbitrator: Address,
    pub removed_at: u64,
}

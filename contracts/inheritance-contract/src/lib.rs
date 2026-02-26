#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, token, vec, Address,
    Bytes, BytesN, Env, IntoVal, InvokeError, String, Symbol, Val, Vec,
};

/// Current contract version - bump this on each upgrade
const CONTRACT_VERSION: u32 = 1;

// ... existing code up to DataKey enum ...

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    NextPlanId,
    Plan(u64),
    Claim(BytesN<32>),         // keyed by hashed_email
    UserPlans(Address),        // keyed by owner Address, value is Vec<u64>
    UserClaimedPlans(Address), // keyed by owner Address, value is Vec<u64>
    DeactivatedPlans,          // value is Vec<u64> of all deactivated plan IDs
    AllClaimedPlans,           // value is Vec<u64> of all claimed plan IDs
    Admin,
    Kyc(Address),
    Version,
    InheritanceTrigger(u64), // per-plan inheritance trigger info
    KycContract,              // Address of the KYC contract
}

// ... existing code continues with all the same structs and enums ...

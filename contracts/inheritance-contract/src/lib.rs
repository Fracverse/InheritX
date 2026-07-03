#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

const MAX_BENEFICIARIES: u32 = 100;
const PLAN_TTL_THRESHOLD: u32 = 500;
const PLAN_TTL_LEEWAY: u32 = 100;
const BRIDGE_FEE_BPS: u32 = 100; // 1% bridge fee
const STELLAR_CHAIN: &str = "Stellar";

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    PlanAlreadyExists = 1,
    PlanNotFound = 2,
    Unauthorized = 3,
    InactivityPeriodNotMet = 4,
    InvalidBasisPoints = 5,
    NegativeAmount = 6,
    InsufficientBalance = 7,
    TooManyBeneficiaries = 8,
    TimelockNotExpired = 9,
    PayoutNotTriggered = 10,
    UnsupportedToken = 11,
    InvalidBridgeMetadata = 12,
    MathOverflow = 13,
    AlreadyInitialized = 14,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Beneficiary {
    pub address: Address,
    pub allocation_bps: u32,
    pub fiat_anchor_info: String,
    pub destination_chain: String,
    pub destination_address: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    pub owner: Address,
    pub token: Address,
    pub amount: i128,
    pub beneficiaries: Vec<Beneficiary>,
    pub last_ping: u64,
    pub grace_period: u64,
    pub earn_yield: bool,
    pub yield_rate_bps: u32,
    pub is_active: bool,
    pub timelock_duration: u64,
    pub source_chain: String,
    pub source_tx_hash: String,
}

pub type InheritancePlan = Plan;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgePayoutEvent {
    pub owner: Address,
    pub token: Address,
    pub beneficiary: Address,
    pub destination_chain: String,
    pub destination_address: String,
    pub gross_amount: i128,
    pub fee_amount: i128,
    pub net_amount: i128,
    pub source_chain: String,
    pub source_tx_hash: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Plan(Address),
    ClaimStatus(Address),
    SupportedWrappedToken(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstanceDataKey {
    Admin,
}

#[contract]
pub struct InheritanceContract;

impl InheritanceContract {
    fn extend_plan_ttl(env: &Env, key: &DataKey) {
        env.storage()
            .persistent()
            .extend_ttl(key, PLAN_TTL_LEEWAY, PLAN_TTL_THRESHOLD);
    }

    fn emit_bridge_payout_event(env: &Env, event: BridgePayoutEvent) {
        let topic = (symbol_short!("BridgePay"), env.current_contract_address());
        env.events().publish(topic, event);
    }

    fn is_stellar_chain(chain: &String, env: &Env) -> bool {
        let stellar = String::from_str(env, STELLAR_CHAIN);
        chain == &stellar
    }

    fn supported_wrapped_token(env: &Env, token: &Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::SupportedWrappedToken(token.clone()))
    }
}

#[contractimpl]
#[allow(clippy::too_many_arguments)]
impl InheritanceContract {
    /// Create a yield-bearing inheritance plan with mass beneficiaries payout allocations.
    /// Contributors: Implement token transfers from owner, validation checks, and storage configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn create_plan(
        env: Env,
        owner: Address,
        token: Address,
        amount: i128,
        beneficiaries: Vec<Beneficiary>,
        grace_period: u64,
        earn_yield: bool,
        yield_rate_bps: u32,
        timelock_duration: u64,
        source_chain: String,
        source_tx_hash: String,
    ) -> Result<(), Error> {
        owner.require_auth();

        if beneficiaries.len() > MAX_BENEFICIARIES {
            return Err(Error::TooManyBeneficiaries);
        }

        let key = DataKey::Plan(owner.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::PlanAlreadyExists);
        }

        if amount <= 0 {
            return Err(Error::NegativeAmount);
        }

        let mut total_bps: u32 = 0;
        for beneficiary in beneficiaries.iter() {
            total_bps += beneficiary.allocation_bps;
            let empty = String::from_str(&env, "");
            if beneficiary.destination_chain == empty || beneficiary.destination_address == empty {
                return Err(Error::InvalidBridgeMetadata);
            }
        }
        if total_bps != 10000 {
            return Err(Error::InvalidBasisPoints);
        }

        let empty = String::from_str(&env, "");
        if source_chain == empty || source_tx_hash == empty {
            return Err(Error::InvalidBridgeMetadata);
        }

        let stellarchain = String::from_str(&env, STELLAR_CHAIN);
        if source_chain != stellarchain && !Self::supported_wrapped_token(&env, &token) {
            return Err(Error::UnsupportedToken);
        }

        let token_client = soroban_sdk::token::Client::new(&env, &token);
        let balance = token_client.balance(&owner);
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }

        token_client.transfer(&owner, &env.current_contract_address(), &amount);

        let plan = Plan {
            owner: owner.clone(),
            token,
            amount,
            beneficiaries,
            last_ping: env.ledger().timestamp(),
            grace_period,
            earn_yield,
            yield_rate_bps,
            is_active: true,
            timelock_duration,
            source_chain: source_chain.clone(),
            source_tx_hash: source_tx_hash.clone(),
        };

        env.storage().persistent().set(&key, &plan);
        Self::extend_plan_ttl(&env, &key);

        Ok(())
    }

    /// Reset the proof-of-life inactivity timer.
    /// Contributors: Recalculate and accrue yield, update last ping timestamp.
    pub fn ping(env: Env, owner: Address) -> Result<(), Error> {
        owner.require_auth();

        let key = DataKey::Plan(owner.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::PlanNotFound);
        }

        let mut plan: Plan = env.storage().persistent().get(&key).unwrap();
        let current_timestamp = env.ledger().timestamp();
        plan.last_ping = current_timestamp;

        env.storage().persistent().set(&key, &plan);
        Self::extend_plan_ttl(&env, &key);
        env.events()
            .publish((symbol_short!("ping"), owner), current_timestamp);

        Ok(())
    }

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        let admin_key = InstanceDataKey::Admin;
        if env.storage().instance().has(&admin_key) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&admin_key, &admin);
        Ok(())
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        let admin_key = InstanceDataKey::Admin;
        let configured_admin: Address = env
            .storage()
            .instance()
            .get(&admin_key)
            .ok_or(Error::Unauthorized)?;
        if &configured_admin != admin {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    pub fn register_supported_wrapped_token(
        env: Env,
        admin: Address,
        token: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        let key = DataKey::SupportedWrappedToken(token);
        env.storage().persistent().set(&key, &true);
        Ok(())
    }

    pub fn unregister_wrapped_token(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        let key = DataKey::SupportedWrappedToken(token);
        env.storage().persistent().remove(&key);
        Ok(())
    }

    pub fn is_supported_wrapped_token(env: Env, token: Address) -> Result<bool, Error> {
        Ok(Self::supported_wrapped_token(&env, &token))
    }

    /// Claim payout once the plan owner has been inactive beyond the grace period.
    /// Contributors: Calculate final yield-bearing payout, split assets among beneficiaries,
    /// emit payout events, and trigger anchor event emissions for fiat recipients.
    pub fn claim(env: Env, owner: Address) -> Result<(), Error> {
        let key = DataKey::Plan(owner.clone());
        let plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;

        if plan.is_active {
            return Err(Error::InactivityPeriodNotMet);
        }

        let current_time = env.ledger().timestamp();
        if current_time < plan.last_ping + plan.grace_period {
            return Err(Error::InactivityPeriodNotMet);
        }

        let claim_key = DataKey::ClaimStatus(owner.clone());
        if env.storage().persistent().has(&claim_key) {
            return Ok(()); // Already claimed
        }

        env.storage().persistent().set(&claim_key, &current_time);
        Self::extend_plan_ttl(&env, &claim_key);

        Ok(())
    }

    /// Cancel a triggered payout during the timelock window.
    pub fn cancel_claim(env: Env, owner: Address) -> Result<(), Error> {
        owner.require_auth();

        let key = DataKey::Plan(owner.clone());
        let mut plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;

        let claim_key = DataKey::ClaimStatus(owner.clone());
        if !env.storage().persistent().has(&claim_key) {
            return Err(Error::PayoutNotTriggered);
        }

        env.storage().persistent().remove(&claim_key);

        plan.is_active = true;
        plan.last_ping = env.ledger().timestamp();
        env.storage().persistent().set(&key, &plan);
        Self::extend_plan_ttl(&env, &key);

        Ok(())
    }

    /// Check if a plan has timed out (grace period elapsed).
    /// Returns true if current_time >= last_ping + grace_period, false otherwise.
    /// This is a read-only query method that does not modify state.
    pub fn is_plan_timed_out(env: Env, owner: Address) -> Result<bool, Error> {
        let key = DataKey::Plan(owner.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::PlanNotFound);
        }

        let plan: Plan = env.storage().persistent().get(&key).unwrap();
        Self::extend_plan_ttl(&env, &key);

        let current_time = env.ledger().timestamp();
        let timeout_deadline = plan.last_ping + plan.grace_period;

        Ok(current_time >= timeout_deadline)
    }

    /// Get the timeout deadline timestamp for a plan.
    /// Returns the timestamp when the grace period expires (last_ping + grace_period).
    /// This is a read-only query method for external monitoring.
    pub fn get_timeout_deadline(env: Env, owner: Address) -> Result<u64, Error> {
        let key = DataKey::Plan(owner.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::PlanNotFound);
        }

        let plan: Plan = env.storage().persistent().get(&key).unwrap();
        Self::extend_plan_ttl(&env, &key);

        Ok(plan.last_ping + plan.grace_period)
    }

    /// Retrieve the current inheritance plan data.
    /// Contributors: Query plan storage, dynamically projects the accumulated yield.
    pub fn get_plan(env: Env, owner: Address) -> Result<InheritancePlan, Error> {
        let key = DataKey::Plan(owner.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::PlanNotFound);
        }

        let plan: Plan = env.storage().persistent().get(&key).unwrap();
        Self::extend_plan_ttl(&env, &key);

        Ok(plan)
    }

    /// Trigger payout to all beneficiaries once the plan is claimable.
    /// Iterates over beneficiaries, computes pro-rata token allocations
    /// using the stored basis points, and transfers tokens safely.
    /// Remaining dust from integer division is allocated to the last beneficiary.
    /// Aborts the entire transaction if any single transfer fails.
    pub fn trigger_payout(env: Env, owner: Address) -> Result<(), Error> {
        let key = DataKey::Plan(owner.clone());
        let plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;

        let claim_key = DataKey::ClaimStatus(owner.clone());
        let claim_time: u64 = env
            .storage()
            .persistent()
            .get(&claim_key)
            .ok_or(Error::PayoutNotTriggered)?;

        let current_time = env.ledger().timestamp();
        if current_time < claim_time + plan.timelock_duration {
            return Err(Error::TimelockNotExpired);
        }

        // Checks-effects-interactions: remove plan before transfers
        // to prevent double payout and guard against re-entrancy
        env.storage().persistent().remove(&key);
        env.storage().persistent().remove(&claim_key);

        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        let n = plan.beneficiaries.len();
        let mut remaining = plan.amount;

        for (i, beneficiary) in plan.beneficiaries.iter().enumerate() {
            let share = if i == (n - 1) as usize {
                remaining
            } else {
                let amount = plan.amount * (beneficiary.allocation_bps as i128) / 10000;
                remaining -= amount;
                amount
            };

            let destination_stellar = Self::is_stellar_chain(&beneficiary.destination_chain, &env);
            let (fee_amount, net_amount) = if destination_stellar {
                (0_i128, share)
            } else {
                let fee = share
                    .checked_mul(BRIDGE_FEE_BPS as i128)
                    .ok_or(Error::MathOverflow)?
                    .checked_div(10000)
                    .ok_or(Error::MathOverflow)?;
                let net = share.checked_sub(fee).ok_or(Error::MathOverflow)?;
                (fee, net)
            };

            token_client.transfer(
                &env.current_contract_address(),
                &beneficiary.address,
                &net_amount,
            );

            if !destination_stellar {
                let event = BridgePayoutEvent {
                    owner: plan.owner.clone(),
                    token: plan.token.clone(),
                    beneficiary: beneficiary.address.clone(),
                    destination_chain: beneficiary.destination_chain.clone(),
                    destination_address: beneficiary.destination_address.clone(),
                    gross_amount: share,
                    fee_amount,
                    net_amount,
                    source_chain: plan.source_chain.clone(),
                    source_tx_hash: plan.source_tx_hash.clone(),
                };
                Self::emit_bridge_payout_event(&env, event);
            }
        }

        Ok(())
    }

    /// Deactivate a plan to start the inactivity grace period.
    /// Used internally by claim logic. This does NOT refund tokens.
    /// The plan owner can call close_plan() for an early refund.
    #[allow(dead_code)]
    fn deactivate_plan(env: &Env, owner: &Address) -> Result<(), Error> {
        let key = DataKey::Plan(owner.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::PlanNotFound);
        }

        let mut plan: Plan = env.storage().persistent().get(&key).unwrap();
        plan.is_active = false;

        env.storage().persistent().set(&key, &plan);
        Self::extend_plan_ttl(env, &key);

        Ok(())
    }

    /// Cancel a plan early and withdraw all remaining assets.
    /// Authenticates that the caller is the plan owner.
    /// Transfers all locked tokens back to the owner and deletes the plan from storage.
    pub fn close_plan(env: Env, owner: Address) -> Result<(), Error> {
        owner.require_auth();

        let key = DataKey::Plan(owner.clone());
        let plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;

        let claim_key = DataKey::ClaimStatus(owner.clone());
        if env.storage().persistent().has(&claim_key) {
            env.storage().persistent().remove(&claim_key);
        }

        env.storage().persistent().remove(&key);

        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        token_client.transfer(&env.current_contract_address(), &owner, &plan.amount);

        Ok(())
    }

    /// Reclaim the locked assets and delete the plan.
    pub fn reclaim(env: Env, owner: Address) -> Result<(), Error> {
        owner.require_auth();

        let key = DataKey::Plan(owner.clone());
        let plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;

        let claim_key = DataKey::ClaimStatus(owner.clone());
        if env.storage().persistent().has(&claim_key) {
            env.storage().persistent().remove(&claim_key);
        }

        env.storage().persistent().remove(&key);

        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        token_client.transfer(&env.current_contract_address(), &owner, &plan.amount);

        Ok(())
    }
}

#[cfg(test)]
mod test;

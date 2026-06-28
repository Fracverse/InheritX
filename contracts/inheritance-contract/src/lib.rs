#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

const MAX_BENEFICIARIES: u32 = 100;
const PLAN_TTL_THRESHOLD: u32 = 500;
const PLAN_TTL_LEEWAY: u32 = 100;
const TEMP_TTL_THRESHOLD: u32 = 100;
const TEMP_TTL_LEEWAY: u32 = 50;

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
    AlreadyInitialized = 9,
    AdminNotSet = 10,
    InvalidYieldRate = 11,
    TokenAlreadySupported = 12,
    TokenNotSupported = 13,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Beneficiary {
    pub address: Address,
    pub allocation_bps: u32,
    pub fiat_anchor_info: String,
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
}

pub type InheritancePlan = Plan;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Plan(Address),
    ClaimStatus(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstanceDataKey {
    Admin,
    GlobalYieldRate,
    SupportedToken(Address),
}

#[contract]
pub struct InheritanceContract;

impl InheritanceContract {
    fn extend_plan_ttl(env: &Env, key: &DataKey) {
        env.storage()
            .persistent()
            .extend_ttl(key, PLAN_TTL_LEEWAY, PLAN_TTL_THRESHOLD);
    }

    fn extend_temp_ttl(env: &Env, key: &DataKey) {
        env.storage()
            .temporary()
            .extend_ttl(key, TEMP_TTL_LEEWAY, TEMP_TTL_THRESHOLD);
    }

    fn extend_instance_ttl(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(PLAN_TTL_LEEWAY, PLAN_TTL_THRESHOLD);
    }

    /// Read the configured admin address, or error if the contract is uninitialized.
    fn read_admin(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&InstanceDataKey::Admin)
            .ok_or(Error::AdminNotSet)
    }

    /// Require the stored admin's signature for governance operations.
    fn require_admin(env: &Env) -> Result<Address, Error> {
        let admin = Self::read_admin(env)?;
        admin.require_auth();
        Ok(admin)
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
        }
        if total_bps != 10000 {
            return Err(Error::InvalidBasisPoints);
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
        plan.last_ping = env.ledger().timestamp();

        env.storage().persistent().set(&key, &plan);
        Self::extend_plan_ttl(&env, &key);

        Ok(())
    }

    /// Claim payout once the plan owner has been inactive beyond the grace period.
    /// Contributors: Calculate final yield-bearing payout, split assets among beneficiaries,
    /// emit payout events, and trigger anchor event emissions for fiat recipients.
    pub fn claim(env: Env, owner: Address) -> Result<(), Error> {
        let key = DataKey::Plan(owner.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::PlanNotFound);
        }

        let plan: Plan = env.storage().persistent().get(&key).unwrap();

        if plan.is_active {
            return Err(Error::InactivityPeriodNotMet);
        }

        let current_time = env.ledger().timestamp();
        if current_time < plan.last_ping + plan.grace_period {
            return Err(Error::InactivityPeriodNotMet);
        }

        let claim_key = DataKey::ClaimStatus(owner.clone());
        env.storage().temporary().set(&claim_key, &true);
        Self::extend_temp_ttl(&env, &claim_key);

        Ok(())
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

        if plan.is_active {
            return Err(Error::InactivityPeriodNotMet);
        }

        let current_time = env.ledger().timestamp();
        if current_time < plan.last_ping + plan.grace_period {
            return Err(Error::InactivityPeriodNotMet);
        }

        // Checks-effects-interactions: remove plan before transfers
        // to prevent double payout and guard against re-entrancy
        env.storage().persistent().remove(&key);

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
            token_client.transfer(
                &env.current_contract_address(),
                &beneficiary.address,
                &share,
            );
        }

        Ok(())
    }

    /// Deactivate the plan and withdraw all remaining assets.
    /// Contributors: Reclaim assets and transfer principal + yield back to the owner.
    pub fn close_plan(env: Env, owner: Address) -> Result<(), Error> {
        owner.require_auth();

        let key = DataKey::Plan(owner.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::PlanNotFound);
        }

        let mut plan: Plan = env.storage().persistent().get(&key).unwrap();
        plan.is_active = false;

        env.storage().persistent().set(&key, &plan);
        Self::extend_plan_ttl(&env, &key);

        Ok(())
    }

    /// Initialize the contract with the governing admin address.
    /// Can only be called once; subsequent calls return `AlreadyInitialized`.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&InstanceDataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&InstanceDataKey::Admin, &admin);
        Self::extend_instance_ttl(&env);

        env.events()
            .publish((symbol_short!("init"), admin.clone()), admin);

        Ok(())
    }

    /// Update the global base yield rate (in basis points).
    /// Restricted to the admin signature. Rate must be <= 10000 bps (100%).
    pub fn set_global_yield_rate(env: Env, rate_bps: u32) -> Result<(), Error> {
        Self::require_admin(&env)?;

        if rate_bps > 10000 {
            return Err(Error::InvalidYieldRate);
        }

        env.storage()
            .instance()
            .set(&InstanceDataKey::GlobalYieldRate, &rate_bps);
        Self::extend_instance_ttl(&env);

        env.events()
            .publish((symbol_short!("yield_set"),), rate_bps);

        Ok(())
    }

    /// Retrieve the global base yield rate in basis points (defaults to 0 if unset).
    pub fn get_global_yield_rate(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&InstanceDataKey::GlobalYieldRate)
            .unwrap_or(0)
    }

    /// Register a supported asset (USDC, XLM, etc.) in the token registry.
    /// Restricted to the admin signature.
    pub fn add_supported_token(env: Env, token: Address) -> Result<(), Error> {
        Self::require_admin(&env)?;

        let key = InstanceDataKey::SupportedToken(token.clone());
        if env.storage().instance().has(&key) {
            return Err(Error::TokenAlreadySupported);
        }

        env.storage().instance().set(&key, &true);
        Self::extend_instance_ttl(&env);

        env.events()
            .publish((symbol_short!("tkn_add"), token.clone()), token);

        Ok(())
    }

    /// Remove an asset from the supported token registry.
    /// Restricted to the admin signature.
    pub fn remove_supported_token(env: Env, token: Address) -> Result<(), Error> {
        Self::require_admin(&env)?;

        let key = InstanceDataKey::SupportedToken(token.clone());
        if !env.storage().instance().has(&key) {
            return Err(Error::TokenNotSupported);
        }

        env.storage().instance().remove(&key);
        Self::extend_instance_ttl(&env);

        env.events()
            .publish((symbol_short!("tkn_rm"), token.clone()), token);

        Ok(())
    }

    /// Check whether a token is registered as a supported asset.
    pub fn is_supported_token(env: Env, token: Address) -> bool {
        env.storage()
            .instance()
            .has(&InstanceDataKey::SupportedToken(token))
    }

    /// Transfer admin/governance control to a new address.
    /// Restricted to the current admin signature.
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let previous_admin = Self::require_admin(&env)?;

        env.storage()
            .instance()
            .set(&InstanceDataKey::Admin, &new_admin);
        Self::extend_instance_ttl(&env);

        env.events()
            .publish((symbol_short!("admin_set"), previous_admin), new_admin);

        Ok(())
    }

    /// Retrieve the current admin address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        Self::read_admin(&env)
    }
}

#[cfg(test)]
mod test;

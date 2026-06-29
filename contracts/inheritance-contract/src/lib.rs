#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec};

const MAX_BENEFICIARIES: u32 = 100;
const PLAN_TTL_THRESHOLD: u32 = 500;
const PLAN_TTL_LEEWAY: u32 = 100;
/// Seconds in a 365-day year (integer arithmetic, no f64 in no_std).
const SECONDS_PER_YEAR: u64 = 365 * 24 * 3600; // 31_536_000

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
    pub timelock_duration: u64,
    /// Yield accumulated so far (in token units). Updated on each ping.
    pub accrued_yield: i128,
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
}

#[contract]
pub struct InheritanceContract;

impl InheritanceContract {
    fn extend_plan_ttl(env: &Env, key: &DataKey) {
        env.storage()
            .persistent()
            .extend_ttl(key, PLAN_TTL_LEEWAY, PLAN_TTL_THRESHOLD);
    }

    /// Simple-interest yield accrual (integer arithmetic, no f64).
    ///
    /// accrued = principal * rate_bps * elapsed_secs / (10_000 * SECONDS_PER_YEAR)
    ///
    /// Returns 0 when `earn_yield` is false or `rate_bps` is 0.
    fn calculate_accrued_yield(amount: i128, rate_bps: u32, elapsed_secs: u64) -> i128 {
        if rate_bps == 0 || elapsed_secs == 0 {
            return 0;
        }
        // Overflow-safe: use i128 for all intermediate products.
        amount * (rate_bps as i128) * (elapsed_secs as i128)
            / (10_000_i128 * SECONDS_PER_YEAR as i128)
    }
}

#[contractimpl]
#[allow(clippy::too_many_arguments)]
impl InheritanceContract {
    /// Create a yield-bearing inheritance plan with mass beneficiaries payout allocations.
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
            timelock_duration,
            accrued_yield: 0,
        };

        env.storage().persistent().set(&key, &plan);
        Self::extend_plan_ttl(&env, &key);

        Ok(())
    }

    /// Reset the proof-of-life inactivity timer, accruing yield for the elapsed period first.
    pub fn ping(env: Env, owner: Address) -> Result<(), Error> {
        owner.require_auth();

        let key = DataKey::Plan(owner.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::PlanNotFound);
        }

        let mut plan: Plan = env.storage().persistent().get(&key).unwrap();

        // Accrue yield for the time elapsed since the last ping.
        if plan.earn_yield {
            let now = env.ledger().timestamp();
            let elapsed = now.saturating_sub(plan.last_ping);
            plan.accrued_yield +=
                Self::calculate_accrued_yield(plan.amount, plan.yield_rate_bps, elapsed);
        }

        plan.last_ping = env.ledger().timestamp();

        env.storage().persistent().set(&key, &plan);
        Self::extend_plan_ttl(&env, &key);

        Ok(())
    }

    /// Claim payout once the plan owner has been inactive beyond the grace period.
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
    pub fn get_timeout_deadline(env: Env, owner: Address) -> Result<u64, Error> {
        let key = DataKey::Plan(owner.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::PlanNotFound);
        }

        let plan: Plan = env.storage().persistent().get(&key).unwrap();
        Self::extend_plan_ttl(&env, &key);

        Ok(plan.last_ping + plan.grace_period)
    }

    /// Retrieve the current inheritance plan, projecting any un-accrued yield dynamically.
    ///
    /// The returned `accrued_yield` reflects all yield accrued since the last ping
    /// without writing to storage.
    pub fn get_plan(env: Env, owner: Address) -> Result<InheritancePlan, Error> {
        let key = DataKey::Plan(owner.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::PlanNotFound);
        }

        let mut plan: Plan = env.storage().persistent().get(&key).unwrap();
        Self::extend_plan_ttl(&env, &key);

        // Project yield since the last ping without persisting the change.
        if plan.earn_yield {
            let now = env.ledger().timestamp();
            let elapsed = now.saturating_sub(plan.last_ping);
            plan.accrued_yield +=
                Self::calculate_accrued_yield(plan.amount, plan.yield_rate_bps, elapsed);
        }

        Ok(plan)
    }

    /// Trigger payout to all beneficiaries (principal + accrued yield).
    pub fn trigger_payout(env: Env, owner: Address) -> Result<(), Error> {
        let key = DataKey::Plan(owner.clone());
        let mut plan: Plan = env
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

        // Accrue any remaining yield up to claim time before distributing.
        if plan.earn_yield {
            let elapsed = current_time.saturating_sub(plan.last_ping);
            plan.accrued_yield +=
                Self::calculate_accrued_yield(plan.amount, plan.yield_rate_bps, elapsed);
        }

        let total_payout = plan.amount + plan.accrued_yield;

        // Checks-effects-interactions: remove plan before transfers.
        env.storage().persistent().remove(&key);
        env.storage().persistent().remove(&claim_key);

        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        let n = plan.beneficiaries.len();
        let mut remaining = total_payout;

        for (i, beneficiary) in plan.beneficiaries.iter().enumerate() {
            let share = if i == (n - 1) as usize {
                remaining
            } else {
                let amount = total_payout * (beneficiary.allocation_bps as i128) / 10000;
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

    /// Deactivate the plan.
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

    /// Reclaim locked assets (principal + accrued yield) and delete the plan.
    pub fn reclaim(env: Env, owner: Address) -> Result<(), Error> {
        owner.require_auth();

        let key = DataKey::Plan(owner.clone());
        let mut plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;

        // Accrue any remaining yield before returning funds.
        if plan.earn_yield {
            let now = env.ledger().timestamp();
            let elapsed = now.saturating_sub(plan.last_ping);
            plan.accrued_yield +=
                Self::calculate_accrued_yield(plan.amount, plan.yield_rate_bps, elapsed);
        }

        let total_return = plan.amount + plan.accrued_yield;

        let claim_key = DataKey::ClaimStatus(owner.clone());
        if env.storage().persistent().has(&claim_key) {
            env.storage().persistent().remove(&claim_key);
        }

        env.storage().persistent().remove(&key);

        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        token_client.transfer(&env.current_contract_address(), &owner, &total_return);

        Ok(())
    }
}

#[cfg(test)]
mod test;

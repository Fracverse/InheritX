#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

pub mod safe_math;

const MAX_BENEFICIARIES: u32 = 100;
const PLAN_TTL_THRESHOLD: u32 = 500;
const PLAN_TTL_LEEWAY: u32 = 100;
const BRIDGE_FEE_BPS: u32 = 100; // 1% bridge fee
const STELLAR_CHAIN: &str = "Stellar";
const MIN_GRACE_PERIOD_SECONDS: u64 = 86_400;

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
    DivisionByZero = 15,
    InvalidYieldRate = 16,
    InvalidGracePeriod = 17,
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
    PaidBeneficiary(Address, Address),
    SupportedWrappedToken(Address),
    YieldState(Address),
}

/// Checkpointed virtual-yield bookkeeping for a plan. `accrued` holds
/// interest locked in by past checkpoints; interest since `last_accrual`
/// is projected on demand with `safe_math::compound_yield`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YieldState {
    pub accrued: i128,
    pub last_accrual: u64,
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

    /// Loads the yield checkpoint for a plan, falling back to a zero
    /// checkpoint anchored at `last_ping` for plans created before the
    /// yield engine existed.
    fn load_yield_state(env: &Env, plan: &Plan, owner: &Address) -> YieldState {
        env.storage()
            .persistent()
            .get(&DataKey::YieldState(owner.clone()))
            .unwrap_or(YieldState {
                accrued: 0,
                last_accrual: plan.last_ping,
            })
    }

    /// Interest earned between the last checkpoint and `at`, compounded
    /// daily on principal + already-accrued interest. Zero while yield is
    /// disabled, and zero for any `at` at or before the last checkpoint.
    fn yield_between(plan: &Plan, state: &YieldState, at: u64) -> Result<i128, Error> {
        if !plan.earn_yield || plan.yield_rate_bps == 0 {
            return Ok(0);
        }
        let elapsed = at.saturating_sub(state.last_accrual);
        let base = safe_math::safe_add(plan.amount, state.accrued)?;
        let compounded = safe_math::compound_yield(base, plan.yield_rate_bps, elapsed)?;
        safe_math::safe_sub(compounded, base)
    }

    /// Interest earned since the last checkpoint, as of the current ledger
    /// timestamp.
    fn live_yield(env: &Env, plan: &Plan, state: &YieldState) -> Result<i128, Error> {
        Self::yield_between(plan, state, env.ledger().timestamp())
    }

    /// Locks in interest accrued so far and re-anchors the accrual clock at
    /// the current ledger time. Returns (gain, new total accrued).
    fn checkpoint_yield(env: &Env, plan: &Plan, owner: &Address) -> Result<(i128, i128), Error> {
        let mut state = Self::load_yield_state(env, plan, owner);
        let gain = Self::live_yield(env, plan, &state)?;
        state.accrued = safe_math::safe_add(state.accrued, gain)?;
        state.last_accrual = env.ledger().timestamp();
        let key = DataKey::YieldState(owner.clone());
        env.storage().persistent().set(&key, &state);
        Self::extend_plan_ttl(env, &key);
        Ok((gain, state.accrued))
    }

    fn clear_yield_state(env: &Env, owner: &Address) {
        env.storage()
            .persistent()
            .remove(&DataKey::YieldState(owner.clone()));
    }

    /// Sums beneficiary allocations with checked math and validates bridge
    /// metadata; returns InvalidBasisPoints when the sum overflows or
    /// deviates from 100%.
    fn validate_beneficiaries(env: &Env, beneficiaries: &Vec<Beneficiary>) -> Result<(), Error> {
        let mut total_bps: u32 = 0;
        let empty = String::from_str(env, "");
        for beneficiary in beneficiaries.iter() {
            total_bps = total_bps
                .checked_add(beneficiary.allocation_bps)
                .ok_or(Error::InvalidBasisPoints)?;
            if beneficiary.destination_chain == empty || beneficiary.destination_address == empty {
                return Err(Error::InvalidBridgeMetadata);
            }
        }
        if total_bps != 10000 {
            return Err(Error::InvalidBasisPoints);
        }
        Ok(())
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

        if grace_period < MIN_GRACE_PERIOD_SECONDS {
            return Err(Error::InvalidGracePeriod);
        }

        safe_math::validate_yield_rate(yield_rate_bps)?;
        Self::validate_beneficiaries(&env, &beneficiaries)?;

        let empty = String::from_str(&env, "");
        if source_chain == empty || source_tx_hash == empty {
            return Err(Error::InvalidBridgeMetadata);
        }

        let stellarchain = String::from_str(&env, STELLAR_CHAIN);
        if source_chain != stellarchain && !Self::supported_wrapped_token(&env, &token) {
            return Err(Error::UnsupportedToken);
        }

        let token_client = soroban_sdk::token::Client::new(&env, &token);

        // Validate token by checking if it's a valid Soroban Token contract
        // Calling decimals() will fail if the token address is not a valid token contract
        let _ = token_client.decimals();

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

        let yield_key = DataKey::YieldState(owner.clone());
        env.storage().persistent().set(
            &yield_key,
            &YieldState {
                accrued: 0,
                last_accrual: plan.last_ping,
            },
        );
        Self::extend_plan_ttl(&env, &yield_key);

        env.events()
            .publish((symbol_short!("PlanCrea"), owner), amount);

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

        let (gain, total_accrued) = Self::checkpoint_yield(&env, &plan, &owner)?;
        if gain > 0 {
            env.events().publish(
                (symbol_short!("yield"), owner.clone()),
                (gain, total_accrued),
            );
        }

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
        let inactivity_deadline = safe_math::safe_add_u64(plan.last_ping, plan.grace_period)?;
        if current_time < inactivity_deadline {
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
        let timeout_deadline = safe_math::safe_add_u64(plan.last_ping, plan.grace_period)?;

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

        safe_math::safe_add_u64(plan.last_ping, plan.grace_period)
    }

    /// Retrieve the current inheritance plan data for client RPC queries.
    pub fn get_plan(env: Env, owner: Address) -> Option<InheritancePlan> {
        env.storage().persistent().get(&DataKey::Plan(owner))
    }

    /// Check whether a beneficiary has already received a plan payout.
    /// This is a read-only query method that does not modify state.
    pub fn is_beneficiary_paid(env: Env, owner: Address, beneficiary: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::PaidBeneficiary(owner, beneficiary))
            .unwrap_or(false)
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
        let payout_time = safe_math::safe_add_u64(claim_time, plan.timelock_duration)?;
        if current_time < payout_time {
            return Err(Error::TimelockNotExpired);
        }

        // Checks-effects-interactions: remove plan before transfers
        // to prevent double payout and guard against re-entrancy
        env.storage().persistent().remove(&key);
        env.storage().persistent().remove(&claim_key);
        Self::clear_yield_state(&env, &owner);

        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        let n = plan.beneficiaries.len();
        let mut remaining = plan.amount;

        for (i, beneficiary) in plan.beneficiaries.iter().enumerate() {
            let share = if i == (n - 1) as usize {
                remaining
            } else {
                let amount = safe_math::apply_bps(plan.amount, beneficiary.allocation_bps)?;
                remaining = safe_math::safe_sub(remaining, amount)?;
                amount
            };

            let paid_key = DataKey::PaidBeneficiary(owner.clone(), beneficiary.address.clone());
            if env.storage().persistent().has(&paid_key) {
                continue;
            }

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
            env.storage().persistent().set(&paid_key, &true);
            Self::extend_plan_ttl(&env, &paid_key);

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

        // A completed payout no longer needs retry markers. Removing them also
        // prevents a later plan for the same owner from inheriting stale state.
        for beneficiary in plan.beneficiaries.iter() {
            env.storage().persistent().remove(&DataKey::PaidBeneficiary(
                owner.clone(),
                beneficiary.address,
            ));
        }

        Ok(())
    }

    /// Distribute the full yield-bearing inheritance to Stellar-native beneficiaries.
    ///
    /// This function is the Stellar-specific payout path: it computes the total
    /// claimable amount as `principal + all accrued yield`, then iterates over
    /// every beneficiary whose `destination_chain` is `"Stellar"` and transfers
    /// their pro-rata share (computed via `allocation_bps / 10 000`) directly to
    /// their on-chain `address`.  Non-Stellar beneficiaries (cross-chain bridge
    /// destinations) are intentionally skipped — they are handled by the separate
    /// bridge settlement flow in `trigger_payout`.
    ///
    /// Dust from integer-division rounding is absorbed by the last Stellar
    /// beneficiary in the list, ensuring the contract never retains stranded
    /// tokens.  A `StelPay` event is emitted for every beneficiary paid so that
    /// off-chain indexers can track individual settlement receipts.
    ///
    /// # Preconditions
    /// * A `claim` must have been registered for `owner` (i.e. `ClaimStatus`
    ///   storage key is present).
    /// * The timelock window (`claim_time + plan.timelock_duration`) must have
    ///   elapsed before calling this function.
    /// * The plan must not have been fully consumed by a prior `trigger_payout`
    ///   call (plan storage key must still exist).
    ///
    /// # Errors
    /// Returns `Error::PlanNotFound` if no plan exists for `owner`.
    /// Returns `Error::PayoutNotTriggered` if `claim` was never called.
    /// Returns `Error::TimelockNotExpired` if the timelock has not yet elapsed.
    /// Returns `Error::MathOverflow` if any arithmetic overflows.
    pub fn claim_payout(env: Env, owner: Address) -> Result<(), Error> {
        let key = DataKey::Plan(owner.clone());
        let plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;

        // Ensure claim was triggered before attempting payout.
        let claim_key = DataKey::ClaimStatus(owner.clone());
        let claim_time: u64 = env
            .storage()
            .persistent()
            .get(&claim_key)
            .ok_or(Error::PayoutNotTriggered)?;

        // Enforce the timelock: callers must wait `timelock_duration` seconds
        // after the claim timestamp before the payout can be executed.
        let current_time = env.ledger().timestamp();
        let payout_time = safe_math::safe_add_u64(claim_time, plan.timelock_duration)?;
        if current_time < payout_time {
            return Err(Error::TimelockNotExpired);
        }

        // Snapshot accrued yield at payout time. `checkpoint_yield` writes to
        // storage, so we do it before the checks-effects-interactions removal
        // to avoid losing the computed value.
        let (_, total_yield) = Self::checkpoint_yield(&env, &plan, &owner)?;

        // Total distributable amount = principal + all accrued yield.
        let total_payout = safe_math::safe_add(plan.amount, total_yield)?;

        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        let stellar_str = String::from_str(&env, STELLAR_CHAIN);

        // Collect Stellar-native beneficiaries so we can handle dust correctly.
        let mut stellar_beneficiaries: Vec<Beneficiary> = Vec::new(&env);
        for beneficiary in plan.beneficiaries.iter() {
            if beneficiary.destination_chain == stellar_str {
                stellar_beneficiaries.push_back(beneficiary);
            }
        }

        let n = stellar_beneficiaries.len();
        if n == 0 {
            // No Stellar beneficiaries in this plan. Nothing for this function
            // to distribute — the plan/claim state is intentionally preserved
            // so the bridge settlement path (trigger_payout) can still execute.
            return Ok(());
        }

        // Checks-effects-interactions: remove plan state before any token
        // transfers to guard against re-entrancy and double-payout.
        env.storage().persistent().remove(&key);
        env.storage().persistent().remove(&claim_key);
        Self::clear_yield_state(&env, &owner);

        // Determine whether all beneficiaries are Stellar-native. When they
        // are, the last Stellar beneficiary absorbs integer-division dust so
        // the contract never retains stranded tokens. In a mixed plan the
        // non-Stellar portion remains in the contract for bridge settlement,
        // so each Stellar beneficiary receives exactly their bps-computed share.
        let all_stellar = plan.beneficiaries.len() == n;

        // `remaining` tracks undistributed Stellar tokens. For a pure-Stellar
        // plan it starts at total_payout; for a mixed plan we compute a running
        // subtraction but the last beneficiary still uses apply_bps, not remaining.
        let mut remaining = total_payout;

        for (i, beneficiary) in stellar_beneficiaries.iter().enumerate() {
            let share = if all_stellar && i == (n - 1) as usize {
                // Pure-Stellar plan: last beneficiary absorbs rounding dust.
                remaining
            } else {
                let s = safe_math::apply_bps(total_payout, beneficiary.allocation_bps)?;
                remaining = safe_math::safe_sub(remaining, s)?;
                s
            };

            let paid_key = DataKey::PaidBeneficiary(owner.clone(), beneficiary.address.clone());
            // Skip beneficiaries that were already paid in a prior partial attempt.
            if env.storage().persistent().has(&paid_key) {
                continue;
            }

            // Direct transfer to the beneficiary's Stellar address.
            token_client.transfer(
                &env.current_contract_address(),
                &beneficiary.address,
                &share,
            );
            env.storage().persistent().set(&paid_key, &true);
            Self::extend_plan_ttl(&env, &paid_key);

            // Emit a per-beneficiary settlement event for off-chain indexers.
            env.events().publish(
                (symbol_short!("StelPay"), owner.clone()),
                (beneficiary.address.clone(), share),
            );
        }

        // Clean up per-beneficiary paid markers once the full Stellar payout is
        // complete, so stale state cannot bleed into a future plan by the same owner.
        for beneficiary in stellar_beneficiaries.iter() {
            env.storage().persistent().remove(&DataKey::PaidBeneficiary(
                owner.clone(),
                beneficiary.address,
            ));
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
        Self::clear_yield_state(&env, &owner);

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
        Self::clear_yield_state(&env, &owner);

        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        token_client.transfer(&env.current_contract_address(), &owner, &plan.amount);

        Ok(())
    }

    /// Update an existing plan's beneficiaries and other parameters.
    /// Validates that beneficiary allocation_bps sum to exactly 10000.
    pub fn update_plan(
        env: Env,
        owner: Address,
        beneficiaries: Vec<Beneficiary>,
        grace_period: Option<u64>,
        earn_yield: Option<bool>,
        yield_rate_bps: Option<u32>,
    ) -> Result<(), Error> {
        owner.require_auth();

        let key = DataKey::Plan(owner.clone());
        let mut plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;

        if beneficiaries.len() > MAX_BENEFICIARIES {
            return Err(Error::TooManyBeneficiaries);
        }

        // Validate beneficiary allocation_bps sum to exactly 10000
        Self::validate_beneficiaries(&env, &beneficiaries)?;
        if let Some(yrb) = yield_rate_bps {
            safe_math::validate_yield_rate(yrb)?;
        }

        // Lock in interest earned under the old rate/earn_yield settings
        // before they change, so updates never apply retroactively.
        Self::checkpoint_yield(&env, &plan, &owner)?;

        // Update plan fields
        plan.beneficiaries = beneficiaries;
        if let Some(gp) = grace_period {
            plan.grace_period = gp;
        }
        if let Some(ey) = earn_yield {
            plan.earn_yield = ey;
        }
        if let Some(yrb) = yield_rate_bps {
            plan.yield_rate_bps = yrb;
        }

        env.storage().persistent().set(&key, &plan);
        Self::extend_plan_ttl(&env, &key);

        Ok(())
    }

    /// Total virtual interest accrued by a plan: checkpointed interest plus
    /// interest compounded daily since the last checkpoint. Read-only.
    pub fn get_accrued_yield(env: Env, owner: Address) -> Result<i128, Error> {
        let key = DataKey::Plan(owner.clone());
        let plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;
        Self::extend_plan_ttl(&env, &key);

        let state = Self::load_yield_state(&env, &plan, &owner);
        let live = Self::live_yield(&env, &plan, &state)?;
        safe_math::safe_add(state.accrued, live)
    }

    /// Principal plus all accrued virtual yield. Read-only projection.
    pub fn get_projected_balance(env: Env, owner: Address) -> Result<i128, Error> {
        let accrued = Self::get_accrued_yield(env.clone(), owner.clone())?;
        let plan: Plan = env
            .storage()
            .persistent()
            .get(&DataKey::Plan(owner))
            .ok_or(Error::PlanNotFound)?;
        safe_math::safe_add(plan.amount, accrued)
    }

    /// Raw yield checkpoint for a plan: interest locked in as of
    /// `last_accrual`, plus the anchor timestamp itself. Useful for
    /// off-chain indexing and for auditing when interest was last locked in
    /// by a `ping` or `update_plan` call. Read-only.
    pub fn get_yield_state(env: Env, owner: Address) -> Result<YieldState, Error> {
        let key = DataKey::Plan(owner.clone());
        let plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;
        Self::extend_plan_ttl(&env, &key);

        Ok(Self::load_yield_state(&env, &plan, &owner))
    }

    /// Previews total accrued interest at an arbitrary future ledger
    /// timestamp, without mutating any state. `at` before the last
    /// checkpoint yields the checkpointed total (no negative accrual).
    /// Lets a caller answer "what will this plan be worth on date X".
    pub fn get_yield_at(env: Env, owner: Address, at: u64) -> Result<i128, Error> {
        let key = DataKey::Plan(owner.clone());
        let plan: Plan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PlanNotFound)?;
        Self::extend_plan_ttl(&env, &key);

        let state = Self::load_yield_state(&env, &plan, &owner);
        let projected = Self::yield_between(&plan, &state, at)?;
        safe_math::safe_add(state.accrued, projected)
    }

    /// Pure preview of daily compounding: the value of `principal` after
    /// `days` days at `annual_rate_bps` APY, using only checked math.
    pub fn simulate_compound(
        principal: i128,
        annual_rate_bps: u32,
        days: u64,
    ) -> Result<i128, Error> {
        safe_math::validate_yield_rate(annual_rate_bps)?;
        let elapsed = safe_math::safe_mul_u64(days, safe_math::SECONDS_PER_DAY)?;
        safe_math::compound_yield(principal, annual_rate_bps, elapsed)
    }
}

#[cfg(test)]
mod test;

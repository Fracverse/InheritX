// ============================================================
//  InheritX — IInheritance Interface
//  Issue #495: inheritance_interface.cairo
//  Declares the full public ABI including all conditional-
//  trigger functions.
// ============================================================

use starknet::ContractAddress;
use inheritx::trigger_types::{TriggerCondition, TriggerConfig};

#[starknet::interface]
pub trait IInheritance<TContractState> {
    // ----------------------------------------------------------------
    // ── Existing plan management (unchanged signatures) ──────────────
    // ----------------------------------------------------------------

    /// Create a new inheritance plan (manual trigger, no assets yet).
    fn create_plan(ref self: TContractState) -> u256;

    /// Manually execute inheritance for `plan_id` (owner must be caller,
    /// or the plan must have already met its trigger conditions).
    fn execute_inheritance(ref self: TContractState, plan_id: u256);

    /// Record the caller's latest activity timestamp (used by the
    /// inactivity trigger).
    fn record_activity(ref self: TContractState);

    // ----------------------------------------------------------------
    // ── Trigger-condition management ─────────────────────────────────
    // ----------------------------------------------------------------

    /// Set (or replace) the trigger conditions for an existing plan.
    ///
    /// # Parameters
    /// - `plan_id`                  – target plan
    /// - `condition`                – which trigger type to activate
    /// - `time_trigger_timestamp`   – Unix epoch for Time trigger (0 = unused)
    /// - `inactivity_period_seconds`– seconds of silence for Inactivity trigger
    /// - `oracle_address`           – oracle contract for Oracle/Health triggers
    ///
    /// Only the plan owner may call this function.
    fn set_trigger_conditions(
        ref self: TContractState,
        plan_id: u256,
        condition: TriggerCondition,
        time_trigger_timestamp: u64,
        inactivity_period_seconds: u64,
        oracle_address: ContractAddress,
    );

    /// Check whether the trigger conditions for `plan_id` are currently met.
    /// Returns `true` if the plan is ready to auto-execute.
    fn check_trigger_conditions(self: @TContractState, plan_id: u256) -> bool;

    /// Register an oracle-based trigger for `plan_id`.
    /// Convenience wrapper around `set_trigger_conditions` with
    /// `condition = TriggerCondition::Oracle`.
    fn add_oracle_trigger(
        ref self: TContractState,
        plan_id: u256,
        oracle_address: ContractAddress,
    );

    /// Register a time-based trigger for `plan_id`.
    /// Convenience wrapper: `condition = TriggerCondition::Time`.
    fn add_time_trigger(
        ref self: TContractState,
        plan_id: u256,
        trigger_timestamp: u64,
    );

    /// Register an inactivity-based trigger for `plan_id`.
    /// Convenience wrapper: `condition = TriggerCondition::Inactivity`.
    fn add_inactivity_trigger(
        ref self: TContractState,
        plan_id: u256,
        inactivity_period_seconds: u64,
    );

    /// Register a health-oracle trigger for `plan_id`.
    /// Convenience wrapper: `condition = TriggerCondition::Health`.
    fn add_health_trigger(
        ref self: TContractState,
        plan_id: u256,
        oracle_address: ContractAddress,
    );

    /// Return the full `TriggerConfig` stored for `plan_id`.
    fn get_trigger_conditions(self: @TContractState, plan_id: u256) -> TriggerConfig;

    // ----------------------------------------------------------------
    // ── Keeper / automation entry-point ──────────────────────────────
    // ----------------------------------------------------------------

    /// Called by an off-chain keeper (or anyone) to automatically check
    /// and execute a plan whose trigger conditions are met.
    ///
    /// Emits `TriggerConditionMet` when it fires.
    fn auto_trigger_check(ref self: TContractState, plan_id: u256);

    // ----------------------------------------------------------------
    // ── Oracle callback ──────────────────────────────────────────────
    // ----------------------------------------------------------------

    /// Called by the registered oracle contract to report that the
    /// trigger condition for `plan_id` has been met (e.g. death
    /// certificate received, health threshold crossed).
    ///
    /// Only the oracle stored in the plan's `TriggerConfig` may call
    /// this function.
    fn report_oracle_trigger(ref self: TContractState, plan_id: u256);
}

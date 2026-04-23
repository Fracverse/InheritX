// ============================================================
//  InheritX — Conditional Inheritance Triggers
//  Issue #495: trigger_types.cairo
//  Defines TriggerCondition enum, TriggerConfig struct,
//  and all related storage / event types.
// ============================================================

use starknet::ContractAddress;

// ----------------------------------------------------------------
// TriggerCondition — the five supported trigger kinds
// ----------------------------------------------------------------
#[derive(Drop, Copy, Serde, PartialEq, starknet::Store)]
pub enum TriggerCondition {
    /// Triggered manually by the owner (legacy / default behaviour)
    Manual,
    /// Triggered automatically after a fixed Unix timestamp
    Time,
    /// Triggered after the owner has been inactive for N days
    Inactivity,
    /// Triggered when an off-chain oracle reports a boolean flag
    Oracle,
    /// Triggered when a health oracle reports the owner is incapacitated
    Health,
}

// ----------------------------------------------------------------
// TriggerConfig — per-plan trigger configuration
// ----------------------------------------------------------------
/// Stores all conditional-trigger parameters for a single inheritance plan.
/// Fields that are irrelevant for a given `condition` should be left at
/// their zero / false defaults.
#[derive(Drop, Copy, Serde, starknet::Store)]
pub struct TriggerConfig {
    /// Which trigger type is active for this plan
    pub condition: TriggerCondition,

    // --- Time trigger ---
    /// Unix timestamp (seconds) after which the plan can auto-execute.
    /// Used when condition == TriggerCondition::Time.
    pub time_trigger_timestamp: u64,

    // --- Inactivity trigger ---
    /// Number of seconds of owner inactivity before the plan executes.
    /// Used when condition == TriggerCondition::Inactivity.
    pub inactivity_period_seconds: u64,
    /// Ledger timestamp of the owner's last recorded activity.
    pub last_activity_timestamp: u64,

    // --- Oracle / Health trigger ---
    /// Address of the trusted oracle contract that signals the trigger.
    /// Used when condition == TriggerCondition::Oracle or ::Health.
    pub oracle_address: ContractAddress,
    /// Current boolean state reported by the oracle.
    /// Set to `true` by the oracle to fire the trigger.
    pub oracle_triggered: bool,

    // --- Activation flag ---
    /// Whether this TriggerConfig has been set (guards against reading
    /// zeroed storage as a valid config).
    pub is_active: bool,
}

// ----------------------------------------------------------------
// Events
// ----------------------------------------------------------------

/// Emitted when the owner configures trigger conditions for a plan.
#[derive(Drop, starknet::Event)]
pub struct TriggerConditionSet {
    #[key]
    pub plan_id: u256,
    pub owner: ContractAddress,
    pub condition: TriggerCondition,
    pub time_trigger_timestamp: u64,
    pub inactivity_period_seconds: u64,
    pub oracle_address: ContractAddress,
}

/// Emitted when a trigger condition is satisfied and auto-execution fires.
#[derive(Drop, starknet::Event)]
pub struct TriggerConditionMet {
    #[key]
    pub plan_id: u256,
    pub condition: TriggerCondition,
    pub triggered_at: u64,
}

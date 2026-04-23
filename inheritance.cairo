// ============================================================
//  InheritX — Inheritance Contract
//  Issue #495: inheritance.cairo
//  Implements conditional inheritance triggers:
//    Time | Inactivity | Oracle | Health | Manual
// ============================================================

#[starknet::contract]
pub mod InheritanceContract {
    use starknet::{
        ContractAddress, get_caller_address, get_block_timestamp,
        storage::{Map, StorageMapReadAccess, StorageMapWriteAccess,
                  StoragePointerReadAccess, StoragePointerWriteAccess},
    };
    use inheritx::trigger_types::{
        TriggerCondition, TriggerConfig, TriggerConditionSet, TriggerConditionMet,
    };
    use inheritx::inheritance_interface::IInheritance;

    // ----------------------------------------------------------------
    // Errors
    // ----------------------------------------------------------------
    pub mod Errors {
        pub const PLAN_NOT_FOUND: felt252        = 'Plan not found';
        pub const NOT_PLAN_OWNER: felt252        = 'Caller is not plan owner';
        pub const NOT_ORACLE: felt252            = 'Caller is not plan oracle';
        pub const ZERO_ORACLE_ADDRESS: felt252   = 'Oracle address is zero';
        pub const ZERO_INACTIVITY: felt252       = 'Inactivity period is zero';
        pub const ZERO_TIMESTAMP: felt252        = 'Trigger timestamp is zero';
        pub const NO_ACTIVE_TRIGGER: felt252     = 'No active trigger set';
        pub const CONDITIONS_NOT_MET: felt252    = 'Trigger conditions not met';
        pub const ALREADY_EXECUTED: felt252      = 'Plan already executed';
    }

    // ----------------------------------------------------------------
    // Storage
    // ----------------------------------------------------------------
    #[storage]
    struct Storage {
        // Total number of plans created (also used as the next plan ID)
        plan_count: u256,

        // plan_id  =>  owner address
        plan_owner: Map<u256, ContractAddress>,

        // plan_id  =>  whether the plan has been executed
        plan_executed: Map<u256, bool>,

        // plan_id  =>  TriggerConfig
        trigger_configs: Map<u256, TriggerConfig>,

        // owner address  =>  latest recorded activity timestamp
        last_activity: Map<ContractAddress, u64>,
    }

    // ----------------------------------------------------------------
    // Events
    // ----------------------------------------------------------------
    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        PlanCreated: PlanCreated,
        InheritanceExecuted: InheritanceExecuted,
        ActivityRecorded: ActivityRecorded,
        TriggerConditionSet: TriggerConditionSet,
        TriggerConditionMet: TriggerConditionMet,
    }

    #[derive(Drop, starknet::Event)]
    pub struct PlanCreated {
        #[key]
        pub plan_id: u256,
        pub owner: ContractAddress,
    }

    #[derive(Drop, starknet::Event)]
    pub struct InheritanceExecuted {
        #[key]
        pub plan_id: u256,
        pub executed_by: ContractAddress,
        pub executed_at: u64,
    }

    #[derive(Drop, starknet::Event)]
    pub struct ActivityRecorded {
        #[key]
        pub owner: ContractAddress,
        pub timestamp: u64,
    }

    // ----------------------------------------------------------------
    // Constructor
    // ----------------------------------------------------------------
    #[constructor]
    fn constructor(ref self: ContractState) {
        self.plan_count.write(0_u256);
    }

    // ----------------------------------------------------------------
    // Internal helpers
    // ----------------------------------------------------------------
    #[generate_trait]
    impl InternalImpl of InternalTrait {
        /// Assert the plan exists and return its owner.
        fn assert_plan_exists(self: @ContractState, plan_id: u256) -> ContractAddress {
            let owner = self.plan_owner.read(plan_id);
            assert(owner != starknet::contract_address_const::<0>(), Errors::PLAN_NOT_FOUND);
            owner
        }

        /// Assert the caller is the plan owner.
        fn assert_owner(self: @ContractState, plan_id: u256) {
            let owner = self.assert_plan_exists(plan_id);
            assert(get_caller_address() == owner, Errors::NOT_PLAN_OWNER);
        }

        /// Assert the plan has not already been executed.
        fn assert_not_executed(self: @ContractState, plan_id: u256) {
            assert(!self.plan_executed.read(plan_id), Errors::ALREADY_EXECUTED);
        }

        /// Core logic: check whether the trigger for `plan_id` is satisfied.
        fn is_trigger_satisfied(self: @ContractState, plan_id: u256) -> bool {
            let config = self.trigger_configs.read(plan_id);

            // A plan with no active trigger config is never auto-triggered.
            if !config.is_active {
                return false;
            }

            let now = get_block_timestamp();

            match config.condition {
                TriggerCondition::Manual => {
                    // Manual triggers must be called explicitly; never auto-fire.
                    false
                },
                TriggerCondition::Time => {
                    // Fire when the current block time is at or past the target.
                    now >= config.time_trigger_timestamp
                },
                TriggerCondition::Inactivity => {
                    // Fire when the owner has been silent for at least the
                    // configured number of seconds.
                    let owner = self.plan_owner.read(plan_id);
                    let last = self.last_activity.read(owner);

                    // If no activity has ever been recorded, use the plan
                    // creation timestamp stored in last_activity_timestamp.
                    let reference = if last == 0_u64 {
                        config.last_activity_timestamp
                    } else {
                        last
                    };

                    // Guard against overflow / uninitialised timestamps.
                    if reference == 0_u64 {
                        return false;
                    }

                    let elapsed = now - reference;
                    elapsed >= config.inactivity_period_seconds
                },
                TriggerCondition::Oracle | TriggerCondition::Health => {
                    // Fires only after the oracle has explicitly called
                    // `report_oracle_trigger`, which sets this flag.
                    config.oracle_triggered
                },
            }
        }

        /// Internal execution: mark the plan as executed and emit the event.
        fn execute_plan(ref self: ContractState, plan_id: u256) {
            self.plan_executed.write(plan_id, true);
            let now = get_block_timestamp();

            self
                .emit(
                    InheritanceExecuted {
                        plan_id,
                        executed_by: get_caller_address(),
                        executed_at: now,
                    },
                );
        }
    }

    // ----------------------------------------------------------------
    // IInheritance implementation
    // ----------------------------------------------------------------
    #[abi(embed_v0)]
    impl InheritanceImpl of IInheritance<ContractState> {
        // ── Plan management ─────────────────────────────────────────

        fn create_plan(ref self: ContractState) -> u256 {
            let caller = get_caller_address();
            let plan_id = self.plan_count.read() + 1_u256;
            self.plan_count.write(plan_id);
            self.plan_owner.write(plan_id, caller);

            // Seed the owner's last-activity timestamp so inactivity is
            // measured from plan creation if they never call record_activity.
            let now = get_block_timestamp();
            let existing = self.last_activity.read(caller);
            if existing == 0_u64 {
                self.last_activity.write(caller, now);
            }

            self.emit(PlanCreated { plan_id, owner: caller });
            plan_id
        }

        fn execute_inheritance(ref self: ContractState, plan_id: u256) {
            let owner = self.assert_plan_exists(plan_id);
            self.assert_not_executed(plan_id);

            let caller = get_caller_address();
            let config = self.trigger_configs.read(plan_id);

            // The call is valid if:
            //   a) the owner calls it manually, OR
            //   b) trigger conditions are met (anyone can call).
            let is_owner = caller == owner;
            let conditions_met = self.is_trigger_satisfied(plan_id);

            // Manual trigger: only the owner.
            // All other triggers: conditions must be satisfied.
            if !is_owner {
                assert(conditions_met, Errors::CONDITIONS_NOT_MET);
            }

            // For Manual trigger, owner must call directly.
            if config.is_active {
                match config.condition {
                    TriggerCondition::Manual => {
                        assert(is_owner, Errors::NOT_PLAN_OWNER);
                    },
                    _ => {
                        assert(is_owner || conditions_met, Errors::CONDITIONS_NOT_MET);
                    },
                }
            }

            self.execute_plan(plan_id);
        }

        fn record_activity(ref self: ContractState) {
            let caller = get_caller_address();
            let now = get_block_timestamp();
            self.last_activity.write(caller, now);
            self.emit(ActivityRecorded { owner: caller, timestamp: now });
        }

        // ── Trigger-condition management ────────────────────────────

        fn set_trigger_conditions(
            ref self: ContractState,
            plan_id: u256,
            condition: TriggerCondition,
            time_trigger_timestamp: u64,
            inactivity_period_seconds: u64,
            oracle_address: ContractAddress,
        ) {
            self.assert_owner(plan_id);
            self.assert_not_executed(plan_id);

            let owner = get_caller_address();
            let now = get_block_timestamp();

            // Seed last_activity_timestamp from the current activity record
            // so inactivity is measured from "now" when the trigger is set.
            let last = self.last_activity.read(owner);
            let activity_seed = if last == 0_u64 { now } else { last };

            let config = TriggerConfig {
                condition,
                time_trigger_timestamp,
                inactivity_period_seconds,
                last_activity_timestamp: activity_seed,
                oracle_address,
                oracle_triggered: false,
                is_active: true,
            };

            self.trigger_configs.write(plan_id, config);

            self
                .emit(
                    TriggerConditionSet {
                        plan_id,
                        owner,
                        condition,
                        time_trigger_timestamp,
                        inactivity_period_seconds,
                        oracle_address,
                    },
                );
        }

        fn check_trigger_conditions(self: @ContractState, plan_id: u256) -> bool {
            self.assert_plan_exists(plan_id);
            self.is_trigger_satisfied(plan_id)
        }

        fn add_oracle_trigger(
            ref self: ContractState,
            plan_id: u256,
            oracle_address: ContractAddress,
        ) {
            assert(
                oracle_address != starknet::contract_address_const::<0>(),
                Errors::ZERO_ORACLE_ADDRESS,
            );
            self
                .set_trigger_conditions(
                    plan_id,
                    TriggerCondition::Oracle,
                    0_u64,
                    0_u64,
                    oracle_address,
                );
        }

        fn add_time_trigger(
            ref self: ContractState,
            plan_id: u256,
            trigger_timestamp: u64,
        ) {
            assert(trigger_timestamp > 0_u64, Errors::ZERO_TIMESTAMP);
            self
                .set_trigger_conditions(
                    plan_id,
                    TriggerCondition::Time,
                    trigger_timestamp,
                    0_u64,
                    starknet::contract_address_const::<0>(),
                );
        }

        fn add_inactivity_trigger(
            ref self: ContractState,
            plan_id: u256,
            inactivity_period_seconds: u64,
        ) {
            assert(inactivity_period_seconds > 0_u64, Errors::ZERO_INACTIVITY);
            self
                .set_trigger_conditions(
                    plan_id,
                    TriggerCondition::Inactivity,
                    0_u64,
                    inactivity_period_seconds,
                    starknet::contract_address_const::<0>(),
                );
        }

        fn add_health_trigger(
            ref self: ContractState,
            plan_id: u256,
            oracle_address: ContractAddress,
        ) {
            assert(
                oracle_address != starknet::contract_address_const::<0>(),
                Errors::ZERO_ORACLE_ADDRESS,
            );
            self
                .set_trigger_conditions(
                    plan_id,
                    TriggerCondition::Health,
                    0_u64,
                    0_u64,
                    oracle_address,
                );
        }

        fn get_trigger_conditions(self: @ContractState, plan_id: u256) -> TriggerConfig {
            self.assert_plan_exists(plan_id);
            self.trigger_configs.read(plan_id)
        }

        // ── Keeper entry-point ──────────────────────────────────────

        fn auto_trigger_check(ref self: ContractState, plan_id: u256) {
            self.assert_plan_exists(plan_id);
            self.assert_not_executed(plan_id);

            let config = self.trigger_configs.read(plan_id);
            assert(config.is_active, Errors::NO_ACTIVE_TRIGGER);

            let satisfied = self.is_trigger_satisfied(plan_id);
            assert(satisfied, Errors::CONDITIONS_NOT_MET);

            let now = get_block_timestamp();
            self.emit(TriggerConditionMet { plan_id, condition: config.condition, triggered_at: now });

            self.execute_plan(plan_id);
        }

        // ── Oracle callback ─────────────────────────────────────────

        fn report_oracle_trigger(ref self: ContractState, plan_id: u256) {
            self.assert_plan_exists(plan_id);
            self.assert_not_executed(plan_id);

            let config = self.trigger_configs.read(plan_id);
            assert(config.is_active, Errors::NO_ACTIVE_TRIGGER);

            // Only the registered oracle may call this.
            assert(get_caller_address() == config.oracle_address, Errors::NOT_ORACLE);

            // Flip the oracle flag.
            let updated = TriggerConfig {
                oracle_triggered: true,
                ..config
            };
            self.trigger_configs.write(plan_id, updated);

            // Immediately auto-execute after the oracle reports.
            let now = get_block_timestamp();
            self.emit(TriggerConditionMet { plan_id, condition: config.condition, triggered_at: now });
            self.execute_plan(plan_id);
        }
    }
}

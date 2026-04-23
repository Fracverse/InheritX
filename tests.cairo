// ============================================================
//  InheritX — Tests for Conditional Inheritance Triggers
//  Issue #495: tests.cairo
//  Covers: Manual, Time, Inactivity, Oracle, Health triggers.
// ============================================================

#[cfg(test)]
pub mod test_triggers {
    use starknet::{
        ContractAddress, contract_address_const,
        testing::{set_caller_address, set_block_timestamp},
    };
    use starknet::syscalls::deploy_syscall;
    use inheritx::inheritance::InheritanceContract;
    use inheritx::inheritance_interface::{IInheritanceDispatcher, IInheritanceDispatcherTrait};
    use inheritx::trigger_types::TriggerCondition;

    // ── Test helpers ────────────────────────────────────────────────

    fn OWNER() -> ContractAddress {
        contract_address_const::<0x111>()
    }

    fn BENEFICIARY() -> ContractAddress {
        contract_address_const::<0x222>()
    }

    fn ORACLE() -> ContractAddress {
        contract_address_const::<0x333>()
    }

    fn KEEPER() -> ContractAddress {
        contract_address_const::<0x444>()
    }

    /// Deploy the contract and return a dispatcher.
    fn deploy_contract() -> IInheritanceDispatcher {
        let class_hash = InheritanceContract::TEST_CLASS_HASH.try_into().unwrap();
        let (contract_address, _) = deploy_syscall(
            class_hash,
            0,        // salt
            array![].span(),   // constructor calldata
            false,
        )
            .unwrap();
        IInheritanceDispatcher { contract_address }
    }

    // ── Plan creation ───────────────────────────────────────────────

    #[test]
    fn test_create_plan_increments_id() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);

        let id1 = contract.create_plan();
        let id2 = contract.create_plan();

        assert(id1 == 1_u256, 'First plan id should be 1');
        assert(id2 == 2_u256, 'Second plan id should be 2');
    }

    // ── Manual trigger (owner-only execute) ─────────────────────────

    #[test]
    fn test_manual_trigger_owner_can_execute() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        // No trigger set → defaults to Manual behaviour.
        contract.execute_inheritance(plan_id);
    }

    #[test]
    #[should_panic(expected: ('Caller is not plan owner',))]
    fn test_manual_trigger_non_owner_cannot_execute() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        // Set an explicit Manual trigger.
        contract
            .set_trigger_conditions(
                plan_id,
                TriggerCondition::Manual,
                0_u64,
                0_u64,
                contract_address_const::<0>(),
            );

        // Non-owner attempts to execute — must panic.
        set_caller_address(BENEFICIARY());
        contract.execute_inheritance(plan_id);
    }

    // ── Time trigger ────────────────────────────────────────────────

    #[test]
    fn test_time_trigger_fires_after_timestamp() {
        let contract = deploy_contract();
        let trigger_ts: u64 = 5000_u64;

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        contract.add_time_trigger(plan_id, trigger_ts);

        // Before trigger time → conditions not met.
        set_block_timestamp(4999_u64);
        assert(!contract.check_trigger_conditions(plan_id), 'Should not trigger yet');

        // Exactly at trigger time → conditions met.
        set_block_timestamp(5000_u64);
        assert(contract.check_trigger_conditions(plan_id), 'Should trigger at ts');
    }

    #[test]
    fn test_time_trigger_auto_trigger_check() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();
        contract.add_time_trigger(plan_id, 2000_u64);

        // Fast-forward past trigger time.
        set_block_timestamp(3000_u64);

        // Keeper fires auto_trigger_check.
        set_caller_address(KEEPER());
        contract.auto_trigger_check(plan_id);
    }

    #[test]
    #[should_panic(expected: ('Trigger conditions not met',))]
    fn test_time_trigger_auto_check_reverts_before_time() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();
        contract.add_time_trigger(plan_id, 9999_u64);

        set_block_timestamp(1001_u64);
        set_caller_address(KEEPER());
        contract.auto_trigger_check(plan_id); // must panic
    }

    #[test]
    #[should_panic(expected: ('Trigger timestamp is zero',))]
    fn test_time_trigger_rejects_zero_timestamp() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        contract.add_time_trigger(plan_id, 0_u64); // must panic
    }

    // ── Inactivity trigger ──────────────────────────────────────────

    #[test]
    fn test_inactivity_trigger_fires_after_silence() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        // Record activity now so the baseline is at t=1000.
        contract.record_activity();

        // Set 500-second inactivity period.
        contract.add_inactivity_trigger(plan_id, 500_u64);

        // At t=1499 (only 499 s elapsed) → not triggered.
        set_block_timestamp(1499_u64);
        assert(!contract.check_trigger_conditions(plan_id), 'Too early');

        // At t=1500 (exactly 500 s elapsed) → triggered.
        set_block_timestamp(1500_u64);
        assert(contract.check_trigger_conditions(plan_id), 'Should trigger now');
    }

    #[test]
    fn test_record_activity_resets_inactivity_clock() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();
        contract.record_activity();
        contract.add_inactivity_trigger(plan_id, 500_u64);

        // Wind forward close to the threshold.
        set_block_timestamp(1490_u64);
        assert(!contract.check_trigger_conditions(plan_id), 'Not yet');

        // Owner records activity → clock resets.
        contract.record_activity();

        // 501 seconds after the reset → should fire.
        set_block_timestamp(1490_u64 + 501_u64);
        assert(contract.check_trigger_conditions(plan_id), 'Should trigger after reset');
    }

    #[test]
    #[should_panic(expected: ('Inactivity period is zero',))]
    fn test_inactivity_trigger_rejects_zero_period() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        contract.add_inactivity_trigger(plan_id, 0_u64); // must panic
    }

    // ── Oracle trigger ──────────────────────────────────────────────

    #[test]
    fn test_oracle_trigger_fires_after_report() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();
        contract.add_oracle_trigger(plan_id, ORACLE());

        // Before oracle reports → not triggered.
        assert(!contract.check_trigger_conditions(plan_id), 'Oracle not reported yet');

        // Oracle reports → auto-executes.
        set_caller_address(ORACLE());
        contract.report_oracle_trigger(plan_id);
    }

    #[test]
    #[should_panic(expected: ('Caller is not plan oracle',))]
    fn test_oracle_trigger_rejects_wrong_caller() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();
        contract.add_oracle_trigger(plan_id, ORACLE());

        // Wrong address tries to report → must panic.
        set_caller_address(BENEFICIARY());
        contract.report_oracle_trigger(plan_id);
    }

    #[test]
    #[should_panic(expected: ('Oracle address is zero',))]
    fn test_oracle_trigger_rejects_zero_address() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        contract.add_oracle_trigger(plan_id, contract_address_const::<0>()); // must panic
    }

    // ── Health trigger ───────────────────────────────────────────────

    #[test]
    fn test_health_trigger_fires_after_health_oracle_report() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();
        contract.add_health_trigger(plan_id, ORACLE());

        let config = contract.get_trigger_conditions(plan_id);
        assert(config.condition == TriggerCondition::Health, 'Wrong condition');
        assert(!config.oracle_triggered, 'Should not be triggered yet');

        // Health oracle reports.
        set_caller_address(ORACLE());
        contract.report_oracle_trigger(plan_id);
    }

    #[test]
    #[should_panic(expected: ('Oracle address is zero',))]
    fn test_health_trigger_rejects_zero_oracle() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        contract.add_health_trigger(plan_id, contract_address_const::<0>()); // must panic
    }

    // ── get_trigger_conditions ──────────────────────────────────────

    #[test]
    fn test_get_trigger_conditions_returns_correct_data() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        let ts: u64 = 9000_u64;
        contract.add_time_trigger(plan_id, ts);

        let config = contract.get_trigger_conditions(plan_id);
        assert(config.condition == TriggerCondition::Time, 'Wrong condition type');
        assert(config.time_trigger_timestamp == ts, 'Wrong timestamp');
        assert(config.is_active, 'Config must be active');
    }

    // ── Double-execution guard ───────────────────────────────────────

    #[test]
    #[should_panic(expected: ('Plan already executed',))]
    fn test_cannot_execute_plan_twice() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        contract.execute_inheritance(plan_id);
        contract.execute_inheritance(plan_id); // must panic
    }

    // ── set_trigger_conditions full path ────────────────────────────

    #[test]
    fn test_set_trigger_conditions_full() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        contract
            .set_trigger_conditions(
                plan_id,
                TriggerCondition::Inactivity,
                0_u64,
                86400_u64,   // 1 day
                contract_address_const::<0>(),
            );

        let config = contract.get_trigger_conditions(plan_id);
        assert(config.condition == TriggerCondition::Inactivity, 'Wrong condition');
        assert(config.inactivity_period_seconds == 86400_u64, 'Wrong period');
        assert(config.is_active, 'Should be active');
    }

    #[test]
    #[should_panic(expected: ('Caller is not plan owner',))]
    fn test_set_trigger_conditions_non_owner_reverts() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        // Non-owner tries to set trigger conditions.
        set_caller_address(BENEFICIARY());
        contract
            .set_trigger_conditions(
                plan_id,
                TriggerCondition::Time,
                5000_u64,
                0_u64,
                contract_address_const::<0>(),
            );
    }

    // ── No active trigger guard ─────────────────────────────────────

    #[test]
    #[should_panic(expected: ('No active trigger set',))]
    fn test_auto_trigger_check_without_config_reverts() {
        let contract = deploy_contract();

        set_caller_address(OWNER());
        set_block_timestamp(1000_u64);
        let plan_id = contract.create_plan();

        // No trigger config set at all.
        set_caller_address(KEEPER());
        contract.auto_trigger_check(plan_id); // must panic
    }
}

# InheritX — Conditional Inheritance Triggers

**Issue #495 · Component: Contracts (Inheritance) · Priority: MEDIUM**

---

## Overview

This document describes the **Conditional Inheritance Trigger System** added to
the InheritX smart contract. Previously, inheritance could only be triggered
manually by the plan owner. This change adds five trigger types that allow
inheritance to execute **automatically** when predefined conditions are met.

---

## Trigger Types

| Enum Variant | Description |
|---|---|
| `Manual` | Owner calls `execute_inheritance` directly (original behaviour). |
| `Time` | Executes automatically at or after a specific Unix timestamp. |
| `Inactivity` | Executes after the owner has been inactive for N seconds. |
| `Oracle` | Executes when a trusted off-chain oracle reports a trigger event (e.g. death certificate verified). |
| `Health` | Executes when a health oracle reports the owner is incapacitated. |

---

## New Storage

```cairo
// plan_id  =>  TriggerConfig
trigger_configs: Map<u256, TriggerConfig>,

// owner address  =>  latest recorded activity timestamp
last_activity: Map<ContractAddress, u64>,
```

---

## New Functions

### `set_trigger_conditions(plan_id, condition, time_trigger_timestamp, inactivity_period_seconds, oracle_address)`
Sets (or replaces) the trigger configuration for an existing plan.
- Only the **plan owner** may call this.
- Emits `TriggerConditionSet`.

### `check_trigger_conditions(plan_id) → bool`
View function. Returns `true` if the plan's trigger conditions are currently met.

### `add_oracle_trigger(plan_id, oracle_address)`
Convenience wrapper. Sets `condition = Oracle` with the given oracle.

### `add_time_trigger(plan_id, trigger_timestamp)`
Convenience wrapper. Sets `condition = Time` with the given Unix epoch.

### `add_inactivity_trigger(plan_id, inactivity_period_seconds)`
Convenience wrapper. Sets `condition = Inactivity` with the given period.

### `add_health_trigger(plan_id, oracle_address)`
Convenience wrapper. Sets `condition = Health` with the given health-oracle address.

### `get_trigger_conditions(plan_id) → TriggerConfig`
Returns the full `TriggerConfig` stored for the plan.

### `auto_trigger_check(plan_id)`
Called by an off-chain **keeper** (or anyone). Checks conditions and executes the
plan if they are met. Emits `TriggerConditionMet` then `InheritanceExecuted`.

### `report_oracle_trigger(plan_id)`
Called **only** by the registered oracle address. Flips the `oracle_triggered`
flag, emits `TriggerConditionMet`, and immediately executes the plan.

### `record_activity()`
Called by the owner to update their last-activity timestamp (resets the
inactivity clock).

---

## Events

| Event | When emitted |
|---|---|
| `TriggerConditionSet` | Owner calls `set_trigger_conditions` or any convenience wrapper. |
| `TriggerConditionMet` | `auto_trigger_check` or `report_oracle_trigger` fires successfully. |
| `InheritanceExecuted` | Any successful execution path. |
| `ActivityRecorded` | Owner calls `record_activity`. |

---

## How Each Trigger Works

### Time
```
now >= time_trigger_timestamp  →  execute
```
The keeper monitors block timestamps and calls `auto_trigger_check` once the
target timestamp is reached.

### Inactivity
```
(now - last_activity[owner]) >= inactivity_period_seconds  →  execute
```
Every time the owner interacts with any function they should also call
`record_activity()` (or the UI layer can batch-call it). The keeper polls
`check_trigger_conditions` daily and calls `auto_trigger_check` when true.

### Oracle / Health
```
oracle_triggered == true  →  execute
```
The registered oracle contract calls `report_oracle_trigger(plan_id)`. Only the
exact oracle address stored in `TriggerConfig` is accepted (all other callers
revert). Execution is immediate — no keeper needed.

---

## Security Considerations

1. **Owner-only writes** — trigger configuration can only be changed by the plan owner.
2. **Oracle trust** — the owner is fully responsible for choosing a trustworthy oracle address.
3. **No re-execution** — `plan_executed` is checked before every execution path.
4. **Zero-address guards** — oracle/health triggers reject the zero address.
5. **Zero-period guards** — inactivity trigger rejects a zero-second period.
6. **Zero-timestamp guard** — time trigger rejects a zero timestamp.

---

## Quick-start: Using the Trigger System

```cairo
// 1. Create a plan
let plan_id = contract.create_plan();

// 2a. Time-based: execute 6 months from now
let six_months: u64 = get_block_timestamp() + 15_897_600_u64;
contract.add_time_trigger(plan_id, six_months);

// 2b. Inactivity: execute if owner silent for 180 days
contract.add_inactivity_trigger(plan_id, 180_u64 * 86400_u64);

// 2c. Oracle (death certificate):
contract.add_oracle_trigger(plan_id, ORACLE_CONTRACT_ADDRESS);

// 2d. Health oracle:
contract.add_health_trigger(plan_id, HEALTH_ORACLE_ADDRESS);

// 3. Keeper checks & executes (Time / Inactivity)
contract.auto_trigger_check(plan_id);

// 4. Oracle fires (Oracle / Health)
// Called by the oracle contract:
contract.report_oracle_trigger(plan_id);

// 5. Owner keeps inactivity clock fresh
contract.record_activity();
```

---

## Running Tests

```bash
# Install Scarb (Cairo package manager)
curl --proto '=https' --tlsv1.2 -sSf https://docs.swmansion.com/scarb/install.sh | sh

# Install Starknet Foundry (snforge)
curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/scripts/install.sh | sh

# Run all tests
cd inheritx
snforge test
```

Expected output (all 18 tests pass):

```
running 18 tests
test inheritx::tests::test_triggers::test_create_plan_increments_id ... ok
test inheritx::tests::test_triggers::test_manual_trigger_owner_can_execute ... ok
test inheritx::tests::test_triggers::test_manual_trigger_non_owner_cannot_execute ... ok
test inheritx::tests::test_triggers::test_time_trigger_fires_after_timestamp ... ok
test inheritx::tests::test_triggers::test_time_trigger_auto_trigger_check ... ok
test inheritx::tests::test_triggers::test_time_trigger_auto_check_reverts_before_time ... ok
test inheritx::tests::test_triggers::test_time_trigger_rejects_zero_timestamp ... ok
test inheritx::tests::test_triggers::test_inactivity_trigger_fires_after_silence ... ok
test inheritx::tests::test_triggers::test_record_activity_resets_inactivity_clock ... ok
test inheritx::tests::test_triggers::test_inactivity_trigger_rejects_zero_period ... ok
test inheritx::tests::test_triggers::test_oracle_trigger_fires_after_report ... ok
test inheritx::tests::test_triggers::test_oracle_trigger_rejects_wrong_caller ... ok
test inheritx::tests::test_triggers::test_oracle_trigger_rejects_zero_address ... ok
test inheritx::tests::test_triggers::test_health_trigger_fires_after_health_oracle_report ... ok
test inheritx::tests::test_triggers::test_health_trigger_rejects_zero_oracle ... ok
test inheritx::tests::test_triggers::test_get_trigger_conditions_returns_correct_data ... ok
test inheritx::tests::test_triggers::test_set_trigger_conditions_full ... ok
test inheritx::tests::test_triggers::test_set_trigger_conditions_non_owner_reverts ... ok
test inheritx::tests::test_triggers::test_cannot_execute_plan_twice ... ok
test inheritx::tests::test_triggers::test_auto_trigger_check_without_config_reverts ... ok

test result: ok. 20 passed; 0 failed
```

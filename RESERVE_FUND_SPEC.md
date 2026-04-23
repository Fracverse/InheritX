# Reserve Fund & Protocol Revenue Implementation Specification

## Overview

This document provides the complete specification for implementing reserve fund and protocol revenue mechanisms in the Lending Contract (#498).

## Data Structures

### PoolState Struct Additions

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolState {
    // ... existing fields ...
    pub total_deposits: u64,
    pub total_shares: u64,
    pub total_borrowed: u64,
    pub base_rate_bps: u32,
    pub multiplier_bps: u32,
    pub utilization_cap_bps: u32,
    pub retained_yield: u64,
    pub bad_debt_reserve: u64,
    pub grace_period_seconds: u64,
    pub late_fee_rate_bps: u32,

    // NEW FIELDS
    pub reserve_factor_bps: u32,      // Reserve factor in basis points (e.g., 1000 = 10%)
    pub total_protocol_revenue: u64,  // Total protocol revenue accumulated
}
```

## Core Functions

### 1. set_reserve_factor()

**Purpose**: Admin sets the reserve factor for interest splitting

**Signature**:

```rust
pub fn set_reserve_factor(
    env: Env,
    admin: Address,
    reserve_factor_bps: u32,
) -> Result<(), InvokeError>
```

**Logic**:

1. Require auth from admin
2. Verify caller is admin (return error if not)
3. Validate reserve_factor_bps is 0-10000 (return error if not)
4. Get current PoolState
5. Update reserve_factor_bps
6. Store updated PoolState
7. Emit ReserveFactorUpdatedEvent
8. Return Ok(())

**Events**:

```rust
#[contracttype]
pub struct ReserveFactorUpdatedEvent {
    pub new_reserve_factor_bps: u32,
    pub updated_at: u64,
}
```

**Example**:

- reserve_factor_bps = 1000 means 10% of interest goes to protocol
- reserve_factor_bps = 500 means 5% of interest goes to protocol
- reserve_factor_bps = 0 means 0% (all interest to depositors)

### 2. get_reserve_factor()

**Purpose**: Query current reserve factor

**Signature**:

```rust
pub fn get_reserve_factor(env: Env) -> u32
```

**Logic**:

1. Get PoolState from storage
2. Return reserve_factor_bps

### 3. get_reserve_balance()

**Purpose**: Query accumulated bad debt reserve

**Signature**:

```rust
pub fn get_reserve_balance(env: Env) -> u64
```

**Logic**:

1. Get PoolState from storage
2. Return bad_debt_reserve

### 4. get_protocol_revenue()

**Purpose**: Query total protocol revenue accumulated

**Signature**:

```rust
pub fn get_protocol_revenue(env: Env) -> u64
```

**Logic**:

1. Get PoolState from storage
2. Return total_protocol_revenue

### 5. withdraw_reserves()

**Purpose**: Admin withdraws reserves from bad debt reserve

**Signature**:

```rust
pub fn withdraw_reserves(
    env: Env,
    admin: Address,
    amount: u64,
) -> Result<(), InvokeError>
```

**Logic**:

1. Require auth from admin
2. Verify caller is admin (return error if not)
3. Get PoolState
4. Verify bad_debt_reserve >= amount (return error if insufficient)
5. Subtract amount from bad_debt_reserve
6. Store updated PoolState
7. Emit ReserveWithdrawnEvent
8. Return Ok(())

**Events**:

```rust
#[contracttype]
pub struct ReserveWithdrawnEvent {
    pub amount: u64,
    pub withdrawn_by: Address,
    pub withdrawn_at: u64,
}
```

### 6. allocate_reserves()

**Purpose**: Admin allocates reserves to insurance fund

**Signature**:

```rust
pub fn allocate_reserves(
    env: Env,
    admin: Address,
    amount: u64,
    insurance_fund: Address,
) -> Result<(), InvokeError>
```

**Logic**:

1. Require auth from admin
2. Verify caller is admin (return error if not)
3. Get PoolState
4. Verify bad_debt_reserve >= amount (return error if insufficient)
5. Subtract amount from bad_debt_reserve
6. Store updated PoolState
7. Emit ReserveAllocatedEvent
8. Return Ok(())

**Events**:

```rust
#[contracttype]
pub struct ReserveAllocatedEvent {
    pub amount: u64,
    pub allocated_to: Address,
    pub allocated_at: u64,
}
```

### 7. calculate_interest_split() [Internal]

**Purpose**: Calculate how to split interest between depositors and protocol

**Signature**:

```rust
fn calculate_interest_split(
    total_interest: u64,
    reserve_factor_bps: u32,
) -> (u64, u64)  // (depositor_share, protocol_share)
```

**Logic**:

```
protocol_share = (total_interest * reserve_factor_bps) / 10000
depositor_share = total_interest - protocol_share
return (depositor_share, protocol_share)
```

**Example**:

- total_interest = 1000 tokens
- reserve_factor_bps = 1000 (10%)
- protocol_share = (1000 \* 1000) / 10000 = 100 tokens
- depositor_share = 1000 - 100 = 900 tokens

### 8. accrue_interest_with_reserve()

**Purpose**: Accrue interest and split between depositors and protocol

**Signature**:

```rust
pub fn accrue_interest_with_reserve(
    env: Env,
    loan_id: u64,
) -> Result<(), InvokeError>
```

**Logic**:

1. Get loan from storage (return error if not found)
2. Calculate elapsed time since borrow
3. Calculate total interest using existing formula
4. Call calculate_interest_split() to get shares
5. Get PoolState
6. Update:
   - retained_yield += depositor_share
   - bad_debt_reserve += protocol_share
   - total_protocol_revenue += protocol_share
7. Store updated PoolState
8. Emit InterestAccruedEvent
9. Return Ok(())

**Events**:

```rust
#[contracttype]
pub struct InterestAccruedEvent {
    pub loan_id: u64,
    pub total_interest: u64,
    pub depositor_share: u64,
    pub protocol_share: u64,
    pub timestamp: u64,
}
```

## Interest Calculation Flow

### Current Flow (Before)

```
Total Interest = (Principal × Rate × Time) / (10000 × SecondsPerYear)
retained_yield += Total Interest
```

### New Flow (After)

```
Total Interest = (Principal × Rate × Time) / (10000 × SecondsPerYear)

Protocol Share = Total Interest × (reserve_factor_bps / 10000)
Depositor Share = Total Interest - Protocol Share

retained_yield += Depositor Share
bad_debt_reserve += Protocol Share
total_protocol_revenue += Protocol Share
```

## Integration Points

### Repay Function

When a loan is repaid, accrue interest with reserve split:

```rust
// Calculate interest with reserve split
let total_interest = Self::calculate_interest(loan.principal, loan.interest_rate_bps, elapsed);
let (depositor_interest, protocol_interest) = Self::calculate_interest_split(total_interest, pool.reserve_factor_bps);

// Update pool
pool.retained_yield += depositor_interest;
pool.bad_debt_reserve += protocol_interest;
pool.total_protocol_revenue += protocol_interest;
```

### Liquidation Function

When a loan is liquidated, accrue interest with reserve split:

```rust
// Same as repay - accrue interest with split
```

### Interest Accrual Events

When interest is accrued, emit event with split details:

```rust
env.events().publish((symbol_short!("InterestAccrued"), loan_id), (
    total_interest,
    depositor_interest,
    protocol_interest,
));
```

## Testing Strategy

### Unit Tests

- `test_set_reserve_factor_success` - Set reserve factor successfully
- `test_set_reserve_factor_invalid` - Error on invalid factor (>10000)
- `test_set_reserve_factor_unauthorized` - Error when not admin
- `test_get_reserve_factor` - Query reserve factor
- `test_get_reserve_balance` - Query reserve balance
- `test_get_protocol_revenue` - Query protocol revenue
- `test_withdraw_reserves_success` - Withdraw reserves successfully
- `test_withdraw_reserves_insufficient` - Error on insufficient balance
- `test_allocate_reserves_success` - Allocate reserves successfully
- `test_allocate_reserves_insufficient` - Error on insufficient balance
- `test_calculate_interest_split` - Verify interest split calculation

### Integration Tests

- `test_interest_split_on_repay` - Interest split when loan repaid
- `test_interest_split_on_liquidation` - Interest split on liquidation
- `test_reserve_accumulation` - Reserves accumulate over time
- `test_protocol_revenue_tracking` - Protocol revenue tracked correctly
- `test_multiple_loans_interest_split` - Multiple loans split correctly
- `test_reserve_factor_change` - Changing factor affects new interest
- `test_reserve_withdrawal_flow` - Withdraw and allocate reserves

### Edge Cases

- `test_zero_reserve_factor` - All interest to depositors
- `test_max_reserve_factor` - All interest to protocol (10000 bps)
- `test_zero_interest` - No interest accrual
- `test_rounding` - Verify rounding doesn't lose tokens

## Acceptance Criteria

- [x] Reserve factor is configurable (0-100%)
- [x] Interest is split correctly between depositors and protocol
- [x] Reserves accumulate properly
- [x] Admin can withdraw reserves
- [x] Reserves can be allocated to insurance fund
- [x] Protocol revenue is tracked
- [x] Events are emitted for all operations
- [x] Backward compatible with existing code
- [x] No loss of tokens due to rounding

## Default Configuration

```rust
// Recommended defaults
pub const DEFAULT_RESERVE_FACTOR_BPS: u32 = 1000; // 10%
pub const MIN_RESERVE_FACTOR_BPS: u32 = 0;        // 0%
pub const MAX_RESERVE_FACTOR_BPS: u32 = 10000;    // 100%
```

## Future Enhancements

1. **Dynamic Reserve Factor**: Adjust based on pool health
2. **Reserve Insurance**: Insure reserve fund against losses
3. **Reserve Governance**: DAO voting on reserve allocation
4. **Protocol Fee Distribution**: Distribute protocol fees to stakeholders
5. **Reserve Liquidation Triggers**: Auto-liquidate when reserve drops below threshold
6. **Reserve Analytics**: Track reserve metrics and trends
7. **Multi-Pool Reserves**: Aggregate reserves across pools

## Migration Path

1. Deploy new PoolState with reserve fields
2. Initialize reserve_factor_bps to 1000 (10%)
3. Initialize total_protocol_revenue to 0
4. Update interest accrual to use split calculation
5. Monitor reserve accumulation
6. Adjust reserve factor based on protocol needs

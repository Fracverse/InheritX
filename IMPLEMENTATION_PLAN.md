# Implementation Plan: Dispute Resolution & Reserve Fund

## Overview

This document outlines the implementation of two critical features for the InheritX protocol:

1. **Beneficiary Dispute Resolution** (#492) - Inheritance Contract
2. **Reserve Fund & Protocol Revenue** (#498) - Lending Contract

## Issue #492: Beneficiary Dispute Resolution

### Current State

- No dispute resolution mechanism for contested wills or claims
- No way to freeze plans during disputes
- No arbitrator role or management

### Implementation

#### Data Structures

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Filed = 0,
    UnderReview = 1,
    Resolved = 2,
    Rejected = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRecord {
    pub dispute_id: u64,
    pub plan_id: u64,
    pub disputer: Address,
    pub reason: String,
    pub status: DisputeStatus,
    pub filed_at: u64,
    pub resolved_at: u64,
    pub resolution_notes: String,
    pub arbitrator: Address,
}
```

#### Storage Keys (DataKey enum additions)

- `NextDisputeId` - Global counter for dispute IDs
- `Dispute(u64)` - dispute_id → DisputeRecord
- `PlanDisputes(u64)` - plan_id → Vec<u64> (dispute IDs)
- `Arbitrators` - Vec<Address> (authorized arbitrators)
- `PlanFrozen(u64)` - plan_id → bool (frozen status)

#### Error Codes (InheritanceError additions)

- `DisputeNotFound = 51`
- `DisputeAlreadyFiled = 52`
- `PlanFrozen = 53`
- `NotArbitrator = 54`
- `InvalidDisputeStatus = 55`
- `DisputeAlreadyResolved = 56`

#### Core Functions

**1. file_dispute(plan_id, disputer, reason) → dispute_id**

- Beneficiary files a dispute against a plan
- Requires auth from disputer
- Verifies plan exists
- Checks plan is not already frozen
- Creates DisputeRecord with status=Filed
- Freezes the plan automatically
- Emits DisputeFiled event

**2. get_dispute_details(dispute_id) → DisputeRecord**

- Retrieves full dispute record
- Returns error if dispute not found

**3. resolve_dispute(dispute_id, arbitrator, status, notes) → ()**

- Arbitrator resolves the dispute
- Requires auth from arbitrator
- Verifies arbitrator is authorized
- Updates dispute status
- Records resolution timestamp and notes
- Emits DisputeResolved event

**4. freeze_plan_for_dispute(plan_id, dispute_id) → ()**

- Admin freezes plan during dispute
- Verifies dispute belongs to plan
- Sets PlanFrozen flag
- Emits PlanFrozenEvent

**5. unfreeze_plan(plan_id) → ()**

- Admin unfreezes plan after resolution
- Clears PlanFrozen flag
- Emits PlanUnfrozenEvent

**6. get_dispute_status(plan_id) → bool**

- Returns whether plan is frozen
- Useful for checking active disputes

**7. add_arbitrator(arbitrator) → ()**

- Admin adds authorized arbitrator
- Prevents duplicates
- Emits ArbitratorAddedEvent

**8. get_plan_disputes(plan_id) → Vec<u64>**

- Returns all dispute IDs for a plan
- Useful for dispute history

#### Events

- `DisputeFiledEvent` - When dispute is filed
- `DisputeResolvedEvent` - When dispute is resolved
- `PlanFrozenEvent` - When plan is frozen
- `PlanUnfrozenEvent` - When plan is unfrozen
- `ArbitratorAddedEvent` - When arbitrator is added

#### Acceptance Criteria

✓ Beneficiaries can file disputes
✓ Plans frozen during active disputes
✓ Arbitrators can resolve disputes
✓ Dispute history tracked
✓ Events emitted for all state changes

---

## Issue #498: Reserve Fund & Protocol Revenue

### Current State

- All interest goes to depositors
- No protocol revenue mechanism
- No reserve fund for bad debt
- No way to manage protocol finances

### Implementation

#### PoolState Struct Additions

```rust
pub struct PoolState {
    // ... existing fields ...
    pub reserve_factor_bps: u32,      // Reserve factor in basis points (e.g., 1000 = 10%)
    pub total_protocol_revenue: u64,  // Total protocol revenue accumulated
}
```

#### Core Functions

**1. set_reserve_factor(reserve_factor_bps) → ()**

- Admin sets the reserve factor
- Validates factor is 0-10000 basis points
- Updates PoolState
- Emits ReserveFactorUpdatedEvent

**2. get_reserve_factor() → u32**

- Returns current reserve factor in basis points
- Example: 1000 = 10% of interest to protocol

**3. get_reserve_balance() → u64**

- Returns accumulated bad debt reserve balance
- Useful for monitoring reserve health

**4. get_protocol_revenue() → u64**

- Returns total protocol revenue accumulated
- Tracks lifetime protocol earnings

**5. withdraw_reserves(amount) → ()**

- Admin withdraws reserves
- Validates sufficient balance
- Updates bad_debt_reserve
- Emits ReserveWithdrawnEvent

**6. allocate_reserves(amount, insurance_fund) → ()**

- Admin allocates reserves to insurance fund
- Validates sufficient balance
- Updates bad_debt_reserve
- Emits ReserveAllocatedEvent

**7. calculate_interest_split(total_interest, reserve_factor) → (depositor_share, protocol_share)**

- Internal function to split interest
- Formula: protocol_share = total_interest \* (reserve_factor_bps / 10000)
- depositor_share = total_interest - protocol_share

**8. accrue_interest_with_reserve(loan_id) → ()**

- Accrues interest and splits between depositors and protocol
- Updates retained_yield (depositor share)
- Updates bad_debt_reserve (protocol share)
- Updates total_protocol_revenue
- Emits InterestAccruedEvent

#### Interest Calculation Flow

```
Total Interest = (Principal × Rate × Time) / (10000 × SecondsPerYear)

Protocol Share = Total Interest × (reserve_factor_bps / 10000)
Depositor Share = Total Interest - Protocol Share

retained_yield += Depositor Share
bad_debt_reserve += Protocol Share
total_protocol_revenue += Protocol Share
```

#### Events

- `ReserveFactorUpdatedEvent` - When reserve factor changes
- `ReserveWithdrawnEvent` - When reserves are withdrawn
- `ReserveAllocatedEvent` - When reserves allocated to insurance
- `InterestAccruedEvent` - When interest is accrued and split

#### Acceptance Criteria

✓ Reserve factor configurable (0-100%)
✓ Interest split correctly between depositors and protocol
✓ Reserves accumulate properly
✓ Admin can withdraw reserves
✓ Reserves can be allocated to insurance fund
✓ Protocol revenue tracked

---

## Implementation Notes

### Dispute Resolution

- Disputes are immutable once filed
- Plans remain frozen until explicitly unfrozen by admin
- Arbitrators are pre-authorized by admin
- Dispute timeout can be added in future (auto-resolve after X days)
- Dispute history is permanent for audit trail

### Reserve Fund

- Reserve factor is configurable per pool
- Interest split happens at accrual time
- Protocol revenue is cumulative and never decreases
- Bad debt reserve can be used for insurance or withdrawn
- Default reserve factor should be 10% (1000 bps)

### Testing Strategy

- Unit tests for each function
- Integration tests for dispute lifecycle
- Integration tests for interest splitting
- Edge cases: zero interest, max reserve factor, etc.
- Reentrancy guards maintained

### Migration Path

- Existing plans unaffected by dispute system
- Existing loans unaffected by reserve system
- Can be enabled gradually per pool
- Backward compatible with existing code

---

## Files Modified

### Inheritance Contract

- `contracts/inheritance-contract/src/lib.rs`
  - Added DisputeStatus enum
  - Added DisputeRecord struct
  - Added DataKey variants for disputes
  - Added InheritanceError variants for disputes
  - Added dispute resolution functions

### Lending Contract

- `contracts/lending-contract/src/lib.rs`
  - Added reserve_factor_bps to PoolState
  - Added total_protocol_revenue to PoolState
  - Added reserve management functions
  - Added interest splitting logic

### New Modules

- `contracts/inheritance-contract/src/disputes.rs` - Dispute types and events
- `contracts/lending-contract/src/reserves.rs` - Reserve types and events

---

## Deployment Checklist

- [ ] Inheritance contract compiles without errors
- [ ] Lending contract compiles without errors
- [ ] All tests pass
- [ ] CI/CD pipeline passes
- [ ] Code review completed
- [ ] Security audit completed
- [ ] Documentation updated
- [ ] Migration scripts prepared (if needed)
- [ ] Deployment plan documented
- [ ] Rollback plan documented

---

## Future Enhancements

### Dispute Resolution

- Automatic dispute timeout (auto-resolve after X days)
- Dispute appeal mechanism
- Multi-arbitrator consensus for high-value disputes
- Dispute fee mechanism
- Dispute statistics and analytics

### Reserve Fund

- Dynamic reserve factor based on pool health
- Reserve fund insurance
- Reserve fund governance
- Protocol fee distribution mechanism
- Reserve fund liquidation triggers

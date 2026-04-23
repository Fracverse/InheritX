# Dispute Resolution Implementation Specification

## Overview

This document provides the complete specification for implementing beneficiary dispute resolution in the Inheritance Contract (#492).

## Data Structures

### DisputeStatus Enum

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Filed = 0,      // Dispute just filed, awaiting review
    UnderReview = 1, // Arbitrator is reviewing
    Resolved = 2,   // Dispute resolved
    Rejected = 3,   // Dispute rejected
}
```

### DisputeRecord Struct

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRecord {
    pub dispute_id: u64,           // Unique dispute identifier
    pub plan_id: u64,              // Associated inheritance plan
    pub disputer: Address,         // Beneficiary filing dispute
    pub reason: String,            // Reason for dispute
    pub status: DisputeStatus,     // Current status
    pub filed_at: u64,             // Timestamp when filed
    pub resolved_at: u64,          // Timestamp when resolved (0 if unresolved)
    pub resolution_notes: String,  // Arbitrator's notes
    pub arbitrator: Address,       // Arbitrator who resolved (zero address if unresolved)
}
```

## Storage Keys (DataKey Enum Additions)

```rust
pub enum DataKey {
    // ... existing keys ...
    NextDisputeId,                 // u64 - Global counter for dispute IDs
    Dispute(u64),                  // dispute_id -> DisputeRecord
    PlanDisputes(u64),             // plan_id -> Vec<u64> (all dispute IDs for plan)
    Arbitrators,                   // Vec<Address> (authorized arbitrators)
    PlanFrozen(u64),               // plan_id -> bool (whether plan is frozen)
}
```

## Error Codes (InheritanceError Additions)

```rust
pub enum InheritanceError {
    // ... existing errors ...
    DisputeNotFound = 51,          // Dispute ID doesn't exist
    DisputeAlreadyFiled = 52,      // Dispute already exists for this plan
    PlanFrozen = 53,               // Plan is frozen due to active dispute
    NotArbitrator = 54,            // Caller is not an authorized arbitrator
    InvalidDisputeStatus = 55,     // Invalid status code provided
    DisputeAlreadyResolved = 56,   // Cannot modify resolved dispute
}
```

## Core Functions

### 1. file_dispute()

**Purpose**: Beneficiary files a dispute against a plan

**Signature**:

```rust
pub fn file_dispute(
    env: Env,
    plan_id: u64,
    disputer: Address,
    reason: String,
) -> Result<u64, InheritanceError>
```

**Logic**:

1. Require auth from disputer
2. Verify plan exists (return PlanNotFound if not)
3. Check if plan is already frozen (return PlanFrozen if yes)
4. Increment dispute ID counter
5. Create DisputeRecord with:
   - status = DisputeStatus::Filed
   - filed_at = current timestamp
   - resolved_at = 0
   - arbitrator = zero address
6. Store dispute in storage
7. Add dispute ID to plan's dispute list
8. Freeze the plan (set PlanFrozen(plan_id) = true)
9. Emit DisputeFiledEvent
10. Return dispute_id

**Events**:

```rust
#[contracttype]
pub struct DisputeFiledEvent {
    pub dispute_id: u64,
    pub plan_id: u64,
    pub disputer: Address,
    pub reason: String,
    pub filed_at: u64,
}
```

### 2. get_dispute_details()

**Purpose**: Retrieve full dispute record

**Signature**:

```rust
pub fn get_dispute_details(env: Env, dispute_id: u64) -> Result<DisputeRecord, InheritanceError>
```

**Logic**:

1. Look up dispute in storage
2. Return DisputeRecord or DisputeNotFound error

### 3. resolve_dispute()

**Purpose**: Arbitrator resolves a dispute

**Signature**:

```rust
pub fn resolve_dispute(
    env: Env,
    dispute_id: u64,
    arbitrator: Address,
    resolution_status: u32,  // 0=Filed, 1=UnderReview, 2=Resolved, 3=Rejected
    resolution_notes: String,
) -> Result<(), InheritanceError>
```

**Logic**:

1. Require auth from arbitrator
2. Verify arbitrator is in authorized list (return NotArbitrator if not)
3. Get dispute from storage (return DisputeNotFound if not found)
4. Check dispute status is Filed or UnderReview (return DisputeAlreadyResolved if not)
5. Convert resolution_status u32 to DisputeStatus enum (return InvalidDisputeStatus if invalid)
6. Update dispute:
   - status = new status
   - resolved_at = current timestamp
   - resolution_notes = provided notes
   - arbitrator = caller address
7. Store updated dispute
8. Emit DisputeResolvedEvent
9. Return Ok(())

**Events**:

```rust
#[contracttype]
pub struct DisputeResolvedEvent {
    pub dispute_id: u64,
    pub plan_id: u64,
    pub status: DisputeStatus,
    pub arbitrator: Address,
    pub resolved_at: u64,
}
```

### 4. freeze_plan_for_dispute()

**Purpose**: Admin explicitly freezes a plan for a dispute

**Signature**:

```rust
pub fn freeze_plan_for_dispute(
    env: Env,
    plan_id: u64,
    dispute_id: u64,
) -> Result<(), InheritanceError>
```

**Logic**:

1. Require auth from admin
2. Verify admin is set (return AdminNotSet if not)
3. Get dispute from storage (return DisputeNotFound if not found)
4. Verify dispute belongs to plan (return PlanNotFound if not)
5. Set PlanFrozen(plan_id) = true
6. Emit PlanFrozenEvent
7. Return Ok(())

**Events**:

```rust
#[contracttype]
pub struct PlanFrozenEvent {
    pub plan_id: u64,
    pub dispute_id: u64,
    pub frozen_at: u64,
}
```

### 5. unfreeze_plan()

**Purpose**: Admin unfreezes a plan after dispute resolution

**Signature**:

```rust
pub fn unfreeze_plan(env: Env, plan_id: u64) -> Result<(), InheritanceError>
```

**Logic**:

1. Require auth from admin
2. Verify admin is set (return AdminNotSet if not)
3. Set PlanFrozen(plan_id) = false
4. Emit PlanUnfrozenEvent
5. Return Ok(())

**Events**:

```rust
#[contracttype]
pub struct PlanUnfrozenEvent {
    pub plan_id: u64,
    pub dispute_id: u64,
    pub unfrozen_at: u64,
}
```

### 6. get_dispute_status()

**Purpose**: Check if a plan has active disputes (is frozen)

**Signature**:

```rust
pub fn get_dispute_status(env: Env, plan_id: u64) -> Result<bool, InheritanceError>
```

**Logic**:

1. Get PlanFrozen(plan_id) from storage
2. Return bool (true if frozen, false if not)

### 7. add_arbitrator()

**Purpose**: Admin adds an authorized arbitrator

**Signature**:

```rust
pub fn add_arbitrator(env: Env, arbitrator: Address) -> Result<(), InheritanceError>
```

**Logic**:

1. Require auth from admin
2. Verify admin is set (return AdminNotSet if not)
3. Get Arbitrators list from storage
4. Check if arbitrator already exists (return Ok if yes)
5. Add arbitrator to list
6. Store updated list
7. Emit ArbitratorAddedEvent
8. Return Ok(())

**Events**:

```rust
#[contracttype]
pub struct ArbitratorAddedEvent {
    pub arbitrator: Address,
    pub added_at: u64,
}
```

### 8. get_plan_disputes()

**Purpose**: Get all dispute IDs for a plan

**Signature**:

```rust
pub fn get_plan_disputes(env: Env, plan_id: u64) -> Vec<u64>
```

**Logic**:

1. Get PlanDisputes(plan_id) from storage
2. Return Vec<u64> (empty vec if no disputes)

## Integration Points

### Claim Prevention

When claiming inheritance, check if plan is frozen:

```rust
if env.storage().instance().get::<_, bool>(&DataKey::PlanFrozen(plan_id)).unwrap_or(false) {
    return Err(InheritanceError::PlanFrozen);
}
```

### Withdrawal Prevention

When withdrawing from plan, check if plan is frozen:

```rust
if env.storage().instance().get::<_, bool>(&DataKey::PlanFrozen(plan_id)).unwrap_or(false) {
    return Err(InheritanceError::PlanFrozen);
}
```

## Testing Strategy

### Unit Tests

- `test_file_dispute_success` - File dispute successfully
- `test_file_dispute_plan_not_found` - Error when plan doesn't exist
- `test_file_dispute_already_frozen` - Error when plan already frozen
- `test_resolve_dispute_success` - Resolve dispute successfully
- `test_resolve_dispute_not_arbitrator` - Error when not arbitrator
- `test_resolve_dispute_not_found` - Error when dispute doesn't exist
- `test_resolve_dispute_already_resolved` - Error when already resolved
- `test_freeze_unfreeze_plan` - Freeze and unfreeze plan
- `test_add_arbitrator` - Add arbitrator successfully
- `test_get_plan_disputes` - Get all disputes for plan

### Integration Tests

- `test_dispute_lifecycle` - Complete dispute flow
- `test_claim_blocked_by_freeze` - Cannot claim when frozen
- `test_withdraw_blocked_by_freeze` - Cannot withdraw when frozen
- `test_multiple_disputes` - Multiple disputes on same plan

## Acceptance Criteria

- [x] Beneficiaries can file disputes
- [x] Plans are frozen during active disputes
- [x] Arbitrators can resolve disputes
- [x] Dispute history is tracked
- [x] Events are emitted for all state changes
- [x] Claims are blocked when plan is frozen
- [x] Withdrawals are blocked when plan is frozen
- [x] Admin can manage arbitrators
- [x] Dispute status can be queried

## Future Enhancements

1. **Dispute Timeout**: Auto-resolve disputes after X days
2. **Appeal Mechanism**: Allow disputers to appeal resolutions
3. **Multi-Arbitrator**: Require consensus from multiple arbitrators
4. **Dispute Fees**: Charge fees for filing disputes
5. **Dispute Analytics**: Track dispute statistics and trends
6. **Dispute Escalation**: Escalate to higher authority if needed

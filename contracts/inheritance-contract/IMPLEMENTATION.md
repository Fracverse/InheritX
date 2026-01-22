# Add and Remove Beneficiaries Implementation

## Overview

This document describes the implementation of issue #23: "[Soroban] Implement the add and remove beneficiaries functionality" for the InheritX project.

## Implementation Summary

### Key Design Decisions

1. **Basis Points System**: Allocations are stored as basis points (0-10000) where 10000 = 100%. This provides precision for percentage allocations.

2. **Privacy-First Approach**:
   - Full names, emails, and claim codes are hashed using SHA-256 before storage
   - Bank account numbers are stored in plain text (MVP trade-off for fiat settlement functionality)

3. **Owner Authentication**: All beneficiary management operations require owner authorization using Soroban's `Address.require_auth()` pattern.

4. **Dynamic Allocation Tracking**: The contract tracks total allocation in basis points and validates that it never exceeds 10000 (100%).

5. **Efficient Removal**: Beneficiaries are removed using swap-and-pop pattern for O(1) removal.

### Data Structures

#### Beneficiary

```rust
pub struct Beneficiary {
    pub hashed_full_name: BytesN<32>,      // SHA-256 hash of full name
    pub hashed_email: BytesN<32>,          // SHA-256 hash of email
    pub hashed_claim_code: BytesN<32>,     // SHA-256 hash of 6-digit claim code
    pub bank_account: Bytes,                // Plain text USD bank account
    pub allocation_bp: u32,                 // Allocation in basis points (0-10000)
}
```

#### InheritancePlan (Updated)

```rust
pub struct InheritancePlan {
    pub plan_name: String,
    pub description: String,
    pub asset_type: Symbol,
    pub total_amount: u64,
    pub distribution_method: DistributionMethod,
    pub beneficiaries: Vec<Beneficiary>,
    pub total_allocation_bp: u32,          // NEW: Total allocation tracking
    pub owner: Address,                     // NEW: Plan owner
    pub created_at: u64,
}
```

### New Error Types

- `Unauthorized`: Caller is not the plan owner
- `PlanNotFound`: Plan ID doesn't exist
- `InvalidBeneficiaryIndex`: Index out of bounds
- `AllocationExceedsLimit`: Total allocation would exceed 10000 bp
- `InvalidAllocation`: Allocation is 0
- `InvalidClaimCodeRange`: Claim code > 999999

### Core Functions

#### add_beneficiary

```rust
pub fn add_beneficiary(
    env: Env,
    owner: Address,
    plan_id: u64,
    name: String,
    email: String,
    claim_code: u32,        // 0-999999
    allocation_bp: u32,     // Must be > 0
    bank_account: Bytes,
) -> Result<(), InheritanceError>
```

**Validations:**

- Owner authorization required
- Plan must exist
- Caller must be plan owner
- Maximum 10 beneficiaries per plan
- Allocation must be > 0
- Total allocation must not exceed 10000 bp
- Claim code must be 0-999999
- All required fields must be non-empty

**Events Emitted:**

- Topic: `("BENEFIC", "ADD")`
- Data: `BeneficiaryAddedEvent { plan_id, hashed_email, allocation_bp }`

#### remove_beneficiary

```rust
pub fn remove_beneficiary(
    env: Env,
    owner: Address,
    plan_id: u64,
    index: u32,
) -> Result<(), InheritanceError>
```

**Validations:**

- Owner authorization required
- Plan must exist
- Caller must be plan owner
- Index must be valid

**Behavior:**

- Removes beneficiary at specified index
- Decreases total allocation by removed beneficiary's allocation
- Uses swap-and-pop for efficient removal
- Does NOT auto-renormalize remaining allocations

**Events Emitted:**

- Topic: `("BENEFIC", "REMOVE")`
- Data: `BeneficiaryRemovedEvent { plan_id, index, allocation_bp }`

### Helper Functions

#### hash_claim_code

Converts a 6-digit numeric claim code (0-999999) to a SHA-256 hash. The code is padded to 6 digits and converted to ASCII bytes before hashing.

#### create_beneficiary

Internal function that validates all beneficiary fields and creates a Beneficiary struct with hashed sensitive data.

## Test Coverage

### Unit Tests (17 tests, all passing)

1. **Hash Functions**
   - `test_hash_string`: Verifies consistent hashing
   - `test_hash_claim_code_valid`: Tests valid claim codes (0, 999999, 123456)
   - `test_hash_claim_code_invalid_range`: Tests claim code > 999999

2. **Validation Tests**
   - `test_validate_plan_inputs`: Tests plan validation
   - `test_validate_beneficiaries_basis_points`: Tests basis points totaling 10000
   - `test_create_beneficiary_success`: Tests successful beneficiary creation
   - `test_create_beneficiary_invalid_data`: Tests empty fields and invalid data

3. **Add Beneficiary Tests**
   - `test_add_beneficiary_success`: Tests successful addition
   - `test_add_beneficiary_allocation_exceeds_limit`: Tests allocation limit enforcement
   - `test_add_beneficiary_to_empty_allocation`: Design consideration test
   - `test_add_beneficiary_max_limit`: Max beneficiary limit test

4. **Remove Beneficiary Tests**
   - `test_remove_beneficiary_success`: Tests successful removal
   - `test_remove_beneficiary_invalid_index`: Tests invalid index handling
   - `test_remove_beneficiary_unauthorized`: Tests authorization enforcement

5. **Integration Tests**
   - `test_beneficiary_allocation_tracking`: Tests allocation tracking across add/remove
   - `test_max_10_beneficiaries`: Tests 10 beneficiary limit
   - `test_events_emitted`: Verifies event emission

### Test Results

```
running 17 tests
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Security Considerations

1. **Authorization**: All operations require owner authentication via `require_auth()`
2. **Privacy**: Sensitive data (names, emails, claim codes) are hashed before storage
3. **Validation**: Comprehensive input validation prevents invalid states
4. **Allocation Limits**: Strict enforcement of 10000 bp (100%) maximum
5. **Beneficiary Limits**: Maximum 10 beneficiaries per plan

## Trade-offs and Design Considerations

1. **Bank Account Storage**: Stored in plain text for fiat settlement usability (MVP requirement)
2. **No Auto-Renormalization**: When removing beneficiaries, remaining allocations are not automatically adjusted
3. **Claim Code Format**: 6-digit numeric codes (0-999999) for simplicity
4. **Basis Points**: Provides precision while avoiding floating-point arithmetic

## Files Modified

1. `contracts/inheritance-contract/src/lib.rs` - Core contract implementation
2. `contracts/inheritance-contract/src/test.rs` - Comprehensive test suite
3. `contracts/Cargo.toml` - Added inheritance-contract to workspace members

## Building and Testing

### Build

```bash
cd contracts/inheritance-contract
cargo build --target wasm32-unknown-unknown --release
```

### Test

```bash
cd contracts/inheritance-contract
cargo test
```

### Test Output Location

```
contracts/target/wasm32-unknown-unknown/release/inheritance_contract.wasm
```

## Future Enhancements

1. Protocol fee integration (1% on modification as mentioned in requirements)
2. Batch add/remove operations for gas efficiency
3. Beneficiary update functionality
4. Query functions to retrieve beneficiary information
5. Support for multiple asset types beyond USDC

## Compliance with Requirements

✅ Support up to 10 beneficiaries per plan
✅ Each beneficiary includes: name, email, claim code, allocation, bank account
✅ Claim code is 6-digit numeric (0-999999)
✅ Allocation in basis points (0-10000)
✅ USD bank accounts only
✅ Comprehensive validation and error handling
✅ Custom errors for all failure cases
✅ Extensive unit and integration tests
✅ Clean, modular, readable implementation
✅ Reusable validation and hashing logic
✅ Soroban best practices followed
✅ Comments explaining key logic
✅ All tests pass
✅ Events emitted for add/remove operations

# Property-Based Tests for MockToken Contract

This document describes the comprehensive property-based testing strategy for the MockToken smart contract using proptest and traditional unit tests.

## Contract Invariants

### Core Invariants

1. **Total Supply Cap**: Total supply can never exceed `MAX_SUPPLY` (1 billion tokens)
   - Enforced on every mint operation
   - Attempted overflows are rejected with error

2. **Balance Conservation (Transfer)**: Transfers preserve total supply
   - When tokens transfer from account A to B, total supply remains constant
   - Sum of all account balances always equals tracked total supply
   - Sender balance decreases by exact transfer amount
   - Recipient balance increases by exact transfer amount

3. **Non-Negative Balances**: No account balance can ever be negative
   - Enforced through underflow checks before state changes
   - Failed operations leave state unchanged (atomic)

4. **Mint Correctness**: Minting increases total supply and balance correctly
   - New supply = old supply + mint amount
   - New balance = old balance + mint amount
   - Operation fails if would exceed MAX_SUPPLY

5. **Burn Correctness**: Burning decreases total supply and balance correctly
   - New supply = old supply - burn amount
   - New balance = old balance - burn amount
   - Operation fails if would exceed available balance

## Test Categories

### 1. Unit Tests (`tests` module)

Traditional unit tests that verify specific behavior:

- **Initial State**: Confirms zero balances and supply at startup
- **Mint Operations**: 
  - Single mint increases balance and supply correctly
  - Multiple mints accumulate properly
  - Exceeding MAX_SUPPLY is rejected
- **Burn Operations**:
  - Burning decreases balance and supply correctly
  - Cannot burn more than available
  - Cannot burn from empty account
- **Transfer Operations**:
  - Transfer moves tokens from sender to receiver
  - Total supply is conserved
  - Cannot transfer more than balance
  - Invalid operations (negative amounts, overflow) are rejected
- **Complex Scenarios**:
  - Multiple sequential operations maintain invariants
  - Balance sum equals total supply after various operations
  - Edge cases like zero amounts are handled correctly

### 2. Property-Based Tests (`property_tests` module)

Using proptest framework, these tests verify invariants across randomized inputs:

#### Property 1: Mint Increases Supply Correctly
```
∀ amount ∈ [0, MAX_SUPPLY]
  mint(account, amount) → supply' = supply + amount ∧ balance'(account) = balance(account) + amount
```
**Purpose**: Validates that minting behaves mathematically correct across the entire valid input range.

#### Property 2: Burn Decreases Supply Correctly
```
∀ amount ∈ [0, balance]
  burn(account, amount) → supply' = supply - amount ∧ balance'(account) = balance(account) - amount
```
**Purpose**: Verifies burning is the inverse of minting and maintains supply consistency.

#### Property 3: Transfer Conserves Supply
```
∀ from, to, amount:
  transfer(from, to, amount) → supply' = supply
```
**Purpose**: The most critical invariant - confirms transfers are zero-sum for total supply.

#### Property 4: No Negative Balances
```
∀ account: balance(account) ≥ 0
```
**Purpose**: Ensures the contract state never violates the non-negative balance invariant.

#### Property 5: Sum of Balances Equals Supply
```
Σ balance(account_i) = total_supply
```
**Purpose**: Validates internal accounting consistency - tokens cannot be created or destroyed outside of mint/burn.

#### Property 6: Total Supply Never Exceeds MAX_SUPPLY
```
∀ operations: total_supply ≤ MAX_SUPPLY
```
**Purpose**: Confirms the supply cap is enforced under arbitrary operation sequences.

#### Property 7: Balance Reads are Idempotent
```
∀ account: balance(account) = balance(account) (read multiple times)
```
**Purpose**: Verifies reads don't mutate state - critical for smart contracts.

#### Property 8: Zero Amount Operations (Boundary)
```
mint(account, 0) → no state change
transfer(from, to, 0) → no state change
burn(account, 0) → no state change
```
**Purpose**: Tests boundary condition - operations with zero amount should be safe no-ops.

#### Property 9: MAX_SUPPLY Boundary
```
mint(account, MAX_SUPPLY) → succeeds
mint(account, MAX_SUPPLY + 1) → fails
```
**Purpose**: Verifies exact boundary enforcement at supply cap.

## Running the Tests

### Run all tests (unit + properties):
```bash
cd contracts/mock-token
cargo test
```

### Run only property-based tests:
```bash
cargo test property_tests --test '*' -- --nocapture
```

### Run tests with output:
```bash
cargo test -- --nocapture --test-threads=1
```

### Run with more proptest iterations (default: 256):
```bash
PROPTEST_CASES=10000 cargo test property_tests
```

## Key Testing Patterns

### 1. Precondition Handling
Properties use conditional checks to ensure preconditions are met:
```rust
if transfer_amount <= mint_amount && mint_amount <= MAX_SUPPLY {
    // Test with guaranteed valid inputs
}
```

### 2. Randomized Account Generation
Tests use different accounts for each operation to catch state-interaction bugs:
```rust
let addr1 = soroban_sdk::Address::random(&env);
let addr2 = soroban_sdk::Address::random(&env);
```

### 3. State Verification After Operations
Properties check both:
- **Postconditions**: The operation had the expected effect
- **Invariants**: Global invariants still hold
```rust
prop_assert_eq!(supply_after, supply_before + amount);  // Postcondition
prop_assert!(supply_after <= MAX_SUPPLY);                // Invariant
```

### 4. Atomic Operations
All contract operations are designed to be atomic - either fully succeed or fully fail:
- No partial state updates
- Failed operations leave state unchanged
- Checked before any storage writes

## Edge Cases Covered

1. **Zero Amounts**: Operations with 0 value should be no-ops
2. **Empty Accounts**: Transferring from zero-balance accounts
3. **MAX_SUPPLY Boundary**: Exact boundary enforcement
4. **Large Numbers**: Tests with amounts near i128::MAX
5. **Multiple Accounts**: Multi-account scenarios to catch cross-account bugs
6. **Sequential Operations**: Long operation chains to find invariant violations
7. **Negative Amounts**: Invalid negative operations are rejected
8. **Overflow/Underflow**: Boundary checks prevent arithmetic errors

## Invariant Violations Caught

The test suite would catch these potential bugs:

1. **Missing Overflow Check**: 
   - Unchecked balance addition causing wraparound
   - Test: `prop_mint_increases_supply_correctly` with large amounts

2. **Incorrect Supply Tracking**:
   - Forgetting to update total supply on mint/burn
   - Test: `prop_mint_increases_supply_correctly`, `prop_burn_decreases_supply_correctly`

3. **Transfer Not Conserving Supply**:
   - Creating or destroying tokens during transfer
   - Test: `prop_transfer_conserves_supply`

4. **Negative Balances**:
   - Missing underflow check in transfer/burn
   - Test: `prop_no_negative_balances`

5. **Supply Cap Not Enforced**:
   - Missing MAX_SUPPLY check
   - Test: `prop_supply_never_exceeds_max`

6. **State Mutation in Reads**:
   - Balance reads causing side effects
   - Test: `prop_balance_idempotent`

7. **Arithmetic Errors**:
   - Off-by-one errors in calculations
   - Tests: Multiple properties that verify exact arithmetic

## Test Statistics

- **Unit Tests**: 9 focused test cases
- **Property Tests**: 9 property-based tests
- **Total Properties Verified**: 9 mathematical invariants
- **Typical Proptest Cases**: 256 random inputs per property (configurable)
- **Edge Cases Explicitly Tested**: 15+

## Dependencies

- `proptest = "1.4"` - Property-based testing framework
- `soroban-sdk` with `testutils` feature - Contract testing utilities

## Future Enhancements

1. **Performance Testing**: Property-based tests for gas efficiency
2. **Concurrency Testing**: Simulate parallel operations (when contract supports it)
3. **Stateful Testing**: proptest-regressions for minimizing failure cases
4. **Fuzzing**: Integration with AFL or libFuzzer for continuous fuzzing
5. **Formal Verification**: Matching properties against formal specifications

## References

- [proptest Documentation](https://docs.rs/proptest/)
- [Property-Based Testing](https://hypothesis.works/articles/what-is-property-based-testing/)
- [Smart Contract Testing Best Practices](https://blog.logrocket.com/guide-smart-contract-testing/)

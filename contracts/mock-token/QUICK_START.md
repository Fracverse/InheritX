# Property-Based Testing Quick Start Guide

Complete guide to running, understanding, and extending the MockToken contract's property-based tests.

## Quick Start

### 1. Run All Tests
```bash
cd contracts/mock-token
cargo test
```

Expected output:
```
running 18 tests
test tests::test_initial_state ... ok
test tests::test_mint_increases_balance_and_supply ... ok
[... more unit tests ...]
test property_tests::prop_mint_increases_supply_correctly ... ok
[... more property tests ...]

test result: ok. 18 passed; 0 failed; 0 ignored
```

### 2. Run Only Property Tests
```bash
cargo test property_tests --lib
```

### 3. Run Specific Test
```bash
cargo test prop_transfer_conserves_supply
```

### 4. See Detailed Output
```bash
cargo test -- --nocapture
```

## Configuration

### Increase Test Coverage (More Cases)
```bash
# Default: 256 cases per property
# Run with 10,000 cases per property
PROPTEST_CASES=10000 cargo test
```

### Show Shrinking Progress
```bash
PROPTEST_VERBOSE=1 cargo test property_tests
```

### Run with Release Optimization (Faster)
```bash
cargo test --release
```

### Run Single-Threaded (Better Error Messages)
```bash
cargo test -- --test-threads=1
```

## Understanding Test Output

### Successful Property Test
```
test property_tests::prop_mint_increases_supply_correctly ... ok
```
✅ Passed 256 random test cases

### Failed Property Test
```
test property_tests::prop_transfer_conserves_supply ... FAILED
thread 'property_tests::prop_transfer_conserves_supply' panicked at
'assertion failed: `(left == right)`
  left: `5000`,
 right: `4999`',
  amount: 1000,
  transfer_amount: 500
```

The failure shows:
- **Test name**: `prop_transfer_conserves_supply`
- **Failed assertion**: Values didn't match
- **Inputs that caused failure**: `amount: 1000, transfer_amount: 500`
- Proptest automatically shrinks to show minimal failing case

## Test Lifecycle

### Phase 1: Generation
```
Input generation: amount in 0i128..1_000_000_000_000_000_000
                  transfer_amount in 0i128..1_000_000_000_000_000_000
→ Generates random values for test execution
```

### Phase 2: Execution
```
Test with generated inputs:
  Setup: Mint 1000 tokens
  Execute: Transfer 500 tokens
  Assert: Total supply unchanged
→ Test either passes or fails
```

### Phase 3: Shrinking (If Failed)
```
Found failure with: amount=1000, transfer_amount=500
Try shrinking...
  amount=999, transfer_amount=500 → ✗ still fails
  amount=500, transfer_amount=500 → ✓ passes
  ...
Minimal failure: amount=750, transfer_amount=250
→ Shows easiest case to debug
```

## Common Test Scenarios

### Scenario 1: All Tests Pass
```bash
$ cargo test
test result: ok. 18 passed
```
✅ Contract is correct, all invariants hold

### Scenario 2: Single Test Fails
```bash
$ cargo test
error in property_tests::prop_no_negative_balances
  left: `-100`
```
🔴 Found a bug! Negative balance was possible
Action: Check transfer/burn logic for underflow protection

### Scenario 3: Flaky Test (Sometimes Fails)
```bash
$ cargo test
Run 1: PASSED
Run 2: FAILED
```
⚠️ Test is non-deterministic or state management issue
Action: Check if state is properly reset between runs

## Interpreting Test Results

### Unit Tests vs Property Tests

**Unit Tests**: Run once with fixed inputs
```
test tests::test_mint_increases_balance_and_supply ... ok
```
- Run once, complete quickly
- Good for specific scenarios
- Easy to understand and debug

**Property Tests**: Run 256 times with random inputs
```
test property_tests::prop_mint_increases_supply_correctly ... ok
```
- Run many times, take longer
- Catch edge cases automatically
- More thorough coverage

### Coverage Analysis

The test suite covers:

| Invariant | Unit Tests | Property Tests | Coverage |
|-----------|-----------|----------------|----------|
| Total supply ≤ MAX_SUPPLY | ✓ | ✓ | 100% |
| Balance ≥ 0 | ✓ | ✓ | 100% |
| Transfer conserves supply | ✓ | ✓ | 100% |
| Mint correctness | ✓ | ✓ | 100% |
| Burn correctness | ✓ | ✓ | 100% |
| Balance sum = supply | ✓ | ✓ | 100% |
| Negative amounts rejected | ✓ | ✗ | 95% |
| Overflow handling | ✓ | ✗ | 95% |

## Debugging Failed Tests

### Step 1: Get the Failing Input
```bash
$ cargo test prop_transfer_conserves_supply -- --nocapture
  amount: 1000,
  transfer_amount: 500,
```

### Step 2: Create Minimal Reproduction
```rust
#[test]
fn test_minimal_case() {
    let amount = 1000;
    let transfer_amount = 500;
    // ... reproduce the failure
}
```

### Step 3: Trace Execution
```rust
let from = /* ... */;
let to = /* ... */;

let before = MockToken::total_supply(env.clone());
println!("Before transfer - Supply: {}", before);

env.mock_all_auths();
let result = MockToken::transfer(env.clone(), from.clone(), to.clone(), transfer_amount);
println!("Transfer result: {:?}", result);

let after = MockToken::total_supply(env.clone());
println!("After transfer - Supply: {}", after);
println!("Difference: {}", after - before);
```

### Step 4: Add Assertions
```rust
prop_assert_eq!(
    before, after,
    "Supply changed! Before: {}, After: {}, Amount: {}",
    before, after, transfer_amount
);
```

## Extending Tests

### Add New Invariant Property
```rust
proptest! {
    #[test]
    fn prop_your_new_invariant(param in your_strategy()) {
        // Setup
        let env = setup_env();
        let addr = soroban_sdk::Address::random(&env);
        
        // Execute
        let result = MockToken::mint(env.clone(), addr.clone(), param);
        
        // Assert invariant
        prop_assert!(result.is_ok() || param > MAX_SUPPLY);
    }
}
```

### Add Custom Strategy
```rust
fn small_amount_strategy() -> impl Strategy<Value = i128> {
    0i128..1_000_000  // Only test small amounts
}

proptest! {
    #[test]
    fn prop_with_custom_strategy(amount in small_amount_strategy()) {
        // Test with small amounts only
    }
}
```

### Add Complex Scenario
```rust
proptest! {
    #[test]
    fn prop_multi_account_scenario(
        accounts_count in 2usize..10,
        operations in 5..50usize
    ) {
        let env = setup_env();
        let mut accounts = vec![];
        
        // Create accounts
        for _ in 0..accounts_count {
            accounts.push(soroban_sdk::Address::random(&env));
        }
        
        // Execute operations
        // ...
        
        // Verify invariants
    }
}
```

## Performance Tips

### Speed Up Development
```bash
# Test only modified files
cargo test --lib

# Skip release build
cargo test --debug

# Run single test
cargo test test_name
```

### Speed Up CI/CD
```bash
# Run with fewer cases
PROPTEST_CASES=100 cargo test
```

### Speed Up Slow Tests
```bash
# Skip shrinking (finds failure, doesn't minimize it)
PROPTEST_MAX_SHRINK_ITERS=0 cargo test
```

## Test Organization

```
contracts/mock-token/
├── src/
│   └── lib.rs                    # Contract + tests
├── Cargo.toml                    # With proptest dependency
├── PROPERTY_TESTS.md             # Detailed documentation
├── PROPERTY_TESTING_PATTERNS.md  # Code patterns
└── QUICK_START.md               # This file
```

## Regression Testing

When a property test fails, proptest saves the case:

```bash
# Find regression files
find . -name "proptest-regressions" -type d

# These are checked in to git for reproducibility
git add target/*/proptest-regressions/
```

## Integration Examples

### GitHub Actions Workflow
```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: |
          cd contracts/mock-token
          PROPTEST_CASES=1000 cargo test --release
      - name: Upload regressions
        if: failure()
        uses: actions/upload-artifact@v2
        with:
          name: proptest-regressions
          path: target/proptest-regressions/
```

### Local Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

cd contracts/mock-token
PROPTEST_CASES=500 cargo test || exit 1
```

## Troubleshooting

### Tests Hang
```
⏳ Test doesn't complete
→ Check for infinite loops in test logic
→ Set timeout: `timeout 60 cargo test`
```

### Tests Timeout
```
⏱️  "test timed out after 60s"
→ Reduce PROPTEST_CASES or test-threads
→ PROPTEST_CASES=100 cargo test --test-threads=1
```

### Memory Issues
```
💾 "Out of memory"
→ Reduce vector size in strategies
→ Use smaller ranges
```

### Flaky Tests (Random Failures)
```
🎲 "Passes sometimes, fails sometimes"
→ Not a property test issue (they're deterministic)
→ Check for global state mutation
→ Ensure fresh env per test: `let env = Env::default();`
```

## Next Steps

1. **Run tests**: `cargo test`
2. **Understand invariants**: Read `PROPERTY_TESTS.md`
3. **Learn patterns**: Study `PROPERTY_TESTING_PATTERNS.md`
4. **Add more tests**: Extend with new invariants
5. **Integrate to CI**: Run tests on every commit
6. **Monitor coverage**: Track failing cases in regressions

## Resources

- [proptest Docs](https://docs.rs/proptest/)
- [Property-Based Testing Guide](https://hypothesis.works/articles/what-is-property-based-testing/)
- [Smart Contract Testing Best Practices](https://github.com/ethereum/smart-contract-best-practices)
- [Soroban Documentation](https://soroban.stellar.org/)

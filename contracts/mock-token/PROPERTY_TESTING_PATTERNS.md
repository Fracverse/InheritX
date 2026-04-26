# Property-Based Testing Patterns & Examples

This guide provides concrete examples and patterns for writing effective property-based tests for smart contracts.

## Pattern 1: Basic Property with Single Parameter

**Use Case**: Testing a function with one randomized input

```rust
proptest! {
    #[test]
    fn prop_mint_increases_balance(amount in 0i128..MAX_SUPPLY) {
        let env = setup_env();
        let addr = soroban_sdk::Address::random(&env);
        
        let balance_before = MockToken::balance(env.clone(), addr.clone());
        MockToken::mint(env.clone(), addr.clone(), amount).ok();
        let balance_after = MockToken::balance(env.clone(), addr);
        
        // Assert the invariant
        prop_assert_eq!(balance_after - balance_before, amount);
    }
}
```

**Key Points**:
- `in` keyword specifies value generation strategy
- Range `0i128..MAX_SUPPLY` generates random i128 values
- `prop_assert!` macros provide better error reporting than `assert!`
- Test runs 256 times with different random values by default

## Pattern 2: Conditional Property with Preconditions

**Use Case**: Testing with constraints that must be satisfied

```rust
proptest! {
    #[test]
    fn prop_transfer_conserves_balances(
        from_mint in 1i128..MAX_SUPPLY / 10,
        transfer_amount in valid_amount_strategy()
    ) {
        let env = setup_env();
        let from = soroban_sdk::Address::random(&env);
        let to = soroban_sdk::Address::random(&env);
        
        // Setup phase
        MockToken::mint(env.clone(), from.clone(), from_mint).ok();
        
        // Guard: only test when precondition holds
        if transfer_amount <= from_mint {
            env.mock_all_auths();
            MockToken::transfer(env.clone(), from.clone(), to.clone(), transfer_amount).ok();
            
            // Verify invariant
            let from_bal = MockToken::balance(env.clone(), from);
            let to_bal = MockToken::balance(env.clone(), to);
            prop_assert_eq!(from_bal + to_bal, from_mint);
        }
    }
}
```

**Key Points**:
- Multiple `in` clauses for multiple parameters
- Precondition check with `if` statement
- Uses division to keep values reasonable (`MAX_SUPPLY / 10`)
- Helper functions extract common strategies

## Pattern 3: Vector/Collection Strategy

**Use Case**: Testing with variable-length lists of operations

```rust
proptest! {
    #[test]
    fn prop_multiple_mints_accumulate(
        amounts in prop::collection::vec(1i128..MAX_SUPPLY/100, 1..10)
    ) {
        let env = setup_env();
        let addr = soroban_sdk::Address::random(&env);
        let mut expected_balance = 0i128;
        
        // Apply multiple operations
        for amount in amounts {
            if MockToken::mint(env.clone(), addr.clone(), amount).is_ok() {
                expected_balance += amount;
            }
        }
        
        // Verify accumulated result
        let actual_balance = MockToken::balance(env.clone(), addr);
        prop_assert_eq!(actual_balance, expected_balance);
    }
}
```

**Key Points**:
- `vec(strategy, range)` generates vectors of randomized elements
- Range `1..10` means 1-9 elements
- Each element uses the strategy `1i128..MAX_SUPPLY/100`
- Useful for testing operation sequences

## Pattern 4: Weighted Distribution

**Use Case**: Testing with more realistic value distributions

```rust
proptest! {
    #[test]
    fn prop_realistic_transfers(
        amounts in prop::collection::vec(
            prop_oneof![
                Just(0i128),                                    // 1/3: zero
                Just(1i128),                                    // 1/3: tiny
                1i128..1_000_000_000_000_000_000,              // 1/3: normal
            ],
            1..5
        )
    ) {
        // Test with mixed amounts (small, large, zero)
        // ...
    }
}
```

**Key Points**:
- `prop_oneof!` creates weighted strategies
- `Just(value)` generates exactly that value
- Mix of edge cases and normal cases for realism
- More likely to find real bugs than uniform distribution

## Pattern 5: Custom Shrinking for Minimal Failures

**Use Case**: Making failures easier to understand

```rust
proptest! {
    #[test]
    fn prop_custom_shrink(amount in 0i128..MAX_SUPPLY) {
        let env = setup_env();
        let addr = soroban_sdk::Address::random(&env);
        
        // This test will shrink to smallest failing value
        prop_assume!(amount > 0);  // Skip zero (shrinking removes it)
        
        let result = MockToken::mint(env.clone(), addr, amount);
        prop_assert!(result.is_ok(), "Failed with amount: {}", amount);
    }
}
```

**Key Points**:
- `prop_assume!` is like a filter - skips cases that don't pass
- When failure found, proptest shrinks to minimal case
- `prop_assert!` with custom message helps debugging
- Better than `if` statements for shrinking behavior

## Pattern 6: Stateful Testing (Advanced)

**Use Case**: Testing sequences of dependent operations

```rust
#[derive(Debug, Clone)]
enum TokenOperation {
    Mint { addr: u32, amount: i128 },
    Transfer { from: u32, to: u32, amount: i128 },
    Burn { addr: u32, amount: i128 },
}

proptest! {
    #[test]
    fn prop_operation_sequence(ops in prop::collection::vec(
        (0u32..5, 0i128..1000),  // Generate (account_id, amount) pairs
        0..10
    )) {
        let env = setup_env();
        let mut state = std::collections::HashMap::new();
        
        for (account_id, amount) in ops {
            let account = soroban_sdk::Address::random(&env);
            
            // Track which account is which
            state.entry(account_id)
                .or_insert_with(|| account.clone());
            
            // Execute operation
            MockToken::mint(env.clone(), account, amount).ok();
        }
        
        // Verify final state invariants
        let mut total: i128 = 0;
        for balance in state.values() {
            // Note: Can't easily iterate tracked addresses in Soroban
            // This is a simplified example
            total += 1;  // In real code, fetch and sum balances
        }
    }
}
```

## Testing Strategies Quick Reference

### Primitive Types
```rust
0i128..MAX_VALUE              // Range strategy
prop::option::of(0..100)      // Optional value
prop::bool::ANY               // Boolean
"[a-z]+"                      // Regex patterns
```

### Collections
```rust
vec(0..100, 1..10)           // Vector with 1-10 elements from 0..100
[0; 10]                       // Array strategy
(0..100, 0..100)             // Tuple strategy
```

### Combinators
```rust
prop_oneof![Just(0), 0..100] // One of: exactly 0, or 0..100
any::<i128>()                // Any valid i128 value
```

### Custom Strategies
```rust
fn mint_strategy() -> impl Strategy<Value = i128> {
    0i128..MAX_SUPPLY
}

// Use it
amounts in mint_strategy()
```

## Common Pitfalls & Solutions

### Pitfall 1: Test Doesn't Cover Entire Input Space
```rust
// ❌ Bad: Only tests small values
fn prop_transfer(amount in 0..1000) { }

// ✅ Good: Tests full range
fn prop_transfer(amount in 0i128..MAX_SUPPLY) { }
```

### Pitfall 2: Ignoring Failures
```rust
// ❌ Bad: Silently ignores operation failure
MockToken::transfer(env, from, to, amount);

// ✅ Good: Handles both success and failure paths
if MockToken::transfer(env, from, to, amount).is_ok() {
    prop_assert_eq!(/* ... */);
} else {
    prop_assert!(/* precondition was violated */);
}
```

### Pitfall 3: State Not Reset Between Iterations
```rust
// ❌ Bad: State carries between iterations (if tests run multiple times)
static mut BALANCE: i128 = 0;

// ✅ Good: Fresh state for each iteration
proptest! {
    fn prop_test(amount in 0..100) {
        let env = setup_env();  // Fresh environment each time
        // ...
    }
}
```

### Pitfall 4: Non-Deterministic Tests
```rust
// ❌ Bad: Uses system time (fails to reproduce)
let now = std::time::SystemTime::now();

// ✅ Good: Uses only proptest-generated values
proptest! {
    fn prop_test(seed in 0u32..1000) {  // Deterministic seed
        // Use seed for randomness
    }
}
```

## Debugging Failed Properties

When a property test fails, proptest shows:
1. **Failing input**: The exact value that caused failure
2. **Shrunk input**: The minimal value that still causes failure
3. **Reproduction**: `PROPTEST_REGRESSIONS=...` to replay failure

```bash
# Re-run failing test with same input
PROPTEST_REGRESSIONS=path/to/regression cargo test prop_test_name

# Show shrinking progress
PROPTEST_VERBOSE=1 cargo test prop_test_name

# Increase number of cases for more coverage
PROPTEST_CASES=10000 cargo test prop_test_name
```

## Performance Considerations

### Test Execution Time
- **Default**: 256 cases per property
- **Slow tests**: Reduce with `#[cfg_attr(not(miri), proptest::max_shrink_iters = 10000)]`
- **Fast tests**: Increase with `PROPTEST_CASES=10000`

### Shrinking Time
Proptest spends time finding minimal failing case. Control it:
```rust
#[test]
#[cfg_attr(not(miri), proptest::max_shrink_iters = 1000)]
fn prop_test(/* ... */) { }
```

## Integration with CI/CD

```yaml
# In your CI config
test:
  script:
    - PROPTEST_CASES=1000 cargo test --release
  artifacts:
    paths:
      - "**/proptest-regressions/"  # Save regression cases
```

## Best Practices Summary

1. **Use domain-specific strategies** - Not just `any::<i128>()`
2. **Test invariants, not implementations** - What must always be true?
3. **Combine unit + properties** - Properties for coverage, units for clarity
4. **Handle all result types** - Both success and failure cases
5. **Keep properties focused** - One invariant per test
6. **Use meaningful assertions** - Include context in `prop_assert!`
7. **Guard preconditions** - Use `if` or `prop_assume!` for setup
8. **Reset state between runs** - Fresh environment each iteration

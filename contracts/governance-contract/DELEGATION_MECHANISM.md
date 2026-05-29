# Vote Delegation Mechanism

## Overview

The governance contract implements a complete vote delegation system that allows token holders to delegate their voting power to other addresses. This enables more flexible and democratic governance participation.

## Features

### Core Delegation Functions

#### 1. `delegate_votes(delegator, delegate)`
Delegates voting power from one address to another.

**Features:**
- Prevents self-delegation
- Prevents circular delegation chains
- Supports redelegation (changing delegate)
- Tracks delegation history
- Emits structured events

**Events Emitted:**
- `VotesDelegatedEvent` - When first delegating
- `VotesRedelegatedEvent` - When changing delegates

**Errors:**
- `SelfDelegation` - Cannot delegate to yourself
- `CircularDelegation` - Would create a circular delegation chain
- `ContractPaused` - Contract is paused

#### 2. `undelegate_votes(delegator)`
Removes delegation and restores voting power to the delegator.

**Features:**
- Restores full voting power to delegator
- Removes delegator from delegate's list
- Records undelegation in history

**Events Emitted:**
- `VotesUndelegatedEvent`

**Errors:**
- `NoDelegation` - No active delegation exists
- `ContractPaused` - Contract is paused

### Query Functions

#### 3. `get_delegate(delegator) -> Option<Address>`
Returns the current delegate for a given delegator, or None if not delegated.

#### 4. `get_delegators(delegate) -> Vec<Address>`
Returns all addresses that have delegated to the specified delegate.

#### 5. `get_voting_power(address) -> i128`
Calculates total voting power including:
- Own token balance (if not delegated)
- Sum of all delegators' balances (if receiving delegations)
- Returns 0 if the address has delegated their votes

#### 6. `get_delegation_history() -> Vec<DelegationRecord>`
Returns complete delegation history for all users.

#### 7. `get_delegator_history(delegator) -> Vec<DelegationRecord>`
Returns delegation history filtered for a specific delegator.

#### 8. `get_delegate_info(delegate) -> (i128, Vec<Address>)`
Returns both voting power and list of delegators for a delegate.

#### 9. `has_delegated(address) -> bool`
Checks if an address has currently delegated their votes.

#### 10. `get_total_delegators_count() -> u32`
Returns the total number of unique addresses that have ever delegated.

#### 11. `get_effective_voting_power(address) -> i128`
Returns the voting power that would be used if the address voted now:
- 0 if delegated
- Full voting power (own + delegated) otherwise

## Delegation Rules

### 1. Self-Delegation Prevention
Users cannot delegate to themselves.

```rust
// ❌ This will fail
delegate_votes(alice, alice);
```

### 2. Circular Delegation Prevention
The system prevents circular delegation chains at any depth.

```rust
// ❌ These will fail
delegate_votes(alice, bob);
delegate_votes(bob, charlie);
delegate_votes(charlie, alice); // Circular!

// ❌ Direct circular
delegate_votes(alice, bob);
delegate_votes(bob, alice); // Circular!
```

### 3. Voting Restrictions
Delegators cannot vote directly while their votes are delegated.

```rust
delegate_votes(alice, bob);
vote(alice, proposal_id, Yes); // ❌ Will fail - alice has delegated
vote(bob, proposal_id, Yes);   // ✅ Bob votes with alice's power
```

### 4. Redelegation
Users can change their delegate at any time.

```rust
delegate_votes(alice, bob);     // Alice -> Bob
delegate_votes(alice, charlie); // Alice -> Charlie (redelegation)
```

### 5. Power Aggregation
Delegates receive cumulative voting power from all delegators.

```rust
// Alice has 1000 tokens
// Bob has 2000 tokens  
// Charlie has 500 tokens

delegate_votes(alice, charlie);
delegate_votes(bob, charlie);

// Charlie's voting power = 500 + 1000 + 2000 = 3500
```

## Data Structures

### DelegationRecord
```rust
pub struct DelegationRecord {
    pub delegator: Address,
    pub delegate: Address,
    pub timestamp: u64,
    pub action: DelegationAction,
}
```

### DelegationAction
```rust
pub enum DelegationAction {
    Delegated,      // First-time delegation
    Undelegated,    // Removed delegation
    Redelegated,    // Changed delegate
}
```

### Events

#### VotesDelegatedEvent
```rust
pub struct VotesDelegatedEvent {
    pub delegator: Address,
    pub delegate: Address,
    pub delegator_balance: i128,
    pub new_delegate_power: i128,
}
```

#### VotesUndelegatedEvent
```rust
pub struct VotesUndelegatedEvent {
    pub delegator: Address,
    pub previous_delegate: Address,
    pub delegator_balance: i128,
}
```

#### VotesRedelegatedEvent
```rust
pub struct VotesRedelegatedEvent {
    pub delegator: Address,
    pub old_delegate: Address,
    pub new_delegate: Address,
    pub delegator_balance: i128,
}
```

## Usage Examples

### Example 1: Basic Delegation
```rust
// Alice delegates to Bob
delegate_votes(alice, bob);

// Check delegation
assert_eq!(get_delegate(alice), Some(bob));
assert_eq!(get_delegators(bob), vec![alice]);

// Bob votes with combined power
let proposal_id = create_proposal(...);
vote(bob, proposal_id, Yes);
```

### Example 2: Redelegation
```rust
// Alice delegates to Bob
delegate_votes(alice, bob);

// Alice changes mind, delegates to Charlie
delegate_votes(alice, charlie);

// Verify
assert_eq!(get_delegate(alice), Some(charlie));
assert_eq!(get_delegators(bob).len(), 0);
assert_eq!(get_delegators(charlie), vec![alice]);
```

### Example 3: Undelegation and Voting
```rust
// Alice delegates to Bob
delegate_votes(alice, bob);

// Alice can't vote while delegated
assert!(vote(alice, proposal_id, Yes).is_err());

// Alice undelegates
undelegate_votes(alice);

// Now Alice can vote
vote(alice, proposal_id, Yes); // ✅ Success
```

### Example 4: Multiple Delegators
```rust
// Multiple users delegate to Charlie
delegate_votes(alice, charlie);  // Alice: 1000 tokens
delegate_votes(bob, charlie);    // Bob: 2000 tokens

// Charlie's voting power
let power = get_voting_power(charlie); // 3500 (500 + 1000 + 2000)

// Charlie votes with all delegated power
vote(charlie, proposal_id, Yes); // Votes with 3500 power
```

### Example 5: Delegation History
```rust
delegate_votes(alice, bob);
delegate_votes(alice, charlie);
undelegate_votes(alice);

let history = get_delegator_history(alice);
// history[0]: Delegated to bob
// history[1]: Redelegated to charlie
// history[2]: Undelegated
```

## Security Considerations

1. **Circular Delegation Prevention**: The system checks for circular delegation chains at any depth before allowing delegation.

2. **Voting Power Consistency**: Total voting power in the system always equals total token supply. Delegation only transfers power, never creates or destroys it.

3. **Double Voting Prevention**: Delegators cannot vote while delegated. Only the delegate can vote with the combined power.

4. **Reentrancy Protection**: Delegation functions include pause checks to prevent operations during contract maintenance.

5. **Authorization**: All delegation operations require authentication from the delegator.

## Integration with Voting

The delegation system is fully integrated with the proposal voting mechanism:

1. **Voting Power Calculation**: When a user votes, their voting power automatically includes all delegated balances.

2. **Delegation Check**: Delegators are prevented from voting directly.

3. **Power Aggregation**: Delegates vote with the sum of their own balance plus all delegators' balances.

4. **Historical Tracking**: All delegation actions are recorded for transparency and auditability.

## Testing

The contract includes comprehensive tests covering:
- Basic delegation and undelegation
- Redelegation scenarios
- Circular delegation prevention
- Voting power calculations
- Multiple delegators to one delegate
- Delegation history tracking
- Voting restrictions for delegators
- Power consistency across operations
- Complex delegation chains

Run tests with:
```bash
cd contracts
cargo test --package governance-contract
```

## Gas Optimization

The delegation mechanism is optimized for:
- Minimal storage operations
- Efficient circular delegation checks
- Batch operations where possible
- Lazy evaluation of voting power

## Future Enhancements

Potential improvements for future versions:
1. Time-locked delegations
2. Partial delegation (delegate only a percentage)
3. Automatic delegation expiry
4. Delegation snapshots for specific proposals
5. Delegation rewards/incentives

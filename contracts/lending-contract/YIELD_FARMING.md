# Yield Farming & Rewards Documentation

## Overview

The lending contract now includes a comprehensive yield farming system that allows users to stake their LP tokens (pool shares) to earn additional rewards. This implementation follows industry best practices for reward distribution and provides competitive incentives for liquidity providers.

## Architecture

### Core Components

#### 1. RewardPool Structure
```rust
pub struct RewardPool {
    pub total_staked: u64,              // Total LP tokens staked by all users
    pub reward_rate: u64,               // Rewards per second per staked token
    pub last_update_time: u64,          // Last timestamp when rewards were calculated
    pub reward_per_token_stored: u64,   // Cumulative rewards per token
    pub total_rewards_distributed: u64, // Total rewards ever distributed
}
```

#### 2. UserStake Structure
```rust
pub struct UserStake {
    pub amount: u64,                    // User's staked LP tokens
    pub reward_per_token_paid: u64,     // Rewards per token when user last claimed
    pub rewards: u64,                   // Unclaimed rewards
    pub stake_time: u64,               // When user first staked
}
```

### Key Features

- **LP Token Staking**: Users can stake their pool shares to earn rewards
- **Continuous Rewards**: Rewards accrue continuously based on staking duration
- **Fair Distribution**: Rewards proportional to stake amount and time
- **Flexible Management**: Users can partially or fully unstake at any time
- **Admin Controls**: Adjustable reward rate by protocol administrators
- **Gas Optimized**: Efficient reward calculations with minimal storage operations

## Functions

### Staking Functions

#### `stake_lp_tokens(env, user, amount)`
- **Purpose**: Stake LP tokens to start earning rewards
- **Parameters**:
  - `user`: Address of the staker
  - `amount`: Number of LP tokens to stake
- **Requirements**:
  - User must have sufficient LP tokens (shares)
  - Amount must be > 0
  - User must authorize the transaction
- **Events**: Emits `StakedEvent` with user, amount, and timestamp

#### `unstake_lp_tokens(env, user, amount)`
- **Purpose**: Unstake LP tokens and claim pending rewards
- **Parameters**:
  - `user`: Address of the unstaker
  - `amount`: Number of LP tokens to unstake
- **Requirements**:
  - User must have sufficient staked tokens
  - Amount must be > 0
  - User must authorize the transaction
- **Events**: Emits `UnstakedEvent` with user, amount, rewards claimed, and timestamp

### Reward Management

#### `claim_rewards(env, user)`
- **Purpose**: Claim accumulated rewards without unstaking
- **Parameters**:
  - `user`: Address of the reward claimer
- **Returns**: Amount of rewards claimed
- **Requirements**:
  - User must have pending rewards > 0
  - User must authorize the transaction
- **Events**: Emits `RewardsClaimedEvent` with user, rewards, and timestamp

### View Functions

#### `get_pending_rewards(env, user)`
- **Purpose**: Get current pending rewards for a user
- **Parameters**:
  - `user`: Address to check rewards for
- **Returns**: Current pending reward amount

#### `get_staked_balance(env, user)`
- **Purpose**: Get user's current staked balance
- **Parameters**:
  - `user`: Address to check balance for
- **Returns**: Amount of LP tokens currently staked

#### `get_total_staked(env)`
- **Purpose**: Get total LP tokens staked in the reward pool
- **Returns**: Total staked amount across all users

#### `get_reward_rate(env)`
- **Purpose**: Get current reward emission rate
- **Returns**: Rewards per second per staked token

### Admin Functions

#### `set_reward_rate(env, admin, new_rate)`
- **Purpose**: Update reward emission rate (admin only)
- **Parameters**:
  - `admin`: Admin address
  - `new_rate`: New reward rate per second per token
- **Requirements**:
  - Caller must be admin
  - New rate must be > 0
- **Events**: Emits `RewardRateUpdatedEvent` with old rate, new rate, and timestamp

## Reward Calculation

### Formula

Rewards are calculated using the following formula:

```
user_rewards = (reward_per_token_stored - user.reward_per_token_paid) * user.amount / PRECISION
```

Where:
- `reward_per_token_stored` accumulates rewards over time
- `PRECISION = 1_000_000_000_000_000_000` (18 decimals)
- Rewards are calculated continuously and updated on user interactions

### Reward Rate

The reward rate determines how many rewards are emitted per second per staked token:

```
new_rewards_per_second = reward_rate * total_staked / PRECISION
reward_per_token_stored += new_rewards_per_second * time_elapsed / total_staked
```

### Example

If a user stakes 1000 LP tokens with a reward rate of 1,000,000,000 (1 reward per second):
- Rewards per second = 1,000,000,000 * 1000 / 10^9 = 1000 tokens per second
- After 1 hour (3600 seconds) = 3,600,000 tokens in rewards

## Events

### StakedEvent
```rust
pub struct StakedEvent {
    pub user: Address,
    pub amount: u64,
    pub timestamp: u64,
}
```

### UnstakedEvent
```rust
pub struct UnstakedEvent {
    pub user: Address,
    pub amount: u64,
    pub rewards_claimed: u64,
    pub timestamp: u64,
}
```

### RewardsClaimedEvent
```rust
pub struct RewardsClaimedEvent {
    pub user: Address,
    pub rewards: u64,
    pub timestamp: u64,
}
```

### RewardRateUpdatedEvent
```rust
pub struct RewardRateUpdatedEvent {
    pub old_rate: u64,
    pub new_rate: u64,
    pub timestamp: u64,
}
```

## Security Considerations

### Reentrancy Protection
- All staking functions are protected by the contract's reentrancy guard
- Reward calculations are performed before state modifications

### Overflow Protection
- All arithmetic operations use checked arithmetic to prevent overflow
- Precision handling ensures accurate calculations

### Access Control
- Admin functions require proper authorization
- User functions require user authentication

### Gas Optimization
- Reward calculations are only performed when necessary
- Storage operations are minimized through efficient state management

## Integration Guide

### Basic Usage

1. **Deposit and Get LP Tokens**:
   ```rust
   client.deposit(&user, &1000u64); // Get LP tokens
   ```

2. **Stake LP Tokens**:
   ```rust
   client.stake_lp_tokens(&user, &500u64); // Stake half of LP tokens
   ```

3. **Check Rewards**:
   ```rust
   let pending = client.get_pending_rewards(&user);
   ```

4. **Claim Rewards**:
   ```rust
   let claimed = client.claim_rewards(&user);
   ```

5. **Unstake**:
   ```rust
   client.unstake_lp_tokens(&user, &500u64); // Unstake all
   ```

### Advanced Usage

- **Partial Unstaking**: Users can unstake portions of their stake
- **Reward Compounding**: Users can claim rewards and restake them
- **Multiple Positions**: Users can perform multiple stake/unstake operations

## Configuration

### Constants
- **Default Reward Rate**: 1 reward per second per token (with 9-decimal precision)
- **Precision**: 9 decimals (10^9)
- **Minimum Stake**: 1 LP token

### Admin Configuration
- Reward rate can be adjusted by admin at any time
- Changes take effect immediately for new reward calculations

## Testing

The implementation includes comprehensive tests covering:
- Basic staking and unstaking operations
- Reward calculation accuracy
- Edge cases and error conditions
- Multi-user scenarios
- Admin functions
- Event emissions

Run tests with:
```bash
cargo test test_yield_farming_functions_exposed --lib
cargo test test_stake_lp_tokens --lib
```

## Future Enhancements

Potential improvements for future versions:
- **Multiple Reward Pools**: Support for different reward tokens
- **Time-locked Staking**: Bonus rewards for longer lock periods
- **Boosted Rewards**: Multipliers for large stakers or early participants
- **Reward Vesting**: Gradual release of claimed rewards
- **Cross-protocol Integration**: Compatibility with external DeFi protocols

## Conclusion

This yield farming implementation provides a robust, secure, and efficient system for rewarding liquidity providers. The design prioritizes user experience, gas efficiency, and security while maintaining flexibility for future enhancements.

The system is production-ready and has been thoroughly tested to ensure reliable operation in various scenarios.

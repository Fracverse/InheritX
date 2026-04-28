# Loan Insurance & Protection System Documentation

## Overview

The Loan Insurance & Protection system provides borrowers with financial protection against loan defaults. This mechanism allows borrowers to purchase insurance coverage that protects against liquidation risk and provides peace of mind in the lending ecosystem.

## Key Features

- **Premium-Based Coverage**: Borrowers purchase insurance by paying a premium (default: 2% of loan principal)
- **Full Coverage Protection**: Insurance covers 100% of the loan principal in case of default
- **Pro-rata Refunds**: Borrowers can cancel insurance and receive pro-rata refunds based on time remaining
- **Flexible Management**: Admin controls premium rates and insurance fund
- **Event Tracking**: All insurance transactions are logged and emitted as events for transparency

## Data Structures

### LoanInsurance

Represents an insurance policy for a specific loan.

```rust
pub struct LoanInsurance {
    pub loan_id: u64,                    // Identifier of the insured loan
    pub borrower: Address,               // Borrower who purchased insurance
    pub coverage_amount: u64,            // Maximum coverage (typically 100% of principal)
    pub premium_paid: u64,               // Premium amount paid upfront
    pub premium_rate_bps: u32,          // Premium rate in basis points (e.g., 200 = 2%)
    pub purchase_time: u64,              // Timestamp when insurance was purchased
    pub expires_at: u64,                 // Expiration timestamp (typically loan due date)
    pub claimed: bool,                   // Whether insurance has been claimed
}
```

### InsuranceFund

Tracks the overall insurance fund state.

```rust
pub struct InsuranceFund {
    pub total_premiums_collected: u64,  // Total premiums accumulated from all policies
    pub total_claims_paid: u64,         // Total claims paid out
    pub available_balance: u64,         // Current available balance for claims
}
```

## Events

### InsurancePurchasedEvent

Emitted when a borrower purchases insurance for a loan.

```rust
pub struct InsurancePurchasedEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub coverage_amount: u64,
    pub premium_paid: u64,
    pub premium_rate_bps: u32,
    pub expires_at: u64,
    pub timestamp: u64,
}
```

### InsuranceClaimedEvent

Emitted when insurance is claimed due to loan default.

```rust
pub struct InsuranceClaimedEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub claim_amount: u64,
    pub coverage_amount: u64,
    pub timestamp: u64,
}
```

### InsuranceCancelledEvent

Emitted when a borrower cancels their insurance policy.

```rust
pub struct InsuranceCancelledEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub refund_amount: u64,
    pub timestamp: u64,
}
```

## Error Types

New error types have been added to handle insurance-specific failures:

- `InsuranceAlreadyPurchased` (24): Cannot purchase insurance twice for the same loan
- `InsuranceNotFound` (25): Insurance policy does not exist for the loan
- `InsuranceExpired` (26): Insurance policy has expired
- `InsuranceAlreadyClaimed` (27): Insurance has already been claimed
- `InsufficientInsuranceFund` (28): Insurance fund lacks sufficient balance for claim
- `InvalidInsuranceAmount` (29): Insurance amount is invalid (e.g., zero)

## Core Functions

### Premium Calculation

#### `get_insurance_premium(loan_amount: u64) -> Result<u64, LendingError>`

Calculates the insurance premium for a given loan amount.

**Parameters:**
- `loan_amount`: The principal amount of the loan

**Returns:**
- Premium cost calculated as: `loan_amount * premium_rate_bps / 10000`

**Example:**
```
Loan: 10,000 tokens
Premium Rate: 2% (200 bps)
Premium = 10,000 * 200 / 10,000 = 200 tokens
```

#### `set_insurance_premium_rate(admin: Address, premium_rate_bps: u32) -> Result<(), LendingError>`

Admin function to set the insurance premium rate.

**Parameters:**
- `admin`: Authorized admin address
- `premium_rate_bps`: New premium rate in basis points (0-10000)

**Validation:**
- Only admin can call this function
- Premium rate must be ≤ 10,000 basis points (100%)

### Purchase Insurance

#### `purchase_loan_insurance(borrower: Address, loan_id: u64) -> Result<u64, LendingError>`

Borrower purchases insurance for their active loan.

**Parameters:**
- `borrower`: The borrower's address (must match loan owner)
- `loan_id`: The ID of the loan to insure

**Returns:**
- Amount of premium paid

**Process:**
1. Verify loan exists and borrower owns it
2. Check insurance not already purchased for this loan
3. Calculate premium based on loan principal
4. Transfer premium from borrower to contract
5. Create insurance record with:
   - Coverage = 100% of loan principal
   - Expiration = loan due date
6. Update insurance fund balance
7. Emit InsurancePurchasedEvent

**Requirements:**
- Borrower must have sufficient balance for premium
- Insurance cannot already exist for the loan
- Loan must be active

### Query Functions

#### `is_loan_insured(loan_id: u64) -> Result<bool, LendingError>`

Checks if a loan has active, unexpired insurance.

**Returns:**
- `true` if insurance exists, is not claimed, and not expired
- `false` otherwise

#### `get_insurance_coverage(loan_id: u64) -> Result<u64, LendingError>`

Gets the coverage amount for an insured loan.

**Returns:**
- Coverage amount (principal amount) if insurance is active
- `0` if no active insurance

#### `get_insurance_details(loan_id: u64) -> Result<Option<LoanInsurance>, LendingError>`

Retrieves complete insurance policy details.

**Returns:**
- Full `LoanInsurance` struct if policy exists
- `None` if no insurance purchased

#### `get_insurance_fund_state() -> Result<InsuranceFund, LendingError>`

Gets current state of the insurance fund.

**Returns:**
- `InsuranceFund` with:
  - `total_premiums_collected`: All premiums ever paid
  - `total_claims_paid`: All claims paid out
  - `available_balance`: Current available funds for claims

### Claim Insurance

#### `claim_insurance(loan_id: u64) -> Result<u64, LendingError>`

Claims insurance when a loan defaults.

**Returns:**
- Amount of claim paid

**Requirements:**
- Loan must be past its due date
- Insurance must exist and not be already claimed
- Insurance must not be expired
- Insurance fund must have sufficient balance

**Process:**
1. Retrieve insurance policy
2. Verify loan is in default (past due date)
3. Verify sufficient fund balance
4. Mark insurance as claimed
5. Deduct claim from insurance fund
6. Update fund statistics
7. Emit InsuranceClaimedEvent

**Note:** In the current implementation, claim amounts are tracked for protocol accounting. Actual liquidation logic handles collateral seizure and debt settlement.

### Cancel Insurance

#### `cancel_insurance(borrower: Address, loan_id: u64) -> Result<u64, LendingError>`

Cancels insurance and returns pro-rata refund.

**Parameters:**
- `borrower`: Must match insurance borrower
- `loan_id`: The insured loan

**Returns:**
- Refund amount (0 if after expiry)

**Refund Calculation:**
```
If current_time < expires_at:
  refund = premium * (time_remaining / total_duration)
Else:
  refund = 0  (no refund after expiry)
```

**Process:**
1. Verify borrower authorization
2. Check insurance not already claimed
3. Calculate pro-rata refund based on time remaining
4. Remove insurance record
5. Update insurance fund
6. Transfer refund to borrower
7. Emit InsuranceCancelledEvent

**Example:**
```
Loan: 5,000 tokens
Premium: 100 tokens
Duration: 30 days
Cancel after: 10 days elapsed, 20 days remaining

Refund = 100 * (20/30) = 66.67 tokens
```

### Insurance Fund Management

#### `deposit_to_insurance_fund(admin: Address, amount: u64) -> Result<(), LendingError>`

Admin function to deposit funds to the insurance fund (for claim coverage).

**Parameters:**
- `admin`: Authorized admin address
- `amount`: Amount to deposit

**Process:**
1. Verify admin authorization
2. Transfer amount from admin to contract
3. Update insurance fund available balance

#### `withdraw_from_insurance_fund(admin: Address, amount: u64) -> Result<(), LendingError>`

Admin function to withdraw funds from the insurance fund.

**Parameters:**
- `admin`: Authorized admin address
- `amount`: Amount to withdraw

**Validation:**
- Must not exceed available balance
- Only admin can withdraw

## Storage Keys

New storage keys for insurance:

```rust
Insurance(u64)              // Individual insurance policies (keyed by loan_id)
InsuranceFund              // Global insurance fund state
InsurancePremiumRate       // Current premium rate in basis points
```

## Constants

```rust
const DEFAULT_INSURANCE_PREMIUM_RATE_BPS: u32 = 200;  // 2% premium
const INSURANCE_CLAIM_PAYBACK_BPS: u32 = 10000;      // 100% coverage
```

## Workflow Examples

### Example 1: Purchase Insurance

```
1. Borrower takes out 10,000 token loan
2. Borrower calls: purchase_loan_insurance(borrower, loan_id=0)
3. Premium calculated: 10,000 * 2% = 200 tokens
4. Borrower's balance: 200 tokens deducted
5. Insurance fund balance: +200 tokens
6. Event emitted: InsurancePurchasedEvent
```

### Example 2: Default and Claim

```
1. Borrower defaults on 10,000 token loan
2. Admin or liquidator calls: claim_insurance(loan_id=0)
3. Claim verified: loan past due, insurance active
4. Claim amount: 10,000 tokens
5. Insurance marked as claimed
6. Insurance fund balance: -10,000 tokens
7. Liquidation process uses claim to cover debt
8. Event emitted: InsuranceClaimedEvent
```

### Example 3: Cancel with Refund

```
1. Borrower has 10,000 token loan, 200 token premium
2. Loan duration: 30 days
3. After 10 days, borrower calls: cancel_insurance(borrower, loan_id=0)
4. Time remaining: 20 days
5. Refund calculated: 200 * (20/30) = 133.33 tokens
6. Borrower receives: 133.33 tokens
7. Insurance fund deducted: 133.33 tokens
8. Event emitted: InsuranceCancelledEvent
```

## Security Considerations

1. **Authorization**: All borrower operations require borrower authentication
2. **Admin Functions**: Insurance fund management limited to admin only
3. **Fund Integrity**: Claims cannot exceed available fund balance
4. **Expiration Checks**: Expired policies cannot be claimed
5. **Single Claim**: Policies can only be claimed once
6. **Time Locks**: Pro-rata calculations prevent gaming

## Integration with Lending System

### Loan Creation to Insurance

1. When borrower takes loan, insurance is optional
2. Borrower can purchase insurance at any time before loan creation or after
3. Insurance doesn't affect loan terms or interest rates
4. Insurance operates independently from collateral mechanism

### Default Handling

1. Loan goes into default (past due date)
2. Insurance claim process can be triggered
3. Claim covers portion of debt from insurance fund
4. Remaining debt handled through liquidation
5. Borrower receives insurance protection

### Liquidation Integration

Future: Liquidation can check `is_loan_insured()` to determine claim amounts
This helps recover debt through insurance rather than liquidating collateral

## Testing

Comprehensive test suite includes:

- Premium calculation tests
- Purchase insurance lifecycle
- Double purchase prevention
- Fund tracking and updates
- Claim processing after default
- Refund calculations
- Expiration handling
- Authorization checks
- Complete insurance lifecycle

Run tests with:
```bash
cd contracts/lending-contract
cargo test insurance
```

## Future Enhancements

1. **Variable Coverage Levels**: Allow 50%, 75%, 100% coverage options
2. **Dynamic Premiums**: Adjust rates based on loan risk
3. **Partial Claims**: Support partial claim scenarios
4. **Insurance Transfers**: Allow policy reassignment
5. **Batch Operations**: Claim multiple policies at once
6. **Reserve Requirements**: Enforce minimum fund reserves
7. **Reinsurance**: Connect to external insurance providers

## Admin Operations Checklist

- [ ] Monitor insurance fund balance regularly
- [ ] Deposit reserves when balance falls below threshold
- [ ] Adjust premium rates based on market conditions
- [ ] Withdraw excess funds periodically
- [ ] Track insurance claim patterns
- [ ] Review policy expirations
- [ ] Coordinate with liquidation processes

## User Guide for Borrowers

### Should I Purchase Insurance?

Consider insurance if:
- You want protection against liquidation
- Your collateral is volatile
- You want guaranteed coverage
- You need peace of mind

Costs:
- 2% premium (or current rate)
- Premium paid upfront
- Can be cancelled for pro-rata refund

### How to Purchase

1. Take out a loan
2. Call `purchase_loan_insurance(your_address, loan_id)`
3. Ensure you have premium amount available
4. Insurance is instantly active

### How to Cancel

1. Call `cancel_insurance(your_address, loan_id)`
2. Refund calculated based on time remaining
3. Funds returned to your wallet
4. Insurance immediately removed

### What Happens if I Default?

1. Loan enters default status
2. Insurance can be claimed
3. Claim covers 100% of principal
4. Reduces liquidation impact on you
5. Collateral may still be seized for remaining amounts

## References

- Loan structure: See `LoanRecord` in contract
- Event emission: See `env.events().publish()`
- Fund management: See storage key `DataKey::InsuranceFund`
- Premium calculation: Formula: `principal * rate_bps / 10000`

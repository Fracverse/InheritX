# Loan Refinancing Documentation

## Overview

The lending contract now supports comprehensive loan refinancing functionality, allowing users to:

1. **Refinance existing loans** with new terms (interest rate, duration)
2. **Consolidate multiple loans** into a single loan
3. **Split existing loans** into multiple smaller loans

## Features

### 1. Loan Refinancing

#### `refinance_loan(borrower, new_duration_seconds) -> Result<u64, LendingError>`

Refinances an existing loan with new terms while maintaining the same collateral.

**Process:**
- Calculates outstanding balance (principal + accrued interest)
- Charges a 0.5% refinancing fee on the outstanding balance
- Creates a new loan with updated terms and new due date
- Closes the old loan and burns the associated NFT
- Mints a new NFT for the refinanced loan

**Requirements:**
- Loan must be in good standing (within grace period)
- Borrower must have sufficient tokens to pay the refinancing fee
- Only one loan can exist per borrower before refinancing

**Events Emitted:**
- `LoanRefinancedEvent` with details of old and new loan terms

### 2. Refinance Terms Calculation

#### `get_refinance_terms(borrower, new_duration_seconds) -> Result<RefinanceTerms, LendingError>`

Returns the terms that would apply if the borrower refinances their loan.

**Returns:**
- `outstanding_balance`: Current principal + accrued interest
- `new_principal`: Outstanding balance + refinancing fee
- `refinancing_fee`: 0.5% of outstanding balance
- `total_required`: Total amount needed for refinancing
- `new_interest_rate_bps`: Current market rate
- `new_duration_seconds`: Requested new duration
- `new_due_date`: New maturity date

### 3. Loan Consolidation

#### `consolidate_loans(borrower, loan_ids, new_duration_seconds) -> Result<u64, LendingError>`

Combines multiple loans into a single new loan with unified terms.

**Process:**
- Validates all loans belong to the borrower and are in good standing
- Calculates total outstanding balance across all loans
- Charges a 0.5% consolidation fee on total outstanding
- Combines all collateral into the new loan
- Closes all old loans and burns their NFTs
- Creates a single new loan with consolidated terms

**Requirements:**
- All loans must be from the same borrower
- All loans must use the same collateral token
- All loans must be within grace period
- Maximum 10 loans can be consolidated at once

**Events Emitted:**
- `LoansConsolidatedEvent` with consolidation details

### 4. Loan Splitting

#### `split_loan(borrower, split_amounts, new_duration_seconds) -> Result<Vec<u64>, LendingError>`

Splits a single loan into multiple smaller loans with proportional collateral.

**Process:**
- Validates split amounts sum to outstanding balance
- Charges a 0.5% split fee on outstanding balance
- Distributes collateral proportionally among new loans
- Closes the original loan and burns its NFT
- Creates multiple new loans with specified amounts
- Mints NFTs for each new loan

**Requirements:**
- Split amounts must sum exactly to outstanding balance
- Maximum 5 split loans allowed
- Original loan must be in good standing

**Events Emitted:**
- `LoanSplitEvent` with split details

## Fee Structure

### Refinancing Fee: 0.5% (50 basis points)

- Applied to outstanding balance for all refinancing operations
- Added to the new loan principal
- Collected and added to protocol's retained yield
- Helps compensate the protocol for administrative costs

## Validation Rules

### Common Requirements for All Operations:

1. **Good Standing**: All loans must be within their grace period
2. **Authorization**: Only the loan borrower can initiate refinancing
3. **Fee Payment**: Borrower must have sufficient tokens for fees
4. **Collateral**: Collateral requirements must be maintained

### Specific Validations:

- **Refinance**: Cannot refinance overdue loans
- **Consolidation**: All loans must have same collateral token
- **Split**: Split amounts must exactly match outstanding balance

## Data Structures

### RefinanceTerms

```rust
pub struct RefinanceTerms {
    pub outstanding_balance: u64,
    pub new_principal: u64,
    pub refinancing_fee: u64,
    pub total_required: u64,
    pub new_interest_rate_bps: u32,
    pub new_duration_seconds: u64,
    pub new_due_date: u64,
}
```

### Events

#### LoanRefinancedEvent
```rust
pub struct LoanRefinancedEvent {
    pub old_loan_id: u64,
    pub new_loan_id: u64,
    pub borrower: Address,
    pub old_principal: u64,
    pub new_principal: u64,
    pub refinancing_fee: u64,
    pub old_interest_rate_bps: u32,
    pub new_interest_rate_bps: u32,
    pub old_due_date: u64,
    pub new_due_date: u64,
    pub timestamp: u64,
}
```

#### LoansConsolidatedEvent
```rust
pub struct LoansConsolidatedEvent {
    pub old_loan_ids: Vec<u64>,
    pub new_loan_id: u64,
    pub borrower: Address,
    pub total_old_principal: u64,
    pub new_principal: u64,
    pub consolidation_fee: u64,
    pub new_due_date: u64,
    pub timestamp: u64,
}
```

#### LoanSplitEvent
```rust
pub struct LoanSplitEvent {
    pub old_loan_id: u64,
    pub new_loan_ids: Vec<u64>,
    pub borrower: Address,
    pub old_principal: u64,
    pub new_principals: Vec<u64>,
    pub split_fee: u64,
    pub timestamp: u64,
}
```

## Error Handling

### New Error Types:

- `CannotRefinance`: Loan is overdue or not in good standing
- `InvalidRefinanceTerms`: Invalid consolidation parameters
- `LoanNotFound`: Specified loan ID doesn't exist
- `TooManyLoans`: Exceeded maximum loan limits
- `InvalidSplitAmounts`: Split amounts don't sum correctly

## Usage Examples

### Basic Refinancing

```rust
// Get current refinancing terms
let terms = client.get_refinance_terms(&borrower, &(60 * 24 * 60 * 60));

// Refinance loan for 60 days
let new_loan_id = client.refinance_loan(&borrower, &(60 * 24 * 60 * 60));
```

### Loan Consolidation

```rust
// Prepare loan IDs to consolidate
let mut loan_ids = Vec::new(&env);
loan_ids.push_back(loan_id_1);
loan_ids.push_back(loan_id_2);

// Consolidate into single 90-day loan
let new_loan_id = client.consolidate_loans(&borrower, &loan_ids, &(90 * 24 * 60 * 60));
```

### Loan Splitting

```rust
// Split outstanding balance into two loans: 70% and 30%
let outstanding = client.get_repayment_amount(&borrower);
let split1 = (outstanding * 70) / 100;
let split2 = outstanding - split1;

let mut split_amounts = Vec::new(&env);
split_amounts.push_back(split1);
split_amounts.push_back(split2);

// Split into 45-day loans
let new_loan_ids = client.split_loan(&borrower, &split_amounts, &(45 * 24 * 60 * 60));
```

## Multi-Loan Support

The contract now supports tracking multiple loans per user through:

- `UserLoans(Address)` storage key tracking loan IDs
- `get_user_loan_ids()` function to retrieve all user loans
- Automatic cleanup when loans are repaid or refinanced

## Security Considerations

1. **Reentrancy Protection**: All refinancing functions use reentrancy guards
2. **Authorization**: Only borrowers can modify their own loans
3. **Validation**: Comprehensive checks prevent invalid operations
4. **Fee Collection**: Fees are collected before loan modifications
5. **Collateral Safety**: Collateral is preserved throughout all operations

## Gas Optimization

- Minimal storage writes during operations
- Efficient vector operations for multi-loan tracking
- Batch operations for consolidation and splitting
- Optimized interest calculations

## Future Enhancements

Potential future improvements could include:

1. **Variable Refinancing Fees**: Tiered fees based on loan size or duration
2. **Partial Refinancing**: Allow refinancing only portions of loans
3. **Cross-Collateral Consolidation**: Support different collateral types
4. **Refinancing Rewards**: Protocol incentives for timely refinancing
5. **Advanced Splitting**: More flexible split ratios and conditions

## Testing

Comprehensive test suite covers:

- Basic refinancing operations
- Edge cases and error conditions
- Fee calculations
- Multi-loan scenarios
- Event emissions
- Security validations

Run tests with: `cargo test --lib`

## Integration Notes

When integrating with frontend applications:

1. Use `get_refinance_terms()` to show users expected costs
2. Monitor events for real-time updates
3. Handle all error cases gracefully
4. Verify loan status before attempting refinancing
5. Account for gas costs in fee calculations

## Conclusion

The loan refinancing system provides flexible options for borrowers to optimize their loan terms while maintaining protocol security and sustainability through appropriate fee mechanisms.

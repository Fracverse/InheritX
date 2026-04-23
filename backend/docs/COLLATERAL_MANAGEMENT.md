# Collateral Management System

## Overview

The Collateral Management System provides advanced collateral handling for active loans in the INHERITX lending protocol. Users can dynamically adjust their collateral to manage risk, optimize capital efficiency, and respond to market conditions.

## Key Features

### 1. Add Collateral (`add_collateral`)

**Purpose**: Add additional collateral to an existing active loan to improve health factor and reduce liquidation risk.

**Endpoint**: `POST /api/loans/lifecycle/:id/collateral/add`

**Request**:

```json
{
  "amount": "1.5",
  "transactionHash": "0x..."
}
```

**Validation**:

- Loan must exist and belong to the authenticated user
- Loan must be in `active` status
- Amount must be positive
- Associated plan must not be paused

**Effects**:

- Increases `collateral_amount` in the loan record
- Emits a `deposit` event
- Creates audit log entry
- Triggers health factor recalculation

**Example**:

```bash
curl -X POST http://localhost:3000/api/loans/lifecycle/550e8400-e29b-41d4-a716-446655440000/collateral/add \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"amount": "2.5"}'
```

### 2. Remove Collateral (`remove_collateral`)

**Purpose**: Withdraw excess collateral from a loan while maintaining minimum health factor.

**Endpoint**: `POST /api/loans/lifecycle/:id/collateral/remove`

**Request**:

```json
{
  "amount": "0.5",
  "transactionHash": "0x..."
}
```

**Validation**:

- Loan must exist and belong to the authenticated user
- Loan must be in `active` status
- Amount must be positive and not exceed current collateral
- Health factor must remain >= 150% (1.5) after removal
- Associated plan must not be paused

**Health Factor Check**:

```
health_factor = collateral_value_usd / debt_value_usd
Required: health_factor >= 1.5
```

**Effects**:

- Decreases `collateral_amount` in the loan record
- Emits a `deposit` event with negative amount
- Creates audit log entry
- Triggers health factor recalculation

**Error Cases**:

- `BadRequest`: Amount exceeds available collateral
- `BadRequest`: Health factor would drop below 150%
- `BadRequest`: Loan not found or plan is paused

**Example**:

```bash
curl -X POST http://localhost:3000/api/loans/lifecycle/550e8400-e29b-41d4-a716-446655440000/collateral/remove \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"amount": "0.5"}'
```

### 3. Swap Collateral (`swap_collateral`)

**Purpose**: Exchange collateral type without closing the loan, useful for portfolio rebalancing or risk management.

**Endpoint**: `POST /api/loans/lifecycle/:id/collateral/swap`

**Request**:

```json
{
  "newCollateralAsset": "BTC",
  "newCollateralAmount": "0.1",
  "transactionHash": "0x..."
}
```

**Validation**:

- Loan must exist and belong to the authenticated user
- Loan must be in `active` status
- New collateral asset must be whitelisted (price feed must exist)
- New collateral amount must maintain health factor >= 150%
- Associated plan must not be paused

**Effects**:

- Updates `collateral_asset` and `collateral_amount`
- Emits two `deposit` events:
  - Negative amount for old collateral (withdrawal)
  - Positive amount for new collateral (deposit)
- Creates audit log entry
- Triggers health factor recalculation

**Example**:

```bash
curl -X POST http://localhost:3000/api/loans/lifecycle/550e8400-e29b-41d4-a716-446655440000/collateral/swap \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "newCollateralAsset": "BTC",
    "newCollateralAmount": "0.1"
  }'
```

### 4. Get Collateral Value (`get_collateral_value`)

**Purpose**: Query current collateral value in USD with pricing information.

**Endpoint**: `GET /api/loans/lifecycle/:id/collateral/value`

**Response**:

```json
{
  "status": "success",
  "data": {
    "loanId": "550e8400-e29b-41d4-a716-446655440000",
    "collateralAsset": "ETH",
    "collateralAmount": "5.0",
    "collateralValueUsd": "10000.00",
    "currentPrice": "2000.00"
  }
}
```

**Example**:

```bash
curl -X GET http://localhost:3000/api/loans/lifecycle/550e8400-e29b-41d4-a716-446655440000/collateral/value \
  -H "Authorization: Bearer <token>"
```

### 5. Get Max Withdrawable Collateral (`get_max_withdrawable_collateral`)

**Purpose**: Calculate the maximum amount of collateral that can be safely withdrawn while maintaining 150% health factor.

**Endpoint**: `GET /api/loans/lifecycle/:id/collateral/max-withdrawable`

**Response**:

```json
{
  "status": "success",
  "data": {
    "loanId": "550e8400-e29b-41d4-a716-446655440000",
    "currentCollateralAmount": "5.0",
    "maxWithdrawableAmount": "1.25",
    "healthFactorAfterWithdrawal": "1.5",
    "minHealthFactor": "1.5"
  }
}
```

**Calculation**:

```
min_collateral_value = debt_value_usd * 1.5
min_collateral_amount = min_collateral_value / collateral_price
max_withdrawable = current_collateral - min_collateral_amount
```

**Example**:

```bash
curl -X GET http://localhost:3000/api/loans/lifecycle/550e8400-e29b-41d4-a716-446655440000/collateral/max-withdrawable \
  -H "Authorization: Bearer <token>"
```

### 6. Get Collateral Requirements (`get_collateral_requirements`)

**Purpose**: Get detailed collateral requirements and current status for a loan.

**Endpoint**: `GET /api/loans/lifecycle/:id/collateral/requirements`

**Response**:

```json
{
  "status": "success",
  "data": {
    "loanId": "550e8400-e29b-41d4-a716-446655440000",
    "borrowAsset": "USDC",
    "principal": "10000.00",
    "collateralAsset": "ETH",
    "requiredCollateralAmount": "7.5",
    "currentCollateralAmount": "5.0",
    "collateralSurplus": "-2.5",
    "healthFactor": "1.0",
    "minHealthFactor": "1.5"
  }
}
```

**Interpretation**:

- `collateralSurplus` > 0: Excess collateral available for withdrawal
- `collateralSurplus` < 0: Collateral deficit; additional collateral needed
- `healthFactor` >= 1.5: Loan is healthy
- `healthFactor` < 1.5: Loan is at risk of liquidation

**Example**:

```bash
curl -X GET http://localhost:3000/api/loans/lifecycle/550e8400-e29b-41d4-a716-446655440000/collateral/requirements \
  -H "Authorization: Bearer <token>"
```

## Health Factor Mechanics

### Definition

```
health_factor = collateral_value_usd / debt_value_usd
```

### Thresholds

- **Healthy**: HF >= 1.5 (150%)
- **At Risk**: 1.2 <= HF < 1.5 (120-150%)
- **Liquidation Risk**: HF < 1.2 (< 120%)

### Minimum Requirement

All collateral management operations enforce a minimum health factor of **1.5** to provide a safety buffer against price volatility.

### Recalculation

Health factor is recalculated:

- After each collateral operation (add, remove, swap)
- Continuously by the Risk Engine (every 60 seconds)
- When prices are updated

## Event Logging

All collateral operations emit events for audit and analytics:

### Deposit Events

- **Type**: `deposit`
- **Metadata**:
  - `collateral_ratio`: Optional collateral ratio
  - `total_deposited`: Total collateral amount after operation

### Audit Log Entries

- `COLLATERAL_ADDED`: When collateral is added
- `COLLATERAL_REMOVED`: When collateral is removed
- `COLLATERAL_SWAPPED`: When collateral type is changed

## Use Cases

### 1. Risk Mitigation

**Scenario**: User's loan health factor drops to 1.3 due to collateral price decline.

**Action**: Add more collateral to increase health factor back to 1.5+

```bash
curl -X POST /api/loans/lifecycle/:id/collateral/add \
  -d '{"amount": "1.0"}'
```

### 2. Capital Efficiency

**Scenario**: User has paid down 50% of their loan and has excess collateral.

**Action**: Withdraw excess collateral to redeploy capital

```bash
curl -X GET /api/loans/lifecycle/:id/collateral/max-withdrawable
# Response shows max_withdrawable_amount: 2.5

curl -X POST /api/loans/lifecycle/:id/collateral/remove \
  -d '{"amount": "2.0"}'
```

### 3. Portfolio Rebalancing

**Scenario**: User wants to shift from ETH to BTC collateral.

**Action**: Swap collateral type

```bash
curl -X POST /api/loans/lifecycle/:id/collateral/swap \
  -d '{
    "newCollateralAsset": "BTC",
    "newCollateralAmount": "0.1"
  }'
```

### 4. Loan Monitoring

**Scenario**: User wants to understand their collateral position.

**Action**: Query collateral requirements and value

```bash
curl -X GET /api/loans/lifecycle/:id/collateral/requirements
curl -X GET /api/loans/lifecycle/:id/collateral/value
```

## Error Handling

### Common Errors

| Error                                                                  | Cause                   | Solution                  |
| ---------------------------------------------------------------------- | ----------------------- | ------------------------- |
| `BadRequest: collateral amount must be greater than zero`              | Invalid amount          | Use positive amount       |
| `BadRequest: cannot remove collateral; health factor would drop to X%` | Insufficient collateral | Add more collateral first |
| `BadRequest: cannot remove X of collateral; only Y available`          | Exceeds available       | Request less collateral   |
| `BadRequest: collateral asset X is not supported`                      | Unsupported asset       | Use whitelisted asset     |
| `BadRequest: cannot add collateral to a loan that is repaid`           | Wrong loan status       | Loan must be active       |
| `NotFound: loan X not found`                                           | Loan doesn't exist      | Verify loan ID            |

## Database Schema

### Loan Lifecycle Table

```sql
CREATE TABLE loan_lifecycle (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    plan_id UUID,
    borrow_asset VARCHAR(20) NOT NULL,
    collateral_asset VARCHAR(20) NOT NULL,
    principal NUMERIC(30, 8) NOT NULL,
    interest_rate_bps INTEGER NOT NULL,
    collateral_amount NUMERIC(30, 8) NOT NULL,  -- Updated by collateral operations
    amount_repaid NUMERIC(30, 8) NOT NULL,
    status loan_lifecycle_status NOT NULL,
    due_date TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    ...
);
```

### Lending Events Table

```sql
CREATE TABLE lending_events (
    id UUID PRIMARY KEY,
    event_type event_type NOT NULL,  -- 'deposit' for collateral operations
    user_id UUID NOT NULL,
    plan_id UUID,
    asset_code VARCHAR(20) NOT NULL,
    amount VARCHAR(50) NOT NULL,  -- Positive for add, negative for remove
    metadata JSONB NOT NULL,
    ...
);
```

## Testing

### Unit Tests

Located in `backend/tests/collateral_management_tests.rs`

Tests cover:

- Health factor calculations
- Max withdrawable calculations
- Required collateral calculations
- Collateral surplus/deficit
- Price volatility impact
- Sequential operations

### Integration Tests

Run with:

```bash
cargo test collateral_management
```

## Performance Considerations

### Price Feed Caching

- Price feeds are cached for 3600 seconds (1 hour)
- Reduces database queries for repeated operations
- Automatic refresh on cache expiration

### Database Indexes

- `idx_loan_lifecycle_user_id`: Fast user lookups
- `idx_loan_lifecycle_status`: Fast status filtering
- `idx_lending_events_user_id`: Fast event queries

### Transaction Safety

- All operations use database transactions
- Row-level locking prevents race conditions
- Atomic updates ensure consistency

## Security Considerations

### Authorization

- All operations require user authentication
- Users can only modify their own loans
- Admin operations require admin authentication

### Input Validation

- All amounts validated as positive decimals
- Asset codes validated against price feed
- Health factor calculations use safe arithmetic

### Audit Trail

- All operations logged with user ID and timestamp
- Audit logs immutable and queryable
- Compliance-ready event tracking

## Future Enhancements

1. **Automated Collateral Management**
   - Auto-add collateral when health factor drops below threshold
   - Auto-remove collateral when surplus exceeds threshold

2. **Collateral Diversification**
   - Support multiple collateral types per loan
   - Weighted collateral calculations

3. **Advanced Risk Management**
   - Collateral liquidation preferences
   - Collateral insurance options
   - Dynamic collateral ratios based on asset volatility

4. **Analytics & Reporting**
   - Collateral utilization metrics
   - Historical collateral value tracking
   - Risk exposure dashboards

## References

- [Loan Lifecycle Documentation](./LEGAL_WILL_AUDIT_LOGS.md)
- [Risk Engine Documentation](./IMPLEMENTATION_SUMMARY.md)
- [Price Feed System](./IMPLEMENTATION_SUMMARY_DOWNLOAD_API.md)
- [Event System](./EVENTS.md)

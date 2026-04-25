# Collateral Management - Quick Reference Guide

## Overview

Advanced collateral management for INHERITX borrowing contracts. Users can add, remove, and swap collateral on active loans while maintaining a minimum 150% health factor.

## Core Functions

### 1. Add Collateral

```rust
CollateralManagementService::add_collateral(pool, &AddCollateralRequest)
```

**Use**: Increase collateral to improve health factor
**Endpoint**: `POST /api/loans/lifecycle/:id/collateral/add`
**Validation**: Amount > 0, Loan active, Plan not paused

### 2. Remove Collateral

```rust
CollateralManagementService::remove_collateral(pool, price_feed, &RemoveCollateralRequest)
```

**Use**: Withdraw excess collateral when loan is healthy
**Endpoint**: `POST /api/loans/lifecycle/:id/collateral/remove`
**Validation**: Amount > 0, HF >= 1.5 after removal, Amount <= current

### 3. Swap Collateral

```rust
CollateralManagementService::swap_collateral(pool, price_feed, &SwapCollateralRequest)
```

**Use**: Change collateral type without closing loan
**Endpoint**: `POST /api/loans/lifecycle/:id/collateral/swap`
**Validation**: New asset whitelisted, HF >= 1.5 with new collateral

### 4. Get Collateral Value

```rust
CollateralManagementService::get_collateral_value(pool, price_feed, loan_id)
```

**Use**: Query current collateral value in USD
**Endpoint**: `GET /api/loans/lifecycle/:id/collateral/value`
**Returns**: CollateralInfo with amount, value, and price

### 5. Get Max Withdrawable

```rust
CollateralManagementService::get_max_withdrawable_collateral(pool, price_feed, loan_id)
```

**Use**: Calculate safe withdrawal amount
**Endpoint**: `GET /api/loans/lifecycle/:id/collateral/max-withdrawable`
**Returns**: SafeWithdrawalInfo with max amount and resulting HF

### 6. Get Requirements

```rust
CollateralManagementService::get_required_collateral(pool, price_feed, loan_id)
```

**Use**: Get detailed collateral requirements
**Endpoint**: `GET /api/loans/lifecycle/:id/collateral/requirements`
**Returns**: CollateralRequirements with required, current, and surplus

## Health Factor Formula

```
health_factor = collateral_value_usd / debt_value_usd

Where:
  collateral_value_usd = collateral_amount * collateral_price
  debt_value_usd = (principal - amount_repaid) * borrow_price

Minimum Required: health_factor >= 1.5 (150%)
```

## Request/Response Examples

### Add Collateral Request

```json
{
  "amount": "1.5",
  "transactionHash": "0x..."
}
```

### Remove Collateral Request

```json
{
  "amount": "0.5",
  "transactionHash": "0x..."
}
```

### Swap Collateral Request

```json
{
  "newCollateralAsset": "BTC",
  "newCollateralAmount": "0.1",
  "transactionHash": "0x..."
}
```

### Collateral Value Response

```json
{
  "loanId": "550e8400-e29b-41d4-a716-446655440000",
  "collateralAsset": "ETH",
  "collateralAmount": "5.0",
  "collateralValueUsd": "10000.00",
  "currentPrice": "2000.00"
}
```

### Max Withdrawable Response

```json
{
  "loanId": "550e8400-e29b-41d4-a716-446655440000",
  "currentCollateralAmount": "5.0",
  "maxWithdrawableAmount": "1.25",
  "healthFactorAfterWithdrawal": "1.5",
  "minHealthFactor": "1.5"
}
```

### Requirements Response

```json
{
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
```

## Common Scenarios

### Scenario 1: Loan at Risk

**Problem**: Health factor dropped to 1.2 due to collateral price decline
**Solution**: Add collateral

```bash
curl -X POST /api/loans/lifecycle/:id/collateral/add \
  -d '{"amount": "1.0"}'
```

### Scenario 2: Excess Collateral

**Problem**: Paid down 50% of loan, have excess collateral
**Solution**: Check max withdrawable, then remove

```bash
curl -X GET /api/loans/lifecycle/:id/collateral/max-withdrawable
# Response: maxWithdrawableAmount: 2.5

curl -X POST /api/loans/lifecycle/:id/collateral/remove \
  -d '{"amount": "2.0"}'
```

### Scenario 3: Portfolio Rebalancing

**Problem**: Want to shift from ETH to BTC collateral
**Solution**: Swap collateral

```bash
curl -X POST /api/loans/lifecycle/:id/collateral/swap \
  -d '{
    "newCollateralAsset": "BTC",
    "newCollateralAmount": "0.1"
  }'
```

### Scenario 4: Loan Monitoring

**Problem**: Need to understand collateral position
**Solution**: Query requirements and value

```bash
curl -X GET /api/loans/lifecycle/:id/collateral/requirements
curl -X GET /api/loans/lifecycle/:id/collateral/value
```

## Error Codes

| Error                                  | Cause                      | Fix                       |
| -------------------------------------- | -------------------------- | ------------------------- |
| `BadRequest: amount must be > 0`       | Invalid amount             | Use positive amount       |
| `BadRequest: health factor would drop` | Insufficient collateral    | Add more collateral first |
| `BadRequest: amount exceeds available` | Exceeds current collateral | Request less collateral   |
| `BadRequest: asset not supported`      | Unsupported collateral     | Use whitelisted asset     |
| `BadRequest: cannot modify non-active` | Wrong loan status          | Loan must be active       |
| `NotFound: loan not found`             | Loan doesn't exist         | Verify loan ID            |

## Database Tables

### loan_lifecycle

```sql
collateral_asset VARCHAR(20)      -- Asset code (ETH, BTC, USDC, etc.)
collateral_amount NUMERIC(30, 8)  -- Amount of collateral
```

### lending_events

```sql
event_type = 'deposit'            -- Collateral operations
amount VARCHAR(50)                -- Positive for add, negative for remove
metadata JSONB                    -- Operation details
```

### audit_logs

```sql
action IN (
  'collateral_added',
  'collateral_removed',
  'collateral_swapped'
)
```

## Integration Checklist

- [ ] Import `CollateralManagementService` in your module
- [ ] Add routes to your router
- [ ] Implement authentication middleware
- [ ] Add error handling for API responses
- [ ] Test with real price feeds
- [ ] Verify health factor calculations
- [ ] Check audit logs
- [ ] Monitor event emissions
- [ ] Test edge cases
- [ ] Load test with concurrent operations

## Performance Tips

1. **Cache Price Feeds**: Prices cached for 1 hour, reduces DB queries
2. **Batch Operations**: Group multiple operations in transactions
3. **Use Indexes**: Queries use existing indexes on user_id and status
4. **Monitor Health Factor**: Check before operations to avoid failures
5. **Async Operations**: All operations are async, use proper await

## Security Checklist

- ✅ User authentication required
- ✅ Users can only modify their own loans
- ✅ All amounts validated as positive
- ✅ Health factor enforced at 150% minimum
- ✅ Transactions are atomic
- ✅ Row-level locking prevents race conditions
- ✅ All operations audited
- ✅ Immutable audit trail

## Testing

### Run Tests

```bash
cargo test collateral_management
```

### Test Coverage

- Add collateral validation
- Remove collateral validation
- Swap collateral validation
- Health factor calculations
- Max withdrawable calculations
- Required collateral calculations
- Price volatility impact
- Sequential operations

## Documentation

- **Full Guide**: `backend/docs/COLLATERAL_MANAGEMENT.md`
- **Implementation**: `COLLATERAL_MANAGEMENT_IMPLEMENTATION.md`
- **API Docs**: See endpoint descriptions above

## Support

For issues or questions:

1. Check error message and error codes table
2. Review full documentation
3. Check test cases for examples
4. Verify loan status is 'active'
5. Verify health factor >= 1.5
6. Check price feed availability

## Key Metrics

- **Health Factor Minimum**: 1.5 (150%)
- **Price Feed Cache**: 3600 seconds (1 hour)
- **Transaction Timeout**: Default (configurable)
- **Max Collateral Amount**: Unlimited (decimal precision)
- **Supported Assets**: Any asset with price feed

## Related Features

- **Loan Lifecycle**: Create, repay, liquidate loans
- **Risk Engine**: Monitors health factor continuously
- **Price Feed**: Provides real-time asset pricing
- **Event System**: Tracks all lending operations
- **Audit Logs**: Immutable operation history

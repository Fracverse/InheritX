# Lending Events System

## Overview

The lending events system provides structured event emission and indexing for DeFi lending operations. All events are stored in the `lending_events` table and can be queried efficiently by user, plan, transaction hash, or event type.

## Event Types

### 1. Deposit
Emitted when a user deposits collateral into the system.

**Metadata:**
```rust
{
    "collateral_ratio": Decimal,      // Optional: Current collateral ratio
    "total_deposited": Decimal        // Total amount deposited by user
}
```

### 2. Borrow
Emitted when a user borrows assets against their collateral.

**Metadata:**
```rust
{
    "interest_rate": Decimal,         // Annual interest rate (e.g., 5.5 for 5.5%)
    "collateral_asset": String,       // Asset used as collateral
    "collateral_amount": Decimal,     // Amount of collateral locked
    "loan_to_value": Decimal,         // LTV ratio (e.g., 75.00 for 75%)
    "maturity_date": Option<DateTime> // Optional loan maturity date
}
```

### 3. Repay
Emitted when a user repays borrowed assets.

**Metadata:**
```rust
{
    "principal_amount": Decimal,      // Principal portion of repayment
    "interest_amount": Decimal,       // Interest portion of repayment
    "remaining_balance": Decimal      // Remaining debt after repayment
}
```

### 4. Liquidation
Emitted when a position is liquidated due to insufficient collateral.

**Metadata:**
```rust
{
    "liquidator_id": Uuid,            // User who performed the liquidation
    "collateral_asset": String,       // Asset that was seized
    "collateral_seized": Decimal,     // Amount of collateral seized
    "debt_covered": Decimal,          // Amount of debt covered
    "liquidation_penalty": Decimal    // Penalty charged to borrower
}
```

### 5. Interest Accrual
Emitted when interest is accrued on a loan position.

**Metadata:**
```rust
{
    "interest_rate": Decimal,         // Current interest rate
    "principal_balance": Decimal,     // Principal balance before accrual
    "accrued_interest": Decimal,      // Amount of interest accrued
    "total_balance": Decimal          // Total balance after accrual
}
```

## Usage

### Emitting Events

Events should be emitted within database transactions to ensure atomicity:

```rust
use inheritx_backend::events::{EventService, DepositMetadata};
use rust_decimal_macros::dec;

async fn handle_deposit(
    pool: &PgPool,
    user_id: Uuid,
    plan_id: Option<Uuid>,
    amount: Decimal,
) -> Result<(), ApiError> {
    let mut tx = pool.begin().await?;
    
    // Perform deposit logic...
    
    // Emit deposit event
    let metadata = DepositMetadata {
        collateral_ratio: Some(dec!(150.00)),
        total_deposited: amount,
    };
    
    EventService::emit_deposit(
        &mut tx,
        user_id,
        plan_id,
        "USDC",
        amount,
        metadata,
        Some("0xabc123".to_string()),  // Transaction hash
        Some(12345),                    // Block number
    ).await?;
    
    tx.commit().await?;
    Ok(())
}
```

### Querying Events

#### Get User Events
```rust
let events = EventService::get_user_events(
    &pool,
    user_id,
    Some(EventType::Deposit),  // Optional: filter by type
    50,                         // Limit
    0                           // Offset
).await?;
```

#### Get Plan Events
```rust
let events = EventService::get_plan_events(
    &pool,
    plan_id,
    None,  // All event types
    50,
    0
).await?;
```

#### Get Events by Transaction Hash
```rust
let events = EventService::get_by_transaction_hash(
    &pool,
    "0xabc123"
).await?;
```

## API Endpoints

### GET /api/events
Get events for the authenticated user.

**Query Parameters:**
- `limit` (optional, default: 50, max: 100): Number of events to return
- `offset` (optional, default: 0): Pagination offset
- `event_type` (optional): Filter by event type (deposit, borrow, repay, liquidation, interest_accrual)

**Response:**
```json
{
  "events": [
    {
      "id": "uuid",
      "event_type": "deposit",
      "user_id": "uuid",
      "plan_id": "uuid",
      "asset_code": "USDC",
      "amount": "1000.00",
      "metadata": {
        "collateral_ratio": "150.00",
        "total_deposited": "1000.00"
      },
      "transaction_hash": "0xabc123",
      "block_number": 12345,
      "event_timestamp": "2026-02-26T10:00:00Z",
      "created_at": "2026-02-26T10:00:00Z"
    }
  ],
  "total": 1,
  "limit": 50,
  "offset": 0
}
```

### GET /api/events/plan/:plan_id
Get events for a specific plan.

**Query Parameters:** Same as above

### GET /api/events/transaction/:transaction_hash
Get all events associated with a transaction hash.

**Response:** Array of events

## Database Schema

```sql
CREATE TYPE event_type AS ENUM (
    'deposit',
    'borrow',
    'repay',
    'liquidation',
    'interest_accrual'
);

CREATE TABLE lending_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type event_type NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id UUID REFERENCES plans(id) ON DELETE SET NULL,
    asset_code VARCHAR(20) NOT NULL,
    amount DECIMAL(20, 8) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    transaction_hash VARCHAR(255),
    block_number BIGINT,
    event_timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT positive_amount CHECK (amount > 0)
);
```

## Indexes

The following indexes are created for efficient querying:

- `idx_lending_events_user_id`: Query by user
- `idx_lending_events_plan_id`: Query by plan
- `idx_lending_events_type`: Query by event type
- `idx_lending_events_asset_code`: Query by asset
- `idx_lending_events_timestamp`: Time-based queries
- `idx_lending_events_transaction_hash`: Query by transaction
- `idx_lending_events_metadata`: JSONB queries on metadata
- `idx_lending_events_user_type`: Composite index for user + type queries
- `idx_lending_events_plan_type`: Composite index for plan + type queries

## Testing

Run the event system tests:

```bash
cd backend
cargo test event_tests
```

## Integration Example

Here's a complete example of integrating event emission into a borrow operation:

```rust
pub async fn borrow_assets(
    pool: &PgPool,
    user_id: Uuid,
    plan_id: Uuid,
    borrow_amount: Decimal,
    collateral_amount: Decimal,
) -> Result<(), ApiError> {
    let mut tx = pool.begin().await?;
    
    // 1. Validate collateral
    let ltv = (borrow_amount / collateral_amount) * dec!(100);
    if ltv > dec!(75.00) {
        return Err(ApiError::BadRequest("LTV too high".to_string()));
    }
    
    // 2. Lock collateral
    sqlx::query(
        "UPDATE plans SET collateral_locked = collateral_locked + $1 WHERE id = $2"
    )
    .bind(collateral_amount)
    .bind(plan_id)
    .execute(&mut *tx)
    .await?;
    
    // 3. Create loan record
    let loan_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO loans (user_id, plan_id, amount, interest_rate) 
         VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(user_id)
    .bind(plan_id)
    .bind(borrow_amount)
    .bind(dec!(5.5))
    .fetch_one(&mut *tx)
    .await?;
    
    // 4. Emit borrow event
    let metadata = BorrowMetadata {
        interest_rate: dec!(5.5),
        collateral_asset: "USDC".to_string(),
        collateral_amount,
        loan_to_value: ltv,
        maturity_date: None,
    };
    
    EventService::emit_borrow(
        &mut tx,
        user_id,
        Some(plan_id),
        "USDC",
        borrow_amount,
        metadata,
        None,
        None,
    ).await?;
    
    // 5. Commit transaction
    tx.commit().await?;
    
    Ok(())
}
```

## Best Practices

1. **Always emit events within transactions**: This ensures atomicity between business logic and event emission.

2. **Include relevant metadata**: Store all contextual information that might be useful for indexing or analytics.

3. **Use transaction hashes when available**: This enables cross-referencing with blockchain transactions.

4. **Query with appropriate limits**: Use pagination to avoid loading too many events at once.

5. **Index on common query patterns**: The provided indexes cover most use cases, but add custom indexes if needed.

6. **Monitor event volume**: Consider archiving old events if the table grows too large.

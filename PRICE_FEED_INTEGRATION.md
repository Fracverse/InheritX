# Price Feed Integration for Collateral Valuation

## Overview

This document describes the price feed integration for collateral valuation in InheritX. The system enables real-time asset pricing, collateral tracking, and valuation calculations across the backend, smart contracts, and frontend.

## Architecture

### Components

1. **Backend Price Feed Service** (`backend/src/price_feed.rs`)
   - Fetches and caches asset prices
   - Manages price feed configurations
   - Calculates collateral valuations
   - Persists price history

2. **Smart Contract Price Module** (`contracts/inheritance-contract/src/price_feed.rs`)
   - Validates collateral ratios on-chain
   - Verifies price freshness
   - Calculates plan valuations

3. **API Endpoints** (`backend/src/price_feed_handlers.rs`)
   - Public price endpoints
   - Admin price management
   - Valuation calculations

4. **Database Schema** (`backend/migrations/20260225_add_price_feeds.sql`)
   - Price feed configurations
   - Asset price history
   - Collateral tracking

## Database Schema

### Tables

#### `price_feeds`
Stores price feed configurations for each asset.

```sql
CREATE TABLE price_feeds (
    id UUID PRIMARY KEY,
    asset_code VARCHAR(20) NOT NULL UNIQUE,
    source VARCHAR(50) NOT NULL,           -- 'pyth', 'chainlink', 'custom'
    feed_id VARCHAR(255) NOT NULL,         -- External feed identifier
    is_active BOOLEAN DEFAULT TRUE,
    last_updated TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
);
```

#### `asset_price_history`
Maintains historical price data for auditing and analysis.

```sql
CREATE TABLE asset_price_history (
    id UUID PRIMARY KEY,
    asset_code VARCHAR(20) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    price_timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    source VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE,
    FOREIGN KEY (asset_code) REFERENCES price_feeds(asset_code)
);
```

#### `plans` (Extended)
Added collateral tracking fields to existing plans table.

```sql
ALTER TABLE plans ADD COLUMN collateral_ratio DECIMAL(5, 2) DEFAULT 100.00;
ALTER TABLE plans ADD COLUMN asset_code VARCHAR(20) DEFAULT 'USDC';
ALTER TABLE plans ADD COLUMN valuation_usd DECIMAL(20, 8);
ALTER TABLE plans ADD COLUMN last_valuation_at TIMESTAMP WITH TIME ZONE;
```

## API Endpoints

### Public Endpoints

#### Get Current Price
```
GET /api/prices/:asset_code
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "asset_code": "USDC",
    "price": "1.00000000",
    "timestamp": "2026-02-25T10:30:00Z",
    "source": "custom"
  }
}
```

#### Get Price History
```
GET /api/prices/:asset_code/history
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "asset_code": "USDC",
    "prices": [
      {
        "price": "1.00000000",
        "timestamp": "2026-02-25T10:30:00Z",
        "source": "custom"
      }
    ]
  }
}
```

#### Calculate Valuation
```
GET /api/valuations/:asset_code/:amount
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "asset_code": "USDC",
    "amount": "1000.00",
    "current_price": "1.00000000",
    "valuation_usd": "1000.00000000",
    "collateral_ratio": "100.00",
    "last_updated": "2026-02-25T10:30:00Z"
  }
}
```

#### Get Plan Valuation
```
GET /api/plans/:plan_id/valuation
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "plan_id": "550e8400-e29b-41d4-a716-446655440000",
    "asset_code": "USDC",
    "amount": "5000.00",
    "current_price": "1.00000000",
    "valuation_usd": "5000.00000000",
    "collateral_ratio": "100.00",
    "last_updated": "2026-02-25T10:30:00Z"
  }
}
```

### Admin Endpoints

#### Register Price Feed
```
POST /api/admin/price-feeds/register
Authorization: Bearer <admin_token>
```

**Request:**
```json
{
  "asset_code": "ETH",
  "source": "pyth",
  "feed_id": "eth-usd-feed-id"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Price feed registered for ETH",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "asset_code": "ETH",
    "source": "pyth",
    "feed_id": "eth-usd-feed-id",
    "is_active": true
  }
}
```

#### Update Price
```
POST /api/admin/prices/:asset_code
Authorization: Bearer <admin_token>
```

**Request:**
```json
{
  "price": "2500.50000000"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Price updated for ETH",
  "data": {
    "asset_code": "ETH",
    "price": "2500.50000000",
    "timestamp": "2026-02-25T10:30:00Z",
    "source": "custom"
  }
}
```

#### Get Active Feeds
```
GET /api/admin/price-feeds
Authorization: Bearer <admin_token>
```

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "asset_code": "USDC",
      "source": "custom",
      "feed_id": "usdc-usd",
      "is_active": true,
      "last_updated": "2026-02-25T10:30:00Z"
    }
  ],
  "count": 1
}
```

## Service Implementation

### PriceFeedService Trait

```rust
#[async_trait]
pub trait PriceFeedService: Send + Sync {
    async fn get_price(&self, asset_code: &str) -> Result<AssetPrice, ApiError>;
    async fn get_price_history(&self, asset_code: &str, limit: i64) -> Result<Vec<AssetPrice>, ApiError>;
    async fn register_feed(&self, asset_code: &str, source: PriceFeedSource, feed_id: &str) -> Result<PriceFeedConfig, ApiError>;
    async fn update_price(&self, asset_code: &str, price: Decimal) -> Result<AssetPrice, ApiError>;
    async fn calculate_valuation(&self, asset_code: &str, amount: Decimal) -> Result<CollateralValuation, ApiError>;
    async fn get_active_feeds(&self) -> Result<Vec<PriceFeedConfig>, ApiError>;
}
```

### DefaultPriceFeedService

Implements the `PriceFeedService` trait with:
- In-memory price caching (configurable TTL)
- Database persistence
- Automatic initialization of default feeds (USDC)
- Price history tracking

**Features:**
- Cache TTL: 300 seconds (5 minutes) by default
- Automatic USDC feed initialization
- Decimal precision: 8 decimal places
- Timestamp tracking for all prices

## Smart Contract Integration

### Price Validation

The smart contract includes price validation for collateral:

```rust
pub fn validate_collateral(
    env: &Env,
    amount: u64,
    price: U128,
    min_ratio_bp: u32,
) -> Result<CollateralValuation, &'static str>
```

**Parameters:**
- `amount`: Plan amount in base units
- `price`: Asset price in USD (8 decimal places)
- `min_ratio_bp`: Minimum collateral ratio in basis points

**Returns:**
- `CollateralValuation` with calculated valuation and ratio
- Error if collateral ratio is insufficient

### Price Freshness Verification

```rust
pub fn verify_price_freshness(
    env: &Env,
    price_timestamp: u64,
    max_age_secs: u64,
) -> Result<(), &'static str>
```

Ensures prices are not stale before using them for critical operations.

## Acceptance Criteria

### ✅ Collateral Value Updates Correctly

1. **Price Updates Propagate**
   - When a price is updated via `/api/admin/prices/:asset_code`, it's immediately available
   - Price history is maintained for audit trail
   - Cache is invalidated and refreshed

2. **Valuation Calculations**
   - Valuations are calculated as: `amount × current_price`
   - Decimal precision is maintained (8 decimal places)
   - Results are consistent across multiple calls

3. **Plan Valuations**
   - Plan valuations reflect current asset prices
   - Collateral ratios are calculated correctly
   - Historical valuations are tracked

4. **Database Consistency**
   - Price history is persisted
   - Collateral ratios are updated in plans table
   - Timestamps are accurate

## Testing

### Unit Tests

Run price feed tests:
```bash
cargo test --test price_feed_tests
```

**Test Coverage:**
- Price feed initialization
- Price registration and updates
- Price history retrieval
- Valuation calculations
- Collateral ratio calculations
- Cache validity
- Error handling

### Integration Tests

The test suite includes:
- Database connectivity
- Price persistence
- Cache behavior
- Multi-asset support
- Timestamp accuracy

## Configuration

### Environment Variables

```bash
# Database connection
DATABASE_URL=postgres://user:password@localhost/inheritx

# Price feed cache TTL (seconds)
PRICE_FEED_CACHE_TTL=300

# Price feed sources
PRICE_FEED_PYTH_ENABLED=true
PRICE_FEED_CHAINLINK_ENABLED=false
```

### Default Configuration

- **Cache TTL**: 300 seconds (5 minutes)
- **Default Asset**: USDC
- **Decimal Precision**: 8 decimal places
- **Collateral Ratio**: 100% (MVP)

## Future Enhancements

1. **Oracle Integration**
   - Pyth Network integration for real-time prices
   - Chainlink fallback support
   - Multi-source price aggregation

2. **Dynamic Collateral Ratios**
   - Risk-based collateral requirements
   - Asset-specific ratios
   - Time-based adjustments

3. **Liquidation Mechanisms**
   - Automatic liquidation triggers
   - Partial liquidation support
   - Liquidation auctions

4. **Advanced Analytics**
   - Price volatility tracking
   - Collateral health monitoring
   - Risk assessment dashboard

5. **Frontend Integration**
   - Real-time price display
   - Collateral ratio visualization
   - Price alert notifications

## Troubleshooting

### Price Not Found
- Ensure price feed is registered: `POST /api/admin/price-feeds/register`
- Verify asset code matches exactly (case-sensitive)
- Check that feed is active: `GET /api/admin/price-feeds`

### Stale Price Data
- Check `last_updated` timestamp
- Update price: `POST /api/admin/prices/:asset_code`
- Verify cache TTL setting

### Valuation Mismatch
- Confirm asset code and amount
- Check decimal precision (8 places)
- Verify price is current

## Security Considerations

1. **Admin-Only Operations**
   - Price registration requires admin authentication
   - Price updates require admin authentication
   - Feed management is restricted to admins

2. **Price Validation**
   - Prices must be positive
   - Amounts must be positive
   - Timestamps must be valid

3. **Audit Trail**
   - All price updates are logged
   - Historical prices are maintained
   - Source tracking for compliance

## References

- [Soroban SDK Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Pyth Network](https://pyth.network/)
- [Chainlink](https://chain.link/)
- [InheritX Smart Contract](./contracts/inheritance-contract/src/lib.rs)

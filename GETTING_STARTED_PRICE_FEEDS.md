# Getting Started with Price Feeds

## 5-Minute Quick Start

### 1. Run Migrations
```bash
cd InheritX/backend
sqlx migrate run
```

### 2. Start Backend
```bash
cargo run
```

### 3. Test It Works
```bash
# In another terminal
curl http://localhost:8000/api/prices/USDC
```

**Expected Response:**
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

---

## Common Operations

### Get Current Price
```bash
curl http://localhost:8000/api/prices/USDC
```

### Update Price (Admin)
```bash
curl -X POST http://localhost:8000/api/admin/prices/USDC \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"price": "1.05000000"}'
```

### Calculate Valuation
```bash
# What's 1000 USDC worth?
curl http://localhost:8000/api/valuations/USDC/1000
```

### Get Plan Valuation
```bash
# What's plan 550e8400-e29b-41d4-a716-446655440000 worth?
curl http://localhost:8000/api/plans/550e8400-e29b-41d4-a716-446655440000/valuation
```

### Register New Asset
```bash
curl -X POST http://localhost:8000/api/admin/price-feeds/register \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "asset_code": "ETH",
    "source": "pyth",
    "feed_id": "eth-usd-feed"
  }'
```

### Get Price History
```bash
curl http://localhost:8000/api/prices/USDC/history
```

---

## Testing

### Run All Tests
```bash
cd InheritX/backend
cargo test --test price_feed_tests
```

### Run Specific Test
```bash
cargo test --test price_feed_tests test_update_and_get_price -- --nocapture
```

### Expected Output
```
test test_price_feed_initialization ... ok
test test_register_price_feed ... ok
test test_update_and_get_price ... ok
test test_price_history ... ok
test test_calculate_valuation ... ok
test test_get_active_feeds ... ok
test test_price_cache_validity ... ok
test test_collateral_ratio_calculation ... ok
test test_price_feed_not_found ... ok
test test_invalid_price_update ... ok

test result: ok. 10 passed
```

---

## Understanding the Data

### Price Response
```json
{
  "asset_code": "USDC",      // Asset identifier
  "price": "1.00000000",     // Price in USD (8 decimals)
  "timestamp": "2026-02-25T10:30:00Z",  // When price was set
  "source": "custom"         // Where price came from
}
```

### Valuation Response
```json
{
  "asset_code": "USDC",
  "amount": "1000.00",       // Amount of asset
  "current_price": "1.00000000",  // Current price per unit
  "valuation_usd": "1000.00000000",  // Total value (amount × price)
  "collateral_ratio": "100.00",  // Collateral ratio %
  "last_updated": "2026-02-25T10:30:00Z"
}
```

### Plan Valuation Response
```json
{
  "plan_id": "550e8400-e29b-41d4-a716-446655440000",
  "asset_code": "USDC",
  "amount": "5000.00",
  "current_price": "1.00000000",
  "valuation_usd": "5000.00000000",
  "collateral_ratio": "100.00",
  "last_updated": "2026-02-25T10:30:00Z"
}
```

---

## Troubleshooting

### "Price not found for asset"
**Problem**: Asset doesn't have a price set
**Solution**: 
1. Register the asset: `POST /api/admin/price-feeds/register`
2. Update the price: `POST /api/admin/prices/:asset_code`

### "Database error"
**Problem**: Database connection failed
**Solution**:
1. Check DATABASE_URL environment variable
2. Ensure PostgreSQL is running
3. Run migrations: `sqlx migrate run`

### "Unauthorized"
**Problem**: Admin endpoint without token
**Solution**: Add Authorization header with valid admin token

### "Invalid amount format"
**Problem**: Amount parameter is not a valid number
**Solution**: Use valid decimal format (e.g., "1000" or "1000.50")

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend/Client                       │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                   API Endpoints                          │
│  GET /api/prices/:asset_code                            │
│  POST /api/admin/prices/:asset_code                     │
│  GET /api/valuations/:asset_code/:amount               │
│  GET /api/plans/:plan_id/valuation                     │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│              PriceFeedService                            │
│  • In-memory caching (300s TTL)                         │
│  • Price calculation                                     │
│  • Valuation computation                                │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                  PostgreSQL Database                     │
│  • price_feeds table                                    │
│  • asset_price_history table                           │
│  • plans table (extended)                              │
└─────────────────────────────────────────────────────────┘
```

---

## Key Concepts

### Price
The current market value of an asset in USD.
- Example: USDC = $1.00
- Precision: 8 decimal places
- Updated via admin endpoint

### Valuation
The total USD value of an amount of an asset.
- Formula: `amount × price`
- Example: 1000 USDC × $1.00 = $1000.00

### Collateral Ratio
The percentage of collateral backing a plan.
- Current: 100% (MVP)
- Future: Risk-based ratios

### Cache
In-memory storage of recent prices.
- TTL: 300 seconds (5 minutes)
- Reduces database queries
- Improves performance

---

## Next Steps

### Learn More
- Read `PRICE_FEED_INTEGRATION.md` for technical details
- Read `IMPLEMENTATION_GUIDE.md` for setup details
- Check `PRICE_FEED_QUICK_REFERENCE.md` for API reference

### Integrate with Frontend
- Display current prices
- Show valuations
- Alert on price changes

### Add Oracle Integration
- Connect to Pyth Network
- Add Chainlink support
- Implement price aggregation

### Implement Advanced Features
- Dynamic collateral ratios
- Liquidation mechanisms
- Risk assessment

---

## File Locations

### Source Code
- Service: `backend/src/price_feed.rs`
- Handlers: `backend/src/price_feed_handlers.rs`
- Tests: `backend/tests/price_feed_tests.rs`

### Database
- Migration: `backend/migrations/20260225_add_price_feeds.sql`

### Smart Contract
- Module: `contracts/inheritance-contract/src/price_feed.rs`

### Documentation
- Technical: `PRICE_FEED_INTEGRATION.md`
- Setup: `IMPLEMENTATION_GUIDE.md`
- Reference: `PRICE_FEED_QUICK_REFERENCE.md`
- Summary: `PRICE_FEED_SUMMARY.md`
- Checklist: `PRICE_FEED_CHECKLIST.md`
- This File: `GETTING_STARTED_PRICE_FEEDS.md`

---

## Support

### Documentation
- **Technical Details**: `PRICE_FEED_INTEGRATION.md`
- **Setup Help**: `IMPLEMENTATION_GUIDE.md`
- **Quick Answers**: `PRICE_FEED_QUICK_REFERENCE.md`

### Code
- **Service**: `backend/src/price_feed.rs`
- **Tests**: `backend/tests/price_feed_tests.rs`
- **Examples**: See test cases

### Questions?
1. Check the relevant documentation file
2. Review the test cases for examples
3. Check the implementation code
4. Review error messages and logs

---

## Success Criteria

You'll know it's working when:
- ✅ `cargo test --test price_feed_tests` passes
- ✅ `GET /api/prices/USDC` returns a price
- ✅ `POST /api/admin/prices/USDC` updates the price
- ✅ `GET /api/valuations/USDC/1000` calculates correctly
- ✅ `GET /api/plans/{id}/valuation` returns plan valuation

---

## Quick Reference

| Task | Command |
|------|---------|
| Run migrations | `sqlx migrate run` |
| Start backend | `cargo run` |
| Run tests | `cargo test --test price_feed_tests` |
| Get price | `curl http://localhost:8000/api/prices/USDC` |
| Update price | `curl -X POST http://localhost:8000/api/admin/prices/USDC -H "Authorization: Bearer TOKEN" -H "Content-Type: application/json" -d '{"price": "1.00000000"}'` |
| Calculate valuation | `curl http://localhost:8000/api/valuations/USDC/1000` |
| Get plan valuation | `curl http://localhost:8000/api/plans/{id}/valuation` |

---

**Ready to go!** 🚀

Start with the 5-minute quick start above, then explore the documentation for more details.

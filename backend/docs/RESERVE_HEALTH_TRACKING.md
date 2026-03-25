# Reserve Health and Coverage Ratio Tracking

## Overview

The Reserve Health Tracking system monitors the health and adequacy of liquidity pool reserves in the lending protocol. It calculates coverage ratios, tracks reserve adequacy, and alerts administrators when reserves fall below safe thresholds.

## Key Metrics

### Coverage Ratio
The ratio of bad debt reserves to utilized liquidity:
```
Coverage Ratio = Bad Debt Reserve / Utilized Liquidity
```

This metric indicates how well the protocol can absorb potential defaults. Higher ratios indicate better protection against bad debt.

### Reserve Adequacy
The percentage of total liquidity held in reserves:
```
Reserve Adequacy = (Bad Debt Reserve / Total Liquidity) × 100
```

### Utilization Rate
The percentage of pool liquidity currently borrowed:
```
Utilization Rate = (Utilized Liquidity / Total Liquidity) × 100
```

### Health Status
Determined based on coverage ratio and utilization:
- **healthy**: Coverage ratio ≥ 10%, normal utilization
- **moderate**: Coverage ratio between 5-10%
- **warning**: Coverage ratio between 5-15% (approaching threshold)
- **critical**: Coverage ratio < 5%
- **high_utilization**: Utilization > 90%

## Thresholds

Default thresholds (configurable):
- Minimum Coverage Ratio: 10%
- Warning Coverage Ratio: 15%
- Critical Coverage Ratio: 5%

## Monitoring

The Reserve Health Engine runs automatically every 5 minutes to:
1. Calculate metrics for all pools
2. Update database with current health status
3. Send alerts to admins on status changes
4. Log critical threshold breaches

## API Endpoints

### Get All Reserve Health
```
GET /api/admin/reserve-health
Authorization: Bearer <admin_token>
```

Returns health metrics for all pools.

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "asset_code": "USDC",
      "coverage_ratio": 0.12,
      "utilization_rate": 75.5,
      "reserve_adequacy": 9.0,
      "health_status": "healthy",
      "bad_debt_reserve": 90000.0,
      "total_liquidity": 1000000.0,
      "utilized_liquidity": 755000.0,
      "available_liquidity": 245000.0
    }
  ]
}
```

### Get Reserve Health by Asset
```
GET /api/admin/reserve-health/:asset_code
Authorization: Bearer <admin_token>
```

Returns health metrics for a specific asset pool.

### Sync Reserve Health
```
POST /api/admin/reserve-health/sync
Authorization: Bearer <admin_token>
```

Manually triggers synchronization of reserves from lending events and recalculates health metrics.

### Analytics Endpoint
```
GET /api/admin/analytics/reserve-health
Authorization: Bearer <admin_token>
```

Returns reserve health metrics as part of the analytics dashboard.

## Database Schema

### Pools Table Extensions
```sql
ALTER TABLE pools ADD COLUMN bad_debt_reserve DECIMAL(20, 8) NOT NULL DEFAULT 0;
ALTER TABLE pools ADD COLUMN retained_yield DECIMAL(20, 8) NOT NULL DEFAULT 0;
ALTER TABLE pools ADD COLUMN coverage_ratio DECIMAL(10, 4);
ALTER TABLE pools ADD COLUMN reserve_health_status VARCHAR(20) DEFAULT 'healthy';
ALTER TABLE pools ADD COLUMN last_health_check_at TIMESTAMP WITH TIME ZONE;
```

## Alerts and Notifications

Administrators receive notifications when:
- Pool status changes (e.g., healthy → warning)
- Coverage ratio falls below critical threshold (5%)
- Utilization exceeds 90%

Notifications are sent via the NotificationService and logged in the audit log.

## Integration with Lending Contract

The system syncs with on-chain lending contract state:
- Tracks `bad_debt_reserve` from protocol interest allocation
- Monitors `retained_yield` for protocol revenue
- Updates `utilized_liquidity` from borrow/repay events

## Usage Example

```rust
// Initialize the engine
let reserve_health_engine = Arc::new(ReserveHealthEngine::new(db.clone()));
reserve_health_engine.clone().start();

// Manual check
let metrics = reserve_health_engine.check_all_reserves().await?;

// Get specific asset health
let usdc_health = reserve_health_engine.get_reserve_health("USDC").await?;

// Sync from events
reserve_health_engine.sync_reserves_from_events().await?;
```

## Best Practices

1. **Monitor Regularly**: The automated 5-minute check interval ensures timely detection of issues
2. **Set Appropriate Thresholds**: Adjust thresholds based on protocol risk tolerance
3. **Respond to Alerts**: Critical alerts require immediate attention
4. **Sync After Major Events**: Manually sync after large borrows or repayments
5. **Track Trends**: Monitor coverage ratio trends over time to anticipate issues

## Future Enhancements

- Historical tracking of reserve health metrics
- Predictive alerts based on trend analysis
- Automated reserve rebalancing
- Integration with liquidation bot for coordinated risk management
- Multi-asset reserve pooling strategies

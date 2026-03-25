# Reserve Health Tracking - Architecture

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Reserve Health System                        │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────────┐         ┌──────────────────────┐
│  Lending Contract    │         │   Lending Events     │
│  (Soroban)           │────────▶│   Database           │
│                      │         │                      │
│  - Borrows           │         │  - Borrow events     │
│  - Repays            │         │  - Repay events      │
│  - Bad debt reserve  │         │  - Liquidations      │
│  - Retained yield    │         │  - Interest accrual  │
└──────────────────────┘         └──────────────────────┘
                                           │
                                           │ Sync
                                           ▼
                                 ┌──────────────────────┐
                                 │   Pools Table        │
                                 │                      │
                                 │  - total_liquidity   │
                                 │  - utilized_liquidity│
                                 │  - bad_debt_reserve  │
                                 │  - retained_yield    │
                                 │  - coverage_ratio    │
                                 │  - health_status     │
                                 └──────────────────────┘
                                           │
                                           │ Read
                                           ▼
                        ┌────────────────────────────────────┐
                        │   Reserve Health Engine            │
                        │   (Background Task - 5 min cycle)  │
                        │                                    │
                        │  1. Load all pools                 │
                        │  2. Calculate metrics:             │
                        │     - Coverage ratio               │
                        │     - Utilization rate             │
                        │     - Reserve adequacy             │
                        │  3. Determine health status        │
                        │  4. Update database                │
                        │  5. Check for alerts               │
                        └────────────────────────────────────┘
                                     │         │
                        ┌────────────┘         └────────────┐
                        ▼                                    ▼
            ┌──────────────────────┐           ┌──────────────────────┐
            │  Notification        │           │  Audit Log           │
            │  Service             │           │  Service             │
            │                      │           │                      │
            │  - Alert admins      │           │  - Log status changes│
            │  - Status changes    │           │  - Track events      │
            │  - Critical warnings │           │  - Compliance trail  │
            └──────────────────────┘           └──────────────────────┘
                        │
                        ▼
            ┌──────────────────────┐
            │  Admin Dashboard     │
            │                      │
            │  - View metrics      │
            │  - Monitor alerts    │
            │  - Trigger sync      │
            └──────────────────────┘
```

## Data Flow

### 1. Event Capture
```
Lending Contract → Blockchain Events → Event Handlers → Database
```

### 2. Reserve Synchronization
```
Lending Events → Aggregate by Asset → Update Pools Table
```

### 3. Health Calculation
```
Pools Data → Calculate Metrics → Determine Status → Update Database
```

### 4. Alert Flow
```
Status Change → Check Thresholds → Send Notifications → Log Audit
```

## Metric Calculations

### Coverage Ratio
```
Coverage Ratio = Bad Debt Reserve / Utilized Liquidity

Example:
  Bad Debt Reserve: 100,000 USDC
  Utilized Liquidity: 750,000 USDC
  Coverage Ratio: 100,000 / 750,000 = 0.133 (13.3%)
```

### Utilization Rate
```
Utilization Rate = (Utilized Liquidity / Total Liquidity) × 100

Example:
  Utilized: 750,000 USDC
  Total: 1,000,000 USDC
  Utilization: (750,000 / 1,000,000) × 100 = 75%
```

### Reserve Adequacy
```
Reserve Adequacy = (Bad Debt Reserve / Total Liquidity) × 100

Example:
  Bad Debt Reserve: 100,000 USDC
  Total Liquidity: 1,000,000 USDC
  Reserve Adequacy: (100,000 / 1,000,000) × 100 = 10%
```

## Health Status Decision Tree

```
                    ┌─────────────────┐
                    │ Calculate       │
                    │ Coverage Ratio  │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │ Coverage < 5%?  │
                    └────────┬────────┘
                         Yes │ No
                    ┌────────▼────────┐
                    │   CRITICAL      │
                    └─────────────────┘
                             │
                    ┌────────▼────────┐
                    │ Coverage < 15%? │
                    └────────┬────────┘
                         Yes │ No
                    ┌────────▼────────┐
                    │   WARNING       │
                    └─────────────────┘
                             │
                    ┌────────▼────────┐
                    │ Utilization>90%?│
                    └────────┬────────┘
                         Yes │ No
                    ┌────────▼────────┐
                    │ HIGH_UTILIZATION│
                    └─────────────────┘
                             │
                    ┌────────▼────────┐
                    │ Coverage ≥ 10%? │
                    └────────┬────────┘
                         Yes │ No
                    ┌────────▼────────┐
                    │   HEALTHY       │
                    └─────────────────┘
                             │
                    ┌────────▼────────┐
                    │   MODERATE      │
                    └─────────────────┘
```

## API Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    API Layer                             │
└─────────────────────────────────────────────────────────┘

GET /api/admin/reserve-health
    │
    ├─▶ Authenticate Admin
    │
    ├─▶ Reserve Health Engine
    │       │
    │       ├─▶ Load all pools from DB
    │       ├─▶ Calculate metrics for each
    │       ├─▶ Update health status
    │       └─▶ Return metrics array
    │
    └─▶ Return JSON response

GET /api/admin/reserve-health/:asset_code
    │
    ├─▶ Authenticate Admin
    │
    ├─▶ Reserve Health Engine
    │       │
    │       ├─▶ Load specific pool
    │       ├─▶ Calculate metrics
    │       └─▶ Return single metric
    │
    └─▶ Return JSON response

POST /api/admin/reserve-health/sync
    │
    ├─▶ Authenticate Admin
    │
    ├─▶ Reserve Health Engine
    │       │
    │       ├─▶ Aggregate lending events
    │       ├─▶ Update pool utilization
    │       ├─▶ Recalculate all metrics
    │       └─▶ Return updated metrics
    │
    └─▶ Return JSON response

GET /api/admin/analytics/reserve-health
    │
    ├─▶ Authenticate Admin
    │
    ├─▶ Reserve Health Engine
    │       │
    │       └─▶ Return cached metrics
    │
    └─▶ Return JSON response (analytics format)
```

## Background Task Lifecycle

```
Application Start
    │
    ├─▶ Initialize Reserve Health Engine
    │       │
    │       ├─▶ Set thresholds
    │       └─▶ Connect to database
    │
    ├─▶ Start background task
    │       │
    │       └─▶ Spawn tokio task
    │
    └─▶ Enter monitoring loop
            │
            ├─▶ Wait 5 minutes
            │
            ├─▶ Check all reserves
            │       │
            │       ├─▶ Load pools
            │       ├─▶ Calculate metrics
            │       ├─▶ Update database
            │       ├─▶ Check alerts
            │       └─▶ Log results
            │
            └─▶ Repeat
```

## Integration Points

### With Risk Engine
```
Reserve Health Engine ←→ Risk Engine
    │                        │
    ├─ Share notification    ├─ Monitor loan health
    │  infrastructure         │
    │                        ├─ Track collateral
    ├─ Pool-level risk       │  ratios
    │  assessment             │
    │                        └─ Liquidation triggers
    └─ Coordinate alerts
```

### With Stress Testing
```
Stress Testing Engine → Reserve Health Engine
    │                        │
    ├─ Simulate price crash  ├─ Monitor impact
    │                        │
    ├─ Drain liquidity       ├─ Track coverage
    │                        │  changes
    │                        │
    └─ Mass default          └─ Alert on critical
                                thresholds
```

### With Analytics
```
Reserve Health Engine → Analytics Dashboard
    │                        │
    ├─ Provide metrics       ├─ Display trends
    │                        │
    ├─ Historical data       ├─ Generate reports
    │                        │
    └─ Real-time status      └─ Admin visibility
```

## Security Considerations

1. **Authentication**: All endpoints require admin authentication
2. **Authorization**: Only admins can view reserve health
3. **Rate Limiting**: API endpoints are rate-limited
4. **Audit Trail**: All status changes are logged
5. **Data Integrity**: Metrics calculated from verified sources

## Performance Characteristics

- **Background Task**: Runs every 5 minutes (configurable)
- **Database Queries**: Optimized with indexes
- **API Response Time**: < 100ms for cached metrics
- **Sync Operation**: < 1s for typical event volumes
- **Memory Usage**: Minimal (stateless calculations)

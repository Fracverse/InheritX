# Reserve Health Testing Guide

## Overview

This guide provides comprehensive testing procedures for the Reserve Health and Coverage Ratio Tracking system.

## Prerequisites

- PostgreSQL database running
- Backend service running (or ready to start)
- Admin authentication token
- `curl` and `jq` installed (for API tests)
- `psql` installed (for SQL tests)

## Testing Approach

We'll test the system at three levels:
1. **Database Level** - SQL queries to verify data structure and calculations
2. **API Level** - HTTP requests to test endpoints
3. **Integration Level** - End-to-end scenarios

## 1. Database Testing

### Step 1: Run Migration

```bash
cd backend
sqlx migrate run
```

**Expected Output:**
```
Applied 20260325180000/migrate add reserve health tracking (XXXms)
```

### Step 2: Verify Schema

```bash
psql -d your_database -f tests/reserve_health_manual_tests.sql
```

Or connect to your database and run:

```sql
-- Check if new columns exist
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'pools'
AND column_name IN ('bad_debt_reserve', 'retained_yield', 'coverage_ratio', 
                     'reserve_health_status', 'last_health_check_at');
```

**Expected Output:**
```
      column_name       |          data_type          | is_nullable 
------------------------+-----------------------------+-------------
 bad_debt_reserve       | numeric                     | NO
 retained_yield         | numeric                     | NO
 coverage_ratio         | numeric                     | YES
 reserve_health_status  | character varying           | YES
 last_health_check_at   | timestamp with time zone    | YES
```

### Step 3: Insert Test Data

```sql
INSERT INTO pools (asset_code, total_liquidity, utilized_liquidity, bad_debt_reserve, retained_yield)
VALUES 
    ('USDC', 1000000.0, 150000.0, 20000.0, 5000.0),
    ('XLM', 500000.0, 50000.0, 3000.0, 1000.0)
ON CONFLICT (asset_code) DO UPDATE SET
    total_liquidity = EXCLUDED.total_liquidity,
    utilized_liquidity = EXCLUDED.utilized_liquidity,
    bad_debt_reserve = EXCLUDED.bad_debt_reserve,
    retained_yield = EXCLUDED.retained_yield;
```

### Step 4: Verify Calculations

```sql
SELECT 
    asset_code,
    total_liquidity,
    utilized_liquidity,
    bad_debt_reserve,
    ROUND((utilized_liquidity / NULLIF(total_liquidity, 0)) * 100, 2) as utilization_pct,
    ROUND((bad_debt_reserve / NULLIF(utilized_liquidity, 0)) * 100, 2) as coverage_pct,
    ROUND((bad_debt_reserve / NULLIF(total_liquidity, 0)) * 100, 2) as reserve_adequacy_pct
FROM pools
ORDER BY asset_code;
```

**Expected Output for USDC:**
- Utilization: 15%
- Coverage: 13.33%
- Reserve Adequacy: 2%

## 2. API Testing

### Step 1: Start Backend Service

```bash
cd backend
cargo run --release
```

Or if using a different method:
```bash
./start_backend.sh
```

### Step 2: Get Admin Token

Login as admin to get authentication token:

```bash
curl -X POST http://localhost:8080/admin/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "your_password"
  }'
```

Save the token from the response.

### Step 3: Run API Tests

Make the test script executable:
```bash
chmod +x backend/tests/reserve_health_api_tests.sh
```

Run the tests:
```bash
export ADMIN_TOKEN="your_token_here"
export BASE_URL="http://localhost:8080"
./backend/tests/reserve_health_api_tests.sh
```

**Expected Output:**
```
==========================================
Reserve Health API Testing
==========================================
Base URL: http://localhost:8080

Test 1: Health Check
✓ PASS: Server is healthy

Test 2: GET /api/admin/reserve-health
✓ PASS: Get all reserve health
...

==========================================
Test Summary
==========================================
Tests Passed: 10
Tests Failed: 0
Total Tests: 10

All tests passed!
```

### Step 4: Manual API Tests

#### Test 1: Get All Reserve Health
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:8080/api/admin/reserve-health | jq
```

**Expected Response:**
```json
{
  "status": "success",
  "data": [
    {
      "asset_code": "USDC",
      "coverage_ratio": 0.1333,
      "utilization_rate": 15.0,
      "reserve_adequacy": 2.0,
      "health_status": "healthy",
      "bad_debt_reserve": 20000.0,
      "total_liquidity": 1000000.0,
      "utilized_liquidity": 150000.0,
      "available_liquidity": 850000.0
    }
  ]
}
```

#### Test 2: Get Specific Asset
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:8080/api/admin/reserve-health/USDC | jq
```

#### Test 3: Sync Reserves
```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:8080/api/admin/reserve-health/sync | jq
```

**Expected Response:**
```json
{
  "status": "success",
  "message": "Reserve health synced successfully",
  "data": [...]
}
```

#### Test 4: Analytics Endpoint
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:8080/api/admin/analytics/reserve-health | jq
```

## 3. Integration Testing

### Scenario 1: Normal Operation

**Setup:**
```sql
UPDATE pools SET 
    total_liquidity = 1000000.0,
    utilized_liquidity = 500000.0,
    bad_debt_reserve = 100000.0
WHERE asset_code = 'USDC';
```

**Test:**
1. Wait 5 minutes for background task to run
2. Check database:
```sql
SELECT asset_code, coverage_ratio, reserve_health_status, last_health_check_at
FROM pools WHERE asset_code = 'USDC';
```

**Expected:**
- coverage_ratio: ~0.20 (20%)
- reserve_health_status: 'healthy'
- last_health_check_at: Recent timestamp

### Scenario 2: Warning Status

**Setup:**
```sql
UPDATE pools SET 
    total_liquidity = 1000000.0,
    utilized_liquidity = 800000.0,
    bad_debt_reserve = 80000.0
WHERE asset_code = 'USDC';
```

**Test:**
1. Trigger manual sync:
```bash
curl -X POST -H "Authorization: Bearer TOKEN" \
  http://localhost:8080/api/admin/reserve-health/sync
```

2. Check notifications:
```sql
SELECT * FROM notifications 
WHERE notification_type = 'reserve_health_alert'
ORDER BY created_at DESC LIMIT 5;
```

**Expected:**
- Status changes to 'warning'
- Notification sent to admins
- Audit log entry created

### Scenario 3: Critical Status

**Setup:**
```sql
UPDATE pools SET 
    total_liquidity = 1000000.0,
    utilized_liquidity = 900000.0,
    bad_debt_reserve = 30000.0
WHERE asset_code = 'USDC';
```

**Test:**
1. Trigger sync
2. Check logs for critical warnings
3. Verify notifications sent

**Expected:**
- Status: 'critical'
- Multiple notifications sent
- Log entry: "CRITICAL: Pool USDC coverage ratio..."

### Scenario 4: High Utilization

**Setup:**
```sql
UPDATE pools SET 
    total_liquidity = 1000000.0,
    utilized_liquidity = 950000.0,
    bad_debt_reserve = 150000.0
WHERE asset_code = 'USDC';
```

**Expected:**
- Status: 'high_utilization'
- Coverage ratio: 15.79% (healthy)
- Utilization: 95% (high)

## 4. Performance Testing

### Test Background Task

**Monitor logs:**
```bash
tail -f logs/app.log | grep "Reserve Health"
```

**Expected:**
- Check runs every 5 minutes
- Completes in < 1 second
- No errors

### Test API Response Time

```bash
time curl -H "Authorization: Bearer TOKEN" \
  http://localhost:8080/api/admin/reserve-health
```

**Expected:**
- Response time < 100ms
- No timeouts

### Test Database Performance

```sql
EXPLAIN ANALYZE
SELECT id, asset_code, total_liquidity, utilized_liquidity, 
       bad_debt_reserve, retained_yield, coverage_ratio, 
       reserve_health_status, last_health_check_at
FROM pools;
```

**Expected:**
- Uses indexes
- Execution time < 10ms

## 5. Error Testing

### Test 1: Invalid Asset
```bash
curl -H "Authorization: Bearer TOKEN" \
  http://localhost:8080/api/admin/reserve-health/INVALID
```

**Expected:**
- Status: 404 or error
- Message: "Pool not found"

### Test 2: Unauthorized Access
```bash
curl http://localhost:8080/api/admin/reserve-health
```

**Expected:**
- Status: 401
- Message: "Unauthorized"

### Test 3: Database Connection Loss

**Simulate:**
1. Stop database
2. Observe logs

**Expected:**
- Error logged: "DB error loading pools"
- Service continues running
- Retries on next interval

## 6. Verification Checklist

- [ ] Migration applied successfully
- [ ] New columns exist in pools table
- [ ] Indexes created
- [ ] Test data inserted
- [ ] Backend service starts without errors
- [ ] Background task is running
- [ ] API endpoints return 200 OK
- [ ] Metrics calculated correctly
- [ ] Health status determined correctly
- [ ] Notifications sent on status change
- [ ] Audit logs created
- [ ] Performance is acceptable
- [ ] Error handling works

## 7. Troubleshooting

### Issue: Engine not running

**Check:**
```bash
# Look for startup message in logs
grep "Reserve Health Engine" logs/app.log
```

**Fix:**
Verify engine is started in `app.rs`:
```rust
reserve_health_engine.clone().start();
```

### Issue: Metrics not updating

**Check:**
```sql
SELECT asset_code, last_health_check_at 
FROM pools;
```

**Fix:**
- Manually trigger sync via API
- Check logs for errors
- Verify database connection

### Issue: Incorrect calculations

**Debug:**
```sql
-- Compare manual calculation with stored value
SELECT 
    asset_code,
    bad_debt_reserve / NULLIF(utilized_liquidity, 0) as manual_coverage,
    coverage_ratio as stored_coverage
FROM pools;
```

## 8. Continuous Monitoring

### Setup Monitoring Query

Run this periodically:
```sql
SELECT 
    asset_code,
    reserve_health_status,
    ROUND(coverage_ratio * 100, 2) as coverage_pct,
    last_health_check_at,
    EXTRACT(EPOCH FROM (NOW() - last_health_check_at)) / 60 as minutes_since_check
FROM pools
ORDER BY 
    CASE reserve_health_status
        WHEN 'critical' THEN 1
        WHEN 'warning' THEN 2
        ELSE 3
    END;
```

### Setup Alerts

Configure your monitoring system to alert on:
- Status = 'critical'
- Coverage ratio < 0.05
- Last check > 10 minutes ago
- Utilization > 90%

## Success Criteria

All tests should pass with:
- ✅ All API endpoints return expected responses
- ✅ Metrics calculated accurately
- ✅ Health status determined correctly
- ✅ Notifications sent appropriately
- ✅ Background task runs reliably
- ✅ Performance meets requirements
- ✅ Error handling works properly

## Next Steps

After successful testing:
1. Deploy to staging environment
2. Run tests again in staging
3. Monitor for 24 hours
4. Deploy to production
5. Set up production monitoring
6. Document any issues found

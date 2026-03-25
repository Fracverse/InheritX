# Reserve Health Testing Checklist

## Quick Testing Guide

Use this checklist to verify the Reserve Health implementation is working correctly.

## ✅ Pre-Testing Setup

- [ ] PostgreSQL database is running
- [ ] Database connection string is configured
- [ ] Backend dependencies are installed
- [ ] You have admin credentials

## ✅ Database Tests

### 1. Run Migration
```bash
cd backend
sqlx migrate run
```
- [ ] Migration completes without errors
- [ ] No rollback occurs

### 2. Verify Schema
```sql
\d pools
```
- [ ] `bad_debt_reserve` column exists (numeric)
- [ ] `retained_yield` column exists (numeric)
- [ ] `coverage_ratio` column exists (numeric)
- [ ] `reserve_health_status` column exists (varchar)
- [ ] `last_health_check_at` column exists (timestamp)

### 3. Check Indexes
```sql
\di pools*
```
- [ ] `idx_pools_health_status` exists
- [ ] `idx_pools_coverage_ratio` exists

### 4. Insert Test Data
```sql
INSERT INTO pools (asset_code, total_liquidity, utilized_liquidity, bad_debt_reserve)
VALUES ('TEST', 1000000, 150000, 20000);
```
- [ ] Insert succeeds
- [ ] Data visible in table

### 5. Manual Calculation Test
```sql
SELECT 
    asset_code,
    ROUND((bad_debt_reserve / utilized_liquidity) * 100, 2) as coverage_pct
FROM pools WHERE asset_code = 'TEST';
```
- [ ] Returns ~13.33% for TEST pool
- [ ] Calculation is correct

## ✅ Code Review Tests

### 1. Check Module Integration
```bash
grep -r "reserve_health" backend/src/lib.rs
```
- [ ] Module is declared
- [ ] Types are exported

### 2. Check App Integration
```bash
grep -r "ReserveHealthEngine" backend/src/app.rs
```
- [ ] Engine is imported
- [ ] Engine is initialized
- [ ] Engine is started
- [ ] Routes are registered

### 3. Check Compilation (if Rust is installed)
```bash
cd backend
cargo check
```
- [ ] No compilation errors
- [ ] No warnings (or acceptable warnings only)

## ✅ API Tests (Manual)

### 1. Start Backend
```bash
cd backend
cargo run
# OR your preferred method
```
- [ ] Server starts without errors
- [ ] Logs show "Reserve Health Engine" initialization
- [ ] No panic or crash

### 2. Health Check
```bash
curl http://localhost:8080/health
```
- [ ] Returns `{"status":"ok"}`
- [ ] Response time < 100ms

### 3. Get Admin Token
```bash
curl -X POST http://localhost:8080/admin/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your_password"}'
```
- [ ] Returns token
- [ ] Token is valid

### 4. Test Reserve Health Endpoint
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:8080/api/admin/reserve-health
```
- [ ] Returns 200 OK
- [ ] Response has `"status":"success"`
- [ ] Response has `data` array
- [ ] Each item has required fields:
  - [ ] `asset_code`
  - [ ] `coverage_ratio`
  - [ ] `utilization_rate`
  - [ ] `health_status`
  - [ ] `total_liquidity`
  - [ ] `utilized_liquidity`
  - [ ] `bad_debt_reserve`

### 5. Test Specific Asset Endpoint
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:8080/api/admin/reserve-health/USDC
```
- [ ] Returns 200 OK
- [ ] Returns single pool data
- [ ] Metrics are calculated correctly

### 6. Test Sync Endpoint
```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:8080/api/admin/reserve-health/sync
```
- [ ] Returns 200 OK
- [ ] Message indicates success
- [ ] Data is updated

### 7. Test Analytics Endpoint
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:8080/api/admin/analytics/reserve-health
```
- [ ] Returns 200 OK
- [ ] Same format as main endpoint
- [ ] Data is consistent

### 8. Test Error Cases
```bash
# Invalid asset
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:8080/api/admin/reserve-health/INVALID

# No auth
curl http://localhost:8080/api/admin/reserve-health
```
- [ ] Invalid asset returns 404 or error
- [ ] No auth returns 401 Unauthorized

## ✅ Automated Tests (if available)

### 1. Run Test Script
```bash
chmod +x backend/tests/reserve_health_api_tests.sh
export ADMIN_TOKEN="your_token"
./backend/tests/reserve_health_api_tests.sh
```
- [ ] All tests pass
- [ ] No failures
- [ ] Output shows green checkmarks

### 2. Run SQL Tests
```bash
psql -d your_database -f backend/tests/reserve_health_manual_tests.sql
```
- [ ] All queries execute successfully
- [ ] Results match expectations

## ✅ Integration Tests

### 1. Background Task Test
Wait 5-10 minutes and check:
```sql
SELECT asset_code, last_health_check_at 
FROM pools;
```
- [ ] `last_health_check_at` is updated
- [ ] Timestamp is recent (within last 5 minutes)
- [ ] Updates continue every 5 minutes

### 2. Status Change Test
```sql
-- Force critical status
UPDATE pools SET 
    utilized_liquidity = 900000,
    bad_debt_reserve = 30000
WHERE asset_code = 'USDC';
```
Wait for next check cycle, then:
```sql
SELECT asset_code, reserve_health_status, coverage_ratio
FROM pools WHERE asset_code = 'USDC';
```
- [ ] Status changes to 'critical'
- [ ] Coverage ratio is updated
- [ ] Notification is sent (check notifications table)

### 3. Alert Test
```sql
SELECT * FROM notifications 
WHERE notification_type = 'reserve_health_alert'
ORDER BY created_at DESC LIMIT 5;
```
- [ ] Notification exists for status change
- [ ] Message is descriptive
- [ ] Sent to admin users

### 4. Audit Log Test
```sql
SELECT * FROM action_logs 
WHERE action = 'system_event'
ORDER BY created_at DESC LIMIT 5;
```
- [ ] Audit log entry exists
- [ ] Entity type is correct
- [ ] Timestamp is correct

## ✅ Performance Tests

### 1. API Response Time
```bash
time curl -H "Authorization: Bearer TOKEN" \
  http://localhost:8080/api/admin/reserve-health
```
- [ ] Response time < 100ms
- [ ] No timeouts

### 2. Database Query Performance
```sql
EXPLAIN ANALYZE SELECT * FROM pools;
```
- [ ] Uses indexes where appropriate
- [ ] Execution time < 10ms

### 3. Background Task Performance
Check logs for:
```
Reserve Health Engine: Check completed in XXXms
```
- [ ] Completes in < 1 second
- [ ] No errors or warnings

## ✅ Monitoring Tests

### 1. Log Monitoring
```bash
tail -f logs/app.log | grep -i "reserve"
```
- [ ] Logs show regular health checks
- [ ] No error messages
- [ ] Info messages are clear

### 2. Metrics Accuracy
```sql
-- Compare manual vs stored calculations
SELECT 
    asset_code,
    bad_debt_reserve / NULLIF(utilized_liquidity, 0) as manual,
    coverage_ratio as stored,
    ABS((bad_debt_reserve / NULLIF(utilized_liquidity, 0)) - coverage_ratio) as diff
FROM pools;
```
- [ ] Difference is < 0.0001 (rounding acceptable)
- [ ] No NULL values where unexpected

## ✅ Documentation Tests

- [ ] README exists and is clear
- [ ] API documentation is accurate
- [ ] Code comments are helpful
- [ ] Examples work as shown

## 🎯 Final Verification

### Critical Checks
- [ ] No compilation errors
- [ ] No runtime errors
- [ ] All API endpoints work
- [ ] Background task runs continuously
- [ ] Metrics are accurate
- [ ] Alerts are sent correctly
- [ ] Performance is acceptable

### Optional Checks
- [ ] Unit tests pass (if written)
- [ ] Integration tests pass
- [ ] Load testing completed
- [ ] Security review done

## 📊 Test Results Summary

**Date:** _____________

**Tester:** _____________

**Environment:** _____________

**Results:**
- Total Tests: _____
- Passed: _____
- Failed: _____
- Skipped: _____

**Issues Found:**
1. _____________________________________________
2. _____________________________________________
3. _____________________________________________

**Notes:**
_________________________________________________
_________________________________________________
_________________________________________________

## ✅ Sign-off

- [ ] All critical tests passed
- [ ] All issues documented
- [ ] Ready for deployment

**Approved by:** _____________

**Date:** _____________

---

## Quick Command Reference

```bash
# Database
psql -d your_db -f backend/tests/reserve_health_manual_tests.sql

# API Tests
export ADMIN_TOKEN="your_token"
./backend/tests/reserve_health_api_tests.sh

# Manual API Test
curl -H "Authorization: Bearer TOKEN" \
  http://localhost:8080/api/admin/reserve-health | jq

# Check Logs
tail -f logs/app.log | grep -i reserve

# Monitor Database
watch -n 5 'psql -d your_db -c "SELECT asset_code, reserve_health_status, last_health_check_at FROM pools"'
```

## Need Help?

- Check `backend/tests/TESTING_GUIDE.md` for detailed instructions
- Review `backend/docs/RESERVE_HEALTH_TRACKING.md` for feature documentation
- Check logs for error messages
- Verify database connection and credentials

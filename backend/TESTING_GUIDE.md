# Payout Persistence Testing Guide

## Pre-Testing Setup

### 1. Environment Configuration

Ensure your `.env` file contains:
```bash
DATABASE_URL=postgres://user:password@localhost:5432/inheritx_db
ANCHOR_API_URL=https://anchor.example.com
RUST_LOG=info,inheritx_backend=debug
```

### 2. Database Migration

Run migrations to add the new payout fields:
```bash
# Navigate to backend directory
cd backend

# Run migrations
cargo run --bin migrate
# OR if using sqlx-cli:
sqlx migrate run
```

Verify migration success:
```sql
-- Connect to database
psql $DATABASE_URL

-- Check payouts table schema
\d+ payouts;

-- Expected output should include:
-- exchange_rate        | numeric(18,6)
-- anchor_fee_usd       | numeric(18,6)
-- external_transaction_id | text
-- updated_at          | timestamp with time zone

-- Check indexes
\di payouts_*;

-- Expected indexes:
-- payouts_status_idx
-- payouts_external_transaction_id_idx
-- payouts_plan_id_idx
-- payouts_beneficiary_address_idx
```

### 3. Compile the Project

```bash
# Clean build to ensure all changes compile
cargo clean
cargo build --release

# Expected: Successful compilation with no errors
```

---

## Unit Tests

### Run All Tests

```bash
# Run all tests with output
cargo test -- --nocapture

# Run only backend library tests
cargo test --lib

# Run specific test module
cargo test stellar_anchor
cargo test api
```

### Expected Test Results

All existing tests should pass with the updated `AnchorRegistry::new()` signature.

If any tests fail, check:
1. All `AnchorRegistry::new()` calls pass `pool.clone()` as second parameter
2. Test database is accessible and migrations are run

---

## Integration Testing

### Test 1: Database Connection and Schema

```bash
# Start PostgreSQL if not running
# Windows: Start from Services
# Linux/Mac: systemctl start postgresql

# Verify schema
psql $DATABASE_URL << EOF
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'payouts'
ORDER BY ordinal_position;
EOF
```

**Expected Output:**
```
column_name              | data_type              | is_nullable
-------------------------+-----------------------+-------------
id                       | uuid                   | NO
plan_id                  | uuid                   | NO
beneficiary_address      | text                   | NO
amount                   | numeric                | NO
payout_type              | USER-DEFINED           | NO
status                   | USER-DEFINED           | NO
exchange_rate            | numeric                | NO
anchor_fee_usd           | numeric                | NO
external_transaction_id  | text                   | YES
created_at               | timestamp with time zone| NO
updated_at               | timestamp with time zone| NO
```

### Test 2: Backend Server Startup

```bash
# Start the backend server
cargo run --release

# Expected logs:
# INFO Successfully connected to PostgreSQL database
# INFO Starting rebranded INHERITX backend skeleton on 0.0.0.0:8080
# INFO Starting payout status polling worker { interval_secs: 30 }
```

**Verification Checklist:**
- [ ] Database connection successful
- [ ] Migrations applied (or already up-to-date)
- [ ] Server listening on configured port
- [ ] Background worker started (check for "Starting payout status polling worker")
- [ ] No panic or error messages

### Test 3: Create a Test Plan

```bash
# Create a test plan with fiat beneficiary
curl -X POST http://localhost:8080/api/plans \
  -H "Content-Type: application/json" \
  -H "x-public-key: <test-public-key>" \
  -H "x-signature: <test-signature>" \
  -d '{
    "owner": "GABC...",
    "token": "USDC",
    "amount": 1000.0,
    "beneficiaries": [
      {
        "address": "GDEF...",
        "name": "Test Beneficiary",
        "allocation_bps": 10000,
        "fiat_anchor_info": "John Doe|USD|Bank of America|1234567890"
      }
    ],
    "last_ping": 1234567890,
    "grace_period": 86400,
    "earn_yield": false,
    "is_active": true
  }'
```

**Expected Response:** HTTP 201 Created with plan details

### Test 4: Trigger Payout (Claim Plan)

```bash
# Wait for grace period to expire, then claim
curl -X POST http://localhost:8080/api/plans/{plan_id}/claim \
  -H "Content-Type: application/json" \
  -H "x-public-key: <beneficiary-public-key>" \
  -H "x-signature: <beneficiary-signature>" \
  -d '{
    "beneficiary_email": "test@example.com",
    "two_fa_code": "123456"
  }'
```

**Expected Response:** HTTP 200 OK

**Verify in Database:**
```sql
-- Check payout was created
SELECT 
    id,
    plan_id,
    beneficiary_address,
    amount,
    payout_type,
    status,
    exchange_rate,
    anchor_fee_usd,
    external_transaction_id,
    created_at,
    updated_at
FROM payouts
ORDER BY created_at DESC
LIMIT 1;
```

**Expected Output:**
- `id`: A valid UUID
- `plan_id`: Matches the plan you claimed
- `beneficiary_address`: Matches beneficiary wallet
- `amount`: Correct payout amount
- `payout_type`: 'fiat'
- `status`: 'pending' or 'processing'
- `exchange_rate`: > 0 (e.g., 1.0 if 1:1)
- `anchor_fee_usd`: >= 0
- `external_transaction_id`: Non-null if anchor returned transaction ID
- `created_at`: Recent timestamp
- `updated_at`: Same as created_at initially

### Test 5: Background Worker Status Updates

```bash
# Wait 35 seconds for the background worker to poll
sleep 35

# Check for status updates
psql $DATABASE_URL << EOF
SELECT 
    id,
    status,
    external_transaction_id,
    created_at,
    updated_at,
    (updated_at > created_at) as was_updated
FROM payouts
WHERE status IN ('pending', 'processing', 'completed', 'failed')
ORDER BY created_at DESC
LIMIT 5;
EOF
```

**Expected Output:**
- If anchor API is reachable and transaction progressed:
  - `was_updated`: true (updated_at > created_at)
  - `status`: May have changed from 'pending' to 'processing' or 'completed'

**Check Server Logs:**
```bash
# Look for log entries like:
# INFO Polling status for active payouts { count: X }
# INFO Payout status updated { payout_id, old_status, new_status }
```

### Test 6: Query Payout Status Endpoint

```bash
# Query all payouts
curl http://localhost:8080/api/anchor/payout-status

# Query payouts for specific beneficiary
curl "http://localhost:8080/api/anchor/payout-status?beneficiary_address=GDEF..."

# Query with pagination
curl "http://localhost:8080/api/anchor/payout-status?page=1&page_size=10"
```

**Expected Response:**
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "plan_id": "650e8400-e29b-41d4-a716-446655440001",
      "beneficiary_address": "GDEF...",
      "amount": "1000",
      "payout_type": "fiat",
      "status": "processing",
      "exchange_rate": "1.0",
      "anchor_fee_usd": "2.5",
      "external_transaction_id": "anchor-tx-12345",
      "created_at": "2026-08-27T10:00:00Z",
      "updated_at": "2026-08-27T10:00:35Z"
    }
  ],
  "page": 1,
  "page_size": 20,
  "total": 1,
  "total_pages": 1
}
```

**Verification:**
- [ ] All new fields present in response
- [ ] Pagination works correctly
- [ ] Filtering by beneficiary_address works
- [ ] HTTP 200 status code

---

## Manual Testing Scenarios

### Scenario 1: Successful Payout Flow

**Steps:**
1. Create plan with fiat beneficiary ✓
2. Wait for grace period to expire
3. Claim plan ✓
4. Verify payout record created with status='pending' ✓
5. Wait 30+ seconds
6. Verify status transitions to 'processing' or 'completed' ✓
7. Check updated_at timestamp changed ✓

### Scenario 2: Failed Anchor API Call

**Setup:** Configure invalid ANCHOR_API_URL or stop mock anchor server

**Steps:**
1. Create and claim plan
2. Verify payout created with status='failed'
3. Check logs for error messages
4. Verify external_transaction_id is NULL
5. Background worker should skip this payout (no external_transaction_id)

**Expected Behavior:**
- Payout record still created
- Status set to 'failed'
- Error logged with context
- System continues operating

### Scenario 3: Multiple Beneficiaries

**Steps:**
1. Create plan with 3 beneficiaries (all fiat)
2. Claim plan
3. Verify 3 payout records created
4. Check total amounts sum to plan amount
5. Wait for background worker
6. Verify all 3 payouts update independently

### Scenario 4: Pagination and Filtering

**Steps:**
1. Create 25+ payout records
2. Query with default pagination: `/api/anchor/payout-status`
3. Verify returns first 20 results
4. Query page 2: `/api/anchor/payout-status?page=2`
5. Filter by address: `/api/anchor/payout-status?beneficiary_address=GDEF...`
6. Verify filtering works correctly

### Scenario 5: Graceful Shutdown

**Steps:**
1. Start server with active payouts
2. Send SIGTERM or press Ctrl+C
3. Check logs for:
   - "Payout status poller shutting down"
   - "Database connections closed"
4. Verify no panic or abrupt termination

---

## Database Verification Queries

### Check Active Payouts Being Polled

```sql
-- Payouts that the background worker will process
SELECT 
    id,
    beneficiary_address,
    status,
    external_transaction_id,
    created_at,
    updated_at,
    (NOW() - created_at) as age
FROM payouts
WHERE status IN ('pending', 'processing')
  AND external_transaction_id IS NOT NULL
ORDER BY created_at ASC;
```

### Check Payout Status Distribution

```sql
-- Count by status
SELECT 
    status,
    COUNT(*) as count,
    SUM(amount) as total_amount,
    AVG(anchor_fee_usd) as avg_fee
FROM payouts
GROUP BY status
ORDER BY count DESC;
```

### Check Recent Status Changes

```sql
-- Payouts updated in last 5 minutes
SELECT 
    id,
    beneficiary_address,
    status,
    external_transaction_id,
    updated_at - created_at as time_to_update
FROM payouts
WHERE updated_at > NOW() - INTERVAL '5 minutes'
  AND updated_at > created_at
ORDER BY updated_at DESC;
```

### Check Payouts Stuck in Pending

```sql
-- Payouts pending for more than 1 hour
SELECT 
    id,
    beneficiary_address,
    status,
    external_transaction_id,
    NOW() - created_at as stuck_duration
FROM payouts
WHERE status = 'pending'
  AND created_at < NOW() - INTERVAL '1 hour'
ORDER BY created_at ASC;
```

---

## Performance Testing

### Test 1: Database Query Performance

```sql
-- Enable timing
\timing on

-- Test indexed query (should be fast)
EXPLAIN ANALYZE
SELECT * FROM payouts
WHERE status IN ('pending', 'processing')
  AND external_transaction_id IS NOT NULL;

-- Expected: Index Scan using payouts_status_idx
-- Execution time: < 10ms for thousands of records

-- Test pagination query
EXPLAIN ANALYZE
SELECT * FROM payouts
WHERE beneficiary_address = 'GDEF...'
ORDER BY created_at DESC
LIMIT 20 OFFSET 0;

-- Expected: Index Scan using payouts_beneficiary_address_idx
-- Execution time: < 5ms
```

### Test 2: Background Worker Load

**Setup:** Create 100+ active payouts

```bash
# Check server resource usage
top -p $(pgrep -f inheritx-backend)

# Monitor background worker activity
tail -f /var/log/inheritx/backend.log | grep "Polling status"

# Expected:
# - CPU usage: < 5% during polling
# - Memory: Stable, no growth over time
# - Polling completes in < 5 seconds for 100 payouts
```

### Test 3: Concurrent API Requests

```bash
# Use Apache Bench or similar tool
ab -n 1000 -c 10 http://localhost:8080/api/anchor/payout-status

# Expected:
# - No failures
# - Average response time < 100ms
# - No database connection errors
```

---

## Error Scenarios Testing

### Test 1: Database Connection Lost

**Steps:**
1. Start server
2. Stop PostgreSQL
3. Try to create payout
4. Check error handling and logs
5. Restart PostgreSQL
6. Verify server recovers

**Expected:**
- HTTP 500 with appropriate error message
- Error logged with context
- No panic or crash
- System recovers when DB available

### Test 2: Anchor API Timeout

**Setup:** Configure very slow mock anchor API

**Expected:**
- Request times out after 10 seconds
- Payout marked as 'failed'
- Error logged
- Background worker continues

### Test 3: Invalid Data

**Test Cases:**
```bash
# Missing plan_id
# Negative amounts
# Invalid UUID
# NULL required fields
```

**Expected:**
- Database constraints prevent invalid data
- Appropriate error messages returned
- Logs include validation failure context

---

## Security Testing

### Test 1: SQL Injection Prevention

```bash
# Try SQL injection in query parameters
curl "http://localhost:8080/api/anchor/payout-status?beneficiary_address='; DROP TABLE payouts; --"

# Expected: Query fails gracefully, no SQL executed
```

### Test 2: Authentication Bypass

```bash
# Try accessing endpoints without authentication
curl -X POST http://localhost:8080/api/plans/{id}/claim

# Expected: HTTP 401 Unauthorized
```

---

## Rollback Testing

### Test Migration Rollback

```bash
# Rollback the migration
sqlx migrate revert

# Verify columns removed
psql $DATABASE_URL -c "\d+ payouts;"

# Expected: exchange_rate, anchor_fee_usd, external_transaction_id, updated_at columns removed

# Re-apply migration
sqlx migrate run
```

---

## Monitoring & Alerting Setup

### Log Monitoring

**Key log patterns to monitor:**

```bash
# Success indicators
grep "Payout persisted to database" logs/backend.log
grep "Payout status updated" logs/backend.log

# Error indicators (should trigger alerts)
grep "ERROR.*Failed to persist payout" logs/backend.log
grep "ERROR.*Failed to query anchor status" logs/backend.log
grep "ERROR.*Failed to update payout status" logs/backend.log
```

### Database Monitoring

```sql
-- Monitor payout table growth
SELECT 
    pg_size_pretty(pg_total_relation_size('payouts')) as total_size,
    COUNT(*) as total_rows
FROM payouts;

-- Monitor stuck payouts (alert if > threshold)
SELECT COUNT(*) as stuck_pending_count
FROM payouts
WHERE status = 'pending'
  AND created_at < NOW() - INTERVAL '24 hours';
```

---

## Acceptance Checklist

Before marking implementation as complete:

### Compilation & Build
- [ ] `cargo build --release` succeeds with no errors
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo fmt --check` confirms code is formatted
- [ ] All dependencies resolve correctly

### Database
- [ ] Migrations run successfully (`sqlx migrate run`)
- [ ] New columns exist in payouts table
- [ ] Indexes created correctly
- [ ] Rollback migration works (`sqlx migrate revert`)

### Functionality
- [ ] Payouts persist to database with all fields
- [ ] `get_payout()` returns correct data
- [ ] `list_payouts()` returns filtered results
- [ ] Background worker polls and updates statuses
- [ ] API endpoint returns new fields
- [ ] Pagination works correctly

### Testing
- [ ] All unit tests pass (`cargo test --lib`)
- [ ] All integration tests pass (`cargo test`)
- [ ] Manual test scenarios verified
- [ ] Error scenarios handled gracefully

### Observability
- [ ] Log entries include sufficient context
- [ ] Status transitions logged
- [ ] Errors logged with payout_id and details
- [ ] Background worker activity visible in logs

### Performance
- [ ] Database queries use indexes
- [ ] Background worker completes in reasonable time
- [ ] No memory leaks or resource exhaustion
- [ ] Concurrent requests handled efficiently

### Security
- [ ] No SQL injection vulnerabilities
- [ ] Authentication enforced on protected endpoints
- [ ] Sensitive data not logged
- [ ] Parameterized queries used throughout

### Documentation
- [ ] IMPLEMENTATION_SUMMARY.md complete
- [ ] PAYOUT_ARCHITECTURE.md accurate
- [ ] TESTING_GUIDE.md (this file) comprehensive
- [ ] Code comments added where needed

---

## Troubleshooting

### Issue: Background worker not polling

**Symptoms:** No "Polling status for active payouts" logs

**Checks:**
1. Is the server running? (`ps aux | grep inheritx`)
2. Are there active payouts? (`SELECT COUNT(*) FROM payouts WHERE status IN ('pending', 'processing')`)
3. Check logs for worker startup message
4. Verify no panic in worker task

**Solution:** Restart server, check logs for startup errors

### Issue: Payouts stuck in pending

**Symptoms:** Status never changes from 'pending'

**Checks:**
1. Is `external_transaction_id` set? (worker skips if NULL)
2. Is anchor API reachable?
3. Check logs for "Failed to query anchor status"
4. Verify anchor API URL correct

**Solution:** Check anchor API configuration, verify network connectivity

### Issue: Database constraint errors

**Symptoms:** INSERT fails with constraint violation

**Checks:**
1. Verify migration ran successfully
2. Check if columns have correct types
3. Verify enum types exist (`SELECT * FROM pg_type WHERE typname LIKE '%payout%'`)

**Solution:** Re-run migrations, verify database schema

### Issue: Tests failing after update

**Symptoms:** `cargo test` fails

**Common Causes:**
1. `AnchorRegistry::new()` not updated with `pool` parameter
2. Test database not accessible
3. Migrations not run on test database

**Solution:**
1. Update all test `AnchorRegistry::new()` calls to pass `pool.clone()`
2. Verify test database connection
3. Run migrations on test database

---

## Next Steps After Testing

Once all tests pass:

1. **Code Review:** Have team review implementation
2. **Staging Deployment:** Deploy to staging environment
3. **Load Testing:** Run extended load tests
4. **Monitoring Setup:** Configure alerts for errors
5. **Documentation:** Update API documentation
6. **Production Deployment:** Deploy to production with rollback plan

---

**Testing Guide Version:** 1.0  
**Last Updated:** August 27, 2026  
**Status:** Ready for QA

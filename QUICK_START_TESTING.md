# Reserve Health - Quick Start Testing

## 🚀 5-Minute Test

The fastest way to verify the implementation works:

### Step 1: Database (1 minute)
```bash
cd backend
sqlx migrate run
```
✅ Should complete without errors

### Step 2: Check Schema (1 minute)
```bash
psql -d your_database -c "\d pools" | grep -E "bad_debt|coverage|health"
```
✅ Should show 5 new columns

### Step 3: Start Backend (1 minute)
```bash
cargo run
# OR your preferred method
```
✅ Should start without errors

### Step 4: Test API (2 minutes)
```bash
# Get admin token first
TOKEN=$(curl -s -X POST http://localhost:8080/admin/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your_password"}' | jq -r '.token')

# Test reserve health endpoint
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/admin/reserve-health | jq
```
✅ Should return JSON with pool metrics

---

## 🧪 10-Minute Test

More thorough verification:

### 1. Run SQL Tests (3 minutes)
```bash
psql -d your_database -f backend/tests/reserve_health_manual_tests.sql
```

### 2. Run API Tests (5 minutes)
```bash
export ADMIN_TOKEN="your_token_here"
chmod +x backend/tests/reserve_health_api_tests.sh
./backend/tests/reserve_health_api_tests.sh
```

### 3. Verify Background Task (2 minutes)
```bash
# Wait 5 minutes, then check:
psql -d your_database -c "SELECT asset_code, last_health_check_at FROM pools"
```
✅ Timestamps should be recent

---

## 📋 What to Check

### ✅ Success Indicators
- Migration completes ✓
- Backend starts without errors ✓
- API returns 200 OK ✓
- Metrics are calculated ✓
- Background task runs ✓

### ❌ Failure Indicators
- Migration fails
- Compilation errors
- API returns 500 or 401
- Metrics are null or incorrect
- Background task doesn't run

---

## 🔧 Troubleshooting

### Problem: Migration fails
```bash
# Check if already applied
psql -d your_db -c "SELECT version FROM _sqlx_migrations"
```

### Problem: API returns 401
```bash
# Get a fresh token
curl -X POST http://localhost:8080/admin/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your_password"}'
```

### Problem: No data returned
```bash
# Insert test data
psql -d your_db -c "INSERT INTO pools (asset_code, total_liquidity, utilized_liquidity, bad_debt_reserve) VALUES ('TEST', 1000000, 150000, 20000)"
```

---

## 📊 Expected Results

### API Response
```json
{
  "status": "success",
  "data": [
    {
      "asset_code": "USDC",
      "coverage_ratio": 0.1333,
      "utilization_rate": 15.0,
      "health_status": "healthy",
      "total_liquidity": 1000000.0,
      "utilized_liquidity": 150000.0,
      "bad_debt_reserve": 20000.0,
      "available_liquidity": 850000.0
    }
  ]
}
```

### Database Query
```sql
SELECT asset_code, reserve_health_status, coverage_ratio 
FROM pools;
```
```
 asset_code | reserve_health_status | coverage_ratio 
------------+-----------------------+----------------
 USDC       | healthy               |         0.1333
 XLM        | healthy               |         0.0600
```

---

## ✅ Minimum Viable Test

If you're short on time, just verify these 3 things:

1. **Migration works:**
   ```bash
   sqlx migrate run
   ```

2. **API responds:**
   ```bash
   curl -H "Authorization: Bearer TOKEN" \
     http://localhost:8080/api/admin/reserve-health
   ```

3. **Data looks correct:**
   ```bash
   psql -d your_db -c "SELECT * FROM pools LIMIT 1"
   ```

If all 3 work, you're good to go! ✅

---

## 📚 More Detailed Testing

For comprehensive testing, see:
- `TESTING_SUMMARY.md` - Overview and quick start
- `TESTING_CHECKLIST.md` - Complete checklist
- `backend/tests/TESTING_GUIDE.md` - Detailed guide

---

## 🎯 Next Steps

After successful testing:
1. ✅ Mark tests as passed
2. 📝 Document any issues
3. 🚀 Deploy to staging
4. 👀 Monitor for 24 hours
5. 🎉 Deploy to production

---

## 💡 Pro Tips

- Use `jq` to format JSON responses
- Keep logs open while testing: `tail -f logs/app.log`
- Test in a separate database first
- Save your admin token in an environment variable
- Run tests in order (database → API → integration)

---

## 🆘 Need Help?

1. Check logs: `tail -f logs/app.log | grep -i reserve`
2. Verify database: `psql -d your_db -c "\d pools"`
3. Test connection: `curl http://localhost:8080/health`
4. Review docs: `backend/docs/RESERVE_HEALTH_TRACKING.md`

---

**Ready to test?** Start with the 5-minute test above! 🚀

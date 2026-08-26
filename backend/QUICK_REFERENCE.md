# Payout Persistence - Quick Reference Card

## 🚀 Quick Start

```bash
# 1. Run migrations
sqlx migrate run

# 2. Build and run
cargo build --release
cargo run --release

# 3. Check background worker started
# Look for: "Starting payout status polling worker"
```

---

## 📋 Key Changes Summary

### Database Schema
```sql
-- New columns in payouts table:
exchange_rate            NUMERIC(18, 6)
anchor_fee_usd           NUMERIC(18, 6)
external_transaction_id  TEXT
updated_at              TIMESTAMPTZ
```

### Code Changes
```rust
// AnchorRegistry now requires PgPool
AnchorRegistry::new(api_url: String, pool: PgPool)

// AnchorPayoutRequest includes plan_id
pub struct AnchorPayoutRequest {
    pub plan_id: Option<Uuid>,  // NEW
    // ... other fields
}

// AnchorPayout uses proper types
pub struct AnchorPayout {
    pub id: Uuid,  // Changed from String
    pub external_transaction_id: Option<String>,  // NEW
    pub created_at: DateTime<Utc>,  // Changed from String
    pub updated_at: DateTime<Utc>,  // NEW
    // ... other fields
}
```

---

## 🔍 Common Operations

### Create Payout (API Layer)
```rust
let req = crate::stellar_anchor::AnchorPayoutRequest {
    plan_id: Some(plan.id),  // Link to plan
    beneficiary_address: beneficiary.wallet_address.clone(),
    beneficiary_name: "John Doe".to_string(),
    token: "USDC".to_string(),
    token_amount: 1000.0,
    fiat_currency: "USD".to_string(),
    bank_name: "Bank of America".to_string(),
    account_number: "1234567890".to_string(),
};

let payout = state.anchor.create_payout(req).await;
// Automatically persists to database
```

### Query Payouts
```rust
// Get single payout
let payout = anchor.get_payout(&payout_id).await;

// List payouts for beneficiary
let payouts = anchor.list_payouts(Some("GDEF...")).await;

// List all payouts
let payouts = anchor.list_payouts(None).await;
```

### Check Payout Status (SQL)
```sql
SELECT id, status, updated_at 
FROM payouts 
WHERE beneficiary_address = 'GDEF...'
ORDER BY created_at DESC;
```

---

## 🔧 Configuration

### Required Environment Variables
```bash
DATABASE_URL=postgres://user:pass@localhost:5432/inheritx
ANCHOR_API_URL=https://anchor.example.com
```

### Background Worker Settings
```rust
// In main.rs - polling interval in seconds
spawn_payout_status_poller(anchor_registry_clone, 30)
```

---

## 📊 Status Flow

```
pending → processing → completed
                    → failed
```

**Anchor API Status Mapping:**
- `"completed"` → `completed`
- `"pending"`, `"processing"`, `"pending_user"`, `"pending_external"` → `processing`
- `"failed"`, `"error"`, `"refunded"` → `failed`

---

## 🐛 Debugging

### Check Background Worker Running
```bash
# Look for this in logs
tail -f logs/backend.log | grep "Polling status"
```

### Verify Payouts Created
```sql
SELECT * FROM payouts ORDER BY created_at DESC LIMIT 5;
```

### Check Active Payouts Being Polled
```sql
SELECT id, status, external_transaction_id, updated_at
FROM payouts
WHERE status IN ('pending', 'processing')
  AND external_transaction_id IS NOT NULL;
```

### Common Log Messages
```
✅ "Payout persisted to database" - Payout created successfully
✅ "Payout status updated" - Background worker updated status
⚠️  "Payout has no external_transaction_id" - Worker skipped payout
❌ "Failed to persist payout" - Database error
❌ "Failed to query anchor status" - Anchor API error
```

---

## 🧪 Quick Test

```bash
# 1. Check schema
psql $DATABASE_URL -c "\d+ payouts"

# 2. Create test payout (via API)
curl -X POST http://localhost:8080/api/plans/{id}/claim \
  -H "Content-Type: application/json" \
  -d '{"beneficiary_email":"test@test.com","two_fa_code":"123456"}'

# 3. Verify in DB
psql $DATABASE_URL -c "SELECT * FROM payouts ORDER BY created_at DESC LIMIT 1;"

# 4. Wait for worker poll
sleep 35

# 5. Check if status updated
psql $DATABASE_URL -c "SELECT id, status, updated_at FROM payouts WHERE updated_at > created_at;"
```

---

## 📡 API Endpoints

### Get Payout Status
```bash
# All payouts (paginated)
GET /api/anchor/payout-status

# Filter by beneficiary
GET /api/anchor/payout-status?beneficiary_address=GDEF...

# Pagination
GET /api/anchor/payout-status?page=2&page_size=10
```

**Response:**
```json
{
  "data": [
    {
      "id": "uuid",
      "plan_id": "uuid",
      "beneficiary_address": "GDEF...",
      "amount": "1000",
      "payout_type": "fiat",
      "status": "processing",
      "exchange_rate": "1.0",
      "anchor_fee_usd": "2.5",
      "external_transaction_id": "anchor-tx-123",
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

---

## 🔐 Security Notes

- ✅ All database queries use parameterized bindings (SQL injection safe)
- ✅ Authentication required for plan claims
- ✅ Background worker respects graceful shutdown
- ✅ No sensitive data logged

---

## ⚡ Performance Tips

### Database Indexes
```sql
-- These indexes optimize queries:
CREATE INDEX payouts_status_idx ON payouts(status) 
    WHERE status IN ('pending', 'processing');
    
CREATE INDEX payouts_beneficiary_address_idx 
    ON payouts(beneficiary_address);
```

### Worker Optimization
- Polls only active payouts (`pending`, `processing`)
- Skips payouts without `external_transaction_id`
- Updates only when status changes
- Configurable polling interval

---

## 📝 Code Snippets

### Initialize AnchorRegistry
```rust
let anchor = Arc::new(AnchorRegistry::new(
    config.anchor_api_url.clone(),
    db_pool.clone(),
));
```

### Start Background Worker
```rust
let anchor_clone = anchor.clone();
let mut shutdown_rx = shutdown_rx.clone();
tokio::spawn(async move {
    tokio::select! {
        _ = spawn_payout_status_poller(anchor_clone, 30) => {},
        _ = shutdown_rx.changed() => {
            info!("Payout poller shutting down");
        }
    }
});
```

### Query Payouts in Handler
```rust
async fn get_payouts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnchorQuery>,
) -> impl IntoResponse {
    let payouts = state.anchor
        .list_payouts(query.beneficiary_address.as_deref())
        .await;
    
    Json(payouts).into_response()
}
```

---

## 🚨 Troubleshooting

| Issue | Solution |
|-------|----------|
| Worker not polling | Check logs for "Starting payout status polling worker" |
| Payouts stuck in pending | Verify `external_transaction_id` is set |
| Database error | Run migrations: `sqlx migrate run` |
| Compilation error | Update all `AnchorRegistry::new()` calls with `pool` |
| Tests failing | Update test setup to pass `pool.clone()` |

---

## 📚 Documentation Files

- **IMPLEMENTATION_SUMMARY.md** - Complete implementation details
- **PAYOUT_ARCHITECTURE.md** - Architecture diagrams and flow
- **TESTING_GUIDE.md** - Comprehensive testing instructions
- **QUICK_REFERENCE.md** - This file

---

## ✅ Migration Checklist

Before deploying to production:

- [ ] Migrations tested on staging
- [ ] All tests pass (`cargo test`)
- [ ] Background worker verified running
- [ ] API endpoints tested
- [ ] Monitoring/alerts configured
- [ ] Rollback plan prepared
- [ ] Database backup taken

---

**Quick Reference Version:** 1.0  
**Last Updated:** August 27, 2026  
**For:** InheritX Backend v0.1.0

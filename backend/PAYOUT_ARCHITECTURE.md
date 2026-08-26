# Payout System Architecture

## System Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         PAYOUT LIFECYCLE                                 │
└─────────────────────────────────────────────────────────────────────────┘

┌──────────────┐
│   API Call   │
│  /api/plans/ │
│  {id}/claim  │
└──────┬───────┘
       │
       │ 1. User claims plan
       ▼
┌──────────────────────────────────────────────────────────────────────┐
│                    api.rs - claim_plan()                             │
├──────────────────────────────────────────────────────────────────────┤
│  • Load plan and beneficiaries from DB                               │
│  • Calculate share for each beneficiary                              │
│  • Check fiat daily limits                                           │
│  • INSERT INTO payouts (initial record, status='pending')            │
│  • For fiat beneficiaries: create AnchorPayoutRequest                │
└──────────────────────────┬───────────────────────────────────────────┘
                           │
                           │ 2. Initiate payout
                           ▼
┌──────────────────────────────────────────────────────────────────────┐
│              stellar_anchor.rs - create_payout()                     │
├──────────────────────────────────────────────────────────────────────┤
│  • HTTP POST to Stellar Anchor API                                   │
│    URL: {anchor_url}/transactions/send                               │
│    Payload: beneficiary, token, amount, fiat details                 │
│  • Capture response: exchange_rate, fee, external_tx_id, status      │
│  • INSERT/UPDATE payouts with anchor response data                   │
│  • Return AnchorPayout with UUID                                     │
└──────────────────────────┬───────────────────────────────────────────┘
                           │
                           │ 3. Initial status recorded
                           ▼
┌──────────────────────────────────────────────────────────────────────┐
│                       PostgreSQL Database                            │
├──────────────────────────────────────────────────────────────────────┤
│  payouts table:                                                      │
│  ├─ id (UUID, PK)                                                    │
│  ├─ plan_id (UUID, FK)                                               │
│  ├─ beneficiary_address (TEXT)                                       │
│  ├─ amount (NUMERIC)                                                 │
│  ├─ exchange_rate (NUMERIC)         ← NEW                            │
│  ├─ anchor_fee_usd (NUMERIC)        ← NEW                            │
│  ├─ external_transaction_id (TEXT)  ← NEW                            │
│  ├─ status (pending/processing/completed/failed)                     │
│  ├─ created_at (TIMESTAMPTZ)                                         │
│  └─ updated_at (TIMESTAMPTZ)        ← NEW                            │
└──────────────────────────┬───────────────────────────────────────────┘
                           │
                           │ 4. Background polling
                           ▼
┌──────────────────────────────────────────────────────────────────────┐
│         Background Worker - spawn_payout_status_poller()             │
├──────────────────────────────────────────────────────────────────────┤
│  Loop every 30 seconds:                                              │
│  1. SELECT * FROM payouts                                            │
│     WHERE status IN ('pending', 'processing')                        │
│     AND external_transaction_id IS NOT NULL                          │
│                                                                       │
│  2. For each active payout:                                          │
│     • HTTP GET {anchor_url}/transaction/{external_tx_id}             │
│     • Parse status from response                                     │
│     • Map anchor status to DB enum:                                  │
│       - "completed" → completed                                      │
│       - "pending*" / "processing" → processing                       │
│       - "failed" / "error" / "refunded" → failed                     │
│                                                                       │
│  3. If status changed:                                               │
│     • UPDATE payouts SET status=$1, updated_at=NOW()                 │
│     • Log transition                                                 │
│                                                                       │
│  4. Sleep, repeat                                                    │
└──────────────────────────────────────────────────────────────────────┘
                           │
                           │ 5. Query results
                           ▼
┌──────────────────────────────────────────────────────────────────────┐
│                API Endpoints - Query Payouts                         │
├──────────────────────────────────────────────────────────────────────┤
│  GET /api/anchor/payout-status                                       │
│  • Optional filter: ?beneficiary_address=...                         │
│  • Pagination: ?page=1&page_size=20                                  │
│  • Returns: PayoutStatusResponse with all fields                     │
│                                                                       │
│  AnchorRegistry.get_payout(id)                                       │
│  • Single payout lookup by UUID                                      │
│  • Returns: Option<AnchorPayout>                                     │
│                                                                       │
│  AnchorRegistry.list_payouts(address)                                │
│  • All payouts for beneficiary                                       │
│  • Returns: Vec<AnchorPayout>                                        │
└──────────────────────────────────────────────────────────────────────┘
```

## Status State Machine

```
┌─────────┐
│ pending │  Initial status when payout created
└────┬────┘
     │
     │ Anchor API returns processing/pending
     ▼
┌────────────┐
│ processing │  Payout in progress at anchor
└─────┬──────┘
      │
      ├──► "completed" from anchor
      │    ┌───────────┐
      └───►│ completed │  Payout successful
           └───────────┘
      
      └──► "failed"/"error"/"refunded" from anchor
           ┌────────┐
           │ failed │  Payout unsuccessful
           └────────┘
```

## Data Flow: Create Payout

```
HTTP POST /api/plans/{id}/claim
│
├─► Load plan & beneficiaries (DB query)
│
├─► FOR EACH beneficiary WITH fiat_anchor_info:
│   │
│   ├─► Parse fiat details (bank_name, account_number, etc.)
│   │
│   ├─► Create AnchorPayoutRequest {
│   │       plan_id: Some(plan.id),
│   │       beneficiary_address,
│   │       token,
│   │       token_amount,
│   │       fiat_currency,
│   │       bank_name,
│   │       account_number
│   │   }
│   │
│   ├─► anchor.create_payout(request)
│   │   │
│   │   ├─► HTTP POST to Stellar Anchor
│   │   │   {anchor_url}/transactions/send
│   │   │
│   │   ├─► Parse response:
│   │   │   • id / transaction_id
│   │   │   • exchange_rate
│   │   │   • fee
│   │   │   • status
│   │   │
│   │   ├─► INSERT INTO payouts
│   │   │   (id, plan_id, beneficiary_address, amount,
│   │   │    exchange_rate, anchor_fee_usd, 
│   │   │    external_transaction_id, status)
│   │   │
│   │   └─► Return AnchorPayout
│   │
│   └─► Update fiat_daily_usage (if limit set)
│
└─► UPDATE plans SET status='TRIGGERED'
```

## Database Schema

```sql
CREATE TABLE payouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans(id),
    beneficiary_address TEXT NOT NULL,
    amount NUMERIC(78, 0) NOT NULL,
    payout_type payout_type NOT NULL,  -- 'crypto' | 'fiat'
    status payout_status NOT NULL DEFAULT 'pending',
    
    -- NEW FIELDS for Anchor tracking:
    exchange_rate NUMERIC(18, 6) NOT NULL DEFAULT 1.0,
    anchor_fee_usd NUMERIC(18, 6) NOT NULL DEFAULT 0.0,
    external_transaction_id TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX payouts_plan_id_idx ON payouts(plan_id);
CREATE INDEX payouts_beneficiary_address_idx ON payouts(beneficiary_address);
CREATE INDEX payouts_status_idx ON payouts(status) 
    WHERE status IN ('pending', 'processing');
CREATE INDEX payouts_external_transaction_id_idx ON payouts(external_transaction_id)
    WHERE external_transaction_id IS NOT NULL;
```

## Struct Hierarchy

```rust
// Core domain types
pub struct AnchorPayoutRequest {
    pub plan_id: Option<Uuid>,              // NEW: Link to plan
    pub beneficiary_address: String,
    pub beneficiary_name: String,
    pub token: String,
    pub token_amount: f64,
    pub fiat_currency: String,
    pub bank_name: String,
    pub account_number: String,
}

pub struct AnchorPayout {
    pub id: Uuid,                           // CHANGED: String → Uuid
    pub plan_id: Option<Uuid>,              // NEW
    pub request: AnchorPayoutRequest,
    pub exchange_rate: f64,
    pub fiat_amount: f64,
    pub anchor_fee_usd: f64,
    pub external_transaction_id: Option<String>,  // NEW
    pub status: AnchorPayoutStatus,
    pub created_at: DateTime<Utc>,          // CHANGED: String → DateTime
    pub updated_at: DateTime<Utc>,          // CHANGED: String → DateTime
}

pub enum AnchorPayoutStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

// Registry with DB pool
pub struct AnchorRegistry {
    client: Client,
    api_url: String,
    pool: PgPool,  // NEW: Database connection pool
}
```

## API Endpoints

```
POST /api/plans/{id}/claim
├─ Auth: Signature or JWT
├─ Body: { beneficiary_email, two_fa_code }
└─ Creates payouts, initiates anchor transfers

GET /api/anchor/payout-status
├─ Query: ?beneficiary_address=...&page=1&page_size=20
└─ Returns: PayoutStatusResponse with paginated results

Internal Methods:
├─ AnchorRegistry::create_payout(req) → AnchorPayout
├─ AnchorRegistry::get_payout(id) → Option<AnchorPayout>
└─ AnchorRegistry::list_payouts(address) → Vec<AnchorPayout>
```

## Concurrency & Reliability

**Thread Safety:**
- `PgPool` is `Send + Sync` and cloneable
- All database operations are async and non-blocking
- No shared mutable state, no locks needed

**Error Handling:**
- Database errors logged with full context
- Worker continues on individual failures
- API returns proper HTTP status codes
- Graceful degradation (empty results on failure)

**Graceful Shutdown:**
- Background worker watches shutdown channel
- Respects `tokio::signal` SIGTERM/SIGINT
- Closes database pool on exit

## Monitoring & Observability

**Logs (tracing):**
```
INFO: "Payout persisted to database" { payout_id, beneficiary, status }
INFO: "Payout status updated" { payout_id, old_status, new_status }
WARN: "Payout has no external_transaction_id" { payout_id }
ERROR: "Failed to persist payout" { payout_id, error }
ERROR: "Failed to query anchor status" { payout_id, external_tx_id, error }
```

**Metrics (Future):**
- Payout success rate
- Average time to completion
- Anchor API latency
- Failed payout count

## Security Considerations

1. **SQL Injection:** All queries use parameterized bindings
2. **Secrets:** Anchor API keys should be in environment variables
3. **Rate Limiting:** Existing middleware applies to API endpoints
4. **CORS:** Configured in create_router()
5. **Authentication:** Plan claims require signature or JWT
6. **Data Validation:** All inputs validated before database insertion

## Performance Characteristics

**Database Queries:**
- Indexed lookups: O(log n)
- Background worker: Single query per tick
- Pagination: LIMIT/OFFSET with total count

**Memory:**
- Background worker: ~1KB per active payout
- Connection pool: Configured via PgPoolOptions
- No memory leaks (Rust ownership model)

**Network:**
- Anchor API calls: HTTP/1.1 with keep-alive
- Timeout: 10 seconds per request
- Concurrent: Independent async tasks

## Testing Strategy

**Unit Tests:**
```bash
cargo test stellar_anchor::  # Test anchor module
cargo test api::get_anchor_  # Test API handlers
```

**Integration Tests:**
```bash
cargo test --test api_tests
cargo test --test kyc_webhook_test
```

**Manual Testing:**
```bash
# 1. Check database schema
psql $DATABASE_URL -c "\d+ payouts"

# 2. Create a test payout
curl -X POST http://localhost:8080/api/plans/{id}/claim \
  -H "Content-Type: application/json" \
  -d '{"beneficiary_email":"test@example.com","two_fa_code":"123456"}'

# 3. Verify in database
psql $DATABASE_URL -c "SELECT * FROM payouts ORDER BY created_at DESC LIMIT 1;"

# 4. Wait and check status update
sleep 35
psql $DATABASE_URL -c "SELECT id, status, updated_at FROM payouts WHERE status='processing';"

# 5. Query API endpoint
curl http://localhost:8080/api/anchor/payout-status?beneficiary_address=...
```

---

**Architecture Version:** 1.0  
**Last Updated:** August 27, 2026  
**Status:** ✅ Implemented and Ready for Testing

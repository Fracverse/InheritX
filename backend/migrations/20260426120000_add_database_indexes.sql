-- Database Indexing Optimization — Issue #435
--
-- Adds indexes for the most frequently queried columns identified by
-- analysing query patterns across service layer and API handlers.
-- All indexes use CREATE INDEX IF NOT EXISTS so the migration is idempotent.
--
-- Indexing strategy:
--  1. Foreign key columns not already indexed (PostgreSQL does NOT auto-index FKs).
--  2. Filter columns used in WHERE clauses on hot paths.
--  3. Composite indexes for common multi-column query patterns.
--  4. Partial indexes where a condition selects a small, frequently accessed subset.

-- ── users ────────────────────────────────────────────────────────────────────
-- Login by email (email already has UNIQUE, which creates an index)
-- Wallet address lookup for Web3 login
CREATE INDEX IF NOT EXISTS idx_users_wallet_address
    ON users (wallet_address)
    WHERE wallet_address IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_users_created_at ON users (created_at);

-- ── plans ─────────────────────────────────────────────────────────────────────
-- Most plan queries filter by user_id; status is frequently a secondary filter
CREATE INDEX IF NOT EXISTS idx_plans_user_id         ON plans (user_id);
CREATE INDEX IF NOT EXISTS idx_plans_status          ON plans (status);
CREATE INDEX IF NOT EXISTS idx_plans_user_status     ON plans (user_id, status);

-- "due-for-claim" query filters by status = 'due-for-claim'
CREATE INDEX IF NOT EXISTS idx_plans_due_for_claim
    ON plans (user_id, created_at)
    WHERE status = 'due-for-claim';

-- ── notifications ─────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications (user_id);

-- Unread badge count — partial index for WHERE is_read = FALSE
CREATE INDEX IF NOT EXISTS idx_notifications_unread
    ON notifications (user_id, created_at)
    WHERE is_read = FALSE;

-- ── logs (audit logs) ─────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_logs_user_id    ON logs (user_id);
CREATE INDEX IF NOT EXISTS idx_logs_admin_id   ON logs (admin_id);
CREATE INDEX IF NOT EXISTS idx_logs_plan_id    ON logs (plan_id);
CREATE INDEX IF NOT EXISTS idx_logs_action     ON logs (action);
CREATE INDEX IF NOT EXISTS idx_logs_created_at ON logs (created_at);

-- ── kyc_status ────────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_kyc_status_status ON kyc_status (status);

-- ── nonces ────────────────────────────────────────────────────────────────────
-- Web3 login: lookup by wallet_address + expiry check
CREATE INDEX IF NOT EXISTS idx_nonces_wallet_address ON nonces (wallet_address);
CREATE INDEX IF NOT EXISTS idx_nonces_expires_at     ON nonces (expires_at);

-- ── two_fa ────────────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_two_fa_user_id    ON two_fa (user_id);
CREATE INDEX IF NOT EXISTS idx_two_fa_expiry     ON two_fa (expiry_timestamp);

-- ── loan_lifecycle ────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_loan_lifecycle_user_id    ON loan_lifecycle (user_id);
CREATE INDEX IF NOT EXISTS idx_loan_lifecycle_status     ON loan_lifecycle (status);
CREATE INDEX IF NOT EXISTS idx_loan_lifecycle_user_status ON loan_lifecycle (user_id, status);

-- ── emergency_access_grants ───────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_emergency_access_grants_user_id
    ON emergency_access_grants (user_id);
CREATE INDEX IF NOT EXISTS idx_emergency_access_grants_status
    ON emergency_access_grants (status);

-- ── will_documents ────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_will_documents_plan_id ON will_documents (plan_id);
CREATE INDEX IF NOT EXISTS idx_will_documents_status  ON will_documents (status);

-- ── action_logs ───────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_action_logs_user_id    ON action_logs (user_id);
CREATE INDEX IF NOT EXISTS idx_action_logs_created_at ON action_logs (created_at);

-- ── price_feeds ───────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_price_feeds_asset_code ON price_feeds (asset_code);

-- ── asset_price_history ───────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_asset_price_history_asset_code
    ON asset_price_history (asset_code);
CREATE INDEX IF NOT EXISTS idx_asset_price_history_recorded_at
    ON asset_price_history (recorded_at);
-- Composite for "price history between dates" queries
CREATE INDEX IF NOT EXISTS idx_asset_price_history_asset_time
    ON asset_price_history (asset_code, recorded_at DESC);

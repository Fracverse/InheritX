-- Issue #1268: backing tables/columns for the admin dashboard and management
-- endpoints (metrics, audit log, KYC moderation, insurance funds and emergency
-- controls).

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT,
    admin_id TEXT,
    action TEXT NOT NULL,
    entity_id TEXT,
    entity_type TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS audit_logs_created_at_idx
    ON audit_logs (created_at DESC);

CREATE TABLE IF NOT EXISTS insurance_funds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    asset_code TEXT NOT NULL,
    balance NUMERIC(78, 0) NOT NULL DEFAULT 0 CHECK (balance >= 0),
    total_deposits NUMERIC(78, 0) NOT NULL DEFAULT 0 CHECK (total_deposits >= 0),
    total_withdrawals NUMERIC(78, 0) NOT NULL DEFAULT 0 CHECK (total_withdrawals >= 0),
    total_claims_paid NUMERIC(78, 0) NOT NULL DEFAULT 0 CHECK (total_claims_paid >= 0),
    reserve_ratio NUMERIC(10, 4) NOT NULL DEFAULT 0 CHECK (reserve_ratio >= 0),
    health_status TEXT NOT NULL DEFAULT 'healthy'
        CHECK (health_status IN ('healthy', 'warning', 'critical')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS insurance_funds_asset_code_idx
    ON insurance_funds (asset_code);

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS is_paused BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS paused_reason TEXT,
    ADD COLUMN IF NOT EXISTS paused_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS risk_override_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS risk_override_reason TEXT,
    ADD COLUMN IF NOT EXISTS risk_override_at TIMESTAMPTZ;

ALTER TABLE kyc_records
    ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'submitted',
    ADD COLUMN IF NOT EXISTS reviewed_by TEXT,
    ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS rejection_reason TEXT;

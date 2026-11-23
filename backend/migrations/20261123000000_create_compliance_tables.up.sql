CREATE TABLE IF NOT EXISTS compliance_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address TEXT NOT NULL,
    user_id UUID,
    transaction_hash TEXT,
    type TEXT NOT NULL CHECK (type IN ('velocity', 'volume', 'flagged_transaction')),
    amount NUMERIC(78, 4),
    asset_code TEXT,
    event_count BIGINT,
    threshold NUMERIC(78, 4),
    window_minutes BIGINT,
    severity TEXT NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    status TEXT NOT NULL CHECK (status IN ('open', 'investigating', 'resolved')),
    reason TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS compliance_alerts_address_created_idx
    ON compliance_alerts (address, created_at DESC);
CREATE INDEX IF NOT EXISTS compliance_alerts_status_type_idx
    ON compliance_alerts (status, type);
CREATE UNIQUE INDEX IF NOT EXISTS compliance_alerts_open_unique_idx
    ON compliance_alerts (address, type)
    WHERE status = 'open';

CREATE TABLE IF NOT EXISTS risk_score_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address TEXT NOT NULL,
    previous_score SMALLINT NOT NULL CHECK (previous_score BETWEEN 0 AND 100),
    new_score SMALLINT NOT NULL CHECK (new_score BETWEEN 0 AND 100),
    previous_level TEXT NOT NULL,
    new_level TEXT NOT NULL,
    justification TEXT NOT NULL,
    admin_id TEXT NOT NULL,
    original_factors JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS risk_score_overrides_address_created_idx
    ON risk_score_overrides (address, created_at DESC);

CREATE TABLE IF NOT EXISTS risk_override_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    override_id UUID NOT NULL REFERENCES risk_score_overrides (id) ON DELETE RESTRICT,
    address TEXT NOT NULL,
    admin_id TEXT NOT NULL,
    previous_score SMALLINT NOT NULL,
    new_score SMALLINT NOT NULL,
    reason TEXT NOT NULL,
    original_factors JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS risk_override_audit_address_created_idx
    ON risk_override_audit (address, created_at DESC);

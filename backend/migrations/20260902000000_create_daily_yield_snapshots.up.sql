-- Issue #1130: Create daily_yield_snapshots table for yield accrual audit ledger
CREATE TABLE IF NOT EXISTS daily_yield_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    snapshot_date DATE NOT NULL,
    principal_balance NUMERIC NOT NULL,
    yield_rate_bps INTEGER NOT NULL,
    daily_accrued_yield NUMERIC NOT NULL,
    total_accrued_yield NUMERIC NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_daily_yield_snapshots_plan_date UNIQUE (plan_id, snapshot_date)
);

CREATE INDEX IF NOT EXISTS idx_daily_yield_snapshots_plan_id ON daily_yield_snapshots(plan_id);
CREATE INDEX IF NOT EXISTS idx_daily_yield_snapshots_date ON daily_yield_snapshots(snapshot_date);

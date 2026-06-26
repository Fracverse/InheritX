-- Issue #810: persist yield_rate_bps for yield calculation in GET /api/plans

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS yield_rate_bps INTEGER NOT NULL DEFAULT 0
    CHECK (yield_rate_bps >= 0 AND yield_rate_bps <= 10000);
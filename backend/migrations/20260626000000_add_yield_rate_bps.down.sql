-- Issue #810: rollback yield_rate_bps column addition

ALTER TABLE plans
    DROP COLUMN IF EXISTS yield_rate_bps;
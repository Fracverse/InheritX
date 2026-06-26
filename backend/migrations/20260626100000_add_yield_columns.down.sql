ALTER TABLE plans
    DROP COLUMN IF EXISTS yield_rate_bps,
    DROP COLUMN IF EXISTS accrued_yield;

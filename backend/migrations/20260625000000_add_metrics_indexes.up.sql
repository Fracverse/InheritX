-- Add indexes for optimized admin metrics queries
CREATE INDEX IF NOT EXISTS plans_is_active_idx ON plans (is_active);
CREATE INDEX IF NOT EXISTS payouts_status_idx ON payouts (status);

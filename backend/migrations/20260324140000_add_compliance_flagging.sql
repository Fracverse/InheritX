-- Add compliance flagging to plans
ALTER TABLE plans ADD COLUMN IF NOT EXISTS is_flagged BOOLEAN DEFAULT FALSE;
ALTER TABLE plans ADD COLUMN IF NOT EXISTS suspicion_flags TEXT;

-- Index for performance on compliance scans
CREATE INDEX IF NOT EXISTS idx_plans_is_flagged ON plans(is_flagged);

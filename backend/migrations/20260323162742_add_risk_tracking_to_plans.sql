-- Add risk tracking fields to plans
ALTER TABLE plans ADD COLUMN IF NOT EXISTS is_risky BOOLEAN DEFAULT false;
ALTER TABLE plans ADD COLUMN IF NOT EXISTS health_factor DECIMAL(10, 4);
ALTER TABLE plans ADD COLUMN IF NOT EXISTS risk_flagged_at TIMESTAMP WITH TIME ZONE;

-- Create index for faster querying of risky loans
CREATE INDEX IF NOT EXISTS idx_plans_is_risky ON plans(is_risky);

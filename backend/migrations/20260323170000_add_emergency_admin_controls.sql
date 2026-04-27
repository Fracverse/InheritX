-- Add emergency admin control fields to plans table
ALTER TABLE plans ADD COLUMN IF NOT EXISTS is_paused BOOLEAN DEFAULT false;
ALTER TABLE plans ADD COLUMN IF NOT EXISTS paused_by UUID REFERENCES admins(id);
ALTER TABLE plans ADD COLUMN IF NOT EXISTS paused_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE plans ADD COLUMN IF NOT EXISTS pause_reason TEXT;

ALTER TABLE plans ADD COLUMN IF NOT EXISTS risk_override_enabled BOOLEAN DEFAULT false;
ALTER TABLE plans ADD COLUMN IF NOT EXISTS risk_override_by UUID REFERENCES admins(id);
ALTER TABLE plans ADD COLUMN IF NOT EXISTS risk_override_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE plans ADD COLUMN IF NOT EXISTS risk_override_reason TEXT;

-- Create indexes for faster querying
CREATE INDEX IF NOT EXISTS idx_plans_is_paused ON plans(is_paused);
CREATE INDEX IF NOT EXISTS idx_plans_risk_override ON plans(risk_override_enabled);

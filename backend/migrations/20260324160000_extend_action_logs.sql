-- Extend action_logs table for detailed audit logging
-- Add admin_id for admin-specific actions
ALTER TABLE action_logs ADD COLUMN IF NOT EXISTS admin_id UUID REFERENCES admins(id) ON DELETE SET NULL;

-- Add value tracking for parameter updates
ALTER TABLE action_logs ADD COLUMN IF NOT EXISTS old_value TEXT;
ALTER TABLE action_logs ADD COLUMN IF NOT EXISTS new_value TEXT;

-- Add metadata for additional context
ALTER TABLE action_logs ADD COLUMN IF NOT EXISTS metadata JSONB;

-- Create index for admin_id
CREATE INDEX IF NOT EXISTS idx_action_logs_admin_id ON action_logs(admin_id);

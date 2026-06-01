-- Add event ordering guarantee to audit logs
-- Ensures concurrent events maintain proper ordering via sequence numbers

ALTER TABLE action_logs ADD COLUMN IF NOT EXISTS sequence_number BIGSERIAL UNIQUE;

-- Create index for ordered queries by sequence number
CREATE INDEX IF NOT EXISTS idx_action_logs_sequence ON action_logs(sequence_number DESC);

-- Create index for user audit trail ordered by sequence
CREATE INDEX IF NOT EXISTS idx_action_logs_user_sequence ON action_logs(user_id, sequence_number DESC) WHERE user_id IS NOT NULL;

-- Create index for admin audit trail ordered by sequence
CREATE INDEX IF NOT EXISTS idx_action_logs_admin_sequence ON action_logs(admin_id, sequence_number DESC) WHERE admin_id IS NOT NULL;

-- Issue #76: Notifications & Logging
-- Add `type` column to existing notifications table
ALTER TABLE notifications
    ADD COLUMN IF NOT EXISTS type VARCHAR(100) NOT NULL DEFAULT 'general';

-- Create canonical action_logs table per spec
-- Columns: id, user_id, action, entity_id, entity_type, timestamp
CREATE TABLE IF NOT EXISTS action_logs (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id      UUID REFERENCES users(id) ON DELETE SET NULL,
    action       TEXT NOT NULL,
    entity_id    UUID,
    entity_type  TEXT,
    timestamp    TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_action_logs_user_id   ON action_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_action_logs_timestamp ON action_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_action_logs_action    ON action_logs(action);

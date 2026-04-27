-- Message Access Audit Logs
-- Tracks all access activity on legacy messages for security auditing

CREATE TABLE IF NOT EXISTS message_access_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID REFERENCES legacy_messages(id) ON DELETE SET NULL,
    user_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_message_access_logs_message_id ON message_access_logs(message_id);
CREATE INDEX idx_message_access_logs_user_id ON message_access_logs(user_id);
CREATE INDEX idx_message_access_logs_action ON message_access_logs(action);
CREATE INDEX idx_message_access_logs_created_at ON message_access_logs(created_at DESC);
CREATE INDEX idx_message_access_logs_user_action ON message_access_logs(user_id, action);

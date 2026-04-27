-- Add user_id to will_event_log for better audit tracking

ALTER TABLE will_event_log
ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE SET NULL;

-- Index for user-based queries
CREATE INDEX idx_will_event_log_user_id ON will_event_log(user_id);

-- Composite index for user + time queries
CREATE INDEX idx_will_event_log_user_created ON will_event_log(user_id, created_at DESC);

-- Add IP address and user agent for security auditing (optional but useful)
ALTER TABLE will_event_log
ADD COLUMN ip_address INET,
ADD COLUMN user_agent TEXT;

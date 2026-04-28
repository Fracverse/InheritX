-- Add webhook system tables
-- Migration: 20260427000000_add_webhook_system.sql

CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    secret TEXT NOT NULL,
    events TEXT[] NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_delivery TIMESTAMP WITH TIME ZONE,
    failure_count INTEGER NOT NULL DEFAULT 0
);

-- Index for efficient webhook lookup by event type
CREATE INDEX idx_webhooks_events ON webhooks USING GIN (events);
CREATE INDEX idx_webhooks_user_id ON webhooks (user_id);
CREATE INDEX idx_webhooks_active ON webhooks (is_active);

-- Add comment
COMMENT ON TABLE webhooks IS 'Stores webhook configurations for external integrations';
CREATE TABLE IF NOT EXISTS emergency_access_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    grant_id UUID NOT NULL REFERENCES emergency_access_grants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emergency_contact_id UUID NOT NULL REFERENCES emergency_contacts(id) ON DELETE CASCADE,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_active_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    ended_at TIMESTAMP WITH TIME ZONE,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_emergency_access_sessions_grant_id
    ON emergency_access_sessions(grant_id);

CREATE INDEX IF NOT EXISTS idx_emergency_access_sessions_user_id
    ON emergency_access_sessions(user_id, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_emergency_access_sessions_active
    ON emergency_access_sessions(ended_at) WHERE ended_at IS NULL;

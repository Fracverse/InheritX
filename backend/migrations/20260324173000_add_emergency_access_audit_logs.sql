CREATE TABLE IF NOT EXISTS emergency_access_grants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emergency_contact_id UUID NOT NULL REFERENCES emergency_contacts(id) ON DELETE CASCADE,
    permissions TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    revoked_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_emergency_access_grants_user_id
    ON emergency_access_grants(user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_emergency_access_grants_contact_id
    ON emergency_access_grants(emergency_contact_id);

CREATE INDEX IF NOT EXISTS idx_emergency_access_grants_active
    ON emergency_access_grants(is_active, expires_at);

CREATE TABLE IF NOT EXISTS emergency_access_audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    grant_id UUID NOT NULL REFERENCES emergency_access_grants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emergency_contact_id UUID NOT NULL REFERENCES emergency_contacts(id) ON DELETE CASCADE,
    action VARCHAR(100) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_emergency_access_audit_logs_user_id
    ON emergency_access_audit_logs(user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_emergency_access_audit_logs_grant_id
    ON emergency_access_audit_logs(grant_id);

CREATE INDEX IF NOT EXISTS idx_emergency_access_audit_logs_action
    ON emergency_access_audit_logs(action);

CREATE TRIGGER update_emergency_access_grants_updated_at
BEFORE UPDATE ON emergency_access_grants
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

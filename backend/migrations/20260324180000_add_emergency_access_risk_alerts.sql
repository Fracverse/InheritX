CREATE TABLE IF NOT EXISTS emergency_access_risk_alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    grant_id UUID NOT NULL REFERENCES emergency_access_grants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emergency_contact_id UUID NOT NULL REFERENCES emergency_contacts(id) ON DELETE CASCADE,
    alert_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    message TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_emergency_access_risk_alerts_user_id
    ON emergency_access_risk_alerts(user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_emergency_access_risk_alerts_grant_id
    ON emergency_access_risk_alerts(grant_id);

CREATE INDEX IF NOT EXISTS idx_emergency_access_risk_alerts_type
    ON emergency_access_risk_alerts(alert_type);

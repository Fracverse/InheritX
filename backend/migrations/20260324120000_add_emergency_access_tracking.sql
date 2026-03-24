-- Issue #293: Emergency Access Notification System
-- Track emergency access grants, revocations, and expirations

CREATE TABLE IF NOT EXISTS emergency_access (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    granted_by UUID NOT NULL REFERENCES admins(id),
    granted_to UUID REFERENCES users(id) ON DELETE SET NULL,
    access_type VARCHAR(50) NOT NULL, -- 'admin_override', 'temporary_access', etc.
    reason TEXT NOT NULL,
    granted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE,
    revoked_at TIMESTAMP WITH TIME ZONE,
    revoked_by UUID REFERENCES admins(id),
    revocation_reason TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'active', -- 'active', 'expired', 'revoked'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_emergency_access_plan_id ON emergency_access(plan_id);
CREATE INDEX IF NOT EXISTS idx_emergency_access_granted_to ON emergency_access(granted_to);
CREATE INDEX IF NOT EXISTS idx_emergency_access_status ON emergency_access(status);
CREATE INDEX IF NOT EXISTS idx_emergency_access_expires_at ON emergency_access(expires_at);

-- Add notification types for emergency access events
-- These are referenced in the notifications table via the 'type' column
-- Types: emergency_access_granted, emergency_access_revoked, emergency_access_expiring

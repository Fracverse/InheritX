-- Governance proposal execution support (Issue #648)
ALTER TABLE governance_proposals
    ADD COLUMN IF NOT EXISTS action_type VARCHAR(50),
    ADD COLUMN IF NOT EXISTS action_payload JSONB,
    ADD COLUMN IF NOT EXISTS executed_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN IF NOT EXISTS executed_by UUID REFERENCES admins(id);

-- Default quorum threshold for governance proposals
INSERT INTO protocol_parameters (name, value)
VALUES ('governance_quorum', '1')
ON CONFLICT (name) DO NOTHING;

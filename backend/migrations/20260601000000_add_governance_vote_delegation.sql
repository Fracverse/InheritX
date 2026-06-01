-- Migration: Add governance vote delegation mechanism (Issue #649)
-- Allows users to delegate their governance voting power to another user.

CREATE TABLE IF NOT EXISTS governance_delegations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    delegator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    delegate_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Each delegator can only have one active delegation at a time
    CONSTRAINT uq_governance_delegations_delegator UNIQUE (delegator_id),
    -- Prevent self-delegation at the DB level
    CONSTRAINT chk_no_self_delegation CHECK (delegator_id <> delegate_id)
);

CREATE INDEX IF NOT EXISTS idx_governance_delegations_delegate
    ON governance_delegations (delegate_id);

-- Audit trail for delegation changes
CREATE TABLE IF NOT EXISTS governance_delegation_history (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    delegator_id UUID NOT NULL,
    delegate_id  UUID,          -- NULL means undelegated
    action       TEXT NOT NULL CHECK (action IN ('delegated', 'redelegated', 'undelegated')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_governance_delegation_history_delegator
    ON governance_delegation_history (delegator_id);

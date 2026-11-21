-- Issue #1133: keep an immutable audit trail of plan modifications.
--
-- `update_plan` replaces the beneficiary rows in place, so without this table
-- the previous grace period and beneficiary allocations are lost forever.
-- Each row snapshots the plan just before an update and the values that
-- replaced it, along with the wallet/public key that authorised the change.

CREATE TABLE plan_revisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans (id) ON DELETE CASCADE,
    revision_number BIGINT NOT NULL CHECK (revision_number > 0),
    previous_values JSONB NOT NULL,
    new_values JSONB NOT NULL,
    changed_fields TEXT[] NOT NULL DEFAULT '{}',
    updated_by TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT plan_revisions_plan_revision_unique UNIQUE (plan_id, revision_number)
);

CREATE INDEX plan_revisions_plan_id_updated_at_idx
    ON plan_revisions (plan_id, updated_at DESC);

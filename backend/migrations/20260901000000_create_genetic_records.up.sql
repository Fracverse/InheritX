-- Issue #1102: persist genetic verification metadata, privacy flags and family
-- kinships for the testnet genetic-consensus feature.
--
-- Assumptions:
-- - `plans` and `beneficiaries` (UUID primary keys) are already created by the
--   core tables migration, so both foreign keys below reference those tables.
-- - verification_status and kinship_degree are constrained with CHECK instead
--   of dedicated enum types, matching the most recent table migrations (e.g.
--   plan_loan_lifecycle).
-- - beneficiary_id is nullable: a record may concern the plan owner (no
--   specific beneficiary) or one beneficiary under the plan.
--
-- Index rationale:
-- - genetic_records(plan_id): load every genetic record for a plan.
-- - genetic_records(beneficiary_id): look up a beneficiary's genetic record.
-- - genetic_records(verification_status): verification backlog/dashboards.

CREATE TABLE genetic_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans (id) ON DELETE CASCADE,
    beneficiary_id UUID REFERENCES beneficiaries (id) ON DELETE CASCADE,
    wallet_address TEXT NOT NULL,
    verification_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (verification_status IN ('pending', 'submitted', 'verified', 'failed', 'revoked')),
    verification_hash TEXT,
    verified_at TIMESTAMPTZ,
    privacy_consent BOOLEAN NOT NULL DEFAULT FALSE,
    data_retention_until TIMESTAMPTZ,
    kinship_degree TEXT
        CHECK (kinship_degree IS NULL OR kinship_degree IN
            ('self', 'parent', 'child', 'sibling', 'grandparent', 'grandchild', 'spouse', 'other')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT genetic_records_plan_wallet_unique UNIQUE (plan_id, wallet_address)
);

CREATE INDEX genetic_records_plan_id_idx ON genetic_records (plan_id);
CREATE INDEX genetic_records_beneficiary_id_idx ON genetic_records (beneficiary_id);
CREATE INDEX genetic_records_verification_status_idx ON genetic_records (verification_status);

-- Issue #1099: genetic verification API.
--
-- Only a client-derived SHA-256 fingerprint of a genetic file is ever stored
-- (genetic_dna_hash); raw genetic material never reaches the backend. The
-- verification lifecycle lives on "plans" because a plan is the unit that owns
-- a genetic record, and invitations get their own table so a plan can host
-- several pending invites without widening the plans row.

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS genetic_dna_hash TEXT,
    ADD COLUMN IF NOT EXISTS genetic_verification_status TEXT NOT NULL DEFAULT 'unverified',
    ADD COLUMN IF NOT EXISTS genetic_verified_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS genetic_verifying_authority TEXT,
    ADD COLUMN IF NOT EXISTS genetic_family_tree_id BIGINT,
    ADD COLUMN IF NOT EXISTS genetic_conditions JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS genetic_privacy_settings JSONB NOT NULL DEFAULT
        '{"shareWithFamily": false, "allowHealthAnalysis": false, "visibleToRelatives": false, "dataRetentionDays": 365}'::jsonb;

DO $$
BEGIN
    ALTER TABLE plans
        ADD CONSTRAINT plans_genetic_verification_status_check
        CHECK (genetic_verification_status IN
            ('unverified', 'pending', 'verified', 'rejected', 'partial_match', 'requires_retest'));
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS genetic_family_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans (id) ON DELETE CASCADE,
    to_email TEXT NOT NULL,
    proposed_relationship TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'expired', 'declined')),
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS genetic_family_invitations_plan_id_idx
    ON genetic_family_invitations (plan_id);
CREATE INDEX IF NOT EXISTS genetic_family_invitations_to_email_idx
    ON genetic_family_invitations (to_email);

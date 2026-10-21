-- Issue #1134: aggregate guardian signatures for an emergency unlock.
--
-- Guardians coordinate out-of-band and submit their signatures one at a time.
-- The backend verifies each ed25519 signature, stores it once per guardian,
-- and submits `guardian_emergency_trigger` to Soroban as soon as the
-- configured threshold is met.

CREATE TABLE guardian_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans (id) ON DELETE CASCADE,
    guardian_address TEXT NOT NULL,
    signature TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT guardian_signatures_plan_guardian_unique UNIQUE (plan_id, guardian_address)
);

CREATE INDEX guardian_signatures_plan_id_created_at_idx
    ON guardian_signatures (plan_id, created_at ASC);

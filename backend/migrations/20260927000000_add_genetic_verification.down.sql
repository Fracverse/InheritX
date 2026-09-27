DROP TABLE IF EXISTS genetic_family_invitations;

ALTER TABLE plans
    DROP COLUMN IF EXISTS genetic_privacy_settings,
    DROP COLUMN IF EXISTS genetic_conditions,
    DROP COLUMN IF EXISTS genetic_family_tree_id,
    DROP COLUMN IF EXISTS genetic_verifying_authority,
    DROP COLUMN IF EXISTS genetic_verified_at,
    DROP COLUMN IF EXISTS genetic_verification_status,
    DROP COLUMN IF EXISTS genetic_dna_hash;

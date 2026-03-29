-- Add unique constraint on plan_id in claims table to prevent duplicate claims
-- This ensures only one claim per plan, preventing race condition vulnerabilities

-- First, remove any existing duplicate claims (keep the first one by created_at)
DELETE FROM claims
WHERE id NOT IN (
    SELECT DISTINCT ON (plan_id) id
    FROM claims
    ORDER BY plan_id, created_at ASC
);

-- Drop the existing unique constraint on (plan_id, beneficiary_email)
ALTER TABLE claims DROP CONSTRAINT IF EXISTS claims_plan_id_beneficiary_email_key;

-- Add unique constraint on just plan_id to ensure only one claim per plan
ALTER TABLE claims ADD CONSTRAINT claims_plan_id_key UNIQUE (plan_id);

-- Re-add unique constraint on (plan_id, beneficiary_email) for the same beneficiary case
ALTER TABLE claims ADD CONSTRAINT claims_plan_id_beneficiary_email_key UNIQUE (plan_id, beneficiary_email);

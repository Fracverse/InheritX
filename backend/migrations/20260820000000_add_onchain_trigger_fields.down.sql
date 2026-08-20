DROP INDEX IF EXISTS idx_plans_trigger_in_flight;
DROP INDEX IF EXISTS plans_onchain_plan_id_key;

ALTER TABLE plans
    DROP CONSTRAINT IF EXISTS plans_onchain_plan_id_non_negative;

ALTER TABLE plans
    DROP COLUMN IF EXISTS last_trigger_error,
    DROP COLUMN IF EXISTS trigger_started_at,
    DROP COLUMN IF EXISTS trigger_attempts,
    DROP COLUMN IF EXISTS trigger_tx_hash,
    DROP COLUMN IF EXISTS onchain_plan_id;

-- Issue #1039: the inactivity watchdog now executes the payout on-chain before
-- flipping a plan to TRIGGERED, so it needs the Soroban plan id to call
-- `trigger_inheritance` with, plus somewhere to record the attempt.

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS onchain_plan_id BIGINT,
    ADD COLUMN IF NOT EXISTS trigger_tx_hash TEXT,
    ADD COLUMN IF NOT EXISTS trigger_attempts INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS trigger_started_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN IF NOT EXISTS last_trigger_error TEXT;

ALTER TABLE plans
    ADD CONSTRAINT plans_onchain_plan_id_non_negative
    CHECK (onchain_plan_id IS NULL OR onchain_plan_id >= 0)
    NOT VALID;

ALTER TABLE plans
    VALIDATE CONSTRAINT plans_onchain_plan_id_non_negative;

-- One database plan per on-chain plan: triggering the same contract plan from
-- two rows would double-submit the payout.
CREATE UNIQUE INDEX IF NOT EXISTS plans_onchain_plan_id_key
    ON plans (onchain_plan_id)
    WHERE onchain_plan_id IS NOT NULL;

-- Lets the watchdog cheaply find submissions left in flight by a crashed worker.
CREATE INDEX IF NOT EXISTS idx_plans_trigger_in_flight
    ON plans (trigger_started_at)
    WHERE status = 'TRIGGERING';

-- Issue #820: persist proof-of-life inactivity timers for the watchdog worker.
--
-- `plans.last_ping` is a BIGINT of Unix seconds (see the core tables
-- migration) and the API binds it as such, so the deadline is derived with
-- to_timestamp() rather than by adding an INTERVAL to a timestamp.

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS status VARCHAR(32) NOT NULL DEFAULT 'ACTIVE';

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS grace_period_seconds BIGINT NOT NULL DEFAULT 7776000;

ALTER TABLE plans
    ADD CONSTRAINT plans_grace_period_seconds_non_negative
    CHECK (grace_period_seconds >= 0)
    NOT VALID;

ALTER TABLE plans
    VALIDATE CONSTRAINT plans_grace_period_seconds_non_negative;

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS inactivity_deadline_at TIMESTAMP WITH TIME ZONE
    GENERATED ALWAYS AS (to_timestamp(last_ping + grace_period_seconds)) STORED;

CREATE INDEX IF NOT EXISTS idx_plans_inactivity_deadline_claimable
    ON plans (inactivity_deadline_at)
    WHERE COALESCE(is_active, true) = true
      AND status <> 'CLAIMABLE';

CREATE INDEX IF NOT EXISTS idx_plans_last_ping
    ON plans (last_ping);

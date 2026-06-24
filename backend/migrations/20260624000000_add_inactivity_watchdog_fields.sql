-- Issue #820: persist proof-of-life inactivity timers for the watchdog worker.

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS status VARCHAR(32) NOT NULL DEFAULT 'ACTIVE',
    ADD COLUMN IF NOT EXISTS last_ping TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS grace_period_seconds BIGINT NOT NULL DEFAULT 7776000;

ALTER TABLE plans
    ADD CONSTRAINT plans_grace_period_seconds_non_negative
    CHECK (grace_period_seconds >= 0)
    NOT VALID;

ALTER TABLE plans
    VALIDATE CONSTRAINT plans_grace_period_seconds_non_negative;

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS inactivity_deadline_at TIMESTAMP WITH TIME ZONE
    GENERATED ALWAYS AS (
        last_ping + (grace_period_seconds::double precision * INTERVAL '1 second')
    ) STORED;

CREATE INDEX IF NOT EXISTS idx_plans_inactivity_deadline_claimable
    ON plans (inactivity_deadline_at)
    WHERE COALESCE(is_active, true) = true
      AND status <> 'CLAIMABLE';

CREATE INDEX IF NOT EXISTS idx_plans_last_ping
    ON plans (last_ping);

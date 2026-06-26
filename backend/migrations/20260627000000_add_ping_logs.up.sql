CREATE TABLE ping_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans (id) ON DELETE CASCADE,
    pinged_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    accrued_yield_snapshot NUMERIC(78, 4) NOT NULL DEFAULT 0
);

CREATE INDEX ping_logs_plan_id_idx ON ping_logs (plan_id);
CREATE INDEX ping_logs_pinged_at_idx ON ping_logs (pinged_at);

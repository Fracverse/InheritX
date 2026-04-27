-- Migration: Add plan_logs table (plans already created in init)

CREATE TABLE IF NOT EXISTS plan_logs (
    id SERIAL PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    action VARCHAR(64) NOT NULL,
    performed_by UUID NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_plan_logs_plan_id ON plan_logs(plan_id);
CREATE INDEX idx_plan_logs_performed_by ON plan_logs(performed_by);

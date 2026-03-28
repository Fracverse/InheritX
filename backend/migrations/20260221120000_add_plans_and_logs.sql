-- Migration: Add plan_logs table (plans already created in init)

CREATE TABLE IF NOT EXISTS plan_logs (
    id SERIAL PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES plans(id),
    action VARCHAR(64) NOT NULL,
    performed_by UUID NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW()
);

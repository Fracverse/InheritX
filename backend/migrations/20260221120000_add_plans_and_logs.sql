-- Migration: Add plan_logs table
-- NOTE:
-- The plans table is already created in the initial migration.
-- This migration should only add the dependent plan_logs table.

CREATE TABLE plan_logs (
    id SERIAL PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    action VARCHAR(64) NOT NULL,
    performed_by UUID NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_plan_logs_plan_id ON plan_logs(plan_id);
CREATE INDEX idx_plan_logs_performed_by ON plan_logs(performed_by);
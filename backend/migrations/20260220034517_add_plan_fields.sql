-- Add fields to plans table to support due-for-claim queries

ALTER TABLE plans
ADD COLUMN IF NOT EXISTS contract_plan_id BIGINT,
ADD COLUMN IF NOT EXISTS distribution_method VARCHAR(20),
ADD COLUMN IF NOT EXISTS is_active BOOLEAN DEFAULT true,
ADD COLUMN IF NOT EXISTS contract_created_at BIGINT;

-- Add index for contract_plan_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_plans_contract_plan_id ON plans(contract_plan_id);

-- Add index for is_active and status combination
CREATE INDEX IF NOT EXISTS idx_plans_active_status ON plans(is_active, status);

-- Create claims table to track which plans have been claimed
CREATE TABLE IF NOT EXISTS claims (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    contract_plan_id BIGINT NOT NULL,
    beneficiary_email VARCHAR(255) NOT NULL,
    claimed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    UNIQUE(plan_id, beneficiary_email)
);

-- Add index for claims lookups
CREATE INDEX IF NOT EXISTS idx_claims_plan_id ON claims(plan_id);
CREATE INDEX IF NOT EXISTS idx_claims_contract_plan_id ON claims(contract_plan_id);

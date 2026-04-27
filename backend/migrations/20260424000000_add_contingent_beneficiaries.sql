-- ──────────────────────────────────────────────────────────────────────────────
-- Contingent Beneficiaries System
-- Adds support for backup beneficiaries with activation conditions
-- ──────────────────────────────────────────────────────────────────────────────

-- Beneficiary type enum
CREATE TYPE beneficiary_type AS ENUM ('primary', 'contingent');

-- Contingency condition enum
CREATE TYPE contingency_condition AS ENUM (
    'primary_declined',
    'primary_deceased',
    'primary_timeout',
    'manual_promotion'
);

-- Extend plan_beneficiaries table with contingent beneficiary support
ALTER TABLE plan_beneficiaries ADD COLUMN IF NOT EXISTS beneficiary_type beneficiary_type DEFAULT 'primary';
ALTER TABLE plan_beneficiaries ADD COLUMN IF NOT EXISTS priority_order INTEGER DEFAULT 0;
ALTER TABLE plan_beneficiaries ADD COLUMN IF NOT EXISTS is_active BOOLEAN DEFAULT true;
ALTER TABLE plan_beneficiaries ADD COLUMN IF NOT EXISTS activated_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE plan_beneficiaries ADD COLUMN IF NOT EXISTS activation_reason VARCHAR(100);

-- Contingency conditions table
CREATE TABLE IF NOT EXISTS contingency_conditions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    primary_beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id) ON DELETE CASCADE,
    contingent_beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id) ON DELETE CASCADE,
    
    -- Activation conditions
    condition_type contingency_condition NOT NULL,
    timeout_days INTEGER,  -- For primary_timeout condition
    
    -- Status tracking
    is_active BOOLEAN DEFAULT true,
    activated_at TIMESTAMP WITH TIME ZONE,
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    UNIQUE(plan_id, primary_beneficiary_id, contingent_beneficiary_id, condition_type)
);

-- Contingent beneficiary activation events table
CREATE TABLE IF NOT EXISTS contingent_activation_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    primary_beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id) ON DELETE CASCADE,
    contingent_beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id) ON DELETE CASCADE,
    
    -- Activation details
    activation_reason contingency_condition NOT NULL,
    activated_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Timestamps
    activated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Contingent beneficiary promotion history table
CREATE TABLE IF NOT EXISTS contingent_promotions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id) ON DELETE CASCADE,
    
    -- Promotion details
    promoted_from_type beneficiary_type NOT NULL,
    promoted_to_type beneficiary_type NOT NULL,
    promotion_reason VARCHAR(255),
    promoted_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Timestamps
    promoted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Contingent beneficiary claim attempts table
CREATE TABLE IF NOT EXISTS contingent_claim_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    beneficiary_id UUID NOT NULL REFERENCES plan_beneficiaries(id) ON DELETE CASCADE,
    
    -- Claim attempt details
    attempt_status VARCHAR(50) NOT NULL,  -- 'pending', 'accepted', 'declined', 'expired'
    decline_reason VARCHAR(255),
    
    -- Timestamps
    attempted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    responded_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_contingency_conditions_plan_id ON contingency_conditions(plan_id);
CREATE INDEX IF NOT EXISTS idx_contingency_conditions_primary_beneficiary ON contingency_conditions(primary_beneficiary_id);
CREATE INDEX IF NOT EXISTS idx_contingency_conditions_contingent_beneficiary ON contingency_conditions(contingent_beneficiary_id);

CREATE INDEX IF NOT EXISTS idx_contingent_activation_events_plan_id ON contingent_activation_events(plan_id);
CREATE INDEX IF NOT EXISTS idx_contingent_activation_events_primary_beneficiary ON contingent_activation_events(primary_beneficiary_id);
CREATE INDEX IF NOT EXISTS idx_contingent_activation_events_contingent_beneficiary ON contingent_activation_events(contingent_beneficiary_id);

CREATE INDEX IF NOT EXISTS idx_contingent_promotions_plan_id ON contingent_promotions(plan_id);
CREATE INDEX IF NOT EXISTS idx_contingent_promotions_beneficiary_id ON contingent_promotions(beneficiary_id);

CREATE INDEX IF NOT EXISTS idx_contingent_claim_attempts_plan_id ON contingent_claim_attempts(plan_id);
CREATE INDEX IF NOT EXISTS idx_contingent_claim_attempts_beneficiary_id ON contingent_claim_attempts(beneficiary_id);
CREATE INDEX IF NOT EXISTS idx_contingent_claim_attempts_status ON contingent_claim_attempts(attempt_status);

-- Auto-update updated_at trigger for contingency_conditions
CREATE OR REPLACE FUNCTION update_contingency_conditions_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_contingency_conditions_updated_at
BEFORE UPDATE ON contingency_conditions
FOR EACH ROW EXECUTE FUNCTION update_contingency_conditions_updated_at();

-- Configuration table for contingency timeouts
CREATE TABLE IF NOT EXISTS contingency_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    
    -- Timeout configuration (in days)
    primary_claim_timeout_days INTEGER DEFAULT 30,
    contingent_claim_timeout_days INTEGER DEFAULT 30,
    
    -- Activation configuration
    auto_activate_on_timeout BOOLEAN DEFAULT true,
    require_manual_confirmation BOOLEAN DEFAULT false,
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    UNIQUE(plan_id)
);

CREATE INDEX IF NOT EXISTS idx_contingency_config_plan_id ON contingency_config(plan_id);

-- Auto-update updated_at trigger for contingency_config
CREATE OR REPLACE FUNCTION update_contingency_config_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_contingency_config_updated_at
BEFORE UPDATE ON contingency_config
FOR EACH ROW EXECUTE FUNCTION update_contingency_config_updated_at();

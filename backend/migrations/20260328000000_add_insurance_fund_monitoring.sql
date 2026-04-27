-- ──────────────────────────────────────────────────────────────────────────────
-- Insurance Fund Monitoring
-- Tracks reserve health, coverage ratios, and fund performance metrics
-- ──────────────────────────────────────────────────────────────────────────────

-- Insurance fund status enum
CREATE TYPE insurance_fund_status AS ENUM (
    'healthy',
    'warning',
    'critical',
    'insolvent'
);

-- Insurance Fund Table
-- Tracks the overall insurance pool for the InheritX platform
CREATE TABLE insurance_fund (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Fund identification
    fund_name               VARCHAR(100) NOT NULL UNIQUE,
    description             TEXT,
    
    -- Asset information
    asset_code              VARCHAR(20) NOT NULL,
    total_reserves          NUMERIC(30, 8) NOT NULL DEFAULT 0 CHECK (total_reserves >= 0),
    available_reserves      NUMERIC(30, 8) NOT NULL DEFAULT 0 CHECK (available_reserves >= 0),
    locked_reserves         NUMERIC(30, 8) NOT NULL DEFAULT 0 CHECK (locked_reserves >= 0),
    
    -- Coverage metrics
    total_covered_liabilities NUMERIC(30, 8) NOT NULL DEFAULT 0 CHECK (total_covered_liabilities >= 0),
    coverage_ratio          NUMERIC(10, 4) NOT NULL DEFAULT 0,  -- reserves / liabilities
    reserve_health_score    NUMERIC(5, 2) NOT NULL DEFAULT 100,  -- 0-100 score
    
    -- Risk thresholds
    min_coverage_ratio      NUMERIC(5, 4) NOT NULL DEFAULT 1.0,  -- Minimum acceptable ratio
    target_coverage_ratio   NUMERIC(5, 4) NOT NULL DEFAULT 1.5,  -- Target ratio
    critical_coverage_ratio NUMERIC(5, 4) NOT NULL DEFAULT 0.5,  -- Critical threshold
    
    -- Status tracking
    status                  insurance_fund_status NOT NULL DEFAULT 'healthy',
    status_changed_at       TIMESTAMP WITH TIME ZONE,
    
    -- Performance tracking
    total_contributions     NUMERIC(30, 8) NOT NULL DEFAULT 0,
    total_payouts           NUMERIC(30, 8) NOT NULL DEFAULT 0,
    yield_earned            NUMERIC(30, 8) NOT NULL DEFAULT 0,
    
    -- Metadata
    metadata                JSONB NOT NULL DEFAULT '{}',
    
    -- Timestamps
    created_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Insurance Fund Metrics History
-- Historical tracking of fund metrics for auditing and trend analysis
CREATE TABLE insurance_fund_metrics_history (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fund_id                 UUID NOT NULL REFERENCES insurance_fund(id) ON DELETE CASCADE,
    
    -- Snapshot metrics
    total_reserves          NUMERIC(30, 8) NOT NULL,
    available_reserves      NUMERIC(30, 8) NOT NULL,
    locked_reserves         NUMERIC(30, 8) NOT NULL,
    total_covered_liabilities NUMERIC(30, 8) NOT NULL,
    coverage_ratio          NUMERIC(10, 4) NOT NULL,
    reserve_health_score    NUMERIC(5, 2) NOT NULL,
    status                  insurance_fund_status NOT NULL,
    
    -- Event that triggered the snapshot (optional)
    trigger_event           VARCHAR(50),
    trigger_event_id        UUID,
    
    -- Timestamp
    recorded_at             TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Insurance Fund Transactions
-- Tracks all movements in and out of the insurance fund
CREATE TABLE insurance_fund_transactions (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fund_id                 UUID NOT NULL REFERENCES insurance_fund(id) ON DELETE CASCADE,
    
    -- Transaction type
    transaction_type        VARCHAR(50) NOT NULL,  -- 'contribution', 'payout', 'yield', 'fee', 'penalty'
    
    -- Related entities
    user_id                 UUID REFERENCES users(id) ON DELETE SET NULL,
    plan_id                 UUID REFERENCES plans(id) ON DELETE SET NULL,
    loan_id                 UUID REFERENCES loan_lifecycle(id) ON DELETE SET NULL,
    
    -- Amount and asset
    asset_code              VARCHAR(20) NOT NULL,
    amount                  NUMERIC(30, 8) NOT NULL,
    balance_after           NUMERIC(30, 8) NOT NULL,
    
    -- Transaction details
    description             TEXT,
    metadata                JSONB NOT NULL DEFAULT '{}',
    transaction_hash        VARCHAR(255),
    
    -- Timestamp
    created_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Insurance Claims against the fund
CREATE TABLE insurance_claims (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fund_id                 UUID NOT NULL REFERENCES insurance_fund(id) ON DELETE CASCADE,
    
    -- Claim context
    user_id                 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id                 UUID REFERENCES plans(id) ON DELETE SET NULL,
    loan_id                 UUID REFERENCES loan_lifecycle(id) ON DELETE SET NULL,
    
    -- Claim details
    claim_type              VARCHAR(50) NOT NULL,  -- 'liquidation', 'default', 'emergency_payout'
    claimed_amount          NUMERIC(30, 8) NOT NULL CHECK (claimed_amount > 0),
    approved_amount         NUMERIC(30, 8),
    payout_amount           NUMERIC(30, 8),
    
    -- Status tracking
    status                  VARCHAR(50) NOT NULL DEFAULT 'pending',  -- 'pending', 'approved', 'rejected', 'paid'
    rejection_reason        TEXT,
    
    -- Review process
    reviewed_by             UUID REFERENCES admins(id) ON DELETE SET NULL,
    reviewed_at             TIMESTAMP WITH TIME ZONE,
    paid_at                 TIMESTAMP WITH TIME ZONE,
    
    -- Metadata
    metadata                JSONB NOT NULL DEFAULT '{}',
    
    -- Timestamps
    created_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- ── Indexes ──────────────────────────────────────────────────────────────────
CREATE INDEX idx_insurance_fund_name ON insurance_fund(fund_name);
CREATE INDEX idx_insurance_fund_status ON insurance_fund(status);
CREATE INDEX idx_insurance_fund_metrics_fund_id ON insurance_fund_metrics_history(fund_id);
CREATE INDEX idx_insurance_fund_metrics_recorded_at ON insurance_fund_metrics_history(recorded_at DESC);
CREATE INDEX idx_insurance_fund_transactions_fund_id ON insurance_fund_transactions(fund_id);
CREATE INDEX idx_insurance_fund_transactions_type ON insurance_fund_transactions(transaction_type);
CREATE INDEX idx_insurance_fund_transactions_created_at ON insurance_fund_transactions(created_at DESC);
CREATE INDEX idx_insurance_claims_fund_id ON insurance_claims(fund_id);
CREATE INDEX idx_insurance_claims_status ON insurance_claims(status);
CREATE INDEX idx_insurance_claims_user_id ON insurance_claims(user_id);
CREATE INDEX idx_insurance_claims_plan_id ON insurance_claims(plan_id);

-- ── Auto-update updated_at via trigger ───────────────────────────────────────
CREATE OR REPLACE FUNCTION update_insurance_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_insurance_fund_updated_at
BEFORE UPDATE ON insurance_fund
FOR EACH ROW EXECUTE FUNCTION update_insurance_updated_at();

CREATE TRIGGER trg_insurance_claims_updated_at
BEFORE UPDATE ON insurance_claims
FOR EACH ROW EXECUTE FUNCTION update_insurance_updated_at();

-- ── Comments ─────────────────────────────────────────────────────────────────
COMMENT ON TABLE insurance_fund IS 'Tracks insurance/reserve funds for platform risk coverage';
COMMENT ON COLUMN insurance_fund.coverage_ratio IS 'Ratio of reserves to covered liabilities (target: > 1.5)';
COMMENT ON COLUMN insurance_fund.reserve_health_score IS 'Composite score 0-100 based on multiple health metrics';
COMMENT ON TABLE insurance_fund_metrics_history IS 'Historical snapshots of insurance fund metrics for trend analysis';
COMMENT ON TABLE insurance_fund_transactions IS 'All movements in/out of insurance funds';
COMMENT ON TABLE insurance_claims IS 'Insurance claims against the reserve fund';

-- ── Seed Data ────────────────────────────────────────────────────────────────
-- Create default insurance fund for the platform
INSERT INTO insurance_fund (
    fund_name,
    description,
    asset_code,
    total_reserves,
    available_reserves,
    locked_reserves,
    total_covered_liabilities,
    coverage_ratio,
    reserve_health_score,
    min_coverage_ratio,
    target_coverage_ratio,
    critical_coverage_ratio,
    status
) VALUES (
    'Platform Reserve Fund',
    'Primary insurance fund covering all platform liabilities and risks',
    'USDC',
    0,
    0,
    0,
    0,
    0,
    100.00,
    1.00,
    1.50,
    0.50,
    'healthy'
);

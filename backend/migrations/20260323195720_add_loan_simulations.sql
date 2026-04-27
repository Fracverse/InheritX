-- Add loan simulation history table
CREATE TABLE IF NOT EXISTS loan_simulations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    
    -- Input parameters
    loan_amount DECIMAL(20, 8) NOT NULL,
    loan_duration_days INTEGER NOT NULL,
    collateral_type VARCHAR(50) NOT NULL,
    collateral_price_usd DECIMAL(20, 8) NOT NULL,
    
    -- Calculation results
    required_collateral DECIMAL(20, 8) NOT NULL,
    collateral_quantity DECIMAL(20, 8) NOT NULL,
    estimated_interest DECIMAL(20, 8) NOT NULL,
    total_repayment DECIMAL(20, 8) NOT NULL,
    liquidation_price DECIMAL(20, 8) NOT NULL,
    loan_to_value_ratio DECIMAL(5, 4) NOT NULL,
    interest_rate DECIMAL(5, 4) NOT NULL,
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Index for querying simulations by user
CREATE INDEX idx_loan_simulations_user_id ON loan_simulations(user_id);
CREATE INDEX idx_loan_simulations_created_at ON loan_simulations(created_at);

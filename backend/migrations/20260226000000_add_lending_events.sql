-- Add structured event system for DeFi lending operations

-- Event types enum
CREATE TYPE event_type AS ENUM (
    'deposit',
    'borrow',
    'repay',
    'liquidation',
    'interest_accrual'
);

-- Lending Events Table
CREATE TABLE lending_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type event_type NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id UUID REFERENCES plans(id) ON DELETE SET NULL,
    
    -- Asset information
    asset_code VARCHAR(20) NOT NULL,
    amount VARCHAR(50) NOT NULL,  -- Stored as string for precision
    
    -- Event-specific data (stored as JSONB for flexibility)
    metadata JSONB NOT NULL DEFAULT '{}',
    
    -- Transaction tracking
    transaction_hash VARCHAR(255),
    block_number BIGINT,
    
    -- Timestamps
    event_timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Create indexes for efficient event querying
CREATE INDEX idx_lending_events_user_id ON lending_events(user_id);
CREATE INDEX idx_lending_events_plan_id ON lending_events(plan_id);
CREATE INDEX idx_lending_events_type ON lending_events(event_type);
CREATE INDEX idx_lending_events_asset_code ON lending_events(asset_code);
CREATE INDEX idx_lending_events_timestamp ON lending_events(event_timestamp DESC);
CREATE INDEX idx_lending_events_transaction_hash ON lending_events(transaction_hash);
CREATE INDEX idx_lending_events_metadata ON lending_events USING GIN (metadata);

-- Composite indexes for common queries
CREATE INDEX idx_lending_events_user_type ON lending_events(user_id, event_type);
CREATE INDEX idx_lending_events_plan_type ON lending_events(plan_id, event_type);

-- Add comment for documentation
COMMENT ON TABLE lending_events IS 'Structured events for DeFi lending operations: deposits, borrows, repayments, liquidations, and interest accruals';
COMMENT ON COLUMN lending_events.metadata IS 'Event-specific data: interest_rate, collateral_ratio, liquidation_penalty, etc.';
COMMENT ON COLUMN lending_events.amount IS 'Amount stored as string for decimal precision';


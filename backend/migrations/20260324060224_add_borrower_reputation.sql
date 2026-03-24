-- Add Borrower Reputation System

-- Borrower Reputation Table (One-to-One with Users)
CREATE TABLE borrower_reputation (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    score INTEGER NOT NULL DEFAULT 100,
    total_borrowed DECIMAL(20, 8) NOT NULL DEFAULT 0,
    total_repaid DECIMAL(20, 8) NOT NULL DEFAULT 0,
    liquidation_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Index for querying reputation by score
CREATE INDEX idx_borrower_reputation_score ON borrower_reputation(score);

-- Trigger to update updated_at timestamp
CREATE TRIGGER update_borrower_reputation_updated_at BEFORE UPDATE ON borrower_reputation FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

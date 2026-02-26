-- Add price feed and collateral valuation support

-- Price Feed Configuration Table
CREATE TABLE price_feeds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_code VARCHAR(20) NOT NULL UNIQUE,
    source VARCHAR(50) NOT NULL, -- 'pyth', 'chainlink', 'custom'
    feed_id VARCHAR(255) NOT NULL, -- External feed identifier
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    last_updated TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Asset Price History Table
CREATE TABLE asset_price_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_code VARCHAR(20) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    price_timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    source VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (asset_code) REFERENCES price_feeds(asset_code) ON DELETE CASCADE
);

-- Add collateral tracking to plans table
ALTER TABLE plans ADD COLUMN IF NOT EXISTS collateral_ratio DECIMAL(5, 2) DEFAULT 100.00;
ALTER TABLE plans ADD COLUMN IF NOT EXISTS asset_code VARCHAR(20) DEFAULT 'USDC';
ALTER TABLE plans ADD COLUMN IF NOT EXISTS valuation_usd DECIMAL(20, 8);
ALTER TABLE plans ADD COLUMN IF NOT EXISTS last_valuation_at TIMESTAMP WITH TIME ZONE;

-- Create indexes for performance
CREATE INDEX idx_asset_price_history_asset_code ON asset_price_history(asset_code);
CREATE INDEX idx_asset_price_history_timestamp ON asset_price_history(price_timestamp DESC);
CREATE INDEX idx_plans_asset_code ON plans(asset_code);
CREATE INDEX idx_plans_collateral_ratio ON plans(collateral_ratio);
CREATE INDEX idx_price_feeds_active ON price_feeds(is_active);

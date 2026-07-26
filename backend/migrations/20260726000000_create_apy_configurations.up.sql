CREATE TABLE apy_configurations (
    token_address TEXT PRIMARY KEY,
    rate_bps INTEGER NOT NULL CHECK (rate_bps >= 0 AND rate_bps <= 10000),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed defaults corresponding to the frontend config
INSERT INTO apy_configurations (token_address, rate_bps) VALUES
('XLM', 200),
('USDC', 300),
('CUSTOM', 100)
ON CONFLICT (token_address) DO NOTHING;

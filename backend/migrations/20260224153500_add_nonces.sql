-- Add nonces table for Web3 login
CREATE TABLE IF NOT EXISTS nonces (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(255) UNIQUE NOT NULL,
    nonce VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Index for searching nonces by wallet address
CREATE INDEX IF NOT EXISTS idx_nonces_wallet_address ON nonces(wallet_address);

-- Add nonce column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS nonce VARCHAR(255);

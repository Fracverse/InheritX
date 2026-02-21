-- Update existing two_fa table to match our 2FA implementation
-- The initial migration created a two_fa table, but we need to ensure it has the right structure

-- Drop the old two_fa table if it exists (from init migration)
DROP TABLE IF EXISTS two_fa CASCADE;

-- Create user_2fa table for storing OTPs with proper structure
CREATE TABLE user_2fa (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    otp_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Index for faster lookups
CREATE INDEX idx_user_2fa_user_id ON user_2fa(user_id);
CREATE INDEX idx_user_2fa_expires_at ON user_2fa(expires_at);

-- Clean up expired OTPs periodically (optional, can be done via cron job)
COMMENT ON TABLE user_2fa IS 'Stores temporary OTPs for two-factor authentication';


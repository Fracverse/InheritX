-- Session management — Issue #436
-- Database-backed JWT session tracking for explicit revocation.

CREATE TABLE IF NOT EXISTS sessions (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- SHA-256 hex digest of the raw JWT — avoids storing the secret token itself
    token_hash   VARCHAR(64) NOT NULL UNIQUE,
    created_at   TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at   TIMESTAMP WITH TIME ZONE NOT NULL,
    revoked      BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at   TIMESTAMP WITH TIME ZONE,
    -- Optional label for "manage devices" UX (e.g., "Chrome / macOS")
    device_label VARCHAR(255)
);

-- Primary guard check: is this token hash revoked?
CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions (token_hash);

-- Logout-all and session list by user
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);

-- Housekeeping: find and clean up expired sessions
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions (expires_at);

-- Partial index for active-session queries
CREATE INDEX IF NOT EXISTS idx_sessions_active
    ON sessions (user_id, expires_at)
    WHERE revoked = FALSE;

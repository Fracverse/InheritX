-- CSRF token storage — Issue #434
-- Stores single-use, time-limited CSRF tokens tied to authenticated users.

CREATE TABLE IF NOT EXISTS csrf_tokens (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token       VARCHAR(64) NOT NULL UNIQUE,
    expires_at  TIMESTAMP WITH TIME ZONE NOT NULL,
    used        BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Fast lookup by token value (the hot path in the validation middleware)
CREATE INDEX IF NOT EXISTS idx_csrf_tokens_token ON csrf_tokens (token);

-- Partial index for the active-token query: token + not-expired + not-used
CREATE INDEX IF NOT EXISTS idx_csrf_tokens_active
    ON csrf_tokens (token)
    WHERE used = FALSE AND expires_at > NOW();

-- Housekeeping: TTL-based cleanup selects by expiry and user
CREATE INDEX IF NOT EXISTS idx_csrf_tokens_user_id    ON csrf_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_csrf_tokens_expires_at ON csrf_tokens (expires_at);

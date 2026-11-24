-- API keys that let authorized integration runners (e.g. E2E testnet bots)
-- bypass the public per-IP rate limit via the `X-API-Key` header (Issue #1138).
-- Only the SHA-256 hash (lowercase hex) of a key is stored, never the key.
--
-- Provision a key (generate a long random string, keep the raw value secret):
--   INSERT INTO api_keys (key_hash, label)
--   VALUES (encode(sha256('<raw-key>'::bytea), 'hex'), 'ci-testnet-runner');
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_hash TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT api_keys_key_hash_check
        CHECK (key_hash ~ '^[0-9a-f]{64}$'),
    CONSTRAINT api_keys_label_check
        CHECK (char_length(label) BETWEEN 1 AND 200)
);

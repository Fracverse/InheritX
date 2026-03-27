-- Secure message encryption, key management, and legacy delivery workflow

CREATE TABLE IF NOT EXISTS message_encryption_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_version INTEGER NOT NULL UNIQUE,
    encrypted_key BYTEA NOT NULL,
    wrapping_nonce BYTEA NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_by_admin_id UUID REFERENCES admins(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    rotated_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX IF NOT EXISTS idx_message_encryption_keys_status
    ON message_encryption_keys(status);

CREATE TABLE IF NOT EXISTS legacy_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    beneficiary_contact VARCHAR(255) NOT NULL,
    encrypted_payload BYTEA NOT NULL,
    payload_nonce BYTEA NOT NULL,
    key_version INTEGER NOT NULL,
    unlock_at TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    delivered_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_legacy_messages_unlock_pending
    ON legacy_messages(unlock_at, status);

CREATE INDEX IF NOT EXISTS idx_legacy_messages_owner
    ON legacy_messages(owner_user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS legacy_message_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES legacy_messages(id) ON DELETE CASCADE,
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    beneficiary_contact VARCHAR(255) NOT NULL,
    decrypted_payload TEXT NOT NULL,
    delivered_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_legacy_message_deliveries_owner
    ON legacy_message_deliveries(owner_user_id, delivered_at DESC);

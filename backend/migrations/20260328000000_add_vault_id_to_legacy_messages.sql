-- Associate legacy messages with vaults to support multiple messages per vault

ALTER TABLE legacy_messages
    ADD COLUMN IF NOT EXISTS vault_id BIGINT;

CREATE INDEX IF NOT EXISTS idx_legacy_messages_vault
    ON legacy_messages(vault_id, created_at DESC);

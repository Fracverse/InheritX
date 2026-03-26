-- Document encryption columns and backup table for secure document storage

ALTER TABLE will_documents
    ADD COLUMN encrypted_content BYTEA,
    ADD COLUMN encryption_nonce  BYTEA,
    ADD COLUMN is_encrypted      BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE document_backups (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id     UUID NOT NULL REFERENCES will_documents(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    backup_hash     VARCHAR(64) NOT NULL,
    encrypted_content BYTEA NOT NULL,
    encryption_nonce  BYTEA NOT NULL,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_document_backups_document_id ON document_backups(document_id);
CREATE INDEX idx_document_backups_user_id     ON document_backups(user_id);

-- Issue #1273: persistence for will-document retrieval, verification and the
-- document audit trail. Storage of newly generated/uploaded documents is a
-- separate ticket; these tables are the read model the client queries.

CREATE TABLE IF NOT EXISTS will_documents (
    document_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES plans (id) ON DELETE CASCADE,
    template_used TEXT NOT NULL DEFAULT 'standard',
    will_hash TEXT NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL CHECK (version > 0),
    filename TEXT NOT NULL,
    content BYTEA,
    is_encrypted BOOLEAN NOT NULL DEFAULT FALSE,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'superseded', 'revoked')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT will_documents_plan_version_unique UNIQUE (plan_id, version)
);

CREATE INDEX IF NOT EXISTS will_documents_plan_id_idx
    ON will_documents (plan_id, version);

CREATE TABLE IF NOT EXISTS will_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    document_id UUID REFERENCES will_documents (document_id) ON DELETE SET NULL,
    plan_id UUID NOT NULL REFERENCES plans (id) ON DELETE CASCADE,
    vault_id TEXT NOT NULL DEFAULT '',
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS will_events_plan_id_created_at_idx
    ON will_events (plan_id, created_at);

CREATE INDEX IF NOT EXISTS will_events_document_id_created_at_idx
    ON will_events (document_id, created_at);

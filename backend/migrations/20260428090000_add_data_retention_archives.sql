-- Issue #453: Data retention and archival support
CREATE TABLE IF NOT EXISTS data_archives (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_table TEXT NOT NULL,
    source_id TEXT NOT NULL,
    payload JSONB NOT NULL,
    archived_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_data_archives_source_table
    ON data_archives (source_table, archived_at DESC);

CREATE INDEX IF NOT EXISTS idx_data_archives_source_id
    ON data_archives (source_id);

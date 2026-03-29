-- Legacy Content Upload System

CREATE TABLE IF NOT EXISTS legacy_content (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    storage_path TEXT NOT NULL,
    file_hash VARCHAR(64) NOT NULL,
    encrypted BOOLEAN NOT NULL DEFAULT false,
    encryption_key_version INTEGER,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_legacy_content_owner
    ON legacy_content(owner_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_legacy_content_type
    ON legacy_content(content_type);

CREATE INDEX IF NOT EXISTS idx_legacy_content_status
    ON legacy_content(status);

CREATE INDEX IF NOT EXISTS idx_legacy_content_hash
    ON legacy_content(file_hash);

-- Content type validation constraint
ALTER TABLE legacy_content
    ADD CONSTRAINT check_content_type
    CHECK (content_type IN (
        'video/mp4', 'video/mpeg', 'video/quicktime', 'video/x-msvideo', 'video/webm',
        'audio/mpeg', 'audio/wav', 'audio/ogg', 'audio/mp4', 'audio/webm',
        'text/plain', 'text/markdown', 'text/html',
        'application/pdf', 'application/msword', 
        'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        'application/vnd.ms-excel',
        'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
    ));

-- File size constraint (max 500MB)
ALTER TABLE legacy_content
    ADD CONSTRAINT check_file_size
    CHECK (file_size > 0 AND file_size <= 524288000);

-- Status constraint
ALTER TABLE legacy_content
    ADD CONSTRAINT check_status
    CHECK (status IN ('active', 'deleted', 'processing', 'failed'));

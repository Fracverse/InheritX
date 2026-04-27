-- Will Event Log Table for auditing and indexing all legal will actions

CREATE TABLE will_event_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type      VARCHAR(50) NOT NULL,
    document_id     UUID NOT NULL,
    plan_id         UUID NOT NULL,
    vault_id        VARCHAR(255) NOT NULL,
    event_data      JSONB NOT NULL,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_will_event_log_document_id ON will_event_log(document_id);
CREATE INDEX idx_will_event_log_plan_id ON will_event_log(plan_id);
CREATE INDEX idx_will_event_log_vault_id ON will_event_log(vault_id);
CREATE INDEX idx_will_event_log_event_type ON will_event_log(event_type);
CREATE INDEX idx_will_event_log_created_at ON will_event_log(created_at DESC);

-- Composite index for common queries
CREATE INDEX idx_will_event_log_plan_created ON will_event_log(plan_id, created_at DESC);
CREATE INDEX idx_will_event_log_document_created ON will_event_log(document_id, created_at DESC);

-- GIN index for JSONB queries
CREATE INDEX idx_will_event_log_event_data ON will_event_log USING GIN (event_data);

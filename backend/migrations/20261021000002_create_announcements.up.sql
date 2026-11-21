CREATE TABLE announcements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'info',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    starts_at TIMESTAMPTZ,
    ends_at TIMESTAMPTZ,
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT announcements_severity_check
        CHECK (severity IN ('info', 'warning', 'critical')),
    CONSTRAINT announcements_title_check
        CHECK (char_length(title) BETWEEN 1 AND 200),
    CONSTRAINT announcements_message_check
        CHECK (char_length(message) BETWEEN 1 AND 2000),
    CONSTRAINT announcements_schedule_check
        CHECK (starts_at IS NULL OR ends_at IS NULL OR starts_at <= ends_at)
);

CREATE INDEX idx_announcements_active_created_at
    ON announcements (active, created_at DESC);

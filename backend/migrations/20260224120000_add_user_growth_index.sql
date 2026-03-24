-- Index on users.created_at for efficient date-range counting
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

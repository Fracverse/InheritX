-- Remove indexes for admin metrics queries
DROP INDEX IF EXISTS plans_is_active_idx;
DROP INDEX IF EXISTS payouts_status_idx;

-- Rollback anchor payout tracking fields
DROP INDEX IF EXISTS payouts_external_transaction_id_idx;
DROP INDEX IF EXISTS payouts_status_idx;

ALTER TABLE payouts
DROP COLUMN IF EXISTS updated_at,
DROP COLUMN IF EXISTS external_transaction_id,
DROP COLUMN IF EXISTS anchor_fee_usd,
DROP COLUMN IF EXISTS exchange_rate;

DROP INDEX IF EXISTS payouts_anchor_payout_id_idx;

ALTER TABLE payouts
DROP COLUMN IF EXISTS anchor_payout_id;

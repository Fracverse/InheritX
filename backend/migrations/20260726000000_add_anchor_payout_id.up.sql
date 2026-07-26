-- Add anchor_payout_id column to payouts table for tracking Stellar Anchor
-- transaction references. This allows the system to correlate internal payout
-- records with the anchor's external transaction ID for status polling.
ALTER TABLE payouts
ADD COLUMN anchor_payout_id TEXT;

CREATE INDEX payouts_anchor_payout_id_idx ON payouts (anchor_payout_id);

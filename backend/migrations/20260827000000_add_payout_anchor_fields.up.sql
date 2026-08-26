-- Add fields for Stellar Anchor off-ramp payout tracking
ALTER TABLE payouts
ADD COLUMN exchange_rate NUMERIC(18, 6) NOT NULL DEFAULT 1.0,
ADD COLUMN anchor_fee_usd NUMERIC(18, 6) NOT NULL DEFAULT 0.0,
ADD COLUMN external_transaction_id TEXT,
ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Index for background worker status polling
CREATE INDEX payouts_status_idx ON payouts (status) WHERE status IN ('pending', 'processing');

-- Index for external transaction ID lookups
CREATE INDEX payouts_external_transaction_id_idx ON payouts (external_transaction_id) WHERE external_transaction_id IS NOT NULL;

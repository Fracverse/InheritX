-- Issue #1017: payout audit log.
--
-- `plans` and `beneficiaries` are already created by the core tables
-- migration, and re-creating them here aborted the whole migration chain
-- ("relation \"plans\" already exists"), so this migration now only adds the
-- table that was genuinely new. It references the existing `beneficiaries`.

CREATE TABLE IF NOT EXISTS payout_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    beneficiary_id UUID NOT NULL REFERENCES beneficiaries (id) ON DELETE CASCADE,
    amount NUMERIC(19, 4) NOT NULL,
    payout_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    transaction_hash TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS payout_logs_beneficiary_id_idx ON payout_logs (beneficiary_id);

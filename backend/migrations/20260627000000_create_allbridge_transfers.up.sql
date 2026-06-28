-- Create table to track Allbridge transfers and their verification status

CREATE TABLE allbridge_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    origin_tx_hash TEXT NOT NULL,
    origin_chain TEXT,
    target_chain TEXT,
    origin_token TEXT,
    target_token TEXT,
    amount NUMERIC(78, 0) NOT NULL,
    target_amount NUMERIC(78, 0),
    fees NUMERIC(78, 0),
    status TEXT NOT NULL DEFAULT 'PENDING',
    last_polled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX allbridge_transfers_origin_tx_hash_idx ON allbridge_transfers (origin_tx_hash);
CREATE INDEX allbridge_transfers_status_idx ON allbridge_transfers (status);

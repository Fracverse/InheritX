CREATE TABLE kyc_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address TEXT NOT NULL,
    full_name TEXT NOT NULL,
    date_of_birth TEXT NOT NULL,
    street_address TEXT NOT NULL,
    city TEXT NOT NULL,
    country TEXT NOT NULL,
    postal_code TEXT NOT NULL,
    document_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '365 days'),
    CONSTRAINT kyc_records_wallet_unique UNIQUE (wallet_address)
);

CREATE INDEX kyc_records_wallet_address_idx ON kyc_records (wallet_address);

CREATE INDEX kyc_records_expires_at_idx ON kyc_records (expires_at);

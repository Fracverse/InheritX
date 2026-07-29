CREATE TABLE plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    apy_rate_bps INT NOT NULL,
    min_amount NUMERIC(19, 4) NOT NULL,
    max_amount NUMERIC(19, 4),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE beneficiaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_wallet_address TEXT NOT NULL,
    beneficiary_wallet_address TEXT NOT NULL,
    share_percentage NUMERIC(5, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE payout_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    beneficiary_id UUID NOT NULL REFERENCES beneficiaries(id),
    amount NUMERIC(19, 4) NOT NULL,
    payout_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    transaction_hash TEXT NOT NULL
);

CREATE INDEX beneficiaries_owner_wallet_idx ON beneficiaries (owner_wallet_address);
CREATE INDEX payout_logs_beneficiary_id_idx ON payout_logs (beneficiary_id);

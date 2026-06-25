CREATE TABLE kyc_webhook_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address TEXT NOT NULL,
    provider_reference TEXT,
    event_type TEXT NOT NULL,
    kyc_status kyc_status NOT NULL,
    raw_payload JSONB NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL DEFAULT TRUE,
    error_message TEXT
);

CREATE INDEX kyc_webhook_logs_wallet_address_idx ON kyc_webhook_logs (wallet_address);
CREATE INDEX kyc_webhook_logs_processed_at_idx ON kyc_webhook_logs (processed_at);

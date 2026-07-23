ALTER TABLE beneficiaries
ADD COLUMN fiat_daily_limit NUMERIC(78, 0) NOT NULL DEFAULT 0;

CREATE TABLE fiat_daily_usage (
    beneficiary_id UUID NOT NULL REFERENCES beneficiaries (id) ON DELETE CASCADE,
    usage_date DATE NOT NULL DEFAULT CURRENT_DATE,
    total_amount NUMERIC(78, 0) NOT NULL DEFAULT 0,
    PRIMARY KEY (beneficiary_id, usage_date)
);

CREATE INDEX fiat_daily_usage_beneficiary_id_idx ON fiat_daily_usage (beneficiary_id);
CREATE INDEX fiat_daily_usage_date_idx ON fiat_daily_usage (usage_date);

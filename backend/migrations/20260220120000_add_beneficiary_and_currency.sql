-- Add beneficiary bank account and currency preference to plans table

-- Currency preference: USDC (crypto) or FIAT (bank transfer)
ALTER TABLE plans
ADD COLUMN IF NOT EXISTS beneficiary_name VARCHAR(255),
ADD COLUMN IF NOT EXISTS bank_account_number VARCHAR(255),
ADD COLUMN IF NOT EXISTS bank_name VARCHAR(255),
ADD COLUMN IF NOT EXISTS currency_preference VARCHAR(10);

-- Restrict currency_preference to USDC or FIAT
ALTER TABLE plans
ADD CONSTRAINT chk_currency_preference
CHECK (currency_preference IS NULL OR currency_preference IN ('USDC', 'FIAT'));

-- Index for querying by currency preference (e.g. payout processing)
CREATE INDEX IF NOT EXISTS idx_plans_currency_preference ON plans(currency_preference);

-- Composite index for status + currency (beneficiary/claim queries)
CREATE INDEX IF NOT EXISTS idx_plans_status_currency ON plans(status, currency_preference);

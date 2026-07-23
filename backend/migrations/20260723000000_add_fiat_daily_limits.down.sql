DROP TABLE IF EXISTS fiat_daily_usage;

ALTER TABLE beneficiaries
DROP COLUMN IF EXISTS fiat_daily_limit;

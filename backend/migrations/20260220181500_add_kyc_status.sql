-- Fix: Ensure kyc_status has all required columns even if created by init.sql
ALTER TABLE kyc_status ADD COLUMN IF NOT EXISTS reviewed_by UUID;
ALTER TABLE kyc_status ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMP WITH TIME ZONE;

-- Status type might be VARCHAR(20) from init, increasing to VARCHAR(50) if needed
ALTER TABLE kyc_status ALTER COLUMN status TYPE VARCHAR(50);

CREATE INDEX IF NOT EXISTS idx_kyc_status_status ON kyc_status(status);

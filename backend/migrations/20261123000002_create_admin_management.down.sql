ALTER TABLE kyc_records
    DROP COLUMN IF EXISTS status,
    DROP COLUMN IF EXISTS reviewed_by,
    DROP COLUMN IF EXISTS reviewed_at,
    DROP COLUMN IF EXISTS rejection_reason;

ALTER TABLE plans
    DROP COLUMN IF EXISTS is_paused,
    DROP COLUMN IF EXISTS paused_reason,
    DROP COLUMN IF EXISTS paused_at,
    DROP COLUMN IF EXISTS risk_override_enabled,
    DROP COLUMN IF EXISTS risk_override_reason,
    DROP COLUMN IF EXISTS risk_override_at;

DROP TABLE IF EXISTS insurance_funds;
DROP TABLE IF EXISTS audit_logs;

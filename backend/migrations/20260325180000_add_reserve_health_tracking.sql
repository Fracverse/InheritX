-- Migration to add reserve health and coverage ratio tracking
ALTER TABLE pools ADD COLUMN IF NOT EXISTS bad_debt_reserve DECIMAL(20, 8) NOT NULL DEFAULT 0;
ALTER TABLE pools ADD COLUMN IF NOT EXISTS retained_yield DECIMAL(20, 8) NOT NULL DEFAULT 0;
ALTER TABLE pools ADD COLUMN IF NOT EXISTS coverage_ratio DECIMAL(10, 4);
ALTER TABLE pools ADD COLUMN IF NOT EXISTS reserve_health_status VARCHAR(20) DEFAULT 'healthy';
ALTER TABLE pools ADD COLUMN IF NOT EXISTS last_health_check_at TIMESTAMP WITH TIME ZONE;

-- Create index for efficient health monitoring queries
CREATE INDEX IF NOT EXISTS idx_pools_health_status ON pools(reserve_health_status);
CREATE INDEX IF NOT EXISTS idx_pools_coverage_ratio ON pools(coverage_ratio);

-- Update existing pools with initial values
UPDATE pools SET 
    bad_debt_reserve = 0,
    retained_yield = 0,
    coverage_ratio = 1.0,
    reserve_health_status = 'healthy',
    last_health_check_at = CURRENT_TIMESTAMP
WHERE bad_debt_reserve IS NULL;

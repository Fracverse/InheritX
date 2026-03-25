-- Manual Testing Script for Reserve Health Tracking
-- Run these queries to test the reserve health system

-- ============================================
-- 1. SETUP: Verify pools table structure
-- ============================================

-- Check if new columns exist
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_name = 'pools'
ORDER BY ordinal_position;

-- Expected columns:
-- - bad_debt_reserve (numeric)
-- - retained_yield (numeric)
-- - coverage_ratio (numeric)
-- - reserve_health_status (varchar)
-- - last_health_check_at (timestamp)

-- ============================================
-- 2. SETUP: Insert test data
-- ============================================

-- Insert test pools if they don't exist
INSERT INTO pools (asset_code, total_liquidity, utilized_liquidity, bad_debt_reserve, retained_yield)
VALUES 
    ('USDC', 1000000.0, 150000.0, 20000.0, 5000.0),
    ('XLM', 500000.0, 50000.0, 3000.0, 1000.0),
    ('BTC', 100000.0, 80000.0, 2000.0, 500.0)
ON CONFLICT (asset_code) DO UPDATE SET
    total_liquidity = EXCLUDED.total_liquidity,
    utilized_liquidity = EXCLUDED.utilized_liquidity,
    bad_debt_reserve = EXCLUDED.bad_debt_reserve,
    retained_yield = EXCLUDED.retained_yield;

-- ============================================
-- 3. TEST: View current pool state
-- ============================================

SELECT 
    asset_code,
    total_liquidity,
    utilized_liquidity,
    bad_debt_reserve,
    retained_yield,
    coverage_ratio,
    reserve_health_status,
    last_health_check_at,
    -- Calculate metrics manually for verification
    ROUND((utilized_liquidity / NULLIF(total_liquidity, 0)) * 100, 2) as calc_utilization_pct,
    ROUND((bad_debt_reserve / NULLIF(utilized_liquidity, 0)) * 100, 2) as calc_coverage_pct,
    ROUND((bad_debt_reserve / NULLIF(total_liquidity, 0)) * 100, 2) as calc_reserve_adequacy_pct
FROM pools
ORDER BY asset_code;

-- Expected results for USDC:
-- - Utilization: 15%
-- - Coverage: 13.33%
-- - Reserve Adequacy: 2%
-- - Status: healthy (coverage > 10%)

-- ============================================
-- 4. TEST: Simulate healthy pool
-- ============================================

UPDATE pools 
SET 
    total_liquidity = 1000000.0,
    utilized_liquidity = 500000.0,
    bad_debt_reserve = 100000.0
WHERE asset_code = 'USDC';

-- Expected: Coverage ratio = 20% (healthy)

-- ============================================
-- 5. TEST: Simulate warning status
-- ============================================

UPDATE pools 
SET 
    total_liquidity = 1000000.0,
    utilized_liquidity = 800000.0,
    bad_debt_reserve = 80000.0
WHERE asset_code = 'XLM';

-- Expected: Coverage ratio = 10% (warning threshold)

-- ============================================
-- 6. TEST: Simulate critical status
-- ============================================

UPDATE pools 
SET 
    total_liquidity = 1000000.0,
    utilized_liquidity = 900000.0,
    bad_debt_reserve = 30000.0
WHERE asset_code = 'BTC';

-- Expected: Coverage ratio = 3.33% (critical)

-- ============================================
-- 7. TEST: Simulate high utilization
-- ============================================

UPDATE pools 
SET 
    total_liquidity = 1000000.0,
    utilized_liquidity = 950000.0,
    bad_debt_reserve = 150000.0
WHERE asset_code = 'USDC';

-- Expected: Utilization = 95% (high_utilization)

-- ============================================
-- 8. TEST: Check lending events sync
-- ============================================

-- View lending events (if table exists)
SELECT 
    asset_code,
    event_type,
    amount,
    created_at
FROM lending_events
WHERE asset_code IN ('USDC', 'XLM', 'BTC')
ORDER BY created_at DESC
LIMIT 20;

-- Aggregate lending activity
SELECT 
    asset_code,
    SUM(CASE WHEN event_type = 'borrow' THEN CAST(amount AS numeric) ELSE 0 END) as total_borrowed,
    SUM(CASE WHEN event_type = 'repay' THEN CAST(amount AS numeric) ELSE 0 END) as total_repaid,
    SUM(CASE WHEN event_type = 'borrow' THEN CAST(amount AS numeric) ELSE 0 END) -
    SUM(CASE WHEN event_type = 'repay' THEN CAST(amount AS numeric) ELSE 0 END) as net_utilized
FROM lending_events
WHERE asset_code IS NOT NULL
GROUP BY asset_code;

-- ============================================
-- 9. TEST: Check notifications
-- ============================================

-- View recent reserve health notifications
SELECT 
    n.id,
    n.user_id,
    n.notification_type,
    n.message,
    n.created_at,
    n.is_read
FROM notifications n
WHERE n.notification_type = 'reserve_health_alert'
ORDER BY n.created_at DESC
LIMIT 10;

-- ============================================
-- 10. TEST: Check audit logs
-- ============================================

-- View reserve health related audit logs
SELECT 
    al.id,
    al.user_id,
    al.action,
    al.entity_id,
    al.entity_type,
    al.created_at
FROM action_logs al
WHERE al.action = 'system_event'
ORDER BY al.created_at DESC
LIMIT 10;

-- ============================================
-- 11. VERIFICATION: Calculate expected metrics
-- ============================================

-- This query shows what the engine should calculate
SELECT 
    asset_code,
    total_liquidity,
    utilized_liquidity,
    bad_debt_reserve,
    
    -- Available liquidity
    (total_liquidity - utilized_liquidity) as available_liquidity,
    
    -- Utilization rate (%)
    ROUND((utilized_liquidity / NULLIF(total_liquidity, 0)) * 100, 2) as utilization_rate,
    
    -- Coverage ratio (decimal)
    ROUND(bad_debt_reserve / NULLIF(utilized_liquidity, 0), 4) as coverage_ratio,
    
    -- Reserve adequacy (%)
    ROUND((bad_debt_reserve / NULLIF(total_liquidity, 0)) * 100, 2) as reserve_adequacy,
    
    -- Expected health status
    CASE
        WHEN (bad_debt_reserve / NULLIF(utilized_liquidity, 0)) < 0.05 THEN 'critical'
        WHEN (bad_debt_reserve / NULLIF(utilized_liquidity, 0)) < 0.15 THEN 'warning'
        WHEN (utilized_liquidity / NULLIF(total_liquidity, 0)) > 0.90 THEN 'high_utilization'
        WHEN (bad_debt_reserve / NULLIF(utilized_liquidity, 0)) >= 0.10 THEN 'healthy'
        ELSE 'moderate'
    END as expected_status,
    
    -- Current status from engine
    reserve_health_status as current_status,
    
    -- Last check time
    last_health_check_at
FROM pools
ORDER BY asset_code;

-- ============================================
-- 12. CLEANUP: Reset to default state
-- ============================================

-- Uncomment to reset pools to default state
/*
UPDATE pools 
SET 
    total_liquidity = 1000000.0,
    utilized_liquidity = 150000.0,
    bad_debt_reserve = 20000.0,
    retained_yield = 5000.0,
    coverage_ratio = NULL,
    reserve_health_status = 'healthy',
    last_health_check_at = NULL
WHERE asset_code = 'USDC';

UPDATE pools 
SET 
    total_liquidity = 500000.0,
    utilized_liquidity = 50000.0,
    bad_debt_reserve = 10000.0,
    retained_yield = 2000.0,
    coverage_ratio = NULL,
    reserve_health_status = 'healthy',
    last_health_check_at = NULL
WHERE asset_code = 'XLM';
*/

-- ============================================
-- 13. PERFORMANCE: Check indexes
-- ============================================

-- Verify indexes exist
SELECT 
    tablename,
    indexname,
    indexdef
FROM pg_indexes
WHERE tablename = 'pools'
ORDER BY indexname;

-- Expected indexes:
-- - idx_pools_health_status
-- - idx_pools_coverage_ratio

-- ============================================
-- 14. MONITORING: Continuous health check
-- ============================================

-- Run this query periodically to monitor health
SELECT 
    asset_code,
    reserve_health_status,
    ROUND(coverage_ratio * 100, 2) as coverage_pct,
    ROUND((utilized_liquidity / total_liquidity) * 100, 2) as utilization_pct,
    last_health_check_at,
    EXTRACT(EPOCH FROM (NOW() - last_health_check_at)) / 60 as minutes_since_check
FROM pools
ORDER BY 
    CASE reserve_health_status
        WHEN 'critical' THEN 1
        WHEN 'warning' THEN 2
        WHEN 'high_utilization' THEN 3
        WHEN 'moderate' THEN 4
        WHEN 'healthy' THEN 5
        ELSE 6
    END,
    asset_code;

-- ============================================
-- EXPECTED RESULTS SUMMARY
-- ============================================

/*
Test Scenario 1 (Healthy):
- Total: 1,000,000
- Utilized: 500,000
- Reserve: 100,000
- Coverage: 20%
- Status: healthy

Test Scenario 2 (Warning):
- Total: 1,000,000
- Utilized: 800,000
- Reserve: 80,000
- Coverage: 10%
- Status: warning

Test Scenario 3 (Critical):
- Total: 1,000,000
- Utilized: 900,000
- Reserve: 30,000
- Coverage: 3.33%
- Status: critical

Test Scenario 4 (High Utilization):
- Total: 1,000,000
- Utilized: 950,000
- Reserve: 150,000
- Coverage: 15.79%
- Utilization: 95%
- Status: high_utilization
*/

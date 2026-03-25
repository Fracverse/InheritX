# Reserve Health Integration Guide

## Quick Start

### 1. Run Database Migration

```bash
cd backend
sqlx migrate run
```

This will add the necessary columns to the `pools` table.

### 2. Verify Engine Initialization

The Reserve Health Engine is automatically initialized in `app.rs`:

```rust
let reserve_health_engine = Arc::new(ReserveHealthEngine::new(db.clone()));
reserve_health_engine.clone().start();
```

### 3. Test the API

```bash
# Get all reserve health metrics
curl -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  http://localhost:8080/api/admin/reserve-health

# Get specific asset health
curl -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  http://localhost:8080/api/admin/reserve-health/USDC

# Manually sync reserves
curl -X POST \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  http://localhost:8080/api/admin/reserve-health/sync
```

## Integration Scenarios

### Scenario 1: Adding Reserve Health to Admin Dashboard

```typescript
// Frontend: Fetch reserve health metrics
async function fetchReserveHealth() {
  const response = await fetch('/api/admin/analytics/reserve-health', {
    headers: {
      'Authorization': `Bearer ${adminToken}`
    }
  });
  
  const { data } = await response.json();
  return data; // Array of ReserveHealthMetrics
}

// Display in dashboard
function ReserveHealthWidget({ metrics }) {
  return (
    <div className="reserve-health-widget">
      {metrics.map(metric => (
        <div key={metric.asset_code} className={`pool-status ${metric.health_status}`}>
          <h3>{metric.asset_code}</h3>
          <div className="metrics">
            <span>Coverage: {(metric.coverage_ratio * 100).toFixed(2)}%</span>
            <span>Utilization: {metric.utilization_rate.toFixed(2)}%</span>
            <span>Status: {metric.health_status}</span>
          </div>
        </div>
      ))}
    </div>
  );
}
```

### Scenario 2: Monitoring Reserve Health in Background Jobs

```rust
use crate::reserve_health::ReserveHealthEngine;

async fn daily_health_report(engine: Arc<ReserveHealthEngine>) {
    let metrics = engine.check_all_reserves().await?;
    
    for metric in metrics {
        if metric.health_status == "critical" || metric.health_status == "warning" {
            // Send email/slack notification
            send_alert(&format!(
                "Pool {} is in {} status with coverage ratio {:.2}%",
                metric.asset_code,
                metric.health_status,
                metric.coverage_ratio * 100.0
            )).await?;
        }
    }
}
```

### Scenario 3: Triggering Actions Based on Health Status

```rust
async fn check_and_pause_risky_operations(
    engine: Arc<ReserveHealthEngine>,
    db: &PgPool
) -> Result<(), ApiError> {
    let metrics = engine.check_all_reserves().await?;
    
    for metric in metrics {
        if metric.health_status == "critical" {
            // Pause new borrows for this asset
            sqlx::query(
                "UPDATE pools SET allow_new_borrows = false WHERE asset_code = $1"
            )
            .bind(&metric.asset_code)
            .execute(db)
            .await?;
            
            info!("Paused new borrows for {} due to critical reserve health", metric.asset_code);
        }
    }
    
    Ok(())
}
```

### Scenario 4: Custom Alert Thresholds

```rust
// Create engine with custom thresholds
pub struct CustomReserveHealthEngine {
    engine: ReserveHealthEngine,
    custom_thresholds: HashMap<String, Decimal>,
}

impl CustomReserveHealthEngine {
    pub fn new(db: PgPool, thresholds: HashMap<String, Decimal>) -> Self {
        Self {
            engine: ReserveHealthEngine::new(db),
            custom_thresholds: thresholds,
        }
    }
    
    pub async fn check_with_custom_thresholds(&self) -> Result<Vec<ReserveHealthMetrics>, ApiError> {
        let metrics = self.engine.check_all_reserves().await?;
        
        // Apply custom thresholds per asset
        for metric in &metrics {
            if let Some(threshold) = self.custom_thresholds.get(&metric.asset_code) {
                if metric.coverage_ratio < *threshold {
                    // Custom alert logic
                }
            }
        }
        
        Ok(metrics)
    }
}
```

## Event Handlers Integration

### Listening to Reserve Health Changes

```rust
use crate::events::{EventService, EventType};

async fn on_reserve_health_change(
    db: &PgPool,
    asset_code: &str,
    old_status: &str,
    new_status: &str,
    coverage_ratio: Decimal
) -> Result<(), ApiError> {
    // Log the event
    EventService::create_event(
        db,
        EventType::SystemEvent,
        None, // No specific user
        None, // No specific plan
        serde_json::json!({
            "event": "reserve_health_change",
            "asset_code": asset_code,
            "old_status": old_status,
            "new_status": new_status,
            "coverage_ratio": coverage_ratio
        })
    ).await?;
    
    // Trigger additional actions
    match new_status {
        "critical" => {
            // Emergency protocol
            trigger_emergency_protocol(db, asset_code).await?;
        },
        "warning" => {
            // Increase monitoring frequency
            increase_monitoring_frequency(asset_code).await?;
        },
        _ => {}
    }
    
    Ok(())
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_reserve_health_calculation() {
        let db = setup_test_db().await;
        
        // Insert test pool
        sqlx::query(
            "INSERT INTO pools (asset_code, total_liquidity, utilized_liquidity, bad_debt_reserve)
             VALUES ($1, $2, $3, $4)"
        )
        .bind("TEST")
        .bind(1000000)
        .bind(750000)
        .bind(100000)
        .execute(&db)
        .await
        .unwrap();
        
        let engine = ReserveHealthEngine::new(db.clone());
        let metrics = engine.get_reserve_health("TEST").await.unwrap();
        
        assert_eq!(metrics.asset_code, "TEST");
        assert_eq!(metrics.coverage_ratio, Decimal::new(133, 3)); // 0.133
        assert_eq!(metrics.utilization_rate, Decimal::from(75));
        assert_eq!(metrics.health_status, "healthy");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_reserve_health_api() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/reserve-health")
                .header("Authorization", "Bearer test_admin_token")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "success");
    assert!(json["data"].is_array());
}
```

## Monitoring and Observability

### Logging

The engine logs important events:

```rust
// In your logging configuration
tracing_subscriber::fmt()
    .with_target(true)
    .with_level(true)
    .with_filter(
        EnvFilter::new("info,reserve_health=debug")
    )
    .init();
```

### Metrics (Prometheus)

```rust
use prometheus::{register_gauge_vec, GaugeVec};

lazy_static! {
    static ref RESERVE_COVERAGE_RATIO: GaugeVec = register_gauge_vec!(
        "reserve_coverage_ratio",
        "Coverage ratio for each pool",
        &["asset_code"]
    ).unwrap();
    
    static ref RESERVE_UTILIZATION_RATE: GaugeVec = register_gauge_vec!(
        "reserve_utilization_rate",
        "Utilization rate for each pool",
        &["asset_code"]
    ).unwrap();
}

// Update metrics after each check
for metric in metrics {
    RESERVE_COVERAGE_RATIO
        .with_label_values(&[&metric.asset_code])
        .set(metric.coverage_ratio.to_f64().unwrap());
    
    RESERVE_UTILIZATION_RATE
        .with_label_values(&[&metric.asset_code])
        .set(metric.utilization_rate.to_f64().unwrap());
}
```

## Configuration

### Environment Variables

```bash
# .env
RESERVE_CHECK_INTERVAL_SECS=300  # 5 minutes
MIN_COVERAGE_RATIO=0.10          # 10%
WARNING_COVERAGE_RATIO=0.15      # 15%
CRITICAL_COVERAGE_RATIO=0.05     # 5%
```

### Loading Configuration

```rust
use crate::config::Config;

impl ReserveHealthEngine {
    pub fn from_config(db: PgPool, config: &Config) -> Self {
        Self {
            db,
            min_coverage_ratio: config.min_coverage_ratio,
            warning_coverage_ratio: config.warning_coverage_ratio,
            critical_coverage_ratio: config.critical_coverage_ratio,
        }
    }
}
```

## Troubleshooting

### Issue: Engine not running

**Check:**
```rust
// Verify engine is started in app.rs
let reserve_health_engine = Arc::new(ReserveHealthEngine::new(db.clone()));
reserve_health_engine.clone().start(); // This line must be present
```

### Issue: Metrics not updating

**Solution:**
```bash
# Manually trigger sync
curl -X POST -H "Authorization: Bearer TOKEN" \
  http://localhost:8080/api/admin/reserve-health/sync
```

### Issue: Incorrect coverage ratios

**Check:**
1. Verify lending events are being captured
2. Check pool utilization is syncing correctly
3. Ensure bad_debt_reserve is being updated from contract

```sql
-- Verify pool data
SELECT asset_code, total_liquidity, utilized_liquidity, 
       bad_debt_reserve, coverage_ratio, reserve_health_status
FROM pools;

-- Check lending events
SELECT asset_code, event_type, amount, created_at
FROM lending_events
ORDER BY created_at DESC
LIMIT 10;
```

## Best Practices

1. **Monitor Logs**: Watch for health status changes
2. **Set Up Alerts**: Configure notifications for critical status
3. **Regular Syncs**: Run manual syncs after major lending activity
4. **Test Thresholds**: Adjust thresholds based on protocol risk tolerance
5. **Historical Tracking**: Store metrics over time for trend analysis
6. **Coordinate with Risk Engine**: Ensure both systems work together
7. **Document Changes**: Log all threshold adjustments

## Support

For issues or questions:
- Check logs: `tail -f logs/reserve_health.log`
- Review documentation: `backend/docs/RESERVE_HEALTH_TRACKING.md`
- Test endpoints: Use provided curl examples
- Verify database: Check pools table structure and data

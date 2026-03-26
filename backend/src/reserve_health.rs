use crate::api_error::ApiError;
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PoolReserveHealth {
    pub id: uuid::Uuid,
    pub asset_code: String,
    pub total_liquidity: Decimal,
    pub utilized_liquidity: Decimal,
    pub bad_debt_reserve: Decimal,
    pub retained_yield: Decimal,
    pub coverage_ratio: Option<Decimal>,
    pub reserve_health_status: Option<String>,
    pub last_health_check_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveHealthMetrics {
    pub asset_code: String,
    pub coverage_ratio: Decimal,
    pub utilization_rate: Decimal,
    pub reserve_adequacy: Decimal,
    pub health_status: String,
    pub bad_debt_reserve: Decimal,
    pub total_liquidity: Decimal,
    pub utilized_liquidity: Decimal,
    pub available_liquidity: Decimal,
}

pub struct ReserveHealthEngine {
    db: PgPool,
    min_coverage_ratio: Decimal,
    warning_coverage_ratio: Decimal,
    critical_coverage_ratio: Decimal,
}

impl ReserveHealthEngine {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            min_coverage_ratio: Decimal::new(10, 2), // 0.10 = 10%
            warning_coverage_ratio: Decimal::new(15, 2), // 0.15 = 15%
            critical_coverage_ratio: Decimal::new(5, 2), // 0.05 = 5%
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Check every 5 minutes
            loop {
                interval.tick().await;
                if let Err(e) = self.check_all_reserves().await {
                    error!("Reserve Health Engine error: {}", e);
                }
            }
        });
    }

    /// Check health of all pool reserves
    pub async fn check_all_reserves(&self) -> Result<Vec<ReserveHealthMetrics>, ApiError> {
        let pools = sqlx::query_as::<_, PoolReserveHealth>(
            r#"
            SELECT id, asset_code, total_liquidity, utilized_liquidity, 
                   bad_debt_reserve, retained_yield, coverage_ratio, 
                   reserve_health_status, last_health_check_at
            FROM pools
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error loading pools: {}", e)))?;

        let mut metrics_list = Vec::new();

        for pool in pools {
            let metrics = self.calculate_reserve_metrics(&pool).await?;
            self.update_pool_health(&pool.id, &metrics).await?;

            // Check for alerts
            self.check_and_alert(&pool, &metrics).await?;

            metrics_list.push(metrics);
        }

        Ok(metrics_list)
    }

    /// Calculate reserve health metrics for a pool
    async fn calculate_reserve_metrics(
        &self,
        pool: &PoolReserveHealth,
    ) -> Result<ReserveHealthMetrics, ApiError> {
        let total_liquidity = pool.total_liquidity;
        let utilized = pool.utilized_liquidity;
        let bad_debt_reserve = pool.bad_debt_reserve;

        // Calculate available liquidity
        let available_liquidity = total_liquidity.saturating_sub(utilized);

        // Calculate utilization rate
        let utilization_rate = if total_liquidity > Decimal::ZERO {
            (utilized / total_liquidity) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        // Calculate coverage ratio: bad_debt_reserve / utilized_liquidity
        let coverage_ratio = if utilized > Decimal::ZERO {
            bad_debt_reserve / utilized
        } else if bad_debt_reserve > Decimal::ZERO {
            Decimal::ONE // If no utilization but reserves exist, consider it healthy
        } else {
            Decimal::ZERO
        };

        // Calculate reserve adequacy: bad_debt_reserve / total_liquidity
        let reserve_adequacy = if total_liquidity > Decimal::ZERO {
            (bad_debt_reserve / total_liquidity) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        // Determine health status
        let health_status = self.determine_health_status(coverage_ratio, utilization_rate);

        Ok(ReserveHealthMetrics {
            asset_code: pool.asset_code.clone(),
            coverage_ratio,
            utilization_rate,
            reserve_adequacy,
            health_status,
            bad_debt_reserve,
            total_liquidity,
            utilized_liquidity: utilized,
            available_liquidity,
        })
    }

    /// Determine health status based on coverage ratio and utilization
    fn determine_health_status(
        &self,
        coverage_ratio: Decimal,
        utilization_rate: Decimal,
    ) -> String {
        if coverage_ratio < self.critical_coverage_ratio {
            "critical".to_string()
        } else if coverage_ratio < self.warning_coverage_ratio {
            "warning".to_string()
        } else if utilization_rate > Decimal::from(90) {
            "high_utilization".to_string()
        } else if coverage_ratio >= self.min_coverage_ratio {
            "healthy".to_string()
        } else {
            "moderate".to_string()
        }
    }

    /// Update pool health in database
    async fn update_pool_health(
        &self,
        pool_id: &uuid::Uuid,
        metrics: &ReserveHealthMetrics,
    ) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            UPDATE pools
            SET coverage_ratio = $1,
                reserve_health_status = $2,
                last_health_check_at = CURRENT_TIMESTAMP
            WHERE id = $3
            "#,
        )
        .bind(metrics.coverage_ratio)
        .bind(&metrics.health_status)
        .bind(pool_id)
        .execute(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error updating pool health: {}", e)))?;

        Ok(())
    }

    /// Check for alerts and notify admins
    async fn check_and_alert(
        &self,
        pool: &PoolReserveHealth,
        metrics: &ReserveHealthMetrics,
    ) -> Result<(), ApiError> {
        let previous_status = pool.reserve_health_status.as_deref().unwrap_or("healthy");
        let current_status = &metrics.health_status;

        // Alert on status degradation
        if previous_status != current_status {
            info!(
                "Pool {} ({}) health status changed: {} -> {}",
                pool.asset_code, pool.id, previous_status, current_status
            );

            // Get admin users to notify
            let admin_ids: Vec<uuid::Uuid> =
                sqlx::query_scalar("SELECT id FROM admins WHERE is_active = true LIMIT 10")
                    .fetch_all(&self.db)
                    .await
                    .unwrap_or_default();

            let message = match current_status.as_str() {
                "critical" => format!(
                    "CRITICAL: Pool {} reserve coverage ratio is critically low at {:.2}%. Immediate action required.",
                    pool.asset_code,
                    metrics.coverage_ratio * Decimal::from(100)
                ),
                "warning" => format!(
                    "WARNING: Pool {} reserve coverage ratio is below threshold at {:.2}%. Please review.",
                    pool.asset_code,
                    metrics.coverage_ratio * Decimal::from(100)
                ),
                "high_utilization" => format!(
                    "NOTICE: Pool {} has high utilization at {:.2}%. Monitor liquidity closely.",
                    pool.asset_code,
                    metrics.utilization_rate
                ),
                _ => return Ok(()),
            };

            let mut tx = self.db.begin().await?;

            for admin_id in admin_ids {
                NotificationService::create(
                    &mut tx,
                    admin_id,
                    notif_type::RESERVE_HEALTH_ALERT,
                    message.clone(),
                )
                .await?;
            }

            AuditLogService::log(
                &mut *tx,
                None,
                audit_action::RESERVE_HEALTH_ALERT,
                Some(pool.id),
                Some(entity_type::POOL),
            )
            .await?;

            tx.commit().await?;
        }

        // Always warn on critical thresholds
        if metrics.coverage_ratio < self.critical_coverage_ratio {
            warn!(
                "CRITICAL: Pool {} coverage ratio {:.4} is below critical threshold {:.4}",
                pool.asset_code, metrics.coverage_ratio, self.critical_coverage_ratio
            );
        }

        Ok(())
    }

    /// Get reserve health for a specific asset
    pub async fn get_reserve_health(
        &self,
        asset_code: &str,
    ) -> Result<ReserveHealthMetrics, ApiError> {
        let pool = sqlx::query_as::<_, PoolReserveHealth>(
            r#"
            SELECT id, asset_code, total_liquidity, utilized_liquidity, 
                   bad_debt_reserve, retained_yield, coverage_ratio, 
                   reserve_health_status, last_health_check_at
            FROM pools
            WHERE asset_code = $1
            "#,
        )
        .bind(asset_code)
        .fetch_one(&self.db)
        .await
        .map_err(|e| ApiError::NotFound(format!("Pool not found for asset {}: {}", asset_code, e)))?;

        self.calculate_reserve_metrics(&pool).await
    }

    /// Update pool reserves from lending contract events
    pub async fn sync_reserves_from_events(&self) -> Result<(), ApiError> {
        info!("Syncing pool reserves from lending events...");

        // Aggregate lending activity per asset
        let activity = sqlx::query_as::<_, (String, Decimal, Decimal)>(
            r#"
            SELECT 
                asset_code,
                SUM(CASE WHEN event_type = 'borrow' THEN CAST(amount AS numeric) ELSE 0 END) as total_borrowed,
                SUM(CASE WHEN event_type = 'repay' THEN CAST(amount AS numeric) ELSE 0 END) as total_repaid
            FROM lending_events
            WHERE asset_code IS NOT NULL
            GROUP BY asset_code
            "#
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error syncing reserves: {}", e)))?;

        for (asset_code, borrowed, repaid) in activity {
            let net_utilized = borrowed.saturating_sub(repaid);

            sqlx::query(
                r#"
                UPDATE pools
                SET utilized_liquidity = $1,
                    updated_at = CURRENT_TIMESTAMP
                WHERE asset_code = $2
                "#,
            )
            .bind(net_utilized)
            .bind(&asset_code)
            .execute(&self.db)
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error updating pool: {}", e)))?;
        }

        Ok(())
    }
}

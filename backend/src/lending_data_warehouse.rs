use crate::api_error::ApiError;
use crate::service::LendingMonitoringService;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

pub struct LendingDataWarehouseService {
    db: PgPool,
}

impl LendingDataWarehouseService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Spawn a background task that snapshots lending metrics every hour.
    pub fn start(self: Arc<Self>) {
        let interval_secs: u64 = std::env::var("LENDING_SNAPSHOT_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                match self.snapshot_current_metrics().await {
                    Ok(_) => info!("Lending metrics snapshot written"),
                    Err(e) => error!("Lending data warehouse snapshot failed: {}", e),
                }
            }
        });
    }

    /// Read the current lending metrics and persist a snapshot row.
    pub async fn snapshot_current_metrics(&self) -> Result<(), ApiError> {
        let m = LendingMonitoringService::get_lending_metrics(&self.db).await?;
        sqlx::query(
            r#"
            INSERT INTO lending_metrics_snapshots
                (tvl, total_borrowed, utilization_rate, active_loans_count)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(m.total_value_locked)
        .bind(m.total_borrowed)
        .bind(m.utilization_rate)
        .bind(m.active_loans_count)
        .execute(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Snapshot insert failed: {}", e)))?;

        Ok(())
    }

    /// Return aggregated historical lending performance, bucketed by `range`.
    ///
    /// `range` accepts "hourly", "daily", "weekly", or "monthly".
    pub async fn get_historical_metrics(
        &self,
        range: &str,
    ) -> Result<Vec<LendingPerformancePoint>, ApiError> {
        let trunc = match range {
            "hourly" => "hour",
            "weekly" => "week",
            "monthly" => "month",
            _ => "day",
        };

        #[derive(sqlx::FromRow)]
        struct Row {
            period: Option<DateTime<Utc>>,
            avg_tvl: Option<f64>,
            avg_borrowed: Option<f64>,
            avg_utilization: Option<f64>,
            avg_active_loans: Option<f64>,
            sample_count: Option<i64>,
        }

        let query = format!(
            r#"
            SELECT
                date_trunc('{trunc}', snapshot_at) AS period,
                AVG(tvl)                          AS avg_tvl,
                AVG(total_borrowed)               AS avg_borrowed,
                AVG(utilization_rate)             AS avg_utilization,
                AVG(active_loans_count::FLOAT8)   AS avg_active_loans,
                COUNT(*)                          AS sample_count
            FROM lending_metrics_snapshots
            GROUP BY 1
            ORDER BY 1 DESC
            LIMIT 365
            "#
        );

        let rows = sqlx::query_as::<_, Row>(&query)
            .fetch_all(&self.db)
            .await
            .map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("Historical metrics query failed: {}", e))
            })?;

        let points = rows
            .into_iter()
            .filter_map(|r| {
                Some(LendingPerformancePoint {
                    period: r.period?,
                    avg_tvl: r.avg_tvl.unwrap_or(0.0),
                    avg_borrowed: r.avg_borrowed.unwrap_or(0.0),
                    avg_utilization_rate: r.avg_utilization.unwrap_or(0.0),
                    avg_active_loans: r.avg_active_loans.unwrap_or(0.0),
                    sample_count: r.sample_count.unwrap_or(0),
                })
            })
            .collect();

        Ok(points)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LendingPerformancePoint {
    pub period: DateTime<Utc>,
    pub avg_tvl: f64,
    pub avg_borrowed: f64,
    pub avg_utilization_rate: f64,
    pub avg_active_loans: f64,
    pub sample_count: i64,
}

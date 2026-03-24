use crate::api_error::ApiError;
use crate::notifications::AuditLogService;
use crate::yield_service::OnChainYieldService;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct InterestReconciliationService {
    db: PgPool,
    yield_service: Arc<dyn OnChainYieldService>,
    discrepancy_threshold: Decimal,
}

impl InterestReconciliationService {
    pub fn new(
        db: PgPool,
        yield_service: Arc<dyn OnChainYieldService>,
        discrepancy_threshold: Decimal,
    ) -> Self {
        Self {
            db,
            yield_service,
            discrepancy_threshold,
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = self.reconcile_yields().await {
                    error!("Interest Reconciliation Engine error: {}", e);
                }
            }
        });
    }

    pub async fn reconcile_yields(&self) -> Result<(), ApiError> {
        #[derive(sqlx::FromRow)]
        struct AssetYieldRow {
            asset_code: String,
            expected_yield: rust_decimal::Decimal,
        }

        // Aggregate total interest_accrual per asset from lending_events
        let asset_yields = sqlx::query_as::<_, AssetYieldRow>(
            r#"
            SELECT asset_code, COALESCE(SUM(CAST(amount AS numeric)), 0) as expected_yield
            FROM lending_events
            WHERE event_type = 'interest_accrual'
            GROUP BY asset_code
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("DB error loading expected yields: {}", e))
        })?;

        for row in asset_yields {
            let on_chain_yield = match self
                .yield_service
                .get_total_on_chain_yield_amount(&row.asset_code)
                .await
            {
                Ok(y) => y,
                Err(e) => {
                    warn!(
                        "Failed to fetch on-chain yield for {}: {}",
                        row.asset_code, e
                    );
                    continue;
                }
            };

            let difference = (row.expected_yield - on_chain_yield).abs();
            if difference > self.discrepancy_threshold {
                warn!(
                    "YIELD DISCREPANCY DETECTED for {}: Expected {}, On-Chain {}, Difference {}",
                    row.asset_code, row.expected_yield, on_chain_yield, difference
                );

                let mut tx =
                    self.db.begin().await.map_err(|e| {
                        ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e))
                    })?;

                // Log discrepancy to audit logs
                AuditLogService::log(
                    &mut *tx,
                    None, // System action
                    "yield_discrepancy_detected",
                    None,
                    Some("system"),
                )
                .await?;

                tx.commit()
                    .await
                    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;
            } else {
                info!(
                    "Yield reconciled for {}. Expected {}, On-Chain {}",
                    row.asset_code, row.expected_yield, on_chain_yield
                );
            }
        }

        Ok(())
    }
}

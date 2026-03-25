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
                    error!("Interest Reconciliation Engine error (yields): {}", e);
                }
                if let Err(e) = self.reconcile_vault_balances().await {
                    error!("Interest Reconciliation Engine error (vaults): {}", e);
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

    pub async fn reconcile_vault_balances(&self) -> Result<(), ApiError> {
        #[derive(sqlx::FromRow)]
        struct AssetBalanceRow {
            asset_code: String,
            total_vault_balance: rust_decimal::Decimal,
        }

        // Aggregate net_amount per asset from plans
        // We assume 'USDC' for now if not specified, or we could join with a table that defines the asset.
        // The 'plans' table has 'net_amount'. Let's assume it's all one asset or we need to filter.
        // Actually, looking at plans table, it doesn't have asset_code?
        // Wait, check_all_loans in risk_engine.rs used p.asset_code.

        let vault_balances = sqlx::query_as::<_, AssetBalanceRow>(
            r#"
            SELECT asset_code, SUM(CAST(net_amount AS numeric)) as total_vault_balance
            FROM plans
            WHERE status NOT IN ('claimed', 'deactivated')
            GROUP BY asset_code
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("DB error loading vault balances: {}", e))
        })?;

        for row in vault_balances {
            let on_chain_balance = match self
                .yield_service
                .get_total_on_chain_balance(&row.asset_code)
                .await
            {
                Ok(b) => b,
                Err(e) => {
                    warn!(
                        "Failed to fetch on-chain balance for {}: {}",
                        row.asset_code, e
                    );
                    continue;
                }
            };

            let difference = (row.total_vault_balance - on_chain_balance).abs();
            if difference > self.discrepancy_threshold {
                warn!(
                    "VAULT BALANCE DISCREPANCY DETECTED for {}: Expected {}, On-Chain {}, Difference {}",
                    row.asset_code, row.total_vault_balance, on_chain_balance, difference
                );

                let mut tx =
                    self.db.begin().await.map_err(|e| {
                        ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e))
                    })?;

                AuditLogService::log(
                    &mut *tx,
                    None,
                    "vault_balance_discrepancy_detected",
                    None,
                    Some("system"),
                )
                .await?;

                tx.commit()
                    .await
                    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;
            } else {
                info!(
                    "Vault balance reconciled for {}. Expected {}, On-Chain {}",
                    row.asset_code, row.total_vault_balance, on_chain_balance
                );
            }
        }

        Ok(())
    }
}

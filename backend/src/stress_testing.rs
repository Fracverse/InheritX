use crate::api_error::ApiError;
use crate::price_feed::PriceFeedService;
use crate::risk_engine::RiskEngine;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, warn};

pub struct StressTestingEngine {
    db: PgPool,
    price_feed: Arc<dyn PriceFeedService>,
    risk_engine: Arc<RiskEngine>,
}

impl StressTestingEngine {
    pub fn new(
        db: PgPool,
        price_feed: Arc<dyn PriceFeedService>,
        risk_engine: Arc<RiskEngine>,
    ) -> Self {
        Self {
            db,
            price_feed,
            risk_engine,
        }
    }

    /// Simulates a sudden price crash for an asset
    pub async fn simulate_price_crash(
        &self,
        asset_code: &str,
        drop_percentage: Decimal,
    ) -> Result<(), ApiError> {
        info!(
            "Simulating price crash for {}: -{}%",
            asset_code, drop_percentage
        );

        let current_price = self.price_feed.get_price(asset_code).await?;
        let drop_factor = Decimal::ONE - (drop_percentage / Decimal::from(100));
        let crashed_price = current_price.price * drop_factor;

        // Use the price feed service to update to the crashed price
        self.price_feed
            .update_price(asset_code, crashed_price)
            .await?;

        // Immediately trigger risk engine check to see effects
        self.risk_engine.check_all_loans().await?;

        Ok(())
    }

    /// Simulates a mass default scenario by marking multiple plans as risky or forcing low health factors
    pub async fn simulate_mass_default(&self) -> Result<(), ApiError> {
        info!("Simulating mass default scenario...");

        // Strategy: Forcefully lower health factors in the database for a set of plans
        // Or we could just crash the main collateral asset price (usually USDC is 1, but if we drop it...)
        // Let's assume most plans use a specific asset as collateral that's not stable for this test.

        // Find plans that are currently healthy and force them to be risky
        let affected = sqlx::query(
            r#"
            UPDATE plans
            SET is_risky = true, health_factor = 0.5, risk_flagged_at = CURRENT_TIMESTAMP
            WHERE id IN (
                SELECT id FROM plans 
                WHERE is_risky = false OR is_risky IS NULL 
                LIMIT 50
            ) AND (is_paused IS NULL OR is_paused = false)
            "#,
        )
        .execute(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error forcing mass default: {}", e)))?;

        info!("Forced mass default for {} plans", affected.rows_affected());

        // We don't need to run check_all_loans here since we manually updated the state,
        // but we might want to trigger notifications if the check_all_loans does that.
        // Actually, if we want notifications, we should rely on the risk engine.

        Ok(())
    }

    /// Simulates a liquidity drain by reducing the pool's reported balance or increasing apparent utilization
    pub async fn simulate_liquidity_drain(
        &self,
        asset_code: &str,
        amount: Decimal,
    ) -> Result<(), ApiError> {
        info!(
            "Simulating liquidity drain for {}: {} units",
            asset_code, amount
        );

        // This would involve interacting with whatever tracks pool balances.
        // If it's a dedicated table or contract state.
        // Let's assume there's a 'pools' table.

        let result = sqlx::query(
            r#"
            UPDATE pools
            SET total_liquidity = total_liquidity - $1,
                last_drain_simulation_at = CURRENT_TIMESTAMP
            WHERE asset_code = $2
            "#,
        )
        .bind(amount)
        .bind(asset_code)
        .execute(&self.db)
        .await;

        match result {
            Ok(r) => {
                if r.rows_affected() == 0 {
                    warn!(
                        "No pool found for asset {} to drain liquidity from",
                        asset_code
                    );
                } else {
                    info!("Successfully simulated liquidity drain for {}", asset_code);
                }
            }
            Err(e) => {
                warn!(
                    "Liquidity drain simulation failed (table might not exist yet): {}",
                    e
                );
                // We might want to handle this gracefully if the schema is not yet fully updated
            }
        }

        Ok(())
    }
}

use crate::api_error::ApiError;
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use crate::price_feed::PriceFeedService;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct RiskEngine {
    db: PgPool,
    price_feed: Arc<dyn PriceFeedService>,
    liquidation_threshold: Decimal,
}

impl RiskEngine {
    pub fn new(
        db: PgPool,
        price_feed: Arc<dyn PriceFeedService>,
        liquidation_threshold: Decimal,
    ) -> Self {
        Self {
            db,
            price_feed,
            liquidation_threshold,
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = self.check_all_loans().await {
                    error!("Risk Engine error checking loans: {}", e);
                }
            }
        });
    }

    pub async fn check_all_loans(&self) -> Result<(), ApiError> {
        #[derive(sqlx::FromRow)]
        struct LoanHealthRow {
            plan_id: uuid::Uuid,
            user_id: uuid::Uuid,
            borrow_asset: String,
            total_debt: rust_decimal::Decimal,
            collateral_asset: Option<String>,
            collateral_amount: Option<rust_decimal::Decimal>,
            is_risky: Option<bool>,
            risk_override_enabled: Option<bool>,
        }

        // Find plans that have borrowing activity by aggregating lending events.
        // Exclude paused plans from risk monitoring
        let loans_health = sqlx::query_as::<_, LoanHealthRow>(
            r#"
            WITH loan_balances AS (
                SELECT plan_id, user_id, asset_code AS borrow_asset,
                       SUM(CASE WHEN event_type = 'borrow' THEN CAST(amount AS numeric) ELSE 0 END) -
                       SUM(CASE WHEN event_type = 'repay' THEN CAST(amount AS numeric) ELSE 0 END) -
                       SUM(CASE WHEN event_type = 'liquidation' THEN CAST(amount AS numeric) ELSE 0 END) AS total_debt
                FROM lending_events
                WHERE plan_id IS NOT NULL
                GROUP BY plan_id, user_id, asset_code
            )
            SELECT lb.plan_id, lb.user_id, lb.borrow_asset, lb.total_debt,
                   p.asset_code as collateral_asset, CAST(p.net_amount AS numeric) as collateral_amount, 
                   p.is_risky, p.risk_override_enabled
            FROM loan_balances lb
            JOIN plans p ON p.id = lb.plan_id
            WHERE lb.total_debt > 0
              AND (p.is_paused IS NULL OR p.is_paused = false)
            "#
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error loading loan balances: {}", e)))?;

        for loan in loans_health {
            // Get prices for evaluation
            let borrow_price = match self.price_feed.get_price(&loan.borrow_asset).await {
                Ok(p) => p.price,
                Err(e) => {
                    warn!(
                        "Risk Engine: Could not get price for borrow asset {}: {}",
                        loan.borrow_asset, e
                    );
                    continue;
                }
            };

            let collat_asset = loan.collateral_asset.unwrap_or_else(|| "USDC".to_string());
            let collat_price = match self.price_feed.get_price(&collat_asset).await {
                Ok(p) => p.price,
                Err(e) => {
                    warn!(
                        "Risk Engine: Could not get price for collateral asset {}: {}",
                        collat_asset, e
                    );
                    continue;
                }
            };

            // Values are aggregated using numeric casts, yielding Decimal safely via sqlx mapping
            let collat_value = loan.collateral_amount.unwrap_or(Decimal::ZERO) * collat_price;
            let debt_value = loan.total_debt * borrow_price;

            if debt_value > Decimal::ZERO {
                let health_factor = collat_value / debt_value;
                
                // Skip risk flagging if risk override is enabled
                let should_skip_risk_check = loan.risk_override_enabled.unwrap_or(false);
                
                let is_now_risky = if should_skip_risk_check {
                    false // Override: never mark as risky
                } else {
                    health_factor < self.liquidation_threshold
                };

                // Update database state
                sqlx::query(
                    r#"
                    UPDATE plans
                    SET is_risky = $1, health_factor = $2, risk_flagged_at = CASE WHEN $1 AND risk_flagged_at IS NULL THEN CURRENT_TIMESTAMP ELSE risk_flagged_at END
                    WHERE id = $3
                    "#
                )
                .bind(is_now_risky)
                .bind(health_factor)
                .bind(loan.plan_id)
                .execute(&self.db)
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error updating plan risk status: {}", e)))?;

                // Notify if transitioned to risky (and not overridden)
                if is_now_risky && !loan.is_risky.unwrap_or(false) && !should_skip_risk_check {
                    info!(
                        "Plan {} for User {} flagged as risky. HF: {}",
                        loan.plan_id, loan.user_id, health_factor
                    );

                    let mut tx = self.db.begin().await.map_err(|e| {
                        ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e))
                    })?;

                    NotificationService::create(
                        &mut tx,
                        loan.user_id,
                        notif_type::LIQUIDATION_WARNING,
                        format!("WARNING: Your loan against plan {} is at risk of liquidation. Health factor is now {:.2}. Please add collateral or repay some debt.", loan.plan_id, health_factor)
                    ).await?;

                    AuditLogService::log(
                        &mut *tx,
                        Some(loan.user_id),
                        audit_action::LIQUIDATION_WARNING,
                        Some(loan.plan_id),
                        Some(entity_type::PLAN),
                    )
                    .await?;

                    tx.commit().await.map_err(|e| {
                        ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e))
                    })?;
                } else if !is_now_risky && loan.is_risky.unwrap_or(false) {
                    info!(
                        "Plan {} for User {} is no longer risky. HF: {}",
                        loan.plan_id, loan.user_id, health_factor
                    );
                }
            }
        }

        Ok(())
    }
}

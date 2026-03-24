use crate::api_error::ApiError;
use crate::events::{EventService, LiquidationMetadata};
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

pub struct LiquidationBotService {
    db: PgPool,
    liquidation_penalty_rate: Decimal, // e.g., 0.05 for 5% penalty
}

impl LiquidationBotService {
    pub fn new(db: PgPool, liquidation_penalty_rate: Decimal) -> Self {
        Self {
            db,
            liquidation_penalty_rate,
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = self.process_liquidations().await {
                    error!("Liquidation Bot error: {}", e);
                }
            }
        });
    }

    pub async fn process_liquidations(&self) -> Result<(), ApiError> {
        #[derive(sqlx::FromRow)]
        struct RiskyLoanRow {
            plan_id: Uuid,
            user_id: Uuid,
            borrow_asset: String,
            total_debt: Decimal,
            collateral_asset: Option<String>,
            collateral_amount: Option<Decimal>,
        }

        // Find plans where is_risky = true and not yet liquidated
        let risky_loans = sqlx::query_as::<_, RiskyLoanRow>(
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
                   p.asset_code as collateral_asset, CAST(p.net_amount AS numeric) as collateral_amount
            FROM loan_balances lb
            JOIN plans p ON p.id = lb.plan_id
            WHERE p.is_risky = true AND p.status != 'liquidated' AND lb.total_debt > 0
            "#
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error loading risky loans: {}", e)))?;

        // System Liquidator Uuid
        let system_liquidator_id = Uuid::nil();

        for loan in risky_loans {
            let collat_asset = loan.collateral_asset.unwrap_or_else(|| "USDC".to_string());
            let collat_amount = loan.collateral_amount.unwrap_or(Decimal::ZERO);

            // Assume we liquidate the entire debt for simplicity.
            let debt_to_cover = loan.total_debt;
            // A realistic liquidation grabs debt + penalty from the collateral.
            let penalty_amount = debt_to_cover * self.liquidation_penalty_rate;
            // Since we assume simple 1:1 price for simplicity in this mock, collateral seized is debt_to_cover + penalty.
            // In a real system, we'd use prices to convert debt_to_cover to collateral token amounts.
            let mut collateral_seized = debt_to_cover + penalty_amount;
            if collateral_seized > collat_amount {
                collateral_seized = collat_amount; // Cap at max collateral available
            }

            info!(
                "Triggering auto-liquidation for Plan {}. Seizing {} {} to cover {} {} debt.",
                loan.plan_id, collateral_seized, collat_asset, debt_to_cover, loan.borrow_asset
            );

            let mut tx = self
                .db
                .begin()
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

            // 1. Emit Liquidation Event
            let metadata = LiquidationMetadata {
                liquidator_id: system_liquidator_id,
                collateral_asset: collat_asset.clone(),
                collateral_seized,
                debt_covered: debt_to_cover,
                liquidation_penalty: penalty_amount,
            };

            EventService::emit_liquidation(
                &mut tx,
                loan.user_id,
                Some(loan.plan_id),
                &loan.borrow_asset,
                debt_to_cover, // Emit the debt covered amount out of the borrow_asset
                metadata,
                None,
                None,
            )
            .await?;

            // 2. Mark Plan as Liquidated
            sqlx::query(
                r#"
                UPDATE plans
                SET status = 'liquidated'
                WHERE id = $1
                "#,
            )
            .bind(loan.plan_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("DB error updating plan status: {}", e))
            })?;

            // 3. Notify User
            #[allow(clippy::explicit_auto_deref)]
            NotificationService::create(
                &mut *tx,
                loan.user_id,
                notif_type::LIQUIDATION_WARNING, // Using LIQUIDATION_WARNING as a fallback if notif_type::LIQUIDATED doesn't exist
                format!(
                    "Your loan against plan {} has been liquidated. {} {} debt was covered by seizing {} {}.",
                    loan.plan_id, debt_to_cover, loan.borrow_asset, collateral_seized, collat_asset
                ),
            ).await?;

            // 4. Audit Log
            #[allow(clippy::explicit_auto_deref)]
            AuditLogService::log(
                &mut *tx,
                Some(loan.user_id),
                audit_action::LIQUIDATION_WARNING, // fallback
                Some(loan.plan_id),
                Some(entity_type::PLAN),
            )
            .await?;

            tx.commit()
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

            info!(
                "Successfully executed liquidation for Plan {}",
                loan.plan_id
            );
        }

        Ok(())
    }
}

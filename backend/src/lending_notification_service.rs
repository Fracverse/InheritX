use crate::api_error::ApiError;
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

pub struct LendingNotificationService {
    db: PgPool,
}

impl LendingNotificationService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = self.process_notifications().await {
                    error!("Lending Notification Service error: {}", e);
                }
            }
        });
    }

    pub async fn process_notifications(&self) -> Result<(), ApiError> {
        self.send_repayment_reminders().await?;
        self.send_yield_updates().await?;
        Ok(())
    }

    async fn send_repayment_reminders(&self) -> Result<(), ApiError> {
        #[derive(sqlx::FromRow)]
        struct LoanReminderRow {
            loan_id: Uuid,
            user_id: Uuid,
            borrow_asset: String,
            principal: Decimal,
            amount_repaid: Decimal,
            due_date: DateTime<Utc>,
        }

        let loans_due_soon = sqlx::query_as::<_, LoanReminderRow>(
            r#"
            SELECT ll.id AS loan_id, ll.user_id, ll.borrow_asset, ll.principal, ll.amount_repaid, ll.due_date
            FROM loan_lifecycle ll
            WHERE ll.status = 'active'
              AND ll.due_date > NOW()
              AND ll.due_date <= NOW() + INTERVAL '24 hours'
              AND NOT EXISTS (
                    SELECT 1
                    FROM action_logs al
                    WHERE al.action = $1
                      AND al.entity_type = $2
                      AND al.entity_id = ll.id
                      AND al.timestamp > NOW() - INTERVAL '24 hours'
              )
            "#,
        )
        .bind(audit_action::REPAYMENT_REMINDER_SENT)
        .bind(entity_type::LOAN)
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error loading due loans: {}", e)))?;

        for loan in loans_due_soon {
            let outstanding = (loan.principal - loan.amount_repaid).max(Decimal::ZERO);

            let mut tx = self
                .db
                .begin()
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

            NotificationService::create(
                &mut tx,
                loan.user_id,
                notif_type::REPAYMENT_REMINDER,
                format!(
                    "Repayment reminder: Loan {} is due at {}. Outstanding balance: {} {}.",
                    loan.loan_id, loan.due_date, outstanding, loan.borrow_asset
                ),
            )
            .await?;

            AuditLogService::log(
                &mut *tx,
                Some(loan.user_id),
                None,
                audit_action::REPAYMENT_REMINDER_SENT,
                Some(loan.loan_id),
                Some(entity_type::LOAN),
                None,
                None,
                None,
            )
            .await?;

            tx.commit()
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

            info!(
                "Sent repayment reminder for loan {} to user {}",
                loan.loan_id, loan.user_id
            );
        }

        Ok(())
    }

    async fn send_yield_updates(&self) -> Result<(), ApiError> {
        #[derive(sqlx::FromRow)]
        struct UserYieldRow {
            user_id: Uuid,
            accrued_yield: Decimal,
        }

        let user_yields = sqlx::query_as::<_, UserYieldRow>(
            r#"
            SELECT le.user_id, COALESCE(SUM(CAST(le.amount AS numeric)), 0) AS accrued_yield
            FROM lending_events le
            WHERE le.event_type = 'interest_accrual'
              AND le.event_timestamp >= NOW() - INTERVAL '24 hours'
            GROUP BY le.user_id
            HAVING COALESCE(SUM(CAST(le.amount AS numeric)), 0) > 0
               AND NOT EXISTS (
                    SELECT 1
                    FROM action_logs al
                    WHERE al.action = $1
                      AND al.entity_type = $2
                      AND al.entity_id = le.user_id
                      AND al.timestamp > NOW() - INTERVAL '24 hours'
               )
            "#,
        )
        .bind(audit_action::YIELD_UPDATE_SENT)
        .bind(entity_type::USER)
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error loading user yields: {}", e)))?;

        for row in user_yields {
            let mut tx = self
                .db
                .begin()
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

            NotificationService::create(
                &mut tx,
                row.user_id,
                notif_type::YIELD_UPDATE,
                format!(
                    "Yield update: You accrued {} in lending yield over the past 24 hours.",
                    row.accrued_yield
                ),
            )
            .await?;

            AuditLogService::log(
                &mut *tx,
                Some(row.user_id),
                None,
                audit_action::YIELD_UPDATE_SENT,
                Some(row.user_id),
                Some(entity_type::USER),
                None,
                None,
                None,
            )
            .await?;

            tx.commit()
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

            info!("Sent yield update to user {}", row.user_id);
        }

        Ok(())
    }
}

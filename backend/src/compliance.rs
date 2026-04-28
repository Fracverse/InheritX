use crate::api_error::ApiError;
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct ComplianceEngine {
    db: PgPool,
    pub velocity_threshold: usize, // e.g., 3 events
    velocity_window_mins: i64,     // e.g., 10 minutes
    pub volume_threshold: Decimal, // e.g., $100k
}

impl ComplianceEngine {
    pub fn new(
        db: PgPool,
        velocity_threshold: usize,
        velocity_window_mins: i64,
        volume_threshold: Decimal,
    ) -> Self {
        Self {
            db,
            velocity_threshold,
            velocity_window_mins,
            volume_threshold,
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
            loop {
                interval.tick().await;
                if let Err(e) = self.scan_suspicious_activity().await {
                    error!("Compliance Engine error: {}", e);
                }
            }
        });
    }

    pub async fn scan_suspicious_activity(&self) -> Result<(), ApiError> {
        info!("Compliance Engine: Scanning for suspicious borrowing patterns...");

        // 1. Detect High Velocity Borrowing
        self.detect_high_velocity().await?;

        // 2. Detect Abnormal Volume
        self.detect_abnormal_volume().await?;

        // 3. Detect Sudden Activity from Inactive Users
        self.detect_sudden_activity_spike().await?;

        Ok(())
    }

    async fn detect_high_velocity(&self) -> Result<(), ApiError> {
        #[derive(sqlx::FromRow)]
        struct VelocityMatch {
            plan_id: Uuid,
            user_id: Uuid,
            event_count: i64,
        }

        let velocity_matches = sqlx::query_as::<_, VelocityMatch>(
            r#"
            SELECT plan_id, user_id, COUNT(*) as event_count
            FROM lending_events
            WHERE event_type IN ('borrow', 'repay')
              AND event_timestamp > NOW() - (INTERVAL '1 minute' * $1)
            GROUP BY plan_id, user_id
            HAVING COUNT(*) >= $2
            "#,
        )
        .bind(self.velocity_window_mins)
        .bind(self.velocity_threshold as i64)
        .fetch_all(&self.db)
        .await?;

        for m in velocity_matches {
            self.flag_plan(
                m.plan_id,
                m.user_id,
                format!(
                    "High velocity detected: {} borrowing events in last {} minutes",
                    m.event_count, self.velocity_window_mins
                ),
            )
            .await?;
        }

        Ok(())
    }

    async fn detect_abnormal_volume(&self) -> Result<(), ApiError> {
        #[derive(sqlx::FromRow)]
        struct VolumeMatch {
            plan_id: Uuid,
            user_id: Uuid,
            asset_code: String,
            amount: rust_decimal::Decimal,
        }

        let volume_matches = sqlx::query_as::<_, VolumeMatch>(
            r#"
            SELECT plan_id, user_id, asset_code, CAST(amount AS numeric) as amount
            FROM lending_events
            WHERE event_type = 'borrow'
              AND CAST(amount AS numeric) >= $1
              AND event_timestamp > NOW() - INTERVAL '5 minutes'
            "#,
        )
        .bind(self.volume_threshold)
        .fetch_all(&self.db)
        .await?;

        for m in volume_matches {
            self.flag_plan(
                m.plan_id,
                m.user_id,
                format!(
                    "Abnormal volume detected: Borrowed {} {} (Threshold: {})",
                    m.amount, m.asset_code, self.volume_threshold
                ),
            )
            .await?;
        }

        Ok(())
    }

    async fn detect_sudden_activity_spike(&self) -> Result<(), ApiError> {
        #[derive(sqlx::FromRow)]
        struct SpikeMatch {
            plan_id: Uuid,
            user_id: Uuid,
        }

        // Flag if a user with no activity for 30 days suddenly borrows
        let spike_matches = sqlx::query_as::<_, SpikeMatch>(
            r#"
            SELECT le.plan_id, le.user_id
            FROM lending_events le
            JOIN plans p ON p.id = le.plan_id
            WHERE le.event_type = 'borrow'
              AND le.event_timestamp > NOW() - INTERVAL '5 minutes'
              AND NOT EXISTS (
                  SELECT 1 FROM lending_events prev
                  WHERE prev.user_id = le.user_id
                    AND prev.event_timestamp < le.event_timestamp
                    AND prev.event_timestamp > le.event_timestamp - INTERVAL '30 days'
              )
              AND p.created_at < NOW() - INTERVAL '30 days' -- Ensure it's an old account that was dormant
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        for m in spike_matches {
            self.flag_plan(
                m.plan_id,
                m.user_id,
                "Sudden activity spike: Borrowing after 30+ days of dormancy".to_string(),
            )
            .await?;
        }

        Ok(())
    }

    async fn flag_plan(
        &self,
        plan_id: Uuid,
        user_id: Uuid,
        reason: String,
    ) -> Result<(), ApiError> {
        // Check if already flagged for this reason to avoid spam
        let current_flags: Option<String> =
            sqlx::query_scalar("SELECT suspicion_flags FROM plans WHERE id = $1")
                .bind(plan_id)
                .fetch_one(&self.db)
                .await?;

        if let Some(flags) = current_flags {
            if flags.contains(&reason) {
                return Ok(());
            }
        }

        warn!(
            "Compliance Engine: Flagging Plan {} due to: {}",
            plan_id, reason
        );

        let mut tx = self.db.begin().await?;

        // 1. Update plan status
        sqlx::query(
            r#"
            UPDATE plans
            SET is_flagged = true, 
                suspicion_flags = COALESCE(suspicion_flags || ' | ', '') || $1
            WHERE id = $2
            "#,
        )
        .bind(&reason)
        .bind(plan_id)
        .execute(&mut *tx)
        .await?;

        // 2. Audit Log
        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            None,
            audit_action::SUSPICIOUS_BORROWING_DETECTED,
            Some(plan_id),
            Some(entity_type::PLAN),
            None,
            None,
            None,
        )
        .await?;

        // 3. Notification
        NotificationService::create(
            &mut tx,
            user_id,
            notif_type::SUSPICIOUS_ACTIVITY_FLAGGED,
            format!("ALARM: Your account has been flagged for abnormal activity: {reason}. A compliance officer has been notified.")
        ).await?;

        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_compliance_engine_new() {
        let db = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let engine = ComplianceEngine::new(db, 5, 15, dec!(50000));
        assert_eq!(engine.velocity_threshold, 5);
        assert_eq!(engine.velocity_window_mins, 15);
        assert_eq!(engine.volume_threshold, dec!(50000));
    }

    // Additional integration tests would go here
    // Test velocity detection logic
    // Test volume threshold detection
    // Test sanctions screening integration
    // Test risk scoring algorithms
    // Add compliance violation scenarios
}

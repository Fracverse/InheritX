use crate::api_error::ApiError;
use crate::events::{EventType, LendingEvent};
use crate::external_integrations::{
    AnchorIntegrationClient, ComplianceApiClient, SanctionsApiClient,
};
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use once_cell::sync::OnceCell;
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use uuid::Uuid;

const ALERT_COOLDOWN_WINDOW: Duration = Duration::from_secs(300);
const EVENT_COMMIT_POLL_INTERVAL: Duration = Duration::from_millis(200);
const EVENT_COMMIT_MAX_RETRIES: usize = 5;

static REALTIME_COMPLIANCE_LISTENER: OnceCell<Arc<dyn RealtimeComplianceListener>> =
    OnceCell::new();

#[async_trait]
pub trait RealtimeComplianceListener: Send + Sync {
    async fn on_event(&self, event: LendingEvent) -> Result<(), ApiError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AlertDedupKey {
    entity_id: Uuid,
    rule_id: String,
    severity: String,
}

#[derive(Debug, Clone)]
struct ComplianceAlert {
    entity_id: Uuid,
    user_id: Uuid,
    plan_id: Option<Uuid>,
    rule_id: String,
    severity: String,
    timestamp: DateTime<Utc>,
    context: serde_json::Value,
}

pub struct ComplianceEngine {
    db: PgPool,
    pub velocity_threshold: usize, // e.g., 3 events
    velocity_window_mins: i64,     // e.g., 10 minutes
    pub volume_threshold: Decimal, // e.g., $100k
    compliance_api_client: Option<ComplianceApiClient>,
    sanctions_client: Option<SanctionsApiClient>,
    anchor_client: Option<AnchorIntegrationClient>,
    alert_history: Mutex<HashMap<AlertDedupKey, DateTime<Utc>>>,
}

pub fn register_realtime_compliance_listener(
    listener: Arc<dyn RealtimeComplianceListener>,
) -> Result<(), &'static str> {
    REALTIME_COMPLIANCE_LISTENER
        .set(listener)
        .map_err(|_| "real-time compliance listener already registered")
}

fn get_realtime_compliance_listener() -> Option<Arc<dyn RealtimeComplianceListener>> {
    REALTIME_COMPLIANCE_LISTENER.get().cloned()
}

pub fn dispatch_realtime_event(event: LendingEvent) {
    if let Some(listener) = get_realtime_compliance_listener() {
        dispatch_realtime_event_with_listener(listener, event);
    }
}

pub fn dispatch_realtime_event_with_listener(
    listener: Arc<dyn RealtimeComplianceListener>,
    event: LendingEvent,
) {
    let event = event.clone();
    tokio::spawn(async move {
        if let Err(err) = listener.on_event(event).await {
            warn!(error = %err, "Real-time compliance listener failed");
        }
    });
}

impl ComplianceEngine {
    const SUDDEN_SPIKE_DORMANCY_DAYS: i64 = 30;
    const SUDDEN_SPIKE_LOOKBACK_MINUTES: i64 = 5;

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
            compliance_api_client: ComplianceApiClient::from_env(),
            sanctions_client: SanctionsApiClient::from_env(),
            anchor_client: AnchorIntegrationClient::from_env(),
            alert_history: Mutex::new(HashMap::new()),
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
            loop {
                interval.tick().await;
                if let Err(e) = self.scan_suspicious_activity().await {
                    error!("Compliance Engine error: {}", e);
                    crate::error_tracking::capture_message(
                        &format!("ComplianceEngine::scan_suspicious_activity failed: {e}"),
                        sentry::Level::Error,
                    );
                }
            }
        });
    }

    pub fn queue_realtime_check(self: Arc<Self>, event: LendingEvent) {
        tokio::spawn(async move {
            if let Err(e) = self.run_realtime_checks_for_event(event).await {
                warn!(error = %e, "Real-time compliance check failed");
            }
        });
    }

    async fn run_realtime_checks_for_event(&self, event: LendingEvent) -> Result<(), ApiError> {
        if !self.wait_for_event_commit(event.id).await? {
            warn!(event_id = %event.id, "Real-time compliance event was not committed; skipping check");
            return Ok(());
        }

        match event.event_type {
            EventType::Borrow => {
                self.check_abnormal_volume_for_event(&event).await?;
                self.check_sudden_activity_spike_for_event(&event).await?;
                self.check_high_velocity_for_event(&event).await?;
            }
            EventType::Repay => {
                self.check_high_velocity_for_event(&event).await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn wait_for_event_commit(&self, event_id: Uuid) -> Result<bool, ApiError> {
        for _ in 0..EVENT_COMMIT_MAX_RETRIES {
            let exists: Option<bool> =
                sqlx::query_scalar("SELECT true FROM lending_events WHERE id = $1")
                    .bind(event_id)
                    .fetch_optional(&self.db)
                    .await?;

            if exists.is_some() {
                return Ok(true);
            }

            tokio::time::sleep(EVENT_COMMIT_POLL_INTERVAL).await;
        }

        Ok(false)
    }

    async fn check_high_velocity_for_event(&self, event: &LendingEvent) -> Result<(), ApiError> {
        let plan_id = match event.plan_id {
            Some(plan_id) => plan_id,
            None => return Ok(()),
        };

        let event_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM lending_events
            WHERE plan_id = $1
              AND user_id = $2
              AND event_type IN ('borrow', 'repay')
              AND event_timestamp > NOW() - (INTERVAL '1 minute' * $3)
            "#,
        )
        .bind(plan_id)
        .bind(event.user_id)
        .bind(self.velocity_window_mins)
        .fetch_one(&self.db)
        .await?;

        let event_count = event_count + 1;
        if event_count >= self.velocity_threshold as i64 {
            let reason = format!(
                "High velocity detected: {} borrowing events in last {} minutes",
                event_count, self.velocity_window_mins
            );

            self.try_emit_violation_alert(
                plan_id,
                event.user_id,
                "high_velocity",
                "high",
                json!({
                    "event_count": event_count,
                    "window_minutes": self.velocity_window_mins,
                    "event_type": event.event_type,
                }),
            )
            .await?;

            self.flag_plan(plan_id, event.user_id, reason).await?;
        }

        Ok(())
    }

    async fn check_abnormal_volume_for_event(&self, event: &LendingEvent) -> Result<(), ApiError> {
        let plan_id = match event.plan_id {
            Some(plan_id) => plan_id,
            None => return Ok(()),
        };

        if event.amount >= self.volume_threshold {
            let reason = format!(
                "Abnormal volume detected: Borrowed {} {} (Threshold: {})",
                event.amount, event.asset_code, self.volume_threshold
            );

            self.try_emit_violation_alert(
                plan_id,
                event.user_id,
                "abnormal_volume",
                "high",
                json!({
                    "amount": event.amount,
                    "asset_code": event.asset_code,
                    "threshold": self.volume_threshold,
                }),
            )
            .await?;

            self.flag_plan(plan_id, event.user_id, reason).await?;
        }

        Ok(())
    }

    async fn check_sudden_activity_spike_for_event(
        &self,
        event: &LendingEvent,
    ) -> Result<(), ApiError> {
        let plan_id = match event.plan_id {
            Some(plan_id) => plan_id,
            None => return Ok(()),
        };

        let plan_created_at: Option<DateTime<Utc>> =
            sqlx::query_scalar("SELECT created_at FROM plans WHERE id = $1")
                .bind(plan_id)
                .fetch_optional(&self.db)
                .await?;

        let plan_created_at = match plan_created_at {
            Some(created_at) => created_at,
            None => return Ok(()),
        };

        if plan_created_at > Utc::now() - ChronoDuration::days(30) {
            return Ok(());
        }

        let prior_activity: Option<bool> = sqlx::query_scalar(
            r#"
            SELECT true
            FROM lending_events
            WHERE user_id = $1
              AND event_type = 'borrow'
              AND event_timestamp < $2
              AND event_timestamp > NOW() - INTERVAL '30 days'
            LIMIT 1
            "#,
        )
        .bind(event.user_id)
        .bind(event.event_timestamp)
        .fetch_optional(&self.db)
        .await?;

        if prior_activity.is_none() {
            let reason = "Sudden activity spike: Borrowing after 30+ days of dormancy".to_string();

            self.try_emit_violation_alert(
                plan_id,
                event.user_id,
                "sudden_activity_spike",
                "medium",
                json!({
                    "plan_created_at": plan_created_at,
                    "event_timestamp": event.event_timestamp,
                }),
            )
            .await?;

            self.flag_plan(plan_id, event.user_id, reason).await?;
        }

        Ok(())
    }

    async fn try_emit_violation_alert(
        &self,
        entity_id: Uuid,
        user_id: Uuid,
        rule_id: &str,
        severity: &str,
        context: serde_json::Value,
    ) -> Result<bool, ApiError> {
        let now = Utc::now();
        let key = AlertDedupKey {
            entity_id,
            rule_id: rule_id.to_string(),
            severity: severity.to_string(),
        };

        let mut alert_history = self.alert_history.lock().await;
        let cutoff = now - ChronoDuration::from_std(ALERT_COOLDOWN_WINDOW).unwrap();
        alert_history.retain(|_, last_seen| *last_seen > cutoff);

        if alert_history.contains_key(&key) {
            warn!(
                entity_id = %entity_id,
                rule_id = %rule_id,
                severity = %severity,
                "Duplicate compliance alert suppressed"
            );
            return Ok(false);
        }

        alert_history.insert(key, now);

        let alert = ComplianceAlert {
            entity_id,
            user_id,
            plan_id: Some(entity_id),
            rule_id: rule_id.to_string(),
            severity: severity.to_string(),
            timestamp: now,
            context,
        };

        self.emit_structured_alert(alert).await;
        Ok(true)
    }

    async fn emit_structured_alert(&self, alert: ComplianceAlert) {
        warn!(
            target = "compliance_alerts",
            entity_id = %alert.entity_id,
            plan_id = ?alert.plan_id,
            user_id = %alert.user_id,
            rule = %alert.rule_id,
            severity = %alert.severity,
            timestamp = %alert.timestamp.to_rfc3339(),
            context = %alert.context,
            "Compliance alert emitted"
        );
    }

    pub async fn scan_suspicious_activity(&self) -> Result<(), ApiError> {
        info!("Compliance Engine: Scanning for suspicious borrowing patterns and sanctions screening...");

        if let Err(e) = self.run_sanctions_screening().await {
            warn!(error = %e, "Sanctions screening failed; continuing with internal compliance checks");
        }

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

        // 1. Single-event detection: any individual borrow >= threshold
        let single_matches = sqlx::query_as::<_, VolumeMatch>(
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

        for m in single_matches {
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

        // 2. Cumulative volume detection: net borrows (borrows - repays) per user >= threshold
        //    Catches split transactions designed to evade single-event detection.
        #[derive(sqlx::FromRow)]
        struct CumulativeMatch {
            plan_id: Uuid,
            user_id: Uuid,
            asset_code: String,
            net_volume: rust_decimal::Decimal,
        }

        let cumulative_matches = sqlx::query_as::<_, CumulativeMatch>(
            r#"
            SELECT
                plan_id,
                user_id,
                asset_code,
                SUM(CASE WHEN event_type = 'borrow' THEN CAST(amount AS numeric)
                         WHEN event_type = 'repay'  THEN -CAST(amount AS numeric)
                         ELSE 0 END) AS net_volume
            FROM lending_events
            WHERE event_type IN ('borrow', 'repay')
              AND event_timestamp > NOW() - (INTERVAL '1 minute' * $2)
            GROUP BY plan_id, user_id, asset_code
            HAVING SUM(CASE WHEN event_type = 'borrow' THEN CAST(amount AS numeric)
                            WHEN event_type = 'repay'  THEN -CAST(amount AS numeric)
                            ELSE 0 END) >= $1
            "#,
        )
        .bind(self.volume_threshold)
        .bind(self.velocity_window_mins)
        .fetch_all(&self.db)
        .await?;

        for m in cumulative_matches {
            self.flag_plan(
                m.plan_id,
                m.user_id,
                format!(
                    "Abnormal cumulative volume detected: Net {} {} in {} minutes (Threshold: {})",
                    m.net_volume, m.asset_code, self.velocity_window_mins, self.volume_threshold
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

        // Flag if an old account has been dormant for 30 days and then suddenly borrows.
        //
        // We use the account creation time as the age check because the database does not
        // store a dedicated "last activity" timestamp for users. The lending_events table
        // provides the activity history needed to detect dormancy.
        let spike_matches = sqlx::query_as::<_, SpikeMatch>(
            r#"
            SELECT DISTINCT le.plan_id, le.user_id
            FROM lending_events le
            JOIN plans p ON p.id = le.plan_id
            JOIN users u ON u.id = le.user_id
            WHERE le.event_type = 'borrow'
              AND le.event_timestamp > NOW() - INTERVAL '1 minute' * $1
              AND NOT EXISTS (
                  SELECT 1 FROM lending_events prev
                  WHERE prev.user_id = le.user_id
                    AND prev.event_timestamp < le.event_timestamp
                    AND prev.event_timestamp > le.event_timestamp - INTERVAL '1 day' * $2
              )
              AND p.created_at < NOW() - INTERVAL '1 day' * $2
              AND u.created_at < NOW() - INTERVAL '1 day' * $2
            "#,
        )
        .bind(Self::SUDDEN_SPIKE_LOOKBACK_MINUTES)
        .bind(Self::SUDDEN_SPIKE_DORMANCY_DAYS)
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

    async fn run_sanctions_screening(&self) -> Result<(), ApiError> {
        let client = match &self.sanctions_client {
            Some(client) => client,
            None => return Ok(()),
        };

        #[derive(sqlx::FromRow)]
        struct SanctionsCandidate {
            plan_id: Uuid,
            user_id: Uuid,
            email: String,
            wallet_address: Option<String>,
        }

        let candidates = sqlx::query_as::<_, SanctionsCandidate>(
            r#"
            SELECT p.id as plan_id, u.id as user_id, u.email, u.wallet_address
            FROM plans p
            JOIN users u ON u.id = p.user_id
            WHERE NOT p.is_flagged
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        for candidate in candidates {
            if candidate.email.is_empty() && candidate.wallet_address.is_none() {
                continue;
            }

            if let Ok(Some(match_reason)) = client
                .screen_user(
                    candidate.user_id,
                    &candidate.email,
                    candidate.wallet_address.as_deref(),
                )
                .await
            {
                self.flag_plan(
                    candidate.plan_id,
                    candidate.user_id,
                    format!("Sanctions screening hit: {}", match_reason),
                )
                .await?;
            }
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

        // External compliance integrations should not block core processing.
        if let Some(client) = &self.compliance_api_client {
            if let Err(e) = client
                .report_suspicious_activity(plan_id, user_id, &reason)
                .await
            {
                warn!(
                    plan_id = %plan_id,
                    user_id = %user_id,
                    error = %e,
                    "Compliance API notification failed"
                );
            }
        }

        if let Some(client) = &self.anchor_client {
            if let Err(e) = client
                .submit_compliance_flag(plan_id, user_id, &reason)
                .await
            {
                warn!(
                    plan_id = %plan_id,
                    user_id = %user_id,
                    error = %e,
                    "Anchor integration notification failed"
                );
            }
        }

        Ok(())
    }
}

#[async_trait]
impl RealtimeComplianceListener for ComplianceEngine {
    async fn on_event(&self, event: LendingEvent) -> Result<(), ApiError> {
        self.run_realtime_checks_for_event(event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use serde_json::json;
    use sqlx::PgPool;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::{oneshot, Mutex};

    #[tokio::test]
    async fn test_compliance_engine_new() {
        let db = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let engine = ComplianceEngine::new(db, 5, 15, dec!(50000));
        assert_eq!(engine.velocity_threshold, 5);
        assert_eq!(engine.velocity_window_mins, 15);
        assert_eq!(engine.volume_threshold, dec!(50000));
    }

    #[tokio::test]
    async fn test_realtime_event_dispatch_does_not_block() {
        struct TestListener {
            handle: Mutex<Option<oneshot::Sender<Uuid>>>,
        }

        #[async_trait]
        impl RealtimeComplianceListener for TestListener {
            async fn on_event(&self, event: LendingEvent) -> Result<(), ApiError> {
                if let Some(tx) = self.handle.lock().await.take() {
                    let _ = tx.send(event.id);
                }
                Ok(())
            }
        }

        let (tx, rx) = oneshot::channel();
        let listener = Arc::new(TestListener {
            handle: Mutex::new(Some(tx)),
        });

        let event = LendingEvent {
            id: Uuid::new_v4(),
            event_type: EventType::Borrow,
            user_id: Uuid::new_v4(),
            plan_id: Some(Uuid::new_v4()),
            asset_code: "USDC".to_string(),
            amount: dec!(1000),
            metadata: json!({}),
            transaction_hash: None,
            block_number: None,
            event_timestamp: Utc::now(),
            created_at: Utc::now(),
        };

        dispatch_realtime_event_with_listener(listener, event.clone());
        let received = tokio::time::timeout(Duration::from_secs(2), rx)
            .await
            .expect("listener should be invoked")
            .expect("listener send succeeded");

        assert_eq!(received, event.id);
    }

    #[tokio::test]
    async fn test_duplicate_alerts_are_suppressed_within_cooldown() {
        let db = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let engine = ComplianceEngine::new(db, 3, 10, dec!(100000));
        let plan_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let first = engine
            .try_emit_violation_alert(
                plan_id,
                user_id,
                "high_velocity",
                "high",
                json!({ "event_count": 5 }),
            )
            .await
            .unwrap();
        assert!(first);

        let second = engine
            .try_emit_violation_alert(
                plan_id,
                user_id,
                "high_velocity",
                "high",
                json!({ "event_count": 5 }),
            )
            .await
            .unwrap();
        assert!(!second);
    }

    #[tokio::test]
    async fn test_realtime_check_error_does_not_block_caller() {
        struct FailingListener {
            handle: Mutex<Option<oneshot::Sender<Uuid>>>,
        }

        #[async_trait]
        impl RealtimeComplianceListener for FailingListener {
            async fn on_event(&self, event: LendingEvent) -> Result<(), ApiError> {
                if let Some(tx) = self.handle.lock().await.take() {
                    let _ = tx.send(event.id);
                }
                Err(ApiError::Internal(anyhow::anyhow!("handler failure")))
            }
        }

        let (tx, rx) = oneshot::channel();
        let listener = Arc::new(FailingListener {
            handle: Mutex::new(Some(tx)),
        });

        let event = LendingEvent {
            id: Uuid::new_v4(),
            event_type: EventType::Borrow,
            user_id: Uuid::new_v4(),
            plan_id: Some(Uuid::new_v4()),
            asset_code: "USDC".to_string(),
            amount: dec!(1000),
            metadata: json!({}),
            transaction_hash: None,
            block_number: None,
            event_timestamp: Utc::now(),
            created_at: Utc::now(),
        };

        dispatch_realtime_event_with_listener(listener, event.clone());
        let received = tokio::time::timeout(Duration::from_secs(2), rx)
            .await
            .expect("listener should be invoked")
            .expect("listener send succeeded");

        assert_eq!(received, event.id);
    }

    // Additional integration tests would go here
    // Test velocity detection logic
    // Test volume threshold detection
    // Test sanctions screening integration
    // Test risk scoring algorithms
    // Add compliance violation scenarios
}

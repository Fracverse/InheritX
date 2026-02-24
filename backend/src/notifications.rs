use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ─── Notification Types ──────────────────────────────────────────────────────

/// Well-known notification type values stored in the `type` column.
pub mod notif_type {
    pub const KYC_APPROVED: &str = "kyc_approved";
    pub const KYC_REJECTED: &str = "kyc_rejected";
    pub const PLAN_CREATED: &str = "plan_created";
    pub const PLAN_CLAIMED: &str = "plan_claimed";
    pub const PLAN_DEACTIVATED: &str = "plan_deactivated";
    pub const TWO_FA_SENT: &str = "2fa_sent";
}

// ─── Notification ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub notif_type: String,
    pub message: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

pub struct NotificationService;

impl NotificationService {
    /// Insert a notification for a user.
    /// Now participates in the caller's transaction.
    pub async fn create(
        executor: &mut sqlx::PgConnection, // Changed from &PgPool
        user_id: Uuid,
        notif_type: &str,
        message: impl Into<String>,
    ) -> Result<Notification, ApiError> {
        let message = message.into();
        let row = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (user_id, type, message, is_read)
            VALUES ($1, $2, $3, false)
            RETURNING id, user_id, type, message, is_read, created_at
            "#,
        )
        .bind(user_id)
        .bind(notif_type)
        .bind(&message)
        .fetch_one(executor) // Use the passed connection/transaction
        .await?;

        Ok(row)
    }

    // REMOVED: create_silent
    // Because atomic safety requires that if a notification fails,
    // the parent transaction MUST rollback.

    /// Return all notifications for a user (Read-only, can stay using &PgPool)
    pub async fn list_for_user(db: &PgPool, user_id: Uuid) -> Result<Vec<Notification>, ApiError> {
        let rows = sqlx::query_as::<_, Notification>(
            r#"
            SELECT id, user_id, type, message, is_read, created_at
            FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(db)
        .await?;

        Ok(rows)
    }

    /// Mark a single notification as read.
    pub async fn mark_read(
        db: &PgPool,
        notif_id: Uuid,
        user_id: Uuid,
    ) -> Result<Notification, ApiError> {
        let row = sqlx::query_as::<_, Notification>(
            r#"
            UPDATE notifications
            SET is_read = true
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, type, message, is_read, created_at
            "#,
        )
        .bind(notif_id)
        .bind(user_id)
        .fetch_optional(db)
        .await?;

        row.ok_or_else(|| ApiError::NotFound(format!("Notification {} not found", notif_id)))
    }
}
// ─── Audit Log ───────────────────────────────────────────────────────────────

/// Well-known action values stored in the `action` column of `action_logs`.
pub mod audit_action {
    pub const KYC_SUBMITTED: &str = "kyc_submitted";
    pub const KYC_APPROVED: &str = "kyc_approved";
    pub const KYC_REJECTED: &str = "kyc_rejected";
    pub const PLAN_CREATED: &str = "plan_created";
    pub const PLAN_CLAIMED: &str = "plan_claimed";
    pub const PLAN_DEACTIVATED: &str = "plan_deactivated";
    pub const TWO_FA_SENT: &str = "2fa_sent";
}

/// Entity type constants — stored in `entity_type` column of `action_logs`.
pub mod entity_type {
    pub const USER: &str = "user";
    pub const PLAN: &str = "plan";
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActionLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub timestamp: DateTime<Utc>,
}

pub struct AuditLogService;

impl AuditLogService {
    pub async fn log(
        // Use an executor that can be a Pool or a Transaction
        executor: impl sqlx::PgExecutor<'_>,
        user_id: Option<Uuid>,
        action: &str,
        entity_id: Option<Uuid>,
        entity_type: Option<&str>,
    ) -> Result<(), ApiError> {
        // Return Result instead of ()
        sqlx::query(
            r#"
            INSERT INTO action_logs (user_id, action, entity_id, entity_type)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id)
        .bind(action)
        .bind(entity_id)
        .bind(entity_type)
        .execute(executor) // Execute on the provided transaction/pool
        .await?;

        Ok(())
    }
    /// Return all audit log entries for admin inspection, newest first.
    pub async fn list_all(db: &PgPool) -> Result<Vec<ActionLog>, ApiError> {
        let rows = sqlx::query_as::<_, ActionLog>(
            r#"
            SELECT id, user_id, action, entity_id, entity_type, timestamp
            FROM action_logs
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(rows)
    }

    /// Return audit log entries for a specific user, newest first.
    pub async fn list_for_user(db: &PgPool, user_id: Uuid) -> Result<Vec<ActionLog>, ApiError> {
        let rows = sqlx::query_as::<_, ActionLog>(
            r#"
            SELECT id, user_id, action, entity_id, entity_type, timestamp
            FROM action_logs
            WHERE user_id = $1
            ORDER BY timestamp DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(db)
        .await?;

        Ok(rows)
    }
}
// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{audit_action, entity_type, notif_type};
    use crate::notifications::{ActionLog, Notification};
    use chrono::Utc;
    use uuid::Uuid;

    // ── notif_type constants ────────────────────────────────────────────────

    #[test]
    fn notif_type_kyc_approved_is_correct_string() {
        assert_eq!(notif_type::KYC_APPROVED, "kyc_approved");
    }

    #[test]
    fn notif_type_kyc_rejected_is_correct_string() {
        assert_eq!(notif_type::KYC_REJECTED, "kyc_rejected");
    }

    #[test]
    fn notif_type_plan_created_is_correct_string() {
        assert_eq!(notif_type::PLAN_CREATED, "plan_created");
    }

    #[test]
    fn notif_type_plan_claimed_is_correct_string() {
        assert_eq!(notif_type::PLAN_CLAIMED, "plan_claimed");
    }

    #[test]
    fn notif_type_plan_deactivated_is_correct_string() {
        assert_eq!(notif_type::PLAN_DEACTIVATED, "plan_deactivated");
    }

    #[test]
    fn notif_type_two_fa_sent_is_correct_string() {
        assert_eq!(notif_type::TWO_FA_SENT, "2fa_sent");
    }

    // ── audit_action constants ──────────────────────────────────────────────

    #[test]
    fn audit_action_kyc_approved_is_correct_string() {
        assert_eq!(audit_action::KYC_APPROVED, "kyc_approved");
    }

    #[test]
    fn audit_action_kyc_rejected_is_correct_string() {
        assert_eq!(audit_action::KYC_REJECTED, "kyc_rejected");
    }

    #[test]
    fn audit_action_plan_created_is_correct_string() {
        assert_eq!(audit_action::PLAN_CREATED, "plan_created");
    }

    #[test]
    fn audit_action_plan_claimed_is_correct_string() {
        assert_eq!(audit_action::PLAN_CLAIMED, "plan_claimed");
    }

    #[test]
    fn audit_action_plan_deactivated_is_correct_string() {
        assert_eq!(audit_action::PLAN_DEACTIVATED, "plan_deactivated");
    }

    #[test]
    fn audit_action_two_fa_sent_is_correct_string() {
        assert_eq!(audit_action::TWO_FA_SENT, "2fa_sent");
    }

    // ── entity_type constants ───────────────────────────────────────────────

    #[test]
    fn entity_type_user_is_correct_string() {
        assert_eq!(entity_type::USER, "user");
    }

    #[test]
    fn entity_type_plan_is_correct_string() {
        assert_eq!(entity_type::PLAN, "plan");
    }

    // ── Cross-module consistency ────────────────────────────────────────────
    // For shared events the notif_type and audit_action values must agree
    // so that log queries filtering by action string stay in sync.

    #[test]
    fn kyc_approved_notif_and_audit_action_agree() {
        assert_eq!(notif_type::KYC_APPROVED, audit_action::KYC_APPROVED);
    }

    #[test]
    fn kyc_rejected_notif_and_audit_action_agree() {
        assert_eq!(notif_type::KYC_REJECTED, audit_action::KYC_REJECTED);
    }

    #[test]
    fn plan_created_notif_and_audit_action_agree() {
        assert_eq!(notif_type::PLAN_CREATED, audit_action::PLAN_CREATED);
    }

    #[test]
    fn plan_claimed_notif_and_audit_action_agree() {
        assert_eq!(notif_type::PLAN_CLAIMED, audit_action::PLAN_CLAIMED);
    }

    #[test]
    fn plan_deactivated_notif_and_audit_action_agree() {
        assert_eq!(notif_type::PLAN_DEACTIVATED, audit_action::PLAN_DEACTIVATED);
    }

    // ── Struct serde round-trips ────────────────────────────────────────────

    #[test]
    fn notification_serializes_type_field_as_type() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notif_type: notif_type::KYC_APPROVED.to_string(),
            message: "Approved!".to_string(),
            is_read: false,
            created_at: Utc::now(),
        };
        let json = serde_json::to_value(&n).unwrap();
        // The `#[serde(rename = "type")]` on notif_type must produce `"type"` in JSON
        assert!(
            json.get("type").is_some(),
            "Expected JSON key 'type', got: {}",
            json
        );
        assert_eq!(json["type"], notif_type::KYC_APPROVED);
        // `notif_type` key must NOT appear (it's renamed)
        assert!(json.get("notif_type").is_none());
    }

    #[test]
    fn notification_is_read_defaults_to_false_in_struct() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notif_type: notif_type::PLAN_CREATED.to_string(),
            message: "Plan created".to_string(),
            is_read: false,
            created_at: Utc::now(),
        };
        assert!(!n.is_read);
    }

    #[test]
    fn action_log_serializes_correctly() {
        let log = ActionLog {
            id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            action: audit_action::PLAN_CLAIMED.to_string(),
            entity_id: Some(Uuid::new_v4()),
            entity_type: Some(entity_type::PLAN.to_string()),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_value(&log).unwrap();
        assert_eq!(json["action"], audit_action::PLAN_CLAIMED);
        assert_eq!(json["entity_type"], entity_type::PLAN);
    }

    #[test]
    fn action_log_optional_fields_can_be_none() {
        let log = ActionLog {
            id: Uuid::new_v4(),
            user_id: None,
            action: audit_action::KYC_APPROVED.to_string(),
            entity_id: None,
            entity_type: None,
            timestamp: Utc::now(),
        };
        let json = serde_json::to_value(&log).unwrap();
        assert!(json["user_id"].is_null());
        assert!(json["entity_id"].is_null());
        assert!(json["entity_type"].is_null());
    }
}

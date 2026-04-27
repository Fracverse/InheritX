//! # Message Access Audit Log Service
//!
//! Tracks all message access activity including creation, viewing, decryption,
//! delivery, key rotation, and administrative actions on legacy messages.

use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

// ─── Access Action Types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageAccessAction {
    Created,
    Viewed,
    Decrypted,
    Delivered,
    DeliveryFailed,
    KeyRotated,
    KeyListed,
    Deleted,
}

impl MessageAccessAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Viewed => "viewed",
            Self::Decrypted => "decrypted",
            Self::Delivered => "delivered",
            Self::DeliveryFailed => "delivery_failed",
            Self::KeyRotated => "key_rotated",
            Self::KeyListed => "key_listed",
            Self::Deleted => "deleted",
        }
    }
}

// ─── Audit Log Entry ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAccessLog {
    pub id: Uuid,
    pub message_id: Option<Uuid>,
    pub user_id: Uuid,
    pub action: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl<'r> FromRow<'r, sqlx::postgres::PgRow> for MessageAccessLog {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            message_id: row.try_get("message_id")?,
            user_id: row.try_get("user_id")?,
            action: row.try_get("action")?,
            ip_address: row.try_get("ip_address")?,
            user_agent: row.try_get("user_agent")?,
            metadata: row.try_get("metadata")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ─── Query Filters ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct MessageAuditFilters {
    pub message_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ─── Audit Summary ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAuditSummary {
    pub total_events: i64,
    pub action_counts: Vec<ActionCount>,
    pub unique_users: i64,
    pub unique_messages: i64,
    pub first_event_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActionCount {
    pub action: String,
    pub count: i64,
}

// ─── User Message Activity ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessageActivity {
    pub user_id: Uuid,
    pub total_actions: i64,
    pub messages_created: i64,
    pub messages_viewed: i64,
    pub messages_delivered: i64,
    pub first_activity: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
}

// ─── Service ─────────────────────────────────────────────────────────────────

pub struct MessageAccessAuditService;

impl MessageAccessAuditService {
    /// Log a message access event
    pub async fn log_access(
        db: &PgPool,
        message_id: Option<Uuid>,
        user_id: Uuid,
        action: MessageAccessAction,
        ip_address: Option<String>,
        user_agent: Option<String>,
        metadata: serde_json::Value,
    ) -> Result<MessageAccessLog, ApiError> {
        let row = sqlx::query(
            r#"
            INSERT INTO message_access_logs
                (message_id, user_id, action, ip_address, user_agent, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, message_id, user_id, action,
                      CAST(ip_address AS TEXT) as ip_address,
                      user_agent, metadata, created_at
            "#,
        )
        .bind(message_id)
        .bind(user_id)
        .bind(action.as_str())
        .bind(ip_address)
        .bind(user_agent)
        .bind(&metadata)
        .fetch_one(db)
        .await?;

        let entry = MessageAccessLog::from_row(&row)?;

        tracing::info!(
            audit_id = %entry.id,
            action = %entry.action,
            user_id = %user_id,
            message_id = ?message_id,
            "Message access logged"
        );

        Ok(entry)
    }

    /// Query audit logs with filters
    pub async fn get_logs(
        db: &PgPool,
        filters: &MessageAuditFilters,
    ) -> Result<Vec<MessageAccessLog>, ApiError> {
        let limit = filters.limit.unwrap_or(100).min(1000);
        let offset = filters.offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT
                id, message_id, user_id, action,
                CAST(ip_address AS TEXT) as ip_address,
                user_agent, metadata, created_at
            FROM message_access_logs
            WHERE
                ($1::UUID IS NULL OR message_id = $1)
                AND ($2::UUID IS NULL OR user_id = $2)
                AND ($3::TEXT IS NULL OR action = $3)
                AND ($4::TIMESTAMPTZ IS NULL OR created_at >= $4)
                AND ($5::TIMESTAMPTZ IS NULL OR created_at <= $5)
            ORDER BY created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(filters.message_id)
        .bind(filters.user_id)
        .bind(&filters.action)
        .bind(filters.start_date)
        .bind(filters.end_date)
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await?;

        rows.into_iter()
            .map(|row| MessageAccessLog::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Get logs for a specific message
    pub async fn get_message_logs(
        db: &PgPool,
        message_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<MessageAccessLog>, ApiError> {
        let limit = limit.unwrap_or(100).min(1000);

        let rows = sqlx::query(
            r#"
            SELECT
                id, message_id, user_id, action,
                CAST(ip_address AS TEXT) as ip_address,
                user_agent, metadata, created_at
            FROM message_access_logs
            WHERE message_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(message_id)
        .bind(limit)
        .fetch_all(db)
        .await?;

        rows.into_iter()
            .map(|row| MessageAccessLog::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Get audit summary (admin dashboard)
    pub async fn get_summary(db: &PgPool) -> Result<MessageAuditSummary, ApiError> {
        let (total_events, unique_users, unique_messages, first_event_at, last_event_at): (
            i64,
            i64,
            i64,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
        ) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_events,
                COUNT(DISTINCT user_id) as unique_users,
                COUNT(DISTINCT message_id) as unique_messages,
                MIN(created_at) as first_event_at,
                MAX(created_at) as last_event_at
            FROM message_access_logs
            "#,
        )
        .fetch_one(db)
        .await?;

        let action_counts = sqlx::query_as::<_, ActionCount>(
            r#"
            SELECT action, COUNT(*) as count
            FROM message_access_logs
            GROUP BY action
            ORDER BY count DESC
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(MessageAuditSummary {
            total_events,
            action_counts,
            unique_users,
            unique_messages,
            first_event_at,
            last_event_at,
        })
    }

    /// Get activity summary for a specific user
    pub async fn get_user_activity(
        db: &PgPool,
        user_id: Uuid,
    ) -> Result<UserMessageActivity, ApiError> {
        #[derive(FromRow)]
        struct ActivityRow {
            total_actions: i64,
            messages_created: i64,
            messages_viewed: i64,
            messages_delivered: i64,
            first_activity: Option<DateTime<Utc>>,
            last_activity: Option<DateTime<Utc>>,
        }

        let row = sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT
                COUNT(*) as total_actions,
                COUNT(*) FILTER (WHERE action = 'created') as messages_created,
                COUNT(*) FILTER (WHERE action = 'viewed') as messages_viewed,
                COUNT(*) FILTER (WHERE action = 'delivered') as messages_delivered,
                MIN(created_at) as first_activity,
                MAX(created_at) as last_activity
            FROM message_access_logs
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(db)
        .await?;

        Ok(UserMessageActivity {
            user_id,
            total_actions: row.total_actions,
            messages_created: row.messages_created,
            messages_viewed: row.messages_viewed,
            messages_delivered: row.messages_delivered,
            first_activity: row.first_activity,
            last_activity: row.last_activity,
        })
    }

    /// Search audit logs by metadata text
    pub async fn search_logs(
        db: &PgPool,
        search_term: &str,
        limit: i64,
    ) -> Result<Vec<MessageAccessLog>, ApiError> {
        let limit = limit.min(1000);

        let rows = sqlx::query(
            r#"
            SELECT
                id, message_id, user_id, action,
                CAST(ip_address AS TEXT) as ip_address,
                user_agent, metadata, created_at
            FROM message_access_logs
            WHERE metadata::text ILIKE $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(format!("%{}%", search_term))
        .bind(limit)
        .fetch_all(db)
        .await?;

        rows.into_iter()
            .map(|row| MessageAccessLog::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_action_serialization() {
        let action = MessageAccessAction::Created;
        assert_eq!(action.as_str(), "created");

        let json = serde_json::to_value(action).unwrap();
        assert_eq!(json, "created");
    }

    #[test]
    fn test_all_action_types() {
        let actions = vec![
            (MessageAccessAction::Created, "created"),
            (MessageAccessAction::Viewed, "viewed"),
            (MessageAccessAction::Decrypted, "decrypted"),
            (MessageAccessAction::Delivered, "delivered"),
            (MessageAccessAction::DeliveryFailed, "delivery_failed"),
            (MessageAccessAction::KeyRotated, "key_rotated"),
            (MessageAccessAction::KeyListed, "key_listed"),
            (MessageAccessAction::Deleted, "deleted"),
        ];

        for (action, expected) in actions {
            assert_eq!(action.as_str(), expected);
        }
    }

    #[test]
    fn test_audit_filters_defaults() {
        let filters = MessageAuditFilters {
            message_id: None,
            user_id: None,
            action: None,
            start_date: None,
            end_date: None,
            limit: None,
            offset: None,
        };

        assert!(filters.message_id.is_none());
        assert!(filters.user_id.is_none());
        assert!(filters.action.is_none());
    }

    #[test]
    fn test_action_count_serialization() {
        let count = ActionCount {
            action: "viewed".to_string(),
            count: 42,
        };

        let json = serde_json::to_value(&count).unwrap();
        assert_eq!(json["action"], "viewed");
        assert_eq!(json["count"], 42);
    }

    #[test]
    fn test_message_access_log_serialization() {
        let log = MessageAccessLog {
            id: Uuid::new_v4(),
            message_id: Some(Uuid::new_v4()),
            user_id: Uuid::new_v4(),
            action: "created".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            metadata: serde_json::json!({"beneficiary": "test@example.com"}),
            created_at: Utc::now(),
        };

        let json = serde_json::to_value(&log).unwrap();
        assert_eq!(json["action"], "created");
        assert!(json["message_id"].is_string());
        assert!(json["ip_address"].is_string());
    }

    #[test]
    fn test_audit_summary_serialization() {
        let summary = MessageAuditSummary {
            total_events: 100,
            action_counts: vec![
                ActionCount {
                    action: "viewed".to_string(),
                    count: 50,
                },
                ActionCount {
                    action: "created".to_string(),
                    count: 30,
                },
            ],
            unique_users: 10,
            unique_messages: 25,
            first_event_at: Some(Utc::now()),
            last_event_at: Some(Utc::now()),
        };

        let json = serde_json::to_value(&summary).unwrap();
        assert_eq!(json["total_events"], 100);
        assert_eq!(json["unique_users"], 10);
        assert_eq!(json["action_counts"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_user_message_activity_serialization() {
        let activity = UserMessageActivity {
            user_id: Uuid::new_v4(),
            total_actions: 15,
            messages_created: 5,
            messages_viewed: 8,
            messages_delivered: 2,
            first_activity: Some(Utc::now()),
            last_activity: Some(Utc::now()),
        };

        let json = serde_json::to_value(&activity).unwrap();
        assert_eq!(json["total_actions"], 15);
        assert_eq!(json["messages_created"], 5);
        assert_eq!(json["messages_viewed"], 8);
        assert_eq!(json["messages_delivered"], 2);
    }

    #[test]
    fn test_action_deserialization() {
        let json = serde_json::json!("key_rotated");
        let action: MessageAccessAction = serde_json::from_value(json).unwrap();
        assert_eq!(action, MessageAccessAction::KeyRotated);
    }
}

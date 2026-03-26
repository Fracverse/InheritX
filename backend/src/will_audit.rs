//! # Legal Will Audit Log Service
//!
//! Comprehensive audit logging for all legal will document actions.
//! Provides queryable API for administrators and users to track document lifecycle.

use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

// ─── Audit Log Entry ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub event_type: String,
    pub document_id: Uuid,
    pub plan_id: Uuid,
    pub vault_id: String,
    pub user_id: Option<Uuid>,
    pub event_data: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl<'r> FromRow<'r, sqlx::postgres::PgRow> for AuditLogEntry {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            event_type: row.try_get("event_type")?,
            document_id: row.try_get("document_id")?,
            plan_id: row.try_get("plan_id")?,
            vault_id: row.try_get("vault_id")?,
            user_id: row.try_get("user_id")?,
            event_data: row.try_get("event_data")?,
            ip_address: row.try_get("ip_address")?,
            user_agent: row.try_get("user_agent")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ─── Audit Log Summary ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogSummary {
    pub total_events: i64,
    pub event_type_counts: Vec<EventTypeCount>,
    pub recent_events: Vec<AuditLogEntry>,
    pub first_event_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventTypeCount {
    pub event_type: String,
    pub count: i64,
}

// ─── Query Filters ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct AuditLogFilters {
    pub document_id: Option<Uuid>,
    pub plan_id: Option<Uuid>,
    pub vault_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ─── User Activity Summary ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivitySummary {
    pub user_id: Uuid,
    pub total_actions: i64,
    pub documents_created: i64,
    pub documents_updated: i64,
    pub documents_signed: i64,
    pub documents_downloaded: i64,
    pub first_activity: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
}

// ─── Audit Log Service ────────────────────────────────────────────────────────

pub struct WillAuditService;

impl WillAuditService {
    /// Get audit logs with filters
    pub async fn get_audit_logs(
        db: &PgPool,
        filters: &AuditLogFilters,
    ) -> Result<Vec<AuditLogEntry>, ApiError> {
        let limit = filters.limit.unwrap_or(100).min(1000);
        let offset = filters.offset.unwrap_or(0);

        Self::get_audit_logs_simple(db, filters, limit, offset).await
    }

    /// Simplified audit log query (more maintainable)
    async fn get_audit_logs_simple(
        db: &PgPool,
        filters: &AuditLogFilters,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLogEntry>, ApiError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, event_type, document_id, plan_id, vault_id, user_id, event_data,
                CAST(ip_address AS TEXT) as ip_address, user_agent, created_at
            FROM will_event_log
            WHERE 
                ($1::UUID IS NULL OR document_id = $1)
                AND ($2::UUID IS NULL OR plan_id = $2)
                AND ($3::TEXT IS NULL OR vault_id = $3)
                AND ($4::UUID IS NULL OR user_id = $4)
                AND ($5::TEXT IS NULL OR event_type = $5)
                AND ($6::TIMESTAMPTZ IS NULL OR created_at >= $6)
                AND ($7::TIMESTAMPTZ IS NULL OR created_at <= $7)
            ORDER BY created_at DESC
            LIMIT $8 OFFSET $9
            "#,
        )
        .bind(filters.document_id)
        .bind(filters.plan_id)
        .bind(&filters.vault_id)
        .bind(filters.user_id)
        .bind(&filters.event_type)
        .bind(filters.start_date)
        .bind(filters.end_date)
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await?;

        let entries: Vec<AuditLogEntry> = rows
            .into_iter()
            .map(|row| AuditLogEntry::from_row(&row))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Get audit log summary for a plan
    pub async fn get_plan_audit_summary(
        db: &PgPool,
        plan_id: Uuid,
    ) -> Result<AuditLogSummary, ApiError> {
        // Get total count
        let total_events: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM will_event_log WHERE plan_id = $1")
                .bind(plan_id)
                .fetch_one(db)
                .await?;

        // Get event type counts
        let event_type_counts = sqlx::query_as::<_, EventTypeCount>(
            r#"
            SELECT event_type, COUNT(*) as count
            FROM will_event_log
            WHERE plan_id = $1
            GROUP BY event_type
            ORDER BY count DESC
            "#,
        )
        .bind(plan_id)
        .fetch_all(db)
        .await?;

        // Get recent events
        let recent_events = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            SELECT 
                id, event_type, document_id, plan_id, vault_id, user_id, event_data,
                CAST(ip_address AS TEXT) as ip_address, user_agent, created_at
            FROM will_event_log
            WHERE plan_id = $1
            ORDER BY created_at DESC
            LIMIT 10
            "#,
        )
        .bind(plan_id)
        .fetch_all(db)
        .await?;

        // Get first and last event timestamps
        let (first_event_at, last_event_at): (Option<DateTime<Utc>>, Option<DateTime<Utc>>) =
            sqlx::query_as(
                r#"
                SELECT MIN(created_at), MAX(created_at)
                FROM will_event_log
                WHERE plan_id = $1
                "#,
            )
            .bind(plan_id)
            .fetch_one(db)
            .await?;

        Ok(AuditLogSummary {
            total_events,
            event_type_counts,
            recent_events,
            first_event_at,
            last_event_at,
        })
    }

    /// Get user activity summary
    pub async fn get_user_activity_summary(
        db: &PgPool,
        user_id: Uuid,
    ) -> Result<UserActivitySummary, ApiError> {
        #[derive(sqlx::FromRow)]
        struct ActivityRow {
            total_actions: i64,
            documents_created: i64,
            documents_updated: i64,
            documents_signed: i64,
            documents_downloaded: i64,
            first_activity: Option<DateTime<Utc>>,
            last_activity: Option<DateTime<Utc>>,
        }

        let row = sqlx::query_as::<_, ActivityRow>(
            r#"
                SELECT
                    COUNT(*) as total_actions,
                    COUNT(*) FILTER (WHERE event_type = 'will_created') as documents_created,
                    COUNT(*) FILTER (WHERE event_type = 'will_updated') as documents_updated,
                    COUNT(*) FILTER (WHERE event_type = 'will_signed') as documents_signed,
                    COUNT(*) FILTER (WHERE event_type = 'will_decrypted') as documents_downloaded,
                    MIN(created_at) as first_activity,
                    MAX(created_at) as last_activity
                FROM will_event_log
                WHERE user_id = $1
                "#,
        )
        .bind(user_id)
        .fetch_one(db)
        .await?;

        Ok(UserActivitySummary {
            user_id,
            total_actions: row.total_actions,
            documents_created: row.documents_created,
            documents_updated: row.documents_updated,
            documents_signed: row.documents_signed,
            documents_downloaded: row.documents_downloaded,
            first_activity: row.first_activity,
            last_activity: row.last_activity,
        })
    }

    /// Get all event types
    pub async fn get_event_types(db: &PgPool) -> Result<Vec<String>, ApiError> {
        let event_types: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT event_type FROM will_event_log ORDER BY event_type",
        )
        .fetch_all(db)
        .await?;

        Ok(event_types)
    }

    /// Get audit log statistics for admin dashboard
    pub async fn get_admin_statistics(db: &PgPool) -> Result<AdminAuditStatistics, ApiError> {
        let row: (i64, i64, i64, Option<DateTime<Utc>>, Option<DateTime<Utc>>) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_events,
                COUNT(DISTINCT plan_id) as unique_plans,
                COUNT(DISTINCT user_id) as unique_users,
                MIN(created_at) as first_event,
                MAX(created_at) as last_event
            FROM will_event_log
            "#,
        )
        .fetch_one(db)
        .await?;

        let event_type_distribution = sqlx::query_as::<_, EventTypeCount>(
            r#"
            SELECT event_type, COUNT(*) as count
            FROM will_event_log
            GROUP BY event_type
            ORDER BY count DESC
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(AdminAuditStatistics {
            total_events: row.0,
            unique_plans: row.1,
            unique_users: row.2,
            first_event: row.3,
            last_event: row.4,
            event_type_distribution,
        })
    }

    /// Search audit logs by text in event_data
    pub async fn search_audit_logs(
        db: &PgPool,
        search_term: &str,
        limit: i64,
    ) -> Result<Vec<AuditLogEntry>, ApiError> {
        let limit = limit.min(1000);

        let rows = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            SELECT 
                id, event_type, document_id, plan_id, vault_id, user_id, event_data,
                CAST(ip_address AS TEXT) as ip_address, user_agent, created_at
            FROM will_event_log
            WHERE event_data::text ILIKE $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(format!("%{}%", search_term))
        .bind(limit)
        .fetch_all(db)
        .await?;

        Ok(rows)
    }
}

// ─── Admin Statistics ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAuditStatistics {
    pub total_events: i64,
    pub unique_plans: i64,
    pub unique_users: i64,
    pub first_event: Option<DateTime<Utc>>,
    pub last_event: Option<DateTime<Utc>>,
    pub event_type_distribution: Vec<EventTypeCount>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_filters_default() {
        let filters = AuditLogFilters {
            document_id: None,
            plan_id: None,
            vault_id: None,
            user_id: None,
            event_type: None,
            start_date: None,
            end_date: None,
            limit: None,
            offset: None,
        };

        assert!(filters.document_id.is_none());
        assert!(filters.plan_id.is_none());
    }

    #[test]
    fn test_event_type_count_serialization() {
        let count = EventTypeCount {
            event_type: "will_created".to_string(),
            count: 42,
        };

        let json = serde_json::to_value(&count).unwrap();
        assert_eq!(json["event_type"], "will_created");
        assert_eq!(json["count"], 42);
    }
}

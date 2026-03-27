//! # Legal Will Event Logging
//!
//! Standardizes and emits events for all major legal will actions to enable
//! backend indexing, auditing, and transparency. Supports real-time frontend
//! updates, audit trails, and monitoring/analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ─── Event Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum WillEvent {
    WillCreated {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        version: u32,
        template: String,
        will_hash: String,
        timestamp: DateTime<Utc>,
    },
    WillUpdated {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        version: u32,
        previous_version: Option<u32>,
        will_hash: String,
        timestamp: DateTime<Utc>,
    },
    WillFinalized {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        version: u32,
        will_hash: String,
        timestamp: DateTime<Utc>,
    },
    WillSigned {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        signer: String,
        signature_hash: String,
        timestamp: DateTime<Utc>,
    },
    WitnessSigned {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        witness: String,
        witness_id: Uuid,
        signature_hash: String,
        timestamp: DateTime<Utc>,
    },
    WillEncrypted {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        encryption_method: String,
        timestamp: DateTime<Utc>,
    },
    WillDecrypted {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        accessed_by: Uuid,
        timestamp: DateTime<Utc>,
    },
    WillBackupCreated {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        backup_id: Uuid,
        backup_hash: String,
        timestamp: DateTime<Utc>,
    },
    WillVerified {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        version: u32,
        verification_result: bool,
        verified_by: Option<Uuid>,
        timestamp: DateTime<Utc>,
    },
    WitnessInvited {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        witness_id: Uuid,
        witness_identifier: String,
        timestamp: DateTime<Utc>,
    },
    WitnessDeclined {
        vault_id: String,
        document_id: Uuid,
        plan_id: Uuid,
        witness_id: Uuid,
        timestamp: DateTime<Utc>,
    },
}

impl WillEvent {
    pub fn event_type(&self) -> &str {
        match self {
            WillEvent::WillCreated { .. } => "will_created",
            WillEvent::WillUpdated { .. } => "will_updated",
            WillEvent::WillFinalized { .. } => "will_finalized",
            WillEvent::WillSigned { .. } => "will_signed",
            WillEvent::WitnessSigned { .. } => "witness_signed",
            WillEvent::WillEncrypted { .. } => "will_encrypted",
            WillEvent::WillDecrypted { .. } => "will_decrypted",
            WillEvent::WillBackupCreated { .. } => "will_backup_created",
            WillEvent::WillVerified { .. } => "will_verified",
            WillEvent::WitnessInvited { .. } => "witness_invited",
            WillEvent::WitnessDeclined { .. } => "witness_declined",
        }
    }

    pub fn document_id(&self) -> Uuid {
        match self {
            WillEvent::WillCreated { document_id, .. }
            | WillEvent::WillUpdated { document_id, .. }
            | WillEvent::WillFinalized { document_id, .. }
            | WillEvent::WillSigned { document_id, .. }
            | WillEvent::WitnessSigned { document_id, .. }
            | WillEvent::WillEncrypted { document_id, .. }
            | WillEvent::WillDecrypted { document_id, .. }
            | WillEvent::WillBackupCreated { document_id, .. }
            | WillEvent::WillVerified { document_id, .. }
            | WillEvent::WitnessInvited { document_id, .. }
            | WillEvent::WitnessDeclined { document_id, .. } => *document_id,
        }
    }

    pub fn plan_id(&self) -> Uuid {
        match self {
            WillEvent::WillCreated { plan_id, .. }
            | WillEvent::WillUpdated { plan_id, .. }
            | WillEvent::WillFinalized { plan_id, .. }
            | WillEvent::WillSigned { plan_id, .. }
            | WillEvent::WitnessSigned { plan_id, .. }
            | WillEvent::WillEncrypted { plan_id, .. }
            | WillEvent::WillDecrypted { plan_id, .. }
            | WillEvent::WillBackupCreated { plan_id, .. }
            | WillEvent::WillVerified { plan_id, .. }
            | WillEvent::WitnessInvited { plan_id, .. }
            | WillEvent::WitnessDeclined { plan_id, .. } => *plan_id,
        }
    }

    pub fn vault_id(&self) -> &str {
        match self {
            WillEvent::WillCreated { vault_id, .. }
            | WillEvent::WillUpdated { vault_id, .. }
            | WillEvent::WillFinalized { vault_id, .. }
            | WillEvent::WillSigned { vault_id, .. }
            | WillEvent::WitnessSigned { vault_id, .. }
            | WillEvent::WillEncrypted { vault_id, .. }
            | WillEvent::WillDecrypted { vault_id, .. }
            | WillEvent::WillBackupCreated { vault_id, .. }
            | WillEvent::WillVerified { vault_id, .. }
            | WillEvent::WitnessInvited { vault_id, .. }
            | WillEvent::WitnessDeclined { vault_id, .. } => vault_id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            WillEvent::WillCreated { timestamp, .. }
            | WillEvent::WillUpdated { timestamp, .. }
            | WillEvent::WillFinalized { timestamp, .. }
            | WillEvent::WillSigned { timestamp, .. }
            | WillEvent::WitnessSigned { timestamp, .. }
            | WillEvent::WillEncrypted { timestamp, .. }
            | WillEvent::WillDecrypted { timestamp, .. }
            | WillEvent::WillBackupCreated { timestamp, .. }
            | WillEvent::WillVerified { timestamp, .. }
            | WillEvent::WitnessInvited { timestamp, .. }
            | WillEvent::WitnessDeclined { timestamp, .. } => *timestamp,
        }
    }
}

// ─── Event Service ────────────────────────────────────────────────────────────

pub struct WillEventService;

impl WillEventService {
    /// Emit a will event and persist it to the database for auditing and indexing
    pub async fn emit(db: &PgPool, event: WillEvent) -> Result<Uuid, crate::api_error::ApiError> {
        let event_id = Uuid::new_v4();
        let event_type = event.event_type();
        let document_id = event.document_id();
        let plan_id = event.plan_id();
        let vault_id = event.vault_id();
        let timestamp = event.timestamp();
        let event_data = serde_json::to_value(&event).map_err(|e| {
            crate::api_error::ApiError::Internal(anyhow::anyhow!(
                "Failed to serialize event: {}",
                e
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO will_event_log
                (id, event_type, document_id, plan_id, vault_id, event_data, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(event_id)
        .bind(event_type)
        .bind(document_id)
        .bind(plan_id)
        .bind(vault_id)
        .bind(event_data)
        .bind(timestamp)
        .execute(db)
        .await?;

        tracing::info!(
            event_id = %event_id,
            event_type = %event_type,
            document_id = %document_id,
            plan_id = %plan_id,
            vault_id = %vault_id,
            "Will event emitted"
        );

        Ok(event_id)
    }

    /// Retrieve all events for a specific document
    pub async fn get_document_events(
        db: &PgPool,
        document_id: Uuid,
    ) -> Result<Vec<WillEvent>, crate::api_error::ApiError> {
        let rows: Vec<EventRow> = sqlx::query_as(
            r#"
            SELECT event_data
            FROM will_event_log
            WHERE document_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(document_id)
        .fetch_all(db)
        .await?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_value(row.event_data).map_err(|e| {
                    crate::api_error::ApiError::Internal(anyhow::anyhow!(
                        "Failed to deserialize event: {}",
                        e
                    ))
                })
            })
            .collect()
    }

    /// Retrieve all events for a specific plan
    pub async fn get_plan_events(
        db: &PgPool,
        plan_id: Uuid,
    ) -> Result<Vec<WillEvent>, crate::api_error::ApiError> {
        let rows: Vec<EventRow> = sqlx::query_as(
            r#"
            SELECT event_data
            FROM will_event_log
            WHERE plan_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(plan_id)
        .fetch_all(db)
        .await?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_value(row.event_data).map_err(|e| {
                    crate::api_error::ApiError::Internal(anyhow::anyhow!(
                        "Failed to deserialize event: {}",
                        e
                    ))
                })
            })
            .collect()
    }

    /// Retrieve all events for a specific vault
    pub async fn get_vault_events(
        db: &PgPool,
        vault_id: &str,
    ) -> Result<Vec<WillEvent>, crate::api_error::ApiError> {
        let rows: Vec<EventRow> = sqlx::query_as(
            r#"
            SELECT event_data
            FROM will_event_log
            WHERE vault_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(vault_id)
        .fetch_all(db)
        .await?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_value(row.event_data).map_err(|e| {
                    crate::api_error::ApiError::Internal(anyhow::anyhow!(
                        "Failed to deserialize event: {}",
                        e
                    ))
                })
            })
            .collect()
    }

    /// Retrieve events by type
    pub async fn get_events_by_type(
        db: &PgPool,
        event_type: &str,
        limit: Option<i64>,
    ) -> Result<Vec<WillEvent>, crate::api_error::ApiError> {
        let limit = limit.unwrap_or(100).min(1000);

        let rows: Vec<EventRow> = sqlx::query_as(
            r#"
            SELECT event_data
            FROM will_event_log
            WHERE event_type = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(event_type)
        .bind(limit)
        .fetch_all(db)
        .await?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_value(row.event_data).map_err(|e| {
                    crate::api_error::ApiError::Internal(anyhow::anyhow!(
                        "Failed to deserialize event: {}",
                        e
                    ))
                })
            })
            .collect()
    }

    /// Get event statistics for a plan
    pub async fn get_plan_event_stats(
        db: &PgPool,
        plan_id: Uuid,
    ) -> Result<EventStats, crate::api_error::ApiError> {
        let row: EventStatsRow = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE event_type = 'will_created') as will_created_count,
                COUNT(*) FILTER (WHERE event_type = 'will_updated') as will_updated_count,
                COUNT(*) FILTER (WHERE event_type = 'will_finalized') as will_finalized_count,
                COUNT(*) FILTER (WHERE event_type = 'will_signed') as will_signed_count,
                COUNT(*) FILTER (WHERE event_type = 'witness_signed') as witness_signed_count,
                COUNT(*) FILTER (WHERE event_type = 'will_verified') as will_verified_count,
                COUNT(*) as total_events,
                MIN(created_at) as first_event_at,
                MAX(created_at) as last_event_at
            FROM will_event_log
            WHERE plan_id = $1
            "#,
        )
        .bind(plan_id)
        .fetch_one(db)
        .await?;

        Ok(EventStats {
            plan_id,
            will_created_count: row.will_created_count.unwrap_or(0),
            will_updated_count: row.will_updated_count.unwrap_or(0),
            will_finalized_count: row.will_finalized_count.unwrap_or(0),
            will_signed_count: row.will_signed_count.unwrap_or(0),
            witness_signed_count: row.witness_signed_count.unwrap_or(0),
            will_verified_count: row.will_verified_count.unwrap_or(0),
            total_events: row.total_events.unwrap_or(0),
            first_event_at: row.first_event_at,
            last_event_at: row.last_event_at,
        })
    }
}

// ─── Data Types ───────────────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
struct EventRow {
    event_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStats {
    pub plan_id: Uuid,
    pub will_created_count: i64,
    pub will_updated_count: i64,
    pub will_finalized_count: i64,
    pub will_signed_count: i64,
    pub witness_signed_count: i64,
    pub will_verified_count: i64,
    pub total_events: i64,
    pub first_event_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
struct EventStatsRow {
    will_created_count: Option<i64>,
    will_updated_count: Option<i64>,
    will_finalized_count: Option<i64>,
    will_signed_count: Option<i64>,
    witness_signed_count: Option<i64>,
    will_verified_count: Option<i64>,
    total_events: Option<i64>,
    first_event_at: Option<DateTime<Utc>>,
    last_event_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_extraction() {
        let event = WillEvent::WillCreated {
            vault_id: "vault123".to_string(),
            document_id: Uuid::new_v4(),
            plan_id: Uuid::new_v4(),
            version: 1,
            template: "formal".to_string(),
            will_hash: "abc123".to_string(),
            timestamp: Utc::now(),
        };

        assert_eq!(event.event_type(), "will_created");
    }

    #[test]
    fn test_event_serialization() {
        let event = WillEvent::WillSigned {
            vault_id: "vault123".to_string(),
            document_id: Uuid::new_v4(),
            plan_id: Uuid::new_v4(),
            signer: "signer_wallet".to_string(),
            signature_hash: "sig_hash".to_string(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event_type"], "will_signed");
        assert_eq!(json["vault_id"], "vault123");
    }

    #[test]
    fn test_event_deserialization() {
        let json = serde_json::json!({
            "event_type": "will_finalized",
            "vault_id": "vault456",
            "document_id": "550e8400-e29b-41d4-a716-446655440000",
            "plan_id": "550e8400-e29b-41d4-a716-446655440001",
            "version": 2,
            "will_hash": "hash123",
            "timestamp": "2024-01-01T00:00:00Z"
        });

        let event: WillEvent = serde_json::from_value(json).unwrap();
        assert_eq!(event.event_type(), "will_finalized");
        assert_eq!(event.vault_id(), "vault456");
    }

    #[test]
    fn test_all_event_types_have_consistent_fields() {
        let doc_id = Uuid::new_v4();
        let plan_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let events = vec![
            WillEvent::WillCreated {
                vault_id: "v1".to_string(),
                document_id: doc_id,
                plan_id,
                version: 1,
                template: "formal".to_string(),
                will_hash: "hash".to_string(),
                timestamp,
            },
            WillEvent::WillUpdated {
                vault_id: "v1".to_string(),
                document_id: doc_id,
                plan_id,
                version: 2,
                previous_version: Some(1),
                will_hash: "hash2".to_string(),
                timestamp,
            },
            WillEvent::WillFinalized {
                vault_id: "v1".to_string(),
                document_id: doc_id,
                plan_id,
                version: 2,
                will_hash: "hash2".to_string(),
                timestamp,
            },
        ];

        for event in events {
            assert_eq!(event.document_id(), doc_id);
            assert_eq!(event.plan_id(), plan_id);
            assert_eq!(event.vault_id(), "v1");
            assert_eq!(event.timestamp(), timestamp);
        }
    }
}

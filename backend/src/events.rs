use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Event types for DeFi lending operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "event_type", rename_all = "lowercase")]
pub enum EventType {
    Deposit,
    Borrow,
    Repay,
    Liquidation,
    #[sqlx(rename = "interest_accrual")]
    InterestAccrual,
}

/// Lending event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub user_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub asset_code: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: rust_decimal::Decimal,
    pub metadata: serde_json::Value,
    pub transaction_hash: Option<String>,
    pub block_number: Option<i64>,
    pub event_timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// Internal row type for database queries
#[derive(sqlx::FromRow)]
struct LendingEventRow {
    id: Uuid,
    event_type: EventType,
    user_id: Uuid,
    plan_id: Option<Uuid>,
    asset_code: String,
    amount: String,
    metadata: serde_json::Value,
    transaction_hash: Option<String>,
    block_number: Option<i64>,
    event_timestamp: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl TryFrom<LendingEventRow> for LendingEvent {
    type Error = ApiError;

    fn try_from(row: LendingEventRow) -> Result<Self, Self::Error> {
        Ok(LendingEvent {
            id: row.id,
            event_type: row.event_type,
            user_id: row.user_id,
            plan_id: row.plan_id,
            asset_code: row.asset_code,
            amount: row.amount.parse().map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("Failed to parse amount: {}", e))
            })?,
            metadata: row.metadata,
            transaction_hash: row.transaction_hash,
            block_number: row.block_number,
            event_timestamp: row.event_timestamp,
            created_at: row.created_at,
        })
    }
}

/// Metadata for deposit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositMetadata {
    pub collateral_ratio: Option<rust_decimal::Decimal>,
    pub total_deposited: rust_decimal::Decimal,
}

/// Metadata for borrow events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowMetadata {
    pub interest_rate: rust_decimal::Decimal,
    pub collateral_asset: String,
    pub collateral_amount: rust_decimal::Decimal,
    pub loan_to_value: rust_decimal::Decimal,
    pub maturity_date: Option<DateTime<Utc>>,
}

/// Metadata for repay events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepayMetadata {
    pub principal_amount: rust_decimal::Decimal,
    pub interest_amount: rust_decimal::Decimal,
    pub remaining_balance: rust_decimal::Decimal,
}

/// Metadata for liquidation events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationMetadata {
    pub liquidator_id: Uuid,
    pub collateral_asset: String,
    pub collateral_seized: rust_decimal::Decimal,
    pub debt_covered: rust_decimal::Decimal,
    pub liquidation_penalty: rust_decimal::Decimal,
}

/// Metadata for interest accrual events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestAccrualMetadata {
    pub interest_rate: rust_decimal::Decimal,
    pub principal_balance: rust_decimal::Decimal,
    pub accrued_interest: rust_decimal::Decimal,
    pub total_balance: rust_decimal::Decimal,
}

/// Parameters for emitting an event
struct EmitEventParams<'a> {
    event_type: EventType,
    user_id: Uuid,
    plan_id: Option<Uuid>,
    asset_code: &'a str,
    amount: rust_decimal::Decimal,
    metadata: serde_json::Value,
    transaction_hash: Option<String>,
    block_number: Option<i64>,
}

/// Service for emitting and querying lending events
pub struct EventService;

impl EventService {
    /// Emit a deposit event
    #[allow(clippy::too_many_arguments)]
    pub async fn emit_deposit(
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        plan_id: Option<Uuid>,
        asset_code: &str,
        amount: rust_decimal::Decimal,
        metadata: DepositMetadata,
        transaction_hash: Option<String>,
        block_number: Option<i64>,
    ) -> Result<LendingEvent, ApiError> {
        let metadata_json = serde_json::to_value(metadata).map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to serialize metadata: {}", e))
        })?;

        Self::emit_event(
            tx,
            EmitEventParams {
                event_type: EventType::Deposit,
                user_id,
                plan_id,
                asset_code,
                amount,
                metadata: metadata_json,
                transaction_hash,
                block_number,
            },
        )
        .await
    }

    /// Emit a borrow event
    #[allow(clippy::too_many_arguments)]
    pub async fn emit_borrow(
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        plan_id: Option<Uuid>,
        asset_code: &str,
        amount: rust_decimal::Decimal,
        metadata: BorrowMetadata,
        transaction_hash: Option<String>,
        block_number: Option<i64>,
    ) -> Result<LendingEvent, ApiError> {
        let metadata_json = serde_json::to_value(metadata).map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to serialize metadata: {}", e))
        })?;

        Self::emit_event(
            tx,
            EmitEventParams {
                event_type: EventType::Borrow,
                user_id,
                plan_id,
                asset_code,
                amount,
                metadata: metadata_json,
                transaction_hash,
                block_number,
            },
        )
        .await
    }

    /// Emit a repay event
    #[allow(clippy::too_many_arguments)]
    pub async fn emit_repay(
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        plan_id: Option<Uuid>,
        asset_code: &str,
        amount: rust_decimal::Decimal,
        metadata: RepayMetadata,
        transaction_hash: Option<String>,
        block_number: Option<i64>,
    ) -> Result<LendingEvent, ApiError> {
        let metadata_json = serde_json::to_value(metadata).map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to serialize metadata: {}", e))
        })?;

        Self::emit_event(
            tx,
            EmitEventParams {
                event_type: EventType::Repay,
                user_id,
                plan_id,
                asset_code,
                amount,
                metadata: metadata_json,
                transaction_hash,
                block_number,
            },
        )
        .await
    }

    /// Emit a liquidation event
    #[allow(clippy::too_many_arguments)]
    pub async fn emit_liquidation(
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        plan_id: Option<Uuid>,
        asset_code: &str,
        amount: rust_decimal::Decimal,
        metadata: LiquidationMetadata,
        transaction_hash: Option<String>,
        block_number: Option<i64>,
    ) -> Result<LendingEvent, ApiError> {
        let metadata_json = serde_json::to_value(metadata).map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to serialize metadata: {}", e))
        })?;

        Self::emit_event(
            tx,
            EmitEventParams {
                event_type: EventType::Liquidation,
                user_id,
                plan_id,
                asset_code,
                amount,
                metadata: metadata_json,
                transaction_hash,
                block_number,
            },
        )
        .await
    }

    /// Emit an interest accrual event
    #[allow(clippy::too_many_arguments)]
    pub async fn emit_interest_accrual(
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        plan_id: Option<Uuid>,
        asset_code: &str,
        amount: rust_decimal::Decimal,
        metadata: InterestAccrualMetadata,
        transaction_hash: Option<String>,
        block_number: Option<i64>,
    ) -> Result<LendingEvent, ApiError> {
        let metadata_json = serde_json::to_value(metadata).map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to serialize metadata: {}", e))
        })?;

        Self::emit_event(
            tx,
            EmitEventParams {
                event_type: EventType::InterestAccrual,
                user_id,
                plan_id,
                asset_code,
                amount,
                metadata: metadata_json,
                transaction_hash,
                block_number,
            },
        )
        .await
    }

    /// Internal method to emit any event type
    async fn emit_event(
        tx: &mut Transaction<'_, Postgres>,
        params: EmitEventParams<'_>,
    ) -> Result<LendingEvent, ApiError> {
        let row = sqlx::query_as::<_, LendingEventRow>(
            r#"
            INSERT INTO lending_events (
                event_type, user_id, plan_id, asset_code, amount,
                metadata, transaction_hash, block_number
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, event_type, user_id, plan_id, asset_code, amount,
                      metadata, transaction_hash, block_number,
                      event_timestamp, created_at
            "#,
        )
        .bind(params.event_type)
        .bind(params.user_id)
        .bind(params.plan_id)
        .bind(params.asset_code)
        .bind(params.amount.to_string())
        .bind(params.metadata)
        .bind(params.transaction_hash)
        .bind(params.block_number)
        .fetch_one(&mut **tx)
        .await?;

        row.try_into()
    }

    /// Query events by user
    pub async fn get_user_events(
        pool: &PgPool,
        user_id: Uuid,
        event_type: Option<EventType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LendingEvent>, ApiError> {
        let rows = if let Some(et) = event_type {
            sqlx::query_as::<_, LendingEventRow>(
                r#"
                SELECT id, event_type, user_id, plan_id, asset_code, amount,
                       metadata, transaction_hash, block_number,
                       event_timestamp, created_at
                FROM lending_events
                WHERE user_id = $1 AND event_type = $2
                ORDER BY event_timestamp DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(user_id)
            .bind(et)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, LendingEventRow>(
                r#"
                SELECT id, event_type, user_id, plan_id, asset_code, amount,
                       metadata, transaction_hash, block_number,
                       event_timestamp, created_at
                FROM lending_events
                WHERE user_id = $1
                ORDER BY event_timestamp DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        rows.into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    /// Query events by plan
    pub async fn get_plan_events(
        pool: &PgPool,
        plan_id: Uuid,
        event_type: Option<EventType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LendingEvent>, ApiError> {
        let rows = if let Some(et) = event_type {
            sqlx::query_as::<_, LendingEventRow>(
                r#"
                SELECT id, event_type, user_id, plan_id, asset_code, amount,
                       metadata, transaction_hash, block_number,
                       event_timestamp, created_at
                FROM lending_events
                WHERE plan_id = $1 AND event_type = $2
                ORDER BY event_timestamp DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(plan_id)
            .bind(et)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, LendingEventRow>(
                r#"
                SELECT id, event_type, user_id, plan_id, asset_code, amount,
                       metadata, transaction_hash, block_number,
                       event_timestamp, created_at
                FROM lending_events
                WHERE plan_id = $1
                ORDER BY event_timestamp DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(plan_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        rows.into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    /// Get event by transaction hash
    pub async fn get_by_transaction_hash(
        pool: &PgPool,
        transaction_hash: &str,
    ) -> Result<Vec<LendingEvent>, ApiError> {
        let rows = sqlx::query_as::<_, LendingEventRow>(
            r#"
            SELECT id, event_type, user_id, plan_id, asset_code, amount,
                   metadata, transaction_hash, block_number,
                   event_timestamp, created_at
            FROM lending_events
            WHERE transaction_hash = $1
            ORDER BY event_timestamp DESC
            "#,
        )
        .bind(transaction_hash)
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<_>, _>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_deposit_metadata_serialization() {
        let metadata = DepositMetadata {
            collateral_ratio: Some(dec!(150.00)),
            total_deposited: dec!(1000.50),
        };

        let json = serde_json::to_value(&metadata).unwrap();
        assert!(json.is_object());
        assert_eq!(json["total_deposited"], "1000.50");
    }

    #[test]
    fn test_borrow_metadata_serialization() {
        let metadata = BorrowMetadata {
            interest_rate: dec!(5.5),
            collateral_asset: "USDC".to_string(),
            collateral_amount: dec!(1500.00),
            loan_to_value: dec!(75.00),
            maturity_date: None,
        };

        let json = serde_json::to_value(&metadata).unwrap();
        assert!(json.is_object());
        assert_eq!(json["collateral_asset"], "USDC");
    }

    #[test]
    fn test_repay_metadata_serialization() {
        let metadata = RepayMetadata {
            principal_amount: dec!(500.00),
            interest_amount: dec!(25.50),
            remaining_balance: dec!(474.50),
        };

        let json = serde_json::to_value(&metadata).unwrap();
        assert!(json.is_object());
        assert_eq!(json["principal_amount"], "500.00");
    }
}

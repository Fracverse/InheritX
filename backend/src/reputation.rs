use crate::api_error::ApiError;
use crate::events::EventType;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowerReputation {
    pub user_id: Uuid,
    pub score: i32,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_borrowed: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_repaid: Decimal,
    pub liquidation_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ReputationService;

impl ReputationService {
    /// Retrieve the borrower's current reputation profile.
    pub async fn get_reputation(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<BorrowerReputation, ApiError> {
        #[derive(sqlx::FromRow)]
        struct ReputationRow {
            user_id: Uuid,
            score: i32,
            total_borrowed: rust_decimal::Decimal,
            total_repaid: rust_decimal::Decimal,
            liquidation_count: i32,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let rep_row = sqlx::query_as::<_, ReputationRow>(
            r#"
            SELECT user_id, score, total_borrowed, total_repaid, liquidation_count, created_at, updated_at
            FROM borrower_reputation
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB Error: {e}")))?;

        if let Some(row) = rep_row {
            Ok(BorrowerReputation {
                user_id: row.user_id,
                score: row.score,
                total_borrowed: row.total_borrowed,
                total_repaid: row.total_repaid,
                liquidation_count: row.liquidation_count,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        } else {
            Ok(BorrowerReputation {
                user_id,
                score: 100, // Default base score
                total_borrowed: Decimal::ZERO,
                total_repaid: Decimal::ZERO,
                liquidation_count: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    /// Update reputation score based on an event type and amount.
    /// This should be called within the same transaction as the event insertion.
    pub async fn update_reputation<'a>(
        tx: &mut Transaction<'a, Postgres>,
        user_id: Uuid,
        event_type: EventType,
        amount: Decimal,
    ) -> Result<BorrowerReputation, ApiError> {
        let add_borrow = if event_type == EventType::Borrow {
            amount
        } else {
            Decimal::ZERO
        };
        let add_repay = if event_type == EventType::Repay {
            amount
        } else {
            Decimal::ZERO
        };
        let add_liquidation = if event_type == EventType::Liquidation {
            1
        } else {
            0
        };

        let score_adjustment = match event_type {
            EventType::Repay => 2,
            EventType::Liquidation => -50,
            _ => 0,
        };

        #[derive(sqlx::FromRow)]
        struct ReputationRow {
            user_id: Uuid,
            score: i32,
            total_borrowed: rust_decimal::Decimal,
            total_repaid: rust_decimal::Decimal,
            liquidation_count: i32,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, ReputationRow>(
            r#"
            INSERT INTO borrower_reputation (user_id, score, total_borrowed, total_repaid, liquidation_count)
            VALUES ($1, 100 + $2, $3, $4, $5)
            ON CONFLICT (user_id) DO UPDATE SET
                score = borrower_reputation.score + $2,
                total_borrowed = borrower_reputation.total_borrowed + $3,
                total_repaid = borrower_reputation.total_repaid + $4,
                liquidation_count = borrower_reputation.liquidation_count + $5
            RETURNING user_id, score, total_borrowed, total_repaid, liquidation_count, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(score_adjustment)
        .bind(add_borrow)
        .bind(add_repay)
        .bind(add_liquidation)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB Error updating reputation: {e}")))?;

        Ok(BorrowerReputation {
            user_id: row.user_id,
            score: row.score,
            total_borrowed: row.total_borrowed,
            total_repaid: row.total_repaid,
            liquidation_count: row.liquidation_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

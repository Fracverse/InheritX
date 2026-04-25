//! # Loan Lifecycle Tracker
//!
//! Tracks every loan through a well-defined state machine:
//!
//! ```text
//!              create_loan
//!                  │
//!              ┌───▼───┐
//!              │ACTIVE │
//!              └───┬───┘
//!         ┌────────┼────────┐
//!         │        │        │
//!    repay_loan  due_date  liquidate_loan
//!         │     exceeded        │
//!      ┌──▼──┐  ┌────────┐ ┌───▼────────┐
//!      │REPAID│  │OVERDUE │ │LIQUIDATED  │
//!      └──────┘  └────────┘ └────────────┘
//! ```
//!
//! Overdue status is set by calling
//! [`LoanLifecycleService::mark_overdue_loans`], which is designed to be
//! invoked periodically by a background sweep or cron job.

use crate::api_error::ApiError;
use crate::notifications::{audit_action, entity_type, AuditLogService};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Status enum
// ─────────────────────────────────────────────────────────────────────────────

/// The four lifecycle states a loan can occupy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoanStatus {
    Active,
    Repaid,
    Overdue,
    Liquidated,
}

impl LoanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoanStatus::Active => "active",
            LoanStatus::Repaid => "repaid",
            LoanStatus::Overdue => "overdue",
            LoanStatus::Liquidated => "liquidated",
        }
    }
}

impl fmt::Display for LoanStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for LoanStatus {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "active" => Ok(LoanStatus::Active),
            "repaid" => Ok(LoanStatus::Repaid),
            "overdue" => Ok(LoanStatus::Overdue),
            "liquidated" => Ok(LoanStatus::Liquidated),
            other => Err(ApiError::BadRequest(format!(
                "unknown loan status '{other}'; valid values: active, repaid, overdue, liquidated"
            ))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DB row / public record types
// ─────────────────────────────────────────────────────────────────────────────

/// Full record returned from the `loan_lifecycle` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoanLifecycleRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub borrow_asset: String,
    pub collateral_asset: String,
    pub principal: Decimal,
    pub interest_rate_bps: i32,
    pub collateral_amount: Decimal,
    pub amount_repaid: Decimal,
    pub status: String,
    pub due_date: DateTime<Utc>,
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub repaid_at: Option<DateTime<Utc>>,
    pub liquidated_at: Option<DateTime<Utc>>,
}

/// Raw sqlx row helper – mirrors the table schema exactly.
#[derive(sqlx::FromRow)]
pub(crate) struct LoanLifecycleRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub borrow_asset: String,
    pub collateral_asset: String,
    pub principal: Decimal,
    pub interest_rate_bps: i32,
    pub collateral_amount: Decimal,
    pub amount_repaid: Decimal,
    pub status: String,
    pub due_date: DateTime<Utc>,
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub repaid_at: Option<DateTime<Utc>>,
    pub liquidated_at: Option<DateTime<Utc>>,
}

impl From<LoanLifecycleRow> for LoanLifecycleRecord {
    fn from(r: LoanLifecycleRow) -> Self {
        LoanLifecycleRecord {
            id: r.id,
            user_id: r.user_id,
            plan_id: r.plan_id,
            borrow_asset: r.borrow_asset,
            collateral_asset: r.collateral_asset,
            principal: r.principal,
            interest_rate_bps: r.interest_rate_bps,
            collateral_amount: r.collateral_amount,
            amount_repaid: r.amount_repaid,
            status: r.status,
            due_date: r.due_date,
            transaction_hash: r.transaction_hash,
            created_at: r.created_at,
            updated_at: r.updated_at,
            repaid_at: r.repaid_at,
            liquidated_at: r.liquidated_at,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Request / filter types
// ─────────────────────────────────────────────────────────────────────────────

/// Payload required to open a new loan.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLoanRequest {
    pub user_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub borrow_asset: String,
    pub collateral_asset: String,
    /// Loan principal in the borrow asset's native units.
    pub principal: Decimal,
    /// Annual interest rate expressed in basis-points (e.g. 800 = 8 %).
    pub interest_rate_bps: i32,
    pub collateral_amount: Decimal,
    /// ISO-8601 datetime when the loan is due.
    pub due_date: DateTime<Utc>,
    /// Optional on-chain transaction hash for cross-reference.
    pub transaction_hash: Option<String>,
}

/// Filter parameters for listing loans.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoanListFilters {
    pub user_id: Option<Uuid>,
    pub plan_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Aggregate counts across all lifecycle states (useful for dashboards).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoanLifecycleSummary {
    pub total: i64,
    pub active: i64,
    pub repaid: i64,
    pub overdue: i64,
    pub liquidated: i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Service
// ─────────────────────────────────────────────────────────────────────────────

pub struct LoanLifecycleService;

impl LoanLifecycleService {
    // ── Read operations ───────────────────────────────────────────────────────

    /// Fetch a single loan by its `id`. Returns `NotFound` when absent.
    pub async fn get_loan(db: &PgPool, id: Uuid) -> Result<LoanLifecycleRecord, ApiError> {
        let row = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            SELECT id, user_id, plan_id, borrow_asset, collateral_asset,
                   principal, interest_rate_bps, collateral_amount, amount_repaid,
                   status, due_date, transaction_hash,
                   created_at, updated_at, repaid_at, liquidated_at
            FROM loan_lifecycle
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("loan {id} not found")))?;

        Ok(row.into())
    }

    /// List loans with optional filters. Results are ordered newest-first.
    pub async fn list_loans(
        db: &PgPool,
        filters: &LoanListFilters,
    ) -> Result<Vec<LoanLifecycleRecord>, ApiError> {
        // Build the query dynamically so we only add WHERE clauses that are
        // actually needed (avoids placeholder mis-alignment in dynamic SQL).
        let mut conditions: Vec<String> = Vec::new();
        let mut idx: i32 = 1;

        if filters.user_id.is_some() {
            conditions.push(format!("user_id = ${idx}"));
            idx += 1;
        }
        if filters.plan_id.is_some() {
            conditions.push(format!("plan_id = ${idx}"));
            idx += 1;
        }
        if filters.status.is_some() {
            conditions.push(format!("status = ${idx}::loan_lifecycle_status"));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT id, user_id, plan_id, borrow_asset, collateral_asset,
                   principal, interest_rate_bps, collateral_amount, amount_repaid,
                   status, due_date, transaction_hash,
                   created_at, updated_at, repaid_at, liquidated_at
            FROM loan_lifecycle
            {where_clause}
            ORDER BY created_at DESC
            "#
        );

        let mut query = sqlx::query_as::<_, LoanLifecycleRow>(&sql);

        if let Some(user_id) = filters.user_id {
            query = query.bind(user_id);
        }
        if let Some(plan_id) = filters.plan_id {
            query = query.bind(plan_id);
        }
        if let Some(ref status) = filters.status {
            query = query.bind(status.clone());
        }

        let rows = query.fetch_all(db).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// List loans with pagination and optional filters.
    pub async fn list_loans_paginated(
        db: &PgPool,
        filters: &LoanListFilters,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LoanLifecycleRecord>, ApiError> {
        let mut conditions: Vec<String> = Vec::new();
        let mut idx: i32 = 1;

        if filters.user_id.is_some() {
            conditions.push(format!("user_id = ${idx}"));
            idx += 1;
        }
        if filters.plan_id.is_some() {
            conditions.push(format!("plan_id = ${idx}"));
            idx += 1;
        }
        if filters.status.is_some() {
            conditions.push(format!("status = ${idx}::loan_lifecycle_status"));
            idx += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT id, user_id, plan_id, borrow_asset, collateral_asset,
                   principal, interest_rate_bps, collateral_amount, amount_repaid,
                   status, due_date, transaction_hash,
                   created_at, updated_at, repaid_at, liquidated_at
            FROM loan_lifecycle
            {where_clause}
            ORDER BY created_at DESC
            LIMIT ${idx} OFFSET ${idx_plus_1}
            "#,
            idx = idx,
            idx_plus_1 = idx + 1
        );

        let mut query = sqlx::query_as::<_, LoanLifecycleRow>(&sql);

        if let Some(user_id) = filters.user_id {
            query = query.bind(user_id);
        }
        if let Some(plan_id) = filters.plan_id {
            query = query.bind(plan_id);
        }
        if let Some(ref status) = filters.status {
            query = query.bind(status.clone());
        }
        query = query.bind(limit).bind(offset);

        let rows = query.fetch_all(db).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Count loans with optional filters.
    pub async fn count_loans(db: &PgPool, filters: &LoanListFilters) -> Result<i64, ApiError> {
        let mut conditions: Vec<String> = Vec::new();
        let mut idx: i32 = 1;

        if filters.user_id.is_some() {
            conditions.push(format!("user_id = ${idx}"));
            idx += 1;
        }
        if filters.plan_id.is_some() {
            conditions.push(format!("plan_id = ${idx}"));
            idx += 1;
        }
        if filters.status.is_some() {
            conditions.push(format!("status = ${idx}::loan_lifecycle_status"));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT COUNT(*)
            FROM loan_lifecycle
            {where_clause}
            "#
        );

        let mut query = sqlx::query_scalar::<_, i64>(&sql);

        if let Some(user_id) = filters.user_id {
            query = query.bind(user_id);
        }
        if let Some(plan_id) = filters.plan_id {
            query = query.bind(plan_id);
        }
        if let Some(ref status) = filters.status {
            query = query.bind(status.clone());
        }

        let count = query.fetch_one(db).await?;
        Ok(count)
    }

    /// Returns aggregate counts of loans grouped by status.
    pub async fn get_lifecycle_summary(
        db: &PgPool,
        user_id: Option<Uuid>,
    ) -> Result<LoanLifecycleSummary, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            total: i64,
            active: i64,
            repaid: i64,
            overdue: i64,
            liquidated: i64,
        }

        let row = if let Some(uid) = user_id {
            sqlx::query_as::<_, Row>(
                r#"
                SELECT
                    COUNT(*)::BIGINT                                                          AS total,
                    COUNT(*) FILTER (WHERE status = 'active')::BIGINT                        AS active,
                    COUNT(*) FILTER (WHERE status = 'repaid')::BIGINT                        AS repaid,
                    COUNT(*) FILTER (WHERE status = 'overdue')::BIGINT                       AS overdue,
                    COUNT(*) FILTER (WHERE status = 'liquidated')::BIGINT                    AS liquidated
                FROM loan_lifecycle
                WHERE user_id = $1
                "#,
            )
            .bind(uid)
            .fetch_one(db)
            .await?
        } else {
            sqlx::query_as::<_, Row>(
                r#"
                SELECT
                    COUNT(*)::BIGINT                                                          AS total,
                    COUNT(*) FILTER (WHERE status = 'active')::BIGINT                        AS active,
                    COUNT(*) FILTER (WHERE status = 'repaid')::BIGINT                        AS repaid,
                    COUNT(*) FILTER (WHERE status = 'overdue')::BIGINT                       AS overdue,
                    COUNT(*) FILTER (WHERE status = 'liquidated')::BIGINT                    AS liquidated
                FROM loan_lifecycle
                "#,
            )
            .fetch_one(db)
            .await?
        };

        Ok(LoanLifecycleSummary {
            total: row.total,
            active: row.active,
            repaid: row.repaid,
            overdue: row.overdue,
            liquidated: row.liquidated,
        })
    }

    // ── Write operations ──────────────────────────────────────────────────────

    /// Open a new loan in the `active` state.
    pub async fn create_loan(
        pool: &PgPool,
        req: &CreateLoanRequest,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        // Input validation
        if req.principal <= Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "principal must be greater than zero".to_string(),
            ));
        }
        if req.collateral_amount <= Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "collateral_amount must be greater than zero".to_string(),
            ));
        }
        if req.interest_rate_bps < 0 {
            return Err(ApiError::BadRequest(
                "interest_rate_bps must be non-negative".to_string(),
            ));
        }
        if req.due_date <= Utc::now() {
            return Err(ApiError::BadRequest(
                "due_date must be in the future".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // If plan_id is provided, check if the plan is paused
        if let Some(plan_id) = req.plan_id {
            let is_paused: Option<bool> =
                sqlx::query_scalar("SELECT is_paused FROM plans WHERE id = $1")
                    .bind(plan_id)
                    .fetch_optional(&mut *tx)
                    .await?
                    .flatten();

            if is_paused == Some(true) {
                return Err(ApiError::BadRequest(
                    "Cannot create a loan for a plan that is currently paused by an administrator"
                        .to_string(),
                ));
            }
        }

        let row = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            INSERT INTO loan_lifecycle (
                user_id, plan_id, borrow_asset, collateral_asset,
                principal, interest_rate_bps, collateral_amount,
                due_date, transaction_hash, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active')
            RETURNING id, user_id, plan_id, borrow_asset, collateral_asset,
                      principal, interest_rate_bps, collateral_amount, amount_repaid,
                      status, due_date, transaction_hash,
                      created_at, updated_at, repaid_at, liquidated_at
            "#,
        )
        .bind(req.user_id)
        .bind(req.plan_id)
        .bind(&req.borrow_asset)
        .bind(&req.collateral_asset)
        .bind(req.principal)
        .bind(req.interest_rate_bps)
        .bind(req.collateral_amount)
        .bind(req.due_date)
        .bind(&req.transaction_hash)
        .fetch_one(&mut *tx)
        .await?;

        let record: LoanLifecycleRecord = row.into();

        AuditLogService::log(
            &mut *tx,
            Some(req.user_id),
            None,
            audit_action::LOAN_CREATED,
            Some(record.id),
            Some(entity_type::LOAN),
            None,
            None,
            None,
        )
        .await?;

        tx.commit().await?;
        Ok(record)
    }

    /// Transition a loan from `active` or `overdue` → `repaid`.
    ///
    /// `amount` is the payment being applied. The transition is committed only
    /// when the cumulative `amount_repaid` reaches the full `principal`.
    pub async fn repay_loan(
        pool: &PgPool,
        loan_id: Uuid,
        user_id: Uuid,
        amount: Decimal,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        if amount <= Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "repayment amount must be greater than zero".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // Lock the row for the duration of the transaction
        // Join with plans to check if the plan is paused
        let row = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            SELECT ll.id, ll.user_id, ll.plan_id, ll.borrow_asset, ll.collateral_asset,
                   ll.principal, ll.interest_rate_bps, ll.collateral_amount, ll.amount_repaid,
                   ll.status, ll.due_date, ll.transaction_hash,
                   ll.created_at, ll.updated_at, ll.repaid_at, ll.liquidated_at
            FROM loan_lifecycle ll
            LEFT JOIN plans p ON p.id = ll.plan_id
            WHERE ll.id = $1 AND ll.user_id = $2
              AND (p.is_paused IS NULL OR p.is_paused = false)
            FOR UPDATE
            "#,
        )
        .bind(loan_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest(
                "Loan not found or its associated plan is paused by an administrator".to_string(),
            )
        })?;

        let current_status = LoanStatus::from_str(&row.status)?;
        if current_status == LoanStatus::Repaid || current_status == LoanStatus::Liquidated {
            return Err(ApiError::BadRequest(format!(
                "cannot repay a loan that is already {current_status}"
            )));
        }

        let new_amount_repaid = row.amount_repaid + amount;
        let fully_repaid = new_amount_repaid >= row.principal;

        let updated = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET amount_repaid  = $1,
                status         = CASE WHEN $2 THEN 'repaid'::loan_lifecycle_status
                                      ELSE status
                                 END,
                repaid_at      = CASE WHEN $2 THEN NOW() ELSE repaid_at END
            WHERE id = $3
            RETURNING id, user_id, plan_id, borrow_asset, collateral_asset,
                      principal, interest_rate_bps, collateral_amount, amount_repaid,
                      status, due_date, transaction_hash,
                      created_at, updated_at, repaid_at, liquidated_at
            "#,
        )
        .bind(new_amount_repaid)
        .bind(fully_repaid)
        .bind(loan_id)
        .fetch_one(&mut *tx)
        .await?;

        let record: LoanLifecycleRecord = updated.into();

        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            None,
            if fully_repaid {
                audit_action::LOAN_REPAID
            } else {
                audit_action::LOAN_PARTIAL_REPAYMENT
            },
            Some(loan_id),
            Some(entity_type::LOAN),
            None,
            None,
            None,
        )
        .await?;

        tx.commit().await?;
        Ok(record)
    }

    /// Transition a loan from `active` or `overdue` → `liquidated`.
    pub async fn liquidate_loan(
        pool: &PgPool,
        loan_id: Uuid,
        admin_id: Uuid,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        let mut tx = pool.begin().await?;

        let row = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            SELECT id, user_id, plan_id, borrow_asset, collateral_asset,
                   principal, interest_rate_bps, collateral_amount, amount_repaid,
                   status, due_date, transaction_hash,
                   created_at, updated_at, repaid_at, liquidated_at
            FROM loan_lifecycle
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(loan_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("loan {loan_id} not found")))?;

        let current_status = LoanStatus::from_str(&row.status)?;
        if current_status == LoanStatus::Repaid || current_status == LoanStatus::Liquidated {
            return Err(ApiError::BadRequest(format!(
                "cannot liquidate a loan that is already {current_status}"
            )));
        }

        let updated = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET status        = 'liquidated',
                liquidated_at = NOW()
            WHERE id = $1
            RETURNING id, user_id, plan_id, borrow_asset, collateral_asset,
                      principal, interest_rate_bps, collateral_amount, amount_repaid,
                      status, due_date, transaction_hash,
                      created_at, updated_at, repaid_at, liquidated_at
            "#,
        )
        .bind(loan_id)
        .fetch_one(&mut *tx)
        .await?;

        let record: LoanLifecycleRecord = updated.into();

        AuditLogService::log(
            &mut *tx,
            None,
            Some(admin_id),
            audit_action::LOAN_LIQUIDATED,
            Some(loan_id),
            Some(entity_type::LOAN),
            None,
            None,
            None,
        )
        .await?;

        tx.commit().await?;
        Ok(record)
    }

    /// Batch-mark all `active` loans whose `due_date` has passed as `overdue`.
    ///
    /// Designed to be called by a periodic background sweep (e.g. every minute).
    /// Returns the IDs of all loans that were transitioned.
    pub async fn mark_overdue_loans(pool: &PgPool) -> Result<Vec<Uuid>, ApiError> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            UPDATE loan_lifecycle
            SET status = 'overdue'
            WHERE status = 'active'
              AND due_date < NOW()
            RETURNING id
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }
}

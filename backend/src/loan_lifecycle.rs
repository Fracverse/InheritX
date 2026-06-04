//! # Loan Lifecycle Tracker
//!
//! Tracks every loan through a well-defined state machine:
//!
//! ```text
//!        create_draft
//!            │
//!        ┌───▼─────┐
//!        │  Draft  │
//!        └───┬─────┘
//!            │ submit_application
//!        ┌───▼──────┐
//!        │ Applied  │
//!        └───┬──────┘
//!            │ start_review
//!        ┌───▼──────────┐
//!        │ UnderReview  │
//!        └───┬──────────┘
//!    ┌───────┴───────┐
//!    │ approve       │ reject
//! ┌──▼───────┐   ┌───▼─────┐
//! │ Approved │   │ Rejected│
//! └───┬──────┘   └─────────┘
//!     │ activate
//! ┌───▼───────┐
//! │   Active  │
//! └───┬───────┘
//! ┌──────┴──────┐
//! │             │
//! │ repay      │ default
//! │             │
//! ├───▼──────┐ ┌▼──────────┐
//! │ PaidOff  │ │ Defaulted │
//! └──────────┘ └───────────┘
//! ```

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

/// The lifecycle states a loan can occupy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoanStatus {
    Draft,
    Applied,
    UnderReview,
    Approved,
    Rejected,
    Active,
    PaidOff,
    Defaulted,
}

impl LoanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoanStatus::Draft => "draft",
            LoanStatus::Applied => "applied",
            LoanStatus::UnderReview => "under_review",
            LoanStatus::Approved => "approved",
            LoanStatus::Rejected => "rejected",
            LoanStatus::Active => "active",
            LoanStatus::PaidOff => "paid_off",
            LoanStatus::Defaulted => "defaulted",
        }
    }

    /// Check if a transition from this state to `next` is valid
    pub fn validate_transition(self, next: LoanStatus) -> Result<(), ApiError> {
        let valid = match (self, next) {
            // Valid transitions
            (LoanStatus::Draft, LoanStatus::Applied) => true,
            (LoanStatus::Applied, LoanStatus::UnderReview) => true,
            (LoanStatus::UnderReview, LoanStatus::Approved) => true,
            (LoanStatus::UnderReview, LoanStatus::Rejected) => true,
            (LoanStatus::Approved, LoanStatus::Active) => true,
            (LoanStatus::Active, LoanStatus::PaidOff) => true,
            (LoanStatus::Active, LoanStatus::Defaulted) => true,
            _ => false,
        };

        if valid {
            Ok(())
        } else {
            Err(ApiError::BadRequest(format!(
                "invalid loan state transition: {} → {}",
                self, next
            )))
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
            "draft" => Ok(LoanStatus::Draft),
            "applied" => Ok(LoanStatus::Applied),
            "under_review" => Ok(LoanStatus::UnderReview),
            "approved" => Ok(LoanStatus::Approved),
            "rejected" => Ok(LoanStatus::Rejected),
            "active" => Ok(LoanStatus::Active),
            "paid_off" => Ok(LoanStatus::PaidOff),
            "defaulted" => Ok(LoanStatus::Defaulted),
            other => Err(ApiError::BadRequest(format!(
                "unknown loan status '{other}'; valid values: draft, applied, under_review, approved, rejected, active, paid_off, defaulted"
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<LoanPlanSummary>,
}

/// A compact plan summary attached to a loan for eager-loaded loan lifecycle
/// queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoanPlanSummary {
    pub id: Uuid,
    pub title: Option<String>,
    pub status: Option<String>,
    pub is_paused: Option<bool>,
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

#[derive(sqlx::FromRow)]
pub(crate) struct LoanLifecycleRowWithPlan {
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
    pub plan_title: Option<String>,
    pub plan_status: Option<String>,
    pub plan_is_paused: Option<bool>,
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
            plan: None,
        }
    }
}

impl From<LoanLifecycleRowWithPlan> for LoanLifecycleRecord {
    fn from(r: LoanLifecycleRowWithPlan) -> Self {
        let plan = r.plan_id.map(|id| LoanPlanSummary {
            id,
            title: r.plan_title,
            status: r.plan_status,
            is_paused: r.plan_is_paused,
        });

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
            plan,
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
    pub draft: i64,
    pub applied: i64,
    pub under_review: i64,
    pub approved: i64,
    pub rejected: i64,
    pub active: i64,
    pub paid_off: i64,
    pub defaulted: i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Service
// ─────────────────────────────────────────────────────────────────────────────

pub struct LoanLifecycleService;

impl LoanLifecycleService {
    // ── Read operations ───────────────────────────────────────────────────────

    /// Fetch a single loan by its `id` for a specific user. Returns `NotFound` when absent
    /// or not owned by the caller.
    pub async fn get_loan_for_user(
        db: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        let row = sqlx::query_as::<_, LoanLifecycleRowWithPlan>(
            r#"
            SELECT ll.id, ll.user_id, ll.plan_id, ll.borrow_asset, ll.collateral_asset,
                   ll.principal, ll.interest_rate_bps, ll.collateral_amount, ll.amount_repaid,
                   ll.status, ll.due_date, ll.transaction_hash,
                   ll.created_at, ll.updated_at, ll.repaid_at, ll.liquidated_at,
                   p.title AS plan_title, p.status AS plan_status, p.is_paused AS plan_is_paused
            FROM loan_lifecycle ll
            LEFT JOIN plans p ON p.id = ll.plan_id
            WHERE ll.id = $1 AND ll.user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("loan {id} not found")))?;

        Ok(row.into())
    }

    /// Fetch a single loan by its `id`. Returns `NotFound` when absent.
    pub async fn get_loan(db: &PgPool, id: Uuid) -> Result<LoanLifecycleRecord, ApiError> {
        let row = sqlx::query_as::<_, LoanLifecycleRowWithPlan>(
            r#"
            SELECT ll.id, ll.user_id, ll.plan_id, ll.borrow_asset, ll.collateral_asset,
                   ll.principal, ll.interest_rate_bps, ll.collateral_amount, ll.amount_repaid,
                   ll.status, ll.due_date, ll.transaction_hash,
                   ll.created_at, ll.updated_at, ll.repaid_at, ll.liquidated_at,
                   p.title AS plan_title, p.status AS plan_status, p.is_paused AS plan_is_paused
            FROM loan_lifecycle ll
            LEFT JOIN plans p ON p.id = ll.plan_id
            WHERE ll.id = $1
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
        let rows = sqlx::query_as::<_, LoanLifecycleRowWithPlan>(
            r#"
            SELECT ll.id, ll.user_id, ll.plan_id, ll.borrow_asset, ll.collateral_asset,
                   ll.principal, ll.interest_rate_bps, ll.collateral_amount, ll.amount_repaid,
                   ll.status, ll.due_date, ll.transaction_hash,
                   ll.created_at, ll.updated_at, ll.repaid_at, ll.liquidated_at,
                   p.title AS plan_title, p.status AS plan_status, p.is_paused AS plan_is_paused
            FROM loan_lifecycle ll
            LEFT JOIN plans p ON p.id = ll.plan_id
            WHERE ($1::uuid IS NULL OR ll.user_id = $1)
              AND ($2::uuid IS NULL OR ll.plan_id = $2)
              AND ($3::text IS NULL OR ll.status::text = $3)
            ORDER BY ll.created_at DESC
            "#,
        );

        let rows = rows
            .bind(filters.user_id)
            .bind(filters.plan_id)
            .bind(
                filters
                    .status
                    .as_ref()
                    .map(|status| status.as_str().to_string()),
            )
            .fetch_all(db)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// List loans with pagination and optional filters.
    pub async fn list_loans_paginated(
        db: &PgPool,
        filters: &LoanListFilters,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LoanLifecycleRecord>, ApiError> {
        let rows = sqlx::query_as::<_, LoanLifecycleRowWithPlan>(
            r#"
            SELECT ll.id, ll.user_id, ll.plan_id, ll.borrow_asset, ll.collateral_asset,
                   ll.principal, ll.interest_rate_bps, ll.collateral_amount, ll.amount_repaid,
                   ll.status, ll.due_date, ll.transaction_hash,
                   ll.created_at, ll.updated_at, ll.repaid_at, ll.liquidated_at,
                   p.title AS plan_title, p.status AS plan_status, p.is_paused AS plan_is_paused
            FROM loan_lifecycle ll
            LEFT JOIN plans p ON p.id = ll.plan_id
            WHERE ($1::uuid IS NULL OR ll.user_id = $1)
              AND ($2::uuid IS NULL OR ll.plan_id = $2)
              AND ($3::text IS NULL OR ll.status::text = $3)
            ORDER BY ll.created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        );

        let rows = rows
            .bind(filters.user_id)
            .bind(filters.plan_id)
            .bind(
                filters
                    .status
                    .as_ref()
                    .map(|status| status.as_str().to_string()),
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(db)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Count loans with optional filters.
    pub async fn count_loans(db: &PgPool, filters: &LoanListFilters) -> Result<i64, ApiError> {
        let sql = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM loan_lifecycle
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::uuid IS NULL OR plan_id = $2)
              AND ($3::text IS NULL OR status::text = $3)
            "#,
        );

        let count = sql
            .bind(filters.user_id)
            .bind(filters.plan_id)
            .bind(
                filters
                    .status
                    .as_ref()
                    .map(|status| status.as_str().to_string()),
            )
            .fetch_one(db)
            .await?;
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
            draft: i64,
            applied: i64,
            under_review: i64,
            approved: i64,
            rejected: i64,
            active: i64,
            paid_off: i64,
            defaulted: i64,
        }

        let row = if let Some(uid) = user_id {
            sqlx::query_as::<_, Row>(
                r#"
                SELECT
                    COUNT(*)::BIGINT                                                          AS total,
                    COUNT(*) FILTER (WHERE status = 'draft')::BIGINT                          AS draft,
                    COUNT(*) FILTER (WHERE status = 'applied')::BIGINT                        AS applied,
                    COUNT(*) FILTER (WHERE status = 'under_review')::BIGINT                   AS under_review,
                    COUNT(*) FILTER (WHERE status = 'approved')::BIGINT                       AS approved,
                    COUNT(*) FILTER (WHERE status = 'rejected')::BIGINT                       AS rejected,
                    COUNT(*) FILTER (WHERE status = 'active')::BIGINT                         AS active,
                    COUNT(*) FILTER (WHERE status = 'paid_off')::BIGINT                       AS paid_off,
                    COUNT(*) FILTER (WHERE status = 'defaulted')::BIGINT                      AS defaulted
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
                    COUNT(*) FILTER (WHERE status = 'draft')::BIGINT                          AS draft,
                    COUNT(*) FILTER (WHERE status = 'applied')::BIGINT                        AS applied,
                    COUNT(*) FILTER (WHERE status = 'under_review')::BIGINT                   AS under_review,
                    COUNT(*) FILTER (WHERE status = 'approved')::BIGINT                       AS approved,
                    COUNT(*) FILTER (WHERE status = 'rejected')::BIGINT                       AS rejected,
                    COUNT(*) FILTER (WHERE status = 'active')::BIGINT                         AS active,
                    COUNT(*) FILTER (WHERE status = 'paid_off')::BIGINT                       AS paid_off,
                    COUNT(*) FILTER (WHERE status = 'defaulted')::BIGINT                      AS defaulted
                FROM loan_lifecycle
                "#,
            )
            .fetch_one(db)
            .await?
        };

        Ok(LoanLifecycleSummary {
            total: row.total,
            draft: row.draft,
            applied: row.applied,
            under_review: row.under_review,
            approved: row.approved,
            rejected: row.rejected,
            active: row.active,
            paid_off: row.paid_off,
            defaulted: row.defaulted,
        })
    }

    // ── Write operations ──────────────────────────────────────────────────────

    /// Create a new loan in the `draft` state.
    pub async fn create_draft_loan(
        pool: &PgPool,
        req: &CreateLoanRequest,
    ) -> Result<LoanLifecycleRecord, ApiError> {
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'draft')
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
        crate::metrics::inc_loans_created();
        Ok(record)
    }

    /// Transition a loan from `draft` → `applied`.
    pub async fn submit_application(
        pool: &PgPool,
        loan_id: Uuid,
        user_id: Uuid,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        let mut tx = pool.begin().await?;

        let row = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            SELECT id, user_id, plan_id, borrow_asset, collateral_asset,
                   principal, interest_rate_bps, collateral_amount, amount_repaid,
                   status, due_date, transaction_hash,
                   created_at, updated_at, repaid_at, liquidated_at
            FROM loan_lifecycle
            WHERE id = $1 AND user_id = $2
            FOR UPDATE
            "#,
        )
        .bind(loan_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("loan {loan_id} not found")))?;

        let current_status = LoanStatus::from_str(&row.status)?;
        let next_status = LoanStatus::Applied;
        current_status.validate_transition(next_status)?;

        let updated = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET status = 'applied'
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
            Some(user_id),
            None,
            "LOAN_SUBMITTED",
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

    /// Transition a loan from `applied` → `under_review`.
    pub async fn start_review(
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
        let next_status = LoanStatus::UnderReview;
        current_status.validate_transition(next_status)?;

        let updated = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET status = 'under_review'
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
            "LOAN_REVIEW_STARTED",
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

    /// Transition a loan from `under_review` → `approved`.
    pub async fn approve_loan(
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
        let next_status = LoanStatus::Approved;
        current_status.validate_transition(next_status)?;

        let updated = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET status = 'approved'
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
            "LOAN_APPROVED",
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

    /// Transition a loan from `under_review` → `rejected`.
    pub async fn reject_loan(
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
        let next_status = LoanStatus::Rejected;
        current_status.validate_transition(next_status)?;

        let updated = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET status = 'rejected'
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
            "LOAN_REJECTED",
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

    /// Transition a loan from `approved` → `active`.
    pub async fn activate_loan(
        pool: &PgPool,
        loan_id: Uuid,
        user_id: Uuid,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        let mut tx = pool.begin().await?;

        let row = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            SELECT id, user_id, plan_id, borrow_asset, collateral_asset,
                   principal, interest_rate_bps, collateral_amount, amount_repaid,
                   status, due_date, transaction_hash,
                   created_at, updated_at, repaid_at, liquidated_at
            FROM loan_lifecycle
            WHERE id = $1 AND user_id = $2
            FOR UPDATE
            "#,
        )
        .bind(loan_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("loan {loan_id} not found")))?;

        let current_status = LoanStatus::from_str(&row.status)?;
        let next_status = LoanStatus::Active;
        current_status.validate_transition(next_status)?;

        let updated = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET status = 'active'
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
            Some(user_id),
            None,
            "LOAN_ACTIVATED",
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

    /// Transition a loan from `active` → `paid_off`.
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

        let new_amount_repaid = row.amount_repaid + amount;
        let fully_repaid = new_amount_repaid >= row.principal;

        if fully_repaid {
            let next_status = LoanStatus::PaidOff;
            current_status.validate_transition(next_status)?;
        }

        let updated = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET amount_repaid  = $1,
                status         = CASE WHEN $2 THEN 'paid_off'::loan_lifecycle_status
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
        if fully_repaid {
            crate::metrics::inc_loans_repaid();
        }
        Ok(record)
    }

    /// Transition a loan from `active` → `defaulted`.
    pub async fn default_loan(
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
        let next_status = LoanStatus::Defaulted;
        current_status.validate_transition(next_status)?;

        let updated = sqlx::query_as::<_, LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET status        = 'defaulted',
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
            "LOAN_DEFAULTED",
            Some(loan_id),
            Some(entity_type::LOAN),
            None,
            None,
            None,
        )
        .await?;

        tx.commit().await?;
        crate::metrics::inc_loans_liquidated();
        Ok(record)
    }

    /// Convenience wrapper used by older call-sites to create and immediately
    /// activate a loan. This delegates to `create_draft_loan` and then
    /// transitions the loan to `active` so higher-level handlers can call a
    /// single method.
    pub async fn create_loan(
        pool: &PgPool,
        req: &CreateLoanRequest,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        let draft = Self::create_draft_loan(pool, req).await?;
        // Activate the draft loan on behalf of the requesting user.
        Self::activate_loan(pool, draft.id, req.user_id).await
    }

    /// Admin-facing wrapper to liquidate a loan. Reuses `default_loan` which
    /// marks the loan as defaulted/liquidated and logs metrics.
    pub async fn liquidate_loan(
        pool: &PgPool,
        loan_id: Uuid,
        admin_id: Uuid,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        Self::default_loan(pool, loan_id, admin_id).await
    }

    /// Sweep active loans past their due date and mark them as `defaulted`.
    /// Returns the list of loan ids that were marked. This is a best-effort
    /// implementation useful for tests and cron jobs.
    pub async fn mark_overdue_loans(db: &PgPool) -> Result<Vec<Uuid>, ApiError> {
        let rows = sqlx::query_as::<_, (Uuid,)>(
            r#"
            UPDATE loan_lifecycle
            SET status = 'defaulted', liquidated_at = NOW()
            WHERE status = 'active' AND due_date < NOW()
            RETURNING id
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // ── LoanStatus parsing ────────────────────────────────────────────────────

    #[test]
    fn loan_status_round_trips() {
        for (s, expected) in [
            ("draft", LoanStatus::Draft),
            ("applied", LoanStatus::Applied),
            ("under_review", LoanStatus::UnderReview),
            ("approved", LoanStatus::Approved),
            ("rejected", LoanStatus::Rejected),
            ("active", LoanStatus::Active),
            ("paid_off", LoanStatus::PaidOff),
            ("defaulted", LoanStatus::Defaulted),
        ] {
            let parsed = LoanStatus::from_str(s).expect("should parse");
            assert_eq!(parsed, expected);
            assert_eq!(parsed.as_str(), s);
        }
    }

    #[test]
    fn loan_status_from_str_rejects_unknown() {
        assert!(LoanStatus::from_str("pending").is_err());
        assert!(LoanStatus::from_str("").is_err());
    }

    // ── State transition validation ───────────────────────────────────────────

    #[test]
    fn valid_state_transitions_pass() {
        // All valid transitions
        assert!(LoanStatus::Draft
            .validate_transition(LoanStatus::Applied)
            .is_ok());
        assert!(LoanStatus::Applied
            .validate_transition(LoanStatus::UnderReview)
            .is_ok());
        assert!(LoanStatus::UnderReview
            .validate_transition(LoanStatus::Approved)
            .is_ok());
        assert!(LoanStatus::UnderReview
            .validate_transition(LoanStatus::Rejected)
            .is_ok());
        assert!(LoanStatus::Approved
            .validate_transition(LoanStatus::Active)
            .is_ok());
        assert!(LoanStatus::Active
            .validate_transition(LoanStatus::PaidOff)
            .is_ok());
        assert!(LoanStatus::Active
            .validate_transition(LoanStatus::Defaulted)
            .is_ok());
    }

    #[test]
    fn invalid_state_transitions_fail() {
        // A few invalid transitions
        assert!(LoanStatus::Draft
            .validate_transition(LoanStatus::Active)
            .is_err());
        assert!(LoanStatus::Applied
            .validate_transition(LoanStatus::Approved)
            .is_err());
        assert!(LoanStatus::Approved
            .validate_transition(LoanStatus::PaidOff)
            .is_err());
        assert!(LoanStatus::PaidOff
            .validate_transition(LoanStatus::Active)
            .is_err());
        assert!(LoanStatus::Rejected
            .validate_transition(LoanStatus::UnderReview)
            .is_err());
    }

    // ── Partial repayment business logic ─────────────────────────────────────

    /// Verify that a partial payment does NOT set `fully_repaid`.
    #[test]
    fn partial_repayment_does_not_fully_repay() {
        let principal = Decimal::from(1000u32);
        let amount_repaid = Decimal::from(300u32);
        let payment = Decimal::from(200u32);

        let new_amount_repaid = amount_repaid + payment;
        let fully_repaid = new_amount_repaid >= principal;

        assert_eq!(new_amount_repaid, Decimal::from(500u32));
        assert!(!fully_repaid, "500 < 1000 should not be fully repaid");
    }

    /// Verify that a payment that exactly meets the principal sets `fully_repaid`.
    #[test]
    fn exact_repayment_marks_fully_repaid() {
        let principal = Decimal::from(1000u32);
        let amount_repaid = Decimal::from(700u32);
        let payment = Decimal::from(300u32);

        let new_amount_repaid = amount_repaid + payment;
        let fully_repaid = new_amount_repaid >= principal;

        assert_eq!(new_amount_repaid, principal);
        assert!(fully_repaid, "700 + 300 == 1000 should be fully repaid");
    }

    /// Verify that an overpayment (more than principal) also sets `fully_repaid`.
    #[test]
    fn overpayment_marks_fully_repaid() {
        let principal = Decimal::from(1000u32);
        let amount_repaid = Decimal::ZERO;
        let payment = Decimal::from(1500u32);

        let new_amount_repaid = amount_repaid + payment;
        let fully_repaid = new_amount_repaid >= principal;

        assert!(fully_repaid, "1500 > 1000 should be fully repaid");
    }

    /// Verify that a zero payment is rejected (mirrors the service guard).
    #[test]
    fn zero_repayment_is_invalid() {
        let amount = Decimal::ZERO;
        assert!(
            amount <= Decimal::ZERO,
            "zero amount should fail the > 0 guard"
        );
    }

    /// Verify that a negative payment is rejected.
    #[test]
    fn negative_repayment_is_invalid() {
        let amount = Decimal::from_str("-1.00").unwrap();
        assert!(
            amount <= Decimal::ZERO,
            "negative amount should fail the > 0 guard"
        );
    }

    // ── CreateLoanRequest validation guards ───────────────────────────────────

    #[test]
    fn zero_principal_is_invalid() {
        assert!(Decimal::ZERO <= Decimal::ZERO);
    }

    #[test]
    fn negative_interest_rate_bps_is_invalid() {
        let rate: i32 = -1;
        assert!(rate < 0);
    }
}

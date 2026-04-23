//! # Contingent Beneficiary Service
//!
//! Manages backup beneficiaries for inheritance plans with automatic activation
//! when primary beneficiaries cannot claim.

use crate::api_error::ApiError;
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Enums
// ─────────────────────────────────────────────────────────────────────────────

/// Beneficiary type: Primary or Contingent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "beneficiary_type", rename_all = "lowercase")]
pub enum BeneficiaryType {
    Primary,
    Contingent,
}

impl BeneficiaryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BeneficiaryType::Primary => "primary",
            BeneficiaryType::Contingent => "contingent",
        }
    }
}

impl fmt::Display for BeneficiaryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for BeneficiaryType {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "primary" => Ok(BeneficiaryType::Primary),
            "contingent" => Ok(BeneficiaryType::Contingent),
            other => Err(ApiError::BadRequest(format!(
                "unknown beneficiary type '{other}'; valid values: primary, contingent"
            ))),
        }
    }
}

/// Contingency activation condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "contingency_condition", rename_all = "snake_case")]
pub enum ContingencyCondition {
    PrimaryDeclined,
    PrimaryDeceased,
    PrimaryTimeout,
    ManualPromotion,
}

impl ContingencyCondition {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContingencyCondition::PrimaryDeclined => "primary_declined",
            ContingencyCondition::PrimaryDeceased => "primary_deceased",
            ContingencyCondition::PrimaryTimeout => "primary_timeout",
            ContingencyCondition::ManualPromotion => "manual_promotion",
        }
    }
}

impl fmt::Display for ContingencyCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ContingencyCondition {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "primary_declined" => Ok(ContingencyCondition::PrimaryDeclined),
            "primary_deceased" => Ok(ContingencyCondition::PrimaryDeceased),
            "primary_timeout" => Ok(ContingencyCondition::PrimaryTimeout),
            "manual_promotion" => Ok(ContingencyCondition::ManualPromotion),
            other => Err(ApiError::BadRequest(format!(
                "unknown contingency condition '{other}'"
            ))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Data Structures
// ─────────────────────────────────────────────────────────────────────────────

/// Contingent beneficiary record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContingentBeneficiary {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub wallet_address: String,
    pub allocation_percent: Decimal,
    pub name: Option<String>,
    pub relationship: Option<String>,
    pub beneficiary_type: String,
    pub priority_order: i32,
    pub is_active: bool,
    pub activated_at: Option<DateTime<Utc>>,
    pub activation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to add a contingent beneficiary
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddContingentBeneficiaryRequest {
    pub plan_id: Uuid,
    pub wallet_address: String,
    pub allocation_percent: Decimal,
    pub name: Option<String>,
    pub relationship: Option<String>,
    pub priority_order: i32,
}

/// Request to remove a contingent beneficiary
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveContingentBeneficiaryRequest {
    pub beneficiary_id: Uuid,
}

/// Request to set contingency conditions
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetContingencyConditionsRequest {
    pub plan_id: Uuid,
    pub primary_beneficiary_id: Uuid,
    pub contingent_beneficiary_id: Uuid,
    pub condition_type: String,
    pub timeout_days: Option<i32>,
}

/// Request to promote contingent to primary
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoteContingentRequest {
    pub beneficiary_id: Uuid,
    pub reason: String,
}

/// Contingency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContingencyConfig {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub primary_claim_timeout_days: i32,
    pub contingent_claim_timeout_days: i32,
    pub auto_activate_on_timeout: bool,
    pub require_manual_confirmation: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Contingent activation event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContingentActivationEvent {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub primary_beneficiary_id: Uuid,
    pub contingent_beneficiary_id: Uuid,
    pub activation_reason: String,
    pub activated_by_user_id: Option<Uuid>,
    pub activated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Database Row Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct BeneficiaryRow {
    id: Uuid,
    plan_id: Uuid,
    wallet_address: String,
    allocation_percent: Decimal,
    name: Option<String>,
    relationship: Option<String>,
    beneficiary_type: String,
    priority_order: i32,
    is_active: bool,
    activated_at: Option<DateTime<Utc>>,
    activation_reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<BeneficiaryRow> for ContingentBeneficiary {
    fn from(r: BeneficiaryRow) -> Self {
        ContingentBeneficiary {
            id: r.id,
            plan_id: r.plan_id,
            wallet_address: r.wallet_address,
            allocation_percent: r.allocation_percent,
            name: r.name,
            relationship: r.relationship,
            beneficiary_type: r.beneficiary_type,
            priority_order: r.priority_order,
            is_active: r.is_active,
            activated_at: r.activated_at,
            activation_reason: r.activation_reason,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ContingencyConfigRow {
    id: Uuid,
    plan_id: Uuid,
    primary_claim_timeout_days: i32,
    contingent_claim_timeout_days: i32,
    auto_activate_on_timeout: bool,
    require_manual_confirmation: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ContingencyConfigRow> for ContingencyConfig {
    fn from(r: ContingencyConfigRow) -> Self {
        ContingencyConfig {
            id: r.id,
            plan_id: r.plan_id,
            primary_claim_timeout_days: r.primary_claim_timeout_days,
            contingent_claim_timeout_days: r.contingent_claim_timeout_days,
            auto_activate_on_timeout: r.auto_activate_on_timeout,
            require_manual_confirmation: r.require_manual_confirmation,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ActivationEventRow {
    id: Uuid,
    plan_id: Uuid,
    primary_beneficiary_id: Uuid,
    contingent_beneficiary_id: Uuid,
    activation_reason: String,
    activated_by_user_id: Option<Uuid>,
    activated_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<ActivationEventRow> for ContingentActivationEvent {
    fn from(r: ActivationEventRow) -> Self {
        ContingentActivationEvent {
            id: r.id,
            plan_id: r.plan_id,
            primary_beneficiary_id: r.primary_beneficiary_id,
            contingent_beneficiary_id: r.contingent_beneficiary_id,
            activation_reason: r.activation_reason,
            activated_by_user_id: r.activated_by_user_id,
            activated_at: r.activated_at,
            created_at: r.created_at,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Service
// ─────────────────────────────────────────────────────────────────────────────

pub struct ContingentBeneficiaryService;

impl ContingentBeneficiaryService {
    /// Add a contingent beneficiary to a plan
    pub async fn add_contingent_beneficiary(
        pool: &PgPool,
        user_id: Uuid,
        req: &AddContingentBeneficiaryRequest,
    ) -> Result<ContingentBeneficiary, ApiError> {
        // Validation
        if req.allocation_percent <= Decimal::ZERO || req.allocation_percent > Decimal::from(100) {
            return Err(ApiError::BadRequest(
                "allocation_percent must be between 0 and 100".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // Verify plan exists and belongs to user
        let plan_exists: Option<bool> =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM plans WHERE id = $1 AND user_id = $2)")
                .bind(req.plan_id)
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?;

        if plan_exists != Some(true) {
            return Err(ApiError::NotFound(format!(
                "Plan {} not found",
                req.plan_id
            )));
        }

        // Insert contingent beneficiary
        let row = sqlx::query_as::<_, BeneficiaryRow>(
            r#"
            INSERT INTO plan_beneficiaries (
                plan_id, wallet_address, allocation_percent, name, relationship,
                beneficiary_type, priority_order, is_active
            )
            VALUES ($1, $2, $3, $4, $5, 'contingent', $6, true)
            RETURNING id, plan_id, wallet_address, allocation_percent, name, relationship,
                      beneficiary_type::text, priority_order, is_active, activated_at, activation_reason,
                      created_at, updated_at
            "#,
        )
        .bind(req.plan_id)
        .bind(&req.wallet_address)
        .bind(req.allocation_percent)
        .bind(&req.name)
        .bind(&req.relationship)
        .bind(req.priority_order)
        .fetch_one(&mut *tx)
        .await?;

        let beneficiary: ContingentBeneficiary = row.into();

        // Audit log
        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            None,
            audit_action::CONTINGENT_BENEFICIARY_ADDED,
            Some(req.plan_id),
            Some(entity_type::PLAN),
            None,
            None,
            None,
        )
        .await?;

        // Notification
        NotificationService::create(
            &mut tx,
            user_id,
            notif_type::CONTINGENT_BENEFICIARY_ADDED,
            format!(
                "Contingent beneficiary {} added to plan",
                req.wallet_address
            ),
        )
        .await?;

        tx.commit().await?;
        Ok(beneficiary)
    }

    /// Remove a contingent beneficiary
    pub async fn remove_contingent_beneficiary(
        pool: &PgPool,
        user_id: Uuid,
        req: &RemoveContingentBeneficiaryRequest,
    ) -> Result<(), ApiError> {
        let mut tx = pool.begin().await?;

        // Verify beneficiary exists and belongs to user's plan
        let beneficiary: Option<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT pb.plan_id, pb.beneficiary_type::text
            FROM plan_beneficiaries pb
            JOIN plans p ON p.id = pb.plan_id
            WHERE pb.id = $1 AND p.user_id = $2
            "#,
        )
        .bind(req.beneficiary_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;

        let (plan_id, beneficiary_type) = beneficiary.ok_or_else(|| {
            ApiError::NotFound(format!("Beneficiary {} not found", req.beneficiary_id))
        })?;

        // Only allow removing contingent beneficiaries
        if beneficiary_type != "contingent" {
            return Err(ApiError::BadRequest(
                "Can only remove contingent beneficiaries".to_string(),
            ));
        }

        // Delete beneficiary
        sqlx::query("DELETE FROM plan_beneficiaries WHERE id = $1")
            .bind(req.beneficiary_id)
            .execute(&mut *tx)
            .await?;

        // Audit log
        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            None,
            audit_action::CONTINGENT_BENEFICIARY_REMOVED,
            Some(plan_id),
            Some(entity_type::PLAN),
            None,
            None,
            None,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Get all contingent beneficiaries for a plan
    pub async fn get_contingent_beneficiaries(
        pool: &PgPool,
        plan_id: Uuid,
    ) -> Result<Vec<ContingentBeneficiary>, ApiError> {
        let rows = sqlx::query_as::<_, BeneficiaryRow>(
            r#"
            SELECT id, plan_id, wallet_address, allocation_percent, name, relationship,
                   beneficiary_type::text, priority_order, is_active, activated_at, activation_reason,
                   created_at, updated_at
            FROM plan_beneficiaries
            WHERE plan_id = $1 AND beneficiary_type = 'contingent'
            ORDER BY priority_order ASC, created_at ASC
            "#,
        )
        .bind(plan_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Set contingency conditions
    pub async fn set_contingency_conditions(
        pool: &PgPool,
        user_id: Uuid,
        req: &SetContingencyConditionsRequest,
    ) -> Result<(), ApiError> {
        let condition = ContingencyCondition::from_str(&req.condition_type)?;

        // Validate timeout_days for timeout condition
        if condition == ContingencyCondition::PrimaryTimeout && req.timeout_days.is_none() {
            return Err(ApiError::BadRequest(
                "timeout_days required for primary_timeout condition".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // Verify plan belongs to user
        let plan_exists: Option<bool> =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM plans WHERE id = $1 AND user_id = $2)")
                .bind(req.plan_id)
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?;

        if plan_exists != Some(true) {
            return Err(ApiError::NotFound(format!(
                "Plan {} not found",
                req.plan_id
            )));
        }

        // Insert or update contingency condition
        sqlx::query(
            r#"
            INSERT INTO contingency_conditions (
                plan_id, primary_beneficiary_id, contingent_beneficiary_id,
                condition_type, timeout_days, is_active
            )
            VALUES ($1, $2, $3, $4, $5, true)
            ON CONFLICT (plan_id, primary_beneficiary_id, contingent_beneficiary_id, condition_type)
            DO UPDATE SET timeout_days = $5, is_active = true, updated_at = NOW()
            "#,
        )
        .bind(req.plan_id)
        .bind(req.primary_beneficiary_id)
        .bind(req.contingent_beneficiary_id)
        .bind(condition.as_str())
        .bind(req.timeout_days)
        .execute(&mut *tx)
        .await?;

        // Audit log
        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            None,
            audit_action::CONTINGENCY_CONDITIONS_SET,
            Some(req.plan_id),
            Some(entity_type::PLAN),
            None,
            None,
            None,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Promote contingent beneficiary to primary
    pub async fn promote_contingent(
        pool: &PgPool,
        user_id: Uuid,
        req: &PromoteContingentRequest,
    ) -> Result<ContingentBeneficiary, ApiError> {
        let mut tx = pool.begin().await?;

        // Fetch beneficiary and verify it's contingent
        let beneficiary: Option<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT pb.plan_id, pb.beneficiary_type::text, p.user_id::text
            FROM plan_beneficiaries pb
            JOIN plans p ON p.id = pb.plan_id
            WHERE pb.id = $1
            "#,
        )
        .bind(req.beneficiary_id)
        .fetch_optional(&mut *tx)
        .await?;

        let (plan_id, beneficiary_type, plan_user_id_str) = beneficiary.ok_or_else(|| {
            ApiError::NotFound(format!("Beneficiary {} not found", req.beneficiary_id))
        })?;

        let plan_user_id = Uuid::parse_str(&plan_user_id_str)
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("Invalid user_id")))?;

        // Verify user owns the plan
        if plan_user_id != user_id {
            return Err(ApiError::Forbidden("Not authorized".to_string()));
        }

        if beneficiary_type != "contingent" {
            return Err(ApiError::BadRequest(
                "Beneficiary is not contingent type".to_string(),
            ));
        }

        // Promote to primary
        let updated_row = sqlx::query_as::<_, BeneficiaryRow>(
            r#"
            UPDATE plan_beneficiaries
            SET beneficiary_type = 'primary',
                activated_at = NOW(),
                activation_reason = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, plan_id, wallet_address, allocation_percent, name, relationship,
                      beneficiary_type::text, priority_order, is_active, activated_at, activation_reason,
                      created_at, updated_at
            "#,
        )
        .bind(req.beneficiary_id)
        .bind(&req.reason)
        .fetch_one(&mut *tx)
        .await?;

        let promoted: ContingentBeneficiary = updated_row.into();

        // Record promotion history
        sqlx::query(
            r#"
            INSERT INTO contingent_promotions (
                plan_id, beneficiary_id, promoted_from_type, promoted_to_type,
                promotion_reason, promoted_by_user_id
            )
            VALUES ($1, $2, 'contingent', 'primary', $3, $4)
            "#,
        )
        .bind(plan_id)
        .bind(req.beneficiary_id)
        .bind(&req.reason)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Record activation event
        sqlx::query(
            r#"
            INSERT INTO contingent_activation_events (
                plan_id, primary_beneficiary_id, contingent_beneficiary_id,
                activation_reason, activated_by_user_id
            )
            VALUES ($1, $2, $2, 'manual_promotion', $3)
            "#,
        )
        .bind(plan_id)
        .bind(req.beneficiary_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Audit log
        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            None,
            audit_action::CONTINGENT_PROMOTED,
            Some(plan_id),
            Some(entity_type::PLAN),
            None,
            None,
            None,
        )
        .await?;

        // Notification
        NotificationService::create(
            &mut tx,
            user_id,
            notif_type::CONTINGENT_PROMOTED,
            format!("Contingent beneficiary promoted to primary"),
        )
        .await?;

        tx.commit().await?;
        Ok(promoted)
    }

    /// Get or create contingency configuration for a plan
    pub async fn get_or_create_config(
        pool: &PgPool,
        plan_id: Uuid,
    ) -> Result<ContingencyConfig, ApiError> {
        let row = sqlx::query_as::<_, ContingencyConfigRow>(
            r#"
            INSERT INTO contingency_config (plan_id)
            VALUES ($1)
            ON CONFLICT (plan_id) DO UPDATE SET plan_id = $1
            RETURNING id, plan_id, primary_claim_timeout_days, contingent_claim_timeout_days,
                      auto_activate_on_timeout, require_manual_confirmation,
                      created_at, updated_at
            "#,
        )
        .bind(plan_id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    /// Check and activate contingent beneficiaries based on timeout
    pub async fn check_and_activate_timeouts(pool: &PgPool) -> Result<Vec<Uuid>, ApiError> {
        let mut activated_plan_ids = Vec::new();

        // Find plans with expired primary claim timeouts
        let expired_plans: Vec<(Uuid, i32)> = sqlx::query_as(
            r#"
            SELECT DISTINCT p.id, cc.primary_claim_timeout_days
            FROM plans p
            JOIN contingency_config cc ON cc.plan_id = p.id
            WHERE p.status = 'active'
              AND cc.auto_activate_on_timeout = true
              AND p.contract_created_at IS NOT NULL
              AND (EXTRACT(EPOCH FROM NOW()) - p.contract_created_at) / 86400 > cc.primary_claim_timeout_days
              AND NOT EXISTS (
                  SELECT 1 FROM claims WHERE plan_id = p.id
              )
            "#,
        )
        .fetch_all(pool)
        .await?;

        for (plan_id, _timeout_days) in expired_plans {
            // Activate contingent beneficiaries for this plan
            match Self::activate_contingent_for_plan(
                pool,
                plan_id,
                ContingencyCondition::PrimaryTimeout,
            )
            .await
            {
                Ok(_) => activated_plan_ids.push(plan_id),
                Err(e) => {
                    tracing::error!("Failed to activate contingent for plan {}: {}", plan_id, e)
                }
            }
        }

        Ok(activated_plan_ids)
    }

    /// Activate contingent beneficiaries for a specific plan
    async fn activate_contingent_for_plan(
        pool: &PgPool,
        plan_id: Uuid,
        reason: ContingencyCondition,
    ) -> Result<(), ApiError> {
        let mut tx = pool.begin().await?;

        // Get contingent beneficiaries
        let contingent_beneficiaries: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM plan_beneficiaries WHERE plan_id = $1 AND beneficiary_type = 'contingent' AND is_active = false"
        )
        .bind(plan_id)
        .fetch_all(&mut *tx)
        .await?;

        if contingent_beneficiaries.is_empty() {
            return Ok(());
        }

        // Activate contingent beneficiaries
        for beneficiary_id in &contingent_beneficiaries {
            sqlx::query(
                r#"
                UPDATE plan_beneficiaries
                SET is_active = true,
                    activated_at = NOW(),
                    activation_reason = $2,
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(beneficiary_id)
            .bind(reason.as_str())
            .execute(&mut *tx)
            .await?;

            // Record activation event
            sqlx::query(
                r#"
                INSERT INTO contingent_activation_events (
                    plan_id, primary_beneficiary_id, contingent_beneficiary_id,
                    activation_reason, activated_by_user_id
                )
                SELECT $1, pb_primary.id, $2, $3, NULL
                FROM plan_beneficiaries pb_primary
                WHERE pb_primary.plan_id = $1 AND pb_primary.beneficiary_type = 'primary'
                LIMIT 1
                "#,
            )
            .bind(plan_id)
            .bind(beneficiary_id)
            .bind(reason.as_str())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

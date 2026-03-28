// Notification stubs
pub fn notify_plan_created(_user_id: uuid::Uuid, _plan_id: uuid::Uuid) {
    // TODO: Implement email or in-app notification for plan creation
}

pub fn notify_plan_claimed(_user_id: uuid::Uuid, _plan_id: uuid::Uuid) {
    // TODO: Implement email or in-app notification for plan claim
}

pub fn notify_plan_deactivated(_user_id: uuid::Uuid, _plan_id: uuid::Uuid) {
    // TODO: Implement email or in-app notification for plan deactivation
}
use crate::api_error::ApiError;
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use crate::yield_service::OnChainYieldService;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Payout currency preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CurrencyPreference {
    Usdc,
    Fiat,
}

impl CurrencyPreference {
    pub fn as_str(&self) -> &'static str {
        match self {
            CurrencyPreference::Usdc => "USDC",
            CurrencyPreference::Fiat => "FIAT",
        }
    }
}

impl FromStr for CurrencyPreference {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USDC" | "usdc" => Ok(CurrencyPreference::Usdc),
            "FIAT" | "fiat" => Ok(CurrencyPreference::Fiat),
            _ => Err(ApiError::BadRequest(
                "currency_preference must be USDC or FIAT".to_string(),
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DueForClaimPlan {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub fee: rust_decimal::Decimal,
    pub net_amount: rust_decimal::Decimal,
    pub status: String,
    pub contract_plan_id: Option<i64>,
    pub distribution_method: Option<String>,
    pub is_active: Option<bool>,
    pub contract_created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beneficiary_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_preference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Plan details including beneficiary
#[derive(Debug, Serialize, Deserialize)]
pub struct PlanWithBeneficiary {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub fee: rust_decimal::Decimal,
    pub net_amount: rust_decimal::Decimal,
    pub status: String,
    pub contract_plan_id: Option<i64>,
    pub distribution_method: Option<String>,
    pub is_active: Option<bool>,
    pub is_paused: Option<bool>,
    pub risk_override_enabled: Option<bool>,
    pub contract_created_at: Option<i64>,
    pub beneficiary_name: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub currency_preference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreatePlanRequest {
    pub title: String,
    pub description: Option<String>,
    pub fee: rust_decimal::Decimal,
    pub net_amount: rust_decimal::Decimal,
    pub beneficiary_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_name: Option<String>,
    pub currency_preference: String,
    pub two_fa_code: String,
}

#[derive(Debug, Deserialize)]
pub struct ClaimPlanRequest {
    pub beneficiary_email: String,
    pub two_fa_code: String,
}

#[derive(sqlx::FromRow)]
struct PlanRowFull {
    id: Uuid,
    user_id: Uuid,
    title: String,
    description: Option<String>,
    fee: String,
    net_amount: String,
    status: String,
    contract_plan_id: Option<i64>,
    distribution_method: Option<String>,
    is_active: Option<bool>,
    is_paused: Option<bool>,
    risk_override_enabled: Option<bool>,
    contract_created_at: Option<i64>,
    beneficiary_name: Option<String>,
    bank_account_number: Option<String>,
    bank_name: Option<String>,
    currency_preference: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn plan_row_to_plan_with_beneficiary(row: &PlanRowFull) -> Result<PlanWithBeneficiary, ApiError> {
    Ok(PlanWithBeneficiary {
        id: row.id,
        user_id: row.user_id,
        title: row.title.clone(),
        description: row.description.clone(),
        fee: row
            .fee
            .parse()
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse fee: {}", e)))?,
        net_amount: row.net_amount.parse().map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to parse net_amount: {}", e))
        })?,
        status: row.status.clone(),
        contract_plan_id: row.contract_plan_id,
        distribution_method: row.distribution_method.clone(),
        is_active: row.is_active,
        is_paused: row.is_paused,
        risk_override_enabled: row.risk_override_enabled,
        contract_created_at: row.contract_created_at,
        beneficiary_name: row.beneficiary_name.clone(),
        bank_name: row.bank_name.clone(),
        bank_account_number: row.bank_account_number.clone(),
        currency_preference: row.currency_preference.clone(),
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

#[derive(Debug, sqlx::FromRow)]
struct InheritanceExecutionSafetyRow {
    outstanding_debt: Option<Decimal>,
    is_risky: Option<bool>,
    risk_override_enabled: Option<bool>,
}

#[derive(Debug, Clone)]
struct InheritanceExecutionSafety {
    outstanding_debt: Decimal,
    utilization_rate: Decimal,
    is_risky: bool,
    risk_override_enabled: bool,
}

impl InheritanceExecutionSafety {
    fn from_plan_state(
        net_amount: Decimal,
        outstanding_debt: Decimal,
        is_risky: bool,
        risk_override_enabled: bool,
    ) -> Self {
        let utilization_rate = if outstanding_debt <= Decimal::ZERO {
            Decimal::ZERO
        } else if net_amount > Decimal::ZERO {
            ((outstanding_debt / net_amount) * Decimal::from(100)).round_dp(2)
        } else {
            Decimal::from(100)
        };

        Self {
            outstanding_debt,
            utilization_rate,
            is_risky,
            risk_override_enabled,
        }
    }

    fn blocking_reason(&self) -> Option<String> {
        if self.is_risky && !self.risk_override_enabled {
            return Some(format!(
                "Inheritance execution is blocked because the plan is currently flagged as risky by the lending monitor (utilization: {}%, outstanding debt: {}).",
                self.utilization_rate.normalize(),
                self.outstanding_debt.round_dp(4).normalize()
            ));
        }

        if self.outstanding_debt > Decimal::ZERO {
            return Some(format!(
                "Inheritance execution is blocked while plan funds remain utilized in lending (utilization: {}%, outstanding debt: {}).",
                self.utilization_rate.normalize(),
                self.outstanding_debt.round_dp(4).normalize()
            ));
        }

        None
    }
}

async fn load_inheritance_execution_safety<'a, E>(
    executor: E,
    plan_id: Uuid,
    net_amount: Decimal,
) -> Result<InheritanceExecutionSafety, ApiError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let row = sqlx::query_as::<_, InheritanceExecutionSafetyRow>(
        r#"
        WITH lending_balance AS (
            SELECT
                COALESCE(SUM(CASE WHEN event_type = 'borrow' THEN CAST(amount AS numeric) ELSE 0 END), 0) -
                COALESCE(SUM(CASE WHEN event_type = 'repay' THEN CAST(amount AS numeric) ELSE 0 END), 0) -
                COALESCE(SUM(CASE WHEN event_type = 'liquidation' THEN CAST(amount AS numeric) ELSE 0 END), 0) AS outstanding_debt
            FROM lending_events
            WHERE plan_id = $1
        )
        SELECT
            lending_balance.outstanding_debt,
            p.is_risky,
            p.risk_override_enabled
        FROM lending_balance
        JOIN plans p ON p.id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_one(executor)
    .await?;

    Ok(InheritanceExecutionSafety::from_plan_state(
        net_amount,
        row.outstanding_debt.unwrap_or(Decimal::ZERO),
        row.is_risky.unwrap_or(false),
        row.risk_override_enabled.unwrap_or(false),
    ))
}

pub struct PlanService;

impl PlanService {
    /// Validates that bank details are present and non-empty when currency is FIAT.
    pub fn validate_beneficiary_for_currency(
        currency: &CurrencyPreference,
        beneficiary_name: Option<&str>,
        bank_name: Option<&str>,
        bank_account_number: Option<&str>,
    ) -> Result<(), ApiError> {
        if *currency == CurrencyPreference::Fiat {
            let name_ok = beneficiary_name
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .is_some();
            let bank_ok = bank_name
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .is_some();
            let account_ok = bank_account_number
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .is_some();
            if !name_ok || !bank_ok || !account_ok {
                return Err(ApiError::BadRequest(
                    "Bank account details (beneficiary_name, bank_name, bank_account_number) are \
                     required for FIAT payouts"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }

    pub async fn create_plan(
        pool: &PgPool,
        user_id: Uuid,
        req: &CreatePlanRequest,
    ) -> Result<PlanWithBeneficiary, ApiError> {
        // 1. Validate input amounts
        crate::safe_math::SafeMath::ensure_non_negative(req.fee, "fee")?;
        crate::safe_math::SafeMath::ensure_non_negative(req.net_amount, "net_amount")?;

        // 2. Check KYC status - only approved users can create plans
        let kyc_record = KycService::get_kyc_status(pool, user_id).await?;
        if kyc_record.status != "approved" {
            return Err(ApiError::Forbidden(
                "KYC not approved: cannot create plan".to_string(),
            ));
        }

        // 3. Start Transaction
        let mut tx = pool.begin().await?;

        let currency = CurrencyPreference::from_str(req.currency_preference.trim())?;
        Self::validate_beneficiary_for_currency(
            &currency,
            req.beneficiary_name.as_deref(),
            req.bank_name.as_deref(),
            req.bank_account_number.as_deref(),
        )?;

        let beneficiary_name = req
            .beneficiary_name
            .as_deref()
            .map(|s| s.trim().to_string());
        let bank_name = req.bank_name.as_deref().map(|s| s.trim().to_string());
        let bank_account_number = req
            .bank_account_number
            .as_deref()
            .map(|s| s.trim().to_string());
        let currency_preference = Some(currency.as_str().to_string());

        // 2. Insert Plan - using the transaction handle
        let row = sqlx::query_as::<_, PlanRowFull>(
            r#"
        INSERT INTO plans (
            user_id, title, description, fee, net_amount, status,
            beneficiary_name, bank_account_number, bank_name, currency_preference
        )
        VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7, $8, $9)
        RETURNING id, user_id, title, description, fee, net_amount, status,
                  contract_plan_id, distribution_method, is_active, contract_created_at,
                  beneficiary_name, bank_account_number, bank_name, currency_preference,
                  created_at, updated_at
        "#,
        )
        .bind(user_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(req.fee.to_string())
        .bind(req.net_amount.to_string())
        .bind(&beneficiary_name)
        .bind(&bank_account_number)
        .bind(&bank_name)
        .bind(&currency_preference)
        .fetch_one(&mut *tx) // CRITICAL: Use the transaction, not the pool
        .await?;

        let plan = plan_row_to_plan_with_beneficiary(&row)?;

        // 3. Audit: This must now return Result and use the transaction
        AuditLogService::log(
            &mut *tx, // Pass the transaction
            Some(user_id),
            audit_action::PLAN_CREATED,
            Some(plan.id),
            Some(entity_type::PLAN),
        )
        .await?; // If this fails, '?' triggers an early return

        // 4. Commit: If we reached here, both Plan and Audit are saved
        tx.commit().await?;

        Ok(plan)
    }
    pub async fn get_plan_by_id<'a, E>(
        executor: E,
        plan_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<PlanWithBeneficiary>, ApiError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let row = sqlx::query_as::<_, PlanRowFull>(
            r#"
        SELECT id, user_id, title, description, fee, net_amount, status,
               contract_plan_id, distribution_method, is_active, is_paused, risk_override_enabled,
               contract_created_at, beneficiary_name, bank_account_number, bank_name, currency_preference,
               created_at, updated_at
        FROM plans
        WHERE id = $1 AND user_id = $2
        "#,
        )
        .bind(plan_id)
        .bind(user_id)
        .fetch_optional(executor)
        .await?;

        match row {
            Some(r) => Ok(Some(plan_row_to_plan_with_beneficiary(&r)?)),
            None => Ok(None),
        }
    }

    pub async fn get_plan_by_id_any_user<'a, E>(
        executor: E,
        plan_id: Uuid,
    ) -> Result<Option<PlanWithBeneficiary>, ApiError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let row = sqlx::query_as::<_, PlanRowFull>(
            r#"
        SELECT id, user_id, title, description, fee, net_amount, status,
               contract_plan_id, distribution_method, is_active, is_paused, risk_override_enabled,
               contract_created_at, beneficiary_name, bank_account_number, bank_name, currency_preference,
               created_at, updated_at
        FROM plans
        WHERE id = $1
        "#,
        )
        .bind(plan_id)
        .fetch_optional(executor)
        .await?;

        match row {
            Some(r) => Ok(Some(plan_row_to_plan_with_beneficiary(&r)?)),
            None => Ok(None),
        }
    }
    pub async fn claim_plan(
        pool: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
        req: &ClaimPlanRequest,
    ) -> Result<PlanWithBeneficiary, ApiError> {
        // 1. Check KYC status - only approved users can claim plans
        let kyc_record = KycService::get_kyc_status(pool, user_id).await?;
        if kyc_record.status != "approved" {
            return Err(ApiError::Forbidden(
                "KYC not approved: cannot claim plan".to_string(),
            ));
        }

        // 2. Start the transaction
        let mut tx = pool.begin().await?;

        // 3. Use SELECT FOR UPDATE to lock the plan row and prevent concurrent claims
        let row = sqlx::query_as::<_, PlanRowFull>(
            r#"
            SELECT id, user_id, title, description, fee, net_amount, status,
                   contract_plan_id, distribution_method, is_active, is_paused, risk_override_enabled,
                   contract_created_at, beneficiary_name, bank_account_number, bank_name, currency_preference,
                   created_at, updated_at
            FROM plans
            WHERE id = $1 AND user_id = $2
            FOR UPDATE
            "#,
        )
        .bind(plan_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;

        let plan = match row {
            Some(r) => plan_row_to_plan_with_beneficiary(&r)?,
            None => return Err(ApiError::NotFound(format!("Plan {} not found", plan_id))),
        };

        // Check if plan is paused
        if plan.is_paused == Some(true) {
            return Err(ApiError::BadRequest(
                "This plan is currently paused by an administrator and cannot be claimed"
                    .to_string(),
            ));
        }

        // Check if plan is already claimed - this prevents concurrent claims
        if plan.status == "claimed" {
            return Err(ApiError::BadRequest(
                "This plan has already been claimed".to_string(),
            ));
        }

        if !Self::is_due_for_claim(
            plan.distribution_method.as_deref(),
            plan.contract_created_at,
        ) {
            return Err(ApiError::BadRequest(
                "Plan is not yet mature for claim".to_string(),
            ));
        }

        let execution_safety =
            load_inheritance_execution_safety(&mut *tx, plan_id, plan.net_amount).await?;
        if let Some(reason) = execution_safety.blocking_reason() {
            return Err(ApiError::BadRequest(reason));
        }

        let contract_plan_id = plan.contract_plan_id.unwrap_or(0_i64);

        // ... (Currency validation logic remains same) ...
        let currency = plan
            .currency_preference
            .as_deref()
            .map(CurrencyPreference::from_str)
            .transpose()?
            .ok_or_else(|| {
                ApiError::BadRequest("Plan has no currency preference set".to_string())
            })?;

        if currency == CurrencyPreference::Fiat {
            Self::validate_beneficiary_for_currency(
                &currency,
                plan.beneficiary_name.as_deref(),
                plan.bank_name.as_deref(),
                plan.bank_account_number.as_deref(),
            )?;
        }

        // 3. FIX: Changed 'db' to '&mut *tx' to keep it atomic
        sqlx::query(
            r#"
        INSERT INTO claims (plan_id, contract_plan_id, beneficiary_email)
        VALUES ($1, $2, $3)
        "#,
        )
        .bind(plan_id)
        .bind(contract_plan_id)
        .bind(req.beneficiary_email.trim())
        .execute(&mut *tx) // <--- Use the transaction here!
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.is_unique_violation() {
                    return ApiError::BadRequest("This plan has already been claimed".to_string());
                }
            }
            ApiError::from(e)
        })?;

        // Update plan status to 'claimed' to prevent future concurrent claims
        sqlx::query(
            r#"
            UPDATE plans
            SET status = 'claimed', updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(plan_id)
        .execute(&mut *tx)
        .await?;

        // 4. Audit Log
        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            audit_action::PLAN_CLAIMED,
            Some(plan_id),
            Some(entity_type::PLAN),
        )
        .await?;

        // Notification: plan claimed
        NotificationService::create(
            &mut tx,
            user_id,
            notif_type::PLAN_CLAIMED,
            format!("Plan '{}' has been successfully claimed", plan.title),
        )
        .await?; // Use ? to ensure failure here rolls back the claim

        // 6. Final Commit
        tx.commit().await?;
        Ok(plan)
    }
    pub fn is_due_for_claim(
        distribution_method: Option<&str>,
        contract_created_at: Option<i64>,
    ) -> bool {
        let Some(method) = distribution_method else {
            return false;
        };
        let Some(created_at) = contract_created_at else {
            return false;
        };

        let now = chrono::Utc::now().timestamp();
        let elapsed = now - created_at;

        match method {
            "LumpSum" => true,
            "Monthly" => elapsed >= 30 * 24 * 60 * 60,
            "Quarterly" => elapsed >= 90 * 24 * 60 * 60,
            "Yearly" => elapsed >= 365 * 24 * 60 * 60,
            _ => false,
        }
    }

    pub async fn get_due_for_claim_plan_by_id(
        db: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<DueForClaimPlan>, ApiError> {
        #[derive(sqlx::FromRow)]
        struct PlanRow {
            id: Uuid,
            user_id: Uuid,
            title: String,
            description: Option<String>,
            fee: String,
            net_amount: String,
            status: String,
            contract_plan_id: Option<i64>,
            distribution_method: Option<String>,
            is_active: Option<bool>,
            contract_created_at: Option<i64>,
            beneficiary_name: Option<String>,
            bank_account_number: Option<String>,
            bank_name: Option<String>,
            currency_preference: Option<String>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let plan_row = sqlx::query_as::<_, PlanRow>(
            r#"
            SELECT p.id, p.user_id, p.title, p.description, p.fee, p.net_amount, p.status,
                   p.contract_plan_id, p.distribution_method, p.is_active, p.contract_created_at,
                   p.beneficiary_name, p.bank_account_number, p.bank_name, p.currency_preference,
                   p.created_at, p.updated_at
            FROM plans p
            WHERE p.id = $1
              AND p.user_id = $2
              AND (p.is_active IS NULL OR p.is_active = true)
              AND p.status != 'claimed'
              AND p.status != 'deactivated'
            "#,
        )
        .bind(plan_id)
        .bind(user_id)
        .fetch_optional(db)
        .await?;

        let plan = if let Some(row) = plan_row {
            Some(DueForClaimPlan {
                id: row.id,
                user_id: row.user_id,
                title: row.title,
                description: row.description,
                fee: row.fee.parse().map_err(|e| {
                    ApiError::Internal(anyhow::anyhow!("Failed to parse fee: {}", e))
                })?,
                net_amount: row.net_amount.parse().map_err(|e| {
                    ApiError::Internal(anyhow::anyhow!("Failed to parse net_amount: {}", e))
                })?,
                status: row.status,
                contract_plan_id: row.contract_plan_id,
                distribution_method: row.distribution_method,
                is_active: row.is_active,
                contract_created_at: row.contract_created_at,
                beneficiary_name: row.beneficiary_name,
                bank_account_number: row.bank_account_number,
                bank_name: row.bank_name,
                currency_preference: row.currency_preference,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        } else {
            None
        };

        if let Some(plan) = plan {
            if Self::is_due_for_claim(
                plan.distribution_method.as_deref(),
                plan.contract_created_at,
            ) {
                let has_claim = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM claims WHERE plan_id = $1)",
                )
                .bind(plan_id)
                .fetch_one(db)
                .await?;

                if !has_claim {
                    let execution_safety =
                        load_inheritance_execution_safety(db, plan.id, plan.net_amount).await?;
                    if execution_safety.blocking_reason().is_none() {
                        return Ok(Some(plan));
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn get_all_due_for_claim_plans_for_user(
        db: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<DueForClaimPlan>, ApiError> {
        #[derive(sqlx::FromRow)]
        struct PlanRow {
            id: Uuid,
            user_id: Uuid,
            title: String,
            description: Option<String>,
            fee: String,
            net_amount: String,
            status: String,
            contract_plan_id: Option<i64>,
            distribution_method: Option<String>,
            is_active: Option<bool>,
            contract_created_at: Option<i64>,
            beneficiary_name: Option<String>,
            bank_account_number: Option<String>,
            bank_name: Option<String>,
            currency_preference: Option<String>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let plan_rows = sqlx::query_as::<_, PlanRow>(
            r#"
            SELECT p.id, p.user_id, p.title, p.description, p.fee, p.net_amount, p.status,
                   p.contract_plan_id, p.distribution_method, p.is_active, p.contract_created_at,
                   p.beneficiary_name, p.bank_account_number, p.bank_name, p.currency_preference,
                   p.created_at, p.updated_at
            FROM plans p
            WHERE p.user_id = $1
              AND (p.is_active IS NULL OR p.is_active = true)
              AND p.status != 'claimed'
              AND p.status != 'deactivated'
            ORDER BY p.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(db)
        .await?;

        let plans: Result<Vec<DueForClaimPlan>, ApiError> = plan_rows
            .into_iter()
            .map(|row| {
                Ok(DueForClaimPlan {
                    id: row.id,
                    user_id: row.user_id,
                    title: row.title,
                    description: row.description,
                    fee: row.fee.parse().map_err(|e| {
                        ApiError::Internal(anyhow::anyhow!("Failed to parse fee: {}", e))
                    })?,
                    net_amount: row.net_amount.parse().map_err(|e| {
                        ApiError::Internal(anyhow::anyhow!("Failed to parse net_amount: {}", e))
                    })?,
                    status: row.status,
                    contract_plan_id: row.contract_plan_id,
                    distribution_method: row.distribution_method,
                    is_active: row.is_active,
                    contract_created_at: row.contract_created_at,
                    beneficiary_name: row.beneficiary_name,
                    bank_account_number: row.bank_account_number,
                    bank_name: row.bank_name,
                    currency_preference: row.currency_preference,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect();
        let plans = plans?;

        let mut due_plans = Vec::new();

        for plan in plans {
            if Self::is_due_for_claim(
                plan.distribution_method.as_deref(),
                plan.contract_created_at,
            ) {
                let has_claim = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM claims WHERE plan_id = $1)",
                )
                .bind(plan.id)
                .fetch_one(db)
                .await?;

                if !has_claim {
                    let execution_safety =
                        load_inheritance_execution_safety(db, plan.id, plan.net_amount).await?;
                    if execution_safety.blocking_reason().is_none() {
                        due_plans.push(plan);
                    }
                }
            }
        }

        Ok(due_plans)
    }

    pub async fn get_all_due_for_claim_plans_admin(
        db: &PgPool,
    ) -> Result<Vec<DueForClaimPlan>, ApiError> {
        #[derive(sqlx::FromRow)]
        struct PlanRow {
            id: Uuid,
            user_id: Uuid,
            title: String,
            description: Option<String>,
            fee: String,
            net_amount: String,
            status: String,
            contract_plan_id: Option<i64>,
            distribution_method: Option<String>,
            is_active: Option<bool>,
            contract_created_at: Option<i64>,
            beneficiary_name: Option<String>,
            bank_account_number: Option<String>,
            bank_name: Option<String>,
            currency_preference: Option<String>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let plan_rows = sqlx::query_as::<_, PlanRow>(
            r#"
            SELECT p.id, p.user_id, p.title, p.description, p.fee, p.net_amount, p.status,
                   p.contract_plan_id, p.distribution_method, p.is_active, p.contract_created_at,
                   p.beneficiary_name, p.bank_account_number, p.bank_name, p.currency_preference,
                   p.created_at, p.updated_at
            FROM plans p
            WHERE (p.is_active IS NULL OR p.is_active = true)
              AND p.status != 'claimed'
              AND p.status != 'deactivated'
            ORDER BY p.created_at DESC
            "#,
        )
        .fetch_all(db)
        .await?;

        let plans: Result<Vec<DueForClaimPlan>, ApiError> = plan_rows
            .into_iter()
            .map(|row| {
                Ok(DueForClaimPlan {
                    id: row.id,
                    user_id: row.user_id,
                    title: row.title,
                    description: row.description,
                    fee: row.fee.parse().map_err(|e| {
                        ApiError::Internal(anyhow::anyhow!("Failed to parse fee: {}", e))
                    })?,
                    net_amount: row.net_amount.parse().map_err(|e| {
                        ApiError::Internal(anyhow::anyhow!("Failed to parse net_amount: {}", e))
                    })?,
                    status: row.status,
                    contract_plan_id: row.contract_plan_id,
                    distribution_method: row.distribution_method,
                    is_active: row.is_active,
                    contract_created_at: row.contract_created_at,
                    beneficiary_name: row.beneficiary_name,
                    bank_account_number: row.bank_account_number,
                    bank_name: row.bank_name,
                    currency_preference: row.currency_preference,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect();
        let plans = plans?;

        let mut due_plans = Vec::new();

        for plan in plans {
            if Self::is_due_for_claim(
                plan.distribution_method.as_deref(),
                plan.contract_created_at,
            ) {
                let has_claim = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM claims WHERE plan_id = $1)",
                )
                .bind(plan.id)
                .fetch_one(db)
                .await?;

                if !has_claim {
                    let execution_safety =
                        load_inheritance_execution_safety(db, plan.id, plan.net_amount).await?;
                    if execution_safety.blocking_reason().is_none() {
                        due_plans.push(plan);
                    }
                }
            }
        }

        Ok(due_plans)
    }

    /// Cancel (deactivate) a plan
    /// Sets the plan status to 'deactivated' and is_active to false
    pub async fn cancel_plan(
        pool: &PgPool, // Required to start a transaction if one isn't provided
        plan_id: Uuid,
        user_id: Uuid,
    ) -> Result<PlanWithBeneficiary, ApiError> {
        // 1. Start the transaction
        let mut tx = pool.begin().await?;

        // 2. Fetch the plan using the transaction handle
        // Note: get_plan_by_id must also use the generic <'a, E> pattern
        let plan = Self::get_plan_by_id(&mut *tx, plan_id, user_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Plan {} not found", plan_id)))?;

        // Business Logic Checks
        if plan.status == "deactivated" {
            return Err(ApiError::BadRequest(
                "Plan is already deactivated".to_string(),
            ));
        }
        if plan.status == "claimed" {
            return Err(ApiError::BadRequest(
                "Cannot cancel a plan that has been claimed".to_string(),
            ));
        }

        // 3. Perform the Update
        let row = sqlx::query_as::<_, PlanRowFull>(
            r#"
        UPDATE plans
        SET status = 'deactivated', is_active = false, updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, title, description, fee, net_amount, status,
                  contract_plan_id, distribution_method, is_active, contract_created_at,
                  beneficiary_name, bank_account_number, bank_name, currency_preference,
                  created_at, updated_at
        "#,
        )
        .bind(plan_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        let updated_plan = plan_row_to_plan_with_beneficiary(&row)?;

        // 4. Atomic Audit Log
        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            audit_action::PLAN_DEACTIVATED,
            Some(plan_id),
            Some(entity_type::PLAN),
        )
        .await?;

        // 5. Atomic Notification
        NotificationService::create(
            &mut tx,
            user_id,
            notif_type::PLAN_DEACTIVATED,
            format!("Plan '{}' has been deactivated", updated_plan.title),
        )
        .await?;

        // 6. Commit
        tx.commit().await?;

        Ok(updated_plan)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum KycStatus {
    Pending,
    Approved,
    Rejected,
}

impl fmt::Display for KycStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            KycStatus::Pending => "pending",
            KycStatus::Approved => "approved",
            KycStatus::Rejected => "rejected",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for KycStatus {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "approved" => KycStatus::Approved,
            "rejected" => KycStatus::Rejected,
            _ => KycStatus::Pending,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KycRecord {
    pub user_id: Uuid,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct KycService;

impl KycService {
    pub async fn submit_kyc(pool: &PgPool, user_id: Uuid) -> Result<KycRecord, ApiError> {
        // 1. Start the transaction
        let mut tx = pool.begin().await?;
        let now = Utc::now();

        // 2. Insert record
        // Adding &mut *tx fixes the "Executor not satisfied" error
        let record = sqlx::query_as::<_, KycRecord>(
            r#"
            INSERT INTO kyc_status (user_id, status, created_at, updated_at)
            VALUES ($1, 'pending', $2, $2)
            ON CONFLICT (user_id) DO UPDATE SET updated_at = EXCLUDED.updated_at
            RETURNING user_id, status, reviewed_by, reviewed_at, created_at
            "#,
        )
        .bind(user_id)
        .bind(now)
        .fetch_one(&mut *tx) // <--- Use the explicit re-borrow here
        .await?;

        // 3. Atomic Audit log
        AuditLogService::log(
            &mut *tx, // Re-borrow here as well
            Some(user_id),
            audit_action::KYC_SUBMITTED,
            Some(user_id),
            Some(entity_type::USER),
        )
        .await?;

        // 4. Commit
        tx.commit().await?;
        Ok(record)
    }

    pub async fn get_kyc_status(db: &PgPool, user_id: Uuid) -> Result<KycRecord, ApiError> {
        let row = sqlx::query_as::<_, KycRecord>(
            "SELECT user_id, status, reviewed_by, reviewed_at, created_at FROM kyc_status WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(db)
        .await?;

        match row {
            Some(record) => Ok(record),
            None => Ok(KycRecord {
                user_id,
                status: "pending".to_string(),
                reviewed_by: None,
                reviewed_at: None,
                created_at: Utc::now(),
            }),
        }
    }

    pub async fn update_kyc_status(
        pool: &PgPool,
        admin_id: Uuid,
        user_id: Uuid,
        status: KycStatus,
    ) -> Result<KycRecord, ApiError> {
        let mut tx = pool.begin().await?; // Start Transaction
        let status_str = status.to_string();
        let now = Utc::now();

        let record = sqlx::query_as::<_, KycRecord>(
            r#"
        INSERT INTO kyc_status (user_id, status, reviewed_by, reviewed_at, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_id) DO UPDATE SET ...
        RETURNING user_id, status, reviewed_by, reviewed_at, created_at
        "#,
        )
        .bind(user_id)
        .bind(status_str)
        .bind(admin_id)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx) // Use Transaction
        .await?;

        // Prepare notification
        let (ntype, msg) = match status {
            KycStatus::Approved => (notif_type::KYC_APPROVED, "Approved".to_string()),
            KycStatus::Rejected => (notif_type::KYC_REJECTED, "Rejected".to_string()),
            _ => (notif_type::KYC_APPROVED, "Updated".to_string()),
        };

        // Notification is now ATOMIC
        NotificationService::create(&mut tx, user_id, ntype, msg).await?;

        // Audit log is now ATOMIC
        AuditLogService::log(
            &mut *tx,
            Some(admin_id),
            if record.status == "approved" {
                audit_action::KYC_APPROVED
            } else {
                audit_action::KYC_REJECTED
            },
            Some(user_id),
            Some(entity_type::USER),
        )
        .await?;

        tx.commit().await?; // Commit all three operations
        Ok(record)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminMetrics {
    pub total_revenue: f64,
    pub total_plans: i64,
    pub total_claims: i64,
    pub active_plans: i64,
    pub total_users: i64,
}

pub struct AdminService;

impl AdminService {
    pub async fn get_metrics_overview(db: &PgPool) -> Result<AdminMetrics, ApiError> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_revenue: f64,
            total_plans: i64,
            total_claims: i64,
            active_plans: i64,
            total_users: i64,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"
            SELECT
                COALESCE(SUM(fee), 0)::FLOAT8 AS total_revenue,
                COUNT(*)::BIGINT AS total_plans,
                (SELECT COUNT(*)::BIGINT FROM claims) AS total_claims,
                COUNT(*) FILTER (
                    WHERE is_active IS NOT FALSE
                      AND status NOT IN ('claimed', 'deactivated')
                )::BIGINT AS active_plans,
                (SELECT COUNT(*)::BIGINT FROM users) AS total_users
            FROM plans
            "#,
        )
        .fetch_one(db)
        .await?;

        Ok(AdminMetrics {
            total_revenue: row.total_revenue,
            total_plans: row.total_plans,
            total_claims: row.total_claims,
            active_plans: row.active_plans,
            total_users: row.total_users,
        })
    }
}

// ── Claim Metrics ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimStatistics {
    pub total_claims: i64,
    pub pending_claims: i64,
    pub approved_claims: i64,
    pub rejected_claims: i64,
    pub average_claim_processing_time_seconds: f64,
}

pub struct ClaimMetricsService;

impl ClaimMetricsService {
    pub async fn get_claim_statistics(db: &PgPool) -> Result<ClaimStatistics, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            total_claims: i64,
            pending_claims: i64,
            approved_claims: i64,
            rejected_claims: i64,
            average_claim_processing_time_seconds: Option<f64>,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                COUNT(c.id)::BIGINT AS total_claims,
                COUNT(c.id) FILTER (
                    WHERE p.status IN ('pending', 'due-for-claim')
                )::BIGINT AS pending_claims,
                COUNT(c.id) FILTER (
                    WHERE p.status = 'claimed'
                )::BIGINT AS approved_claims,
                COUNT(c.id) FILTER (
                    WHERE p.status IN ('rejected', 'deactivated')
                )::BIGINT AS rejected_claims,
                AVG(
                    CASE
                        WHEN p.status IN ('claimed', 'rejected', 'deactivated')
                         AND p.updated_at >= c.claimed_at
                        THEN EXTRACT(EPOCH FROM (p.updated_at - c.claimed_at))
                        ELSE NULL
                    END
                )::FLOAT8 AS average_claim_processing_time_seconds
            FROM claims c
            INNER JOIN plans p ON p.id = c.plan_id
            "#,
        )
        .fetch_one(db)
        .await?;

        Ok(ClaimStatistics {
            total_claims: row.total_claims,
            pending_claims: row.pending_claims,
            approved_claims: row.approved_claims,
            rejected_claims: row.rejected_claims,
            average_claim_processing_time_seconds: row
                .average_claim_processing_time_seconds
                .unwrap_or(0.0),
        })
    }
}

// ── User Growth Metrics ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGrowthMetrics {
    pub total_users: i64,
    pub new_users_last_7_days: i64,
    pub new_users_last_30_days: i64,
    pub active_users: i64,
}

pub struct UserMetricsService;

impl UserMetricsService {
    pub async fn get_user_growth_metrics(db: &PgPool) -> Result<UserGrowthMetrics, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            total_users: i64,
            new_users_last_7_days: i64,
            new_users_last_30_days: i64,
            active_users: i64,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                COUNT(*)::BIGINT AS total_users,
                COUNT(*) FILTER (
                    WHERE created_at >= NOW() - INTERVAL '7 days'
                )::BIGINT AS new_users_last_7_days,
                COUNT(*) FILTER (
                    WHERE created_at >= NOW() - INTERVAL '30 days'
                )::BIGINT AS new_users_last_30_days,
                COUNT(*) FILTER (
                    WHERE id IN (
                        SELECT DISTINCT user_id FROM action_logs
                        WHERE timestamp >= NOW() - INTERVAL '30 days'
                          AND user_id IS NOT NULL
                    )
                )::BIGINT AS active_users
            FROM users
            "#,
        )
        .fetch_one(db)
        .await?;

        Ok(UserGrowthMetrics {
            total_users: row.total_users,
            new_users_last_7_days: row.new_users_last_7_days,
            new_users_last_30_days: row.new_users_last_30_days,
            active_users: row.active_users,
        })
    }
}

// ── Plan Statistics ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanStatistics {
    pub total_plans: i64,
    pub active_plans: i64,
    pub expired_plans: i64,
    pub triggered_plans: i64,
    pub claimed_plans: i64,
    pub by_status: Vec<PlanStatusCount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanStatusCount {
    pub status: String,
    pub count: i64,
}

pub struct PlanStatisticsService;

impl PlanStatisticsService {
    pub async fn get_plan_statistics(db: &PgPool) -> Result<PlanStatistics, ApiError> {
        // Get total plans count
        let total_plans: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plans")
            .fetch_one(db)
            .await?;

        // Get active plans (is_active = true or NULL, and not deactivated/claimed)
        let active_plans: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM plans
            WHERE (is_active IS NULL OR is_active = true)
              AND status NOT IN ('deactivated', 'claimed')
            "#,
        )
        .fetch_one(db)
        .await?;

        // Get expired plans (plans that are past their claim period but not claimed)
        // This is a simplified version - you may need to adjust based on your business logic
        let expired_plans: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM plans
            WHERE status = 'deactivated'
            "#,
        )
        .fetch_one(db)
        .await?;

        // Get triggered plans (plans that are due for claim)
        // Plans with distribution_method set and contract_created_at set
        let triggered_plans: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM plans
            WHERE distribution_method IS NOT NULL
              AND contract_created_at IS NOT NULL
              AND (is_active IS NULL OR is_active = true)
              AND status NOT IN ('claimed', 'deactivated')
            "#,
        )
        .fetch_one(db)
        .await?;

        // Get claimed plans
        let claimed_plans: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM plans
            WHERE status = 'claimed'
            "#,
        )
        .fetch_one(db)
        .await?;

        // Get counts grouped by status
        let by_status: Vec<PlanStatusCount> = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT status, COUNT(*) as count
            FROM plans
            GROUP BY status
            ORDER BY count DESC
            "#,
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|(status, count)| PlanStatusCount { status, count })
        .collect();

        Ok(PlanStatistics {
            total_plans,
            active_plans,
            expired_plans,
            triggered_plans,
            claimed_plans,
            by_status,
        })
    }
}

// ── Revenue Metrics ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueMetric {
    pub date: String,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueMetricsResponse {
    pub range: String,
    pub data: Vec<RevenueMetric>,
}

pub struct RevenueMetricsService;

impl RevenueMetricsService {
    pub async fn get_revenue_breakdown(
        pool: &PgPool,
        range: &str,
    ) -> Result<RevenueMetricsResponse, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            date: String,
            amount: f64,
        }

        let (interval, trunc) = match range {
            "daily" => ("30 days", "day"),
            "weekly" => ("12 weeks", "week"),
            "monthly" => ("12 months", "month"),
            _ => {
                return Err(ApiError::BadRequest(
                    "Invalid range. Use daily, weekly, or monthly.".to_string(),
                ))
            }
        };

        let query = format!(
            r#"
            SELECT 
                DATE_TRUNC('{}', created_at)::DATE::TEXT as date,
                COALESCE(SUM(fee), 0)::FLOAT8 as amount
            FROM plans
            WHERE created_at >= NOW() - INTERVAL '{}'
            GROUP BY 1
            ORDER BY 1
            "#,
            trunc, interval
        );

        let rows = sqlx::query_as::<_, Row>(&query).fetch_all(pool).await?;

        let data = rows
            .into_iter()
            .map(|r| RevenueMetric {
                date: r.date,
                amount: r.amount,
            })
            .collect();

        Ok(RevenueMetricsResponse {
            range: range.to_string(),
            data,
        })
    }
}

// ── Lending Pool Metrics ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LendingMetrics {
    pub total_value_locked: f64,
    pub total_borrowed: f64,
    pub utilization_rate: f64,
    pub active_loans_count: i64,
}

pub struct LendingMonitoringService;

impl LendingMonitoringService {
    pub async fn get_lending_metrics(db: &PgPool) -> Result<LendingMetrics, ApiError> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_deposited: f64,
            total_borrowed: f64,
            total_repaid_principal: f64,
            active_loans_count: i64,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"
            SELECT
                (SELECT COALESCE(SUM(CAST(amount AS DECIMAL)), 0)::FLOAT8 FROM lending_events WHERE event_type = 'deposit') as total_deposited,
                (SELECT COALESCE(SUM(CAST(amount AS DECIMAL)), 0)::FLOAT8 FROM lending_events WHERE event_type = 'borrow') as total_borrowed,
                (SELECT COALESCE(SUM(CAST(metadata->>'principal_amount' AS DECIMAL)), 0)::FLOAT8 FROM lending_events WHERE event_type = 'repay') as total_repaid_principal,
                (SELECT COUNT(*)::BIGINT FROM (
                    SELECT plan_id, 
                           SUM(CASE 
                                WHEN event_type = 'borrow' THEN CAST(amount AS DECIMAL) 
                                WHEN event_type = 'repay' THEN -CAST(metadata->>'principal_amount' AS DECIMAL)
                                ELSE 0 
                           END) as balance
                    FROM lending_events 
                    WHERE plan_id IS NOT NULL 
                    GROUP BY plan_id
                ) t WHERE balance > 0) as active_loans_count
            "#,
        )
        .fetch_one(db)
        .await?;

        let current_debt = row.total_borrowed - row.total_repaid_principal;
        let tvl = row.total_deposited; // Simplified TVL as total deposits

        let utilization_rate = if tvl > 0.0 {
            (current_debt / tvl) * 100.0
        } else {
            0.0
        };

        Ok(LendingMetrics {
            total_value_locked: tvl,
            total_borrowed: current_debt,
            utilization_rate,
            active_loans_count: row.active_loans_count,
        })
    }
}

// ── Yield Reporting ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YieldReportFilters {
    pub asset_code: Option<String>,
    pub user_id: Option<Uuid>,
    pub plan_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YieldVaultSummary {
    pub asset_code: String,
    pub realized_yield: f64,
    pub on_chain_yield: Option<f64>,
    pub apy: f64,
    pub total_principal_balance: f64,
    pub accrual_event_count: i64,
    pub last_accrual_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YieldSummaryResponse {
    pub filters: YieldReportFilters,
    pub total_realized_yield: f64,
    pub total_on_chain_yield: Option<f64>,
    pub average_apy: f64,
    pub vaults: Vec<YieldVaultSummary>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsHistoryPoint {
    pub period: String,
    pub asset_code: String,
    pub earnings: f64,
    pub event_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsHistoryResponse {
    pub filters: YieldReportFilters,
    pub range: String,
    pub total_earnings: f64,
    pub history: Vec<EarningsHistoryPoint>,
}

#[derive(sqlx::FromRow)]
struct YieldAccrualEventRow {
    asset_code: String,
    user_id: Uuid,
    plan_id: Option<Uuid>,
    amount: String,
    metadata: serde_json::Value,
    event_timestamp: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct EarningsHistoryRow {
    period: String,
    asset_code: String,
    earnings: f64,
    event_count: i64,
}

#[derive(Default)]
struct YieldVaultAccumulator {
    total_realized_yield: Decimal,
    total_principal_balance: Decimal,
    weighted_rate_numerator: Decimal,
    accrual_event_count: i64,
    last_accrual_at: Option<DateTime<Utc>>,
    seen_positions: HashSet<String>,
}

pub struct YieldReportingService;

impl YieldReportingService {
    pub async fn get_yield_summary(
        db: &PgPool,
        filters: YieldReportFilters,
        yield_service: &dyn OnChainYieldService,
    ) -> Result<YieldSummaryResponse, ApiError> {
        let events = Self::get_interest_accrual_events(db, &filters).await?;
        let is_asset_pool_scope = filters.user_id.is_none() && filters.plan_id.is_none();
        let mut vaults: BTreeMap<String, YieldVaultAccumulator> = BTreeMap::new();

        for event in events {
            let amount = Self::parse_decimal(&event.amount, "accrual amount")?;
            let asset_code = event.asset_code.to_uppercase();
            let position_key = event
                .plan_id
                .map(|plan_id| format!("plan:{plan_id}"))
                .unwrap_or_else(|| format!("user:{}", event.user_id));

            let accumulator = vaults.entry(asset_code).or_default();
            accumulator.total_realized_yield += amount;
            accumulator.accrual_event_count += 1;
            accumulator.last_accrual_at = match accumulator.last_accrual_at {
                Some(last_seen) if last_seen >= event.event_timestamp => Some(last_seen),
                _ => Some(event.event_timestamp),
            };

            if accumulator.seen_positions.insert(position_key) {
                if let Some(principal_balance) =
                    Self::decimal_from_metadata(&event.metadata, "principal_balance")
                {
                    if principal_balance > Decimal::ZERO {
                        let normalized_rate = Self::normalize_rate(
                            Self::decimal_from_metadata(&event.metadata, "interest_rate")
                                .unwrap_or(Decimal::ZERO),
                        );

                        accumulator.total_principal_balance += principal_balance;
                        accumulator.weighted_rate_numerator += normalized_rate * principal_balance;
                    }
                }
            }
        }

        let mut total_realized_yield = 0.0;
        let mut total_on_chain_yield = 0.0;
        let mut any_on_chain_yield = false;
        let mut aggregate_principal_balance = Decimal::ZERO;
        let mut aggregate_weighted_rate_numerator = Decimal::ZERO;
        let mut summaries = Vec::with_capacity(vaults.len());

        for (asset_code, accumulator) in vaults {
            let apy = if accumulator.total_principal_balance > Decimal::ZERO {
                Self::annual_rate_to_apy(
                    accumulator.weighted_rate_numerator / accumulator.total_principal_balance,
                )
            } else {
                0.0
            };

            let realized_yield = Self::decimal_to_f64(accumulator.total_realized_yield)?;
            total_realized_yield += realized_yield;
            aggregate_principal_balance += accumulator.total_principal_balance;
            aggregate_weighted_rate_numerator += accumulator.weighted_rate_numerator;

            let on_chain_yield = if is_asset_pool_scope {
                let yield_amount = yield_service
                    .get_total_on_chain_yield_amount(&asset_code)
                    .await?;
                let value = Self::decimal_to_f64(yield_amount)?;
                total_on_chain_yield += value;
                any_on_chain_yield = true;
                Some(value)
            } else {
                None
            };

            summaries.push(YieldVaultSummary {
                asset_code,
                realized_yield,
                on_chain_yield,
                apy,
                total_principal_balance: Self::decimal_to_f64(accumulator.total_principal_balance)?,
                accrual_event_count: accumulator.accrual_event_count,
                last_accrual_at: accumulator.last_accrual_at,
            });
        }

        let average_apy = if aggregate_principal_balance > Decimal::ZERO {
            Self::annual_rate_to_apy(
                aggregate_weighted_rate_numerator / aggregate_principal_balance,
            )
        } else {
            0.0
        };

        Ok(YieldSummaryResponse {
            filters,
            total_realized_yield,
            total_on_chain_yield: any_on_chain_yield.then_some(total_on_chain_yield),
            average_apy,
            vaults: summaries,
            generated_at: Utc::now(),
        })
    }

    pub async fn get_earnings_history(
        db: &PgPool,
        filters: YieldReportFilters,
        range: &str,
    ) -> Result<EarningsHistoryResponse, ApiError> {
        let (interval, trunc) = match range {
            "daily" => ("30 days", "day"),
            "weekly" => ("12 weeks", "week"),
            "monthly" => ("12 months", "month"),
            _ => {
                return Err(ApiError::BadRequest(
                    "Invalid range. Use daily, weekly, or monthly.".to_string(),
                ))
            }
        };

        let mut query = QueryBuilder::<Postgres>::new("SELECT DATE_TRUNC('");
        query.push(trunc);
        query.push("', event_timestamp)::DATE::TEXT AS period,");
        query.push(" asset_code, COALESCE(SUM(CAST(amount AS NUMERIC)), 0)::FLOAT8 AS earnings,");
        query.push(" COUNT(*)::BIGINT AS event_count FROM lending_events WHERE event_type = '");
        query.push("interest_accrual");
        query.push("' AND event_timestamp >= NOW() - INTERVAL '");
        query.push(interval);
        query.push("'");

        Self::push_yield_filters(&mut query, &filters);

        query.push(" GROUP BY 1, 2 ORDER BY 1 ASC, 2 ASC");

        let rows = query
            .build_query_as::<EarningsHistoryRow>()
            .fetch_all(db)
            .await?;

        let history: Vec<EarningsHistoryPoint> = rows
            .into_iter()
            .map(|row| EarningsHistoryPoint {
                period: row.period,
                asset_code: row.asset_code,
                earnings: row.earnings,
                event_count: row.event_count,
            })
            .collect();

        let total_earnings = history.iter().map(|point| point.earnings).sum();

        Ok(EarningsHistoryResponse {
            filters,
            range: range.to_string(),
            total_earnings,
            history,
        })
    }

    async fn get_interest_accrual_events(
        db: &PgPool,
        filters: &YieldReportFilters,
    ) -> Result<Vec<YieldAccrualEventRow>, ApiError> {
        let mut query = QueryBuilder::<Postgres>::new(
            "SELECT asset_code, user_id, plan_id, amount, metadata, event_timestamp FROM lending_events WHERE event_type = '",
        );
        query.push("interest_accrual");
        query.push("'");
        Self::push_yield_filters(&mut query, filters);
        query.push(" ORDER BY asset_code ASC, event_timestamp DESC");

        Ok(query
            .build_query_as::<YieldAccrualEventRow>()
            .fetch_all(db)
            .await?)
    }

    fn push_yield_filters(query: &mut QueryBuilder<'_, Postgres>, filters: &YieldReportFilters) {
        if let Some(asset_code) = &filters.asset_code {
            query.push(" AND UPPER(asset_code) = ");
            query.push_bind(asset_code.to_uppercase());
        }

        if let Some(user_id) = filters.user_id {
            query.push(" AND user_id = ");
            query.push_bind(user_id);
        }

        if let Some(plan_id) = filters.plan_id {
            query.push(" AND plan_id = ");
            query.push_bind(plan_id);
        }
    }

    fn parse_decimal(value: &str, field_name: &str) -> Result<Decimal, ApiError> {
        Decimal::from_str(value).map_err(|error| {
            ApiError::Internal(anyhow::anyhow!(
                "Failed to parse {field_name} from yield reporting query: {error}"
            ))
        })
    }

    fn decimal_from_metadata(metadata: &serde_json::Value, key: &str) -> Option<Decimal> {
        let value = metadata.get(key)?;
        match value {
            serde_json::Value::String(raw) => Decimal::from_str(raw).ok(),
            serde_json::Value::Number(number) => Decimal::from_str(&number.to_string()).ok(),
            _ => None,
        }
    }

    fn normalize_rate(rate: Decimal) -> Decimal {
        if rate > Decimal::ONE {
            rate / Decimal::new(100, 0)
        } else {
            rate
        }
    }

    fn annual_rate_to_apy(rate: Decimal) -> f64 {
        let normalized_rate = Self::normalize_rate(rate);
        let rate_f64 = normalized_rate.to_string().parse::<f64>().unwrap_or(0.0);
        ((1.0 + (rate_f64 / 365.0)).powf(365.0) - 1.0) * 100.0
    }

    fn decimal_to_f64(value: Decimal) -> Result<f64, ApiError> {
        value.to_string().parse::<f64>().map_err(|error| {
            ApiError::Internal(anyhow::anyhow!(
                "Failed to convert decimal to f64 for yield reporting: {error}"
            ))
        })
    }
}

// =============================================================================
// Loan Simulation Types and Service
// =============================================================================

/// Collateral type for loan simulations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CollateralType {
    Usdc,
    Eth,
    Btc,
    StellarXlm,
}

impl CollateralType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CollateralType::Usdc => "USDC",
            CollateralType::Eth => "ETH",
            CollateralType::Btc => "BTC",
            CollateralType::StellarXlm => "STELLAR_XLM",
        }
    }

    /// Get the Loan-to-Value ratio for this collateral type
    /// Higher quality collateral = higher LTV
    pub fn get_ltv_ratio(&self) -> rust_decimal::Decimal {
        match self {
            // Stablecoin - lowest risk
            CollateralType::Usdc => rust_decimal::Decimal::new(90, 2), // 0.90
            // Major crypto assets
            CollateralType::Eth => rust_decimal::Decimal::new(75, 2), // 0.75
            CollateralType::Btc => rust_decimal::Decimal::new(75, 2), // 0.75
            // Smaller cap crypto
            CollateralType::StellarXlm => rust_decimal::Decimal::new(60, 2), // 0.60
        }
    }

    /// Get the annual interest rate for loans with this collateral
    pub fn get_annual_interest_rate(&self) -> rust_decimal::Decimal {
        match self {
            // Stablecoin - lower risk = lower rate
            CollateralType::Usdc => rust_decimal::Decimal::new(5, 2), // 5%
            // Major crypto
            CollateralType::Eth => rust_decimal::Decimal::new(8, 2), // 8%
            CollateralType::Btc => rust_decimal::Decimal::new(8, 2), // 8%
            // Higher volatility
            CollateralType::StellarXlm => rust_decimal::Decimal::new(12, 2), // 12%
        }
    }

    /// Get the liquidation threshold for this collateral type
    /// When collateral value drops below this % of loan value, liquidation occurs
    pub fn get_liquidation_threshold(&self) -> rust_decimal::Decimal {
        match self {
            CollateralType::Usdc => rust_decimal::Decimal::new(95, 2), // 0.95
            CollateralType::Eth => rust_decimal::Decimal::new(85, 2),  // 0.85
            CollateralType::Btc => rust_decimal::Decimal::new(85, 2),  // 0.85
            CollateralType::StellarXlm => rust_decimal::Decimal::new(80, 2), // 0.80
        }
    }
}

impl FromStr for CollateralType {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "USDC" => Ok(CollateralType::Usdc),
            "ETH" => Ok(CollateralType::Eth),
            "BTC" => Ok(CollateralType::Btc),
            "STELLAR_XLM" | "XLM" => Ok(CollateralType::StellarXlm),
            _ => Err(ApiError::BadRequest(
                "collateral_type must be USDC, ETH, BTC, or STELLAR_XLM".to_string(),
            )),
        }
    }
}

/// Request to simulate a loan
#[derive(Debug, Deserialize)]
pub struct LoanSimulationRequest {
    /// Amount the user wants to borrow in USDC
    pub loan_amount: rust_decimal::Decimal,
    /// Duration of the loan in days
    pub loan_duration_days: u32,
    /// Type of collateral being used
    pub collateral_type: String,
    /// Current price of the collateral in USD
    pub collateral_price_usd: rust_decimal::Decimal,
}

/// Response containing loan simulation results
#[derive(Debug, Serialize, Deserialize)]
pub struct LoanSimulationResult {
    /// Input parameters
    pub loan_amount: rust_decimal::Decimal,
    pub loan_duration_days: u32,
    pub collateral_type: String,
    pub collateral_price_usd: rust_decimal::Decimal,

    /// Calculation results
    /// Minimum collateral value required (loan_amount / LTV)
    pub required_collateral_usd: rust_decimal::Decimal,
    /// Quantity of collateral needed
    pub collateral_quantity: rust_decimal::Decimal,
    /// Interest to be paid for this loan duration
    pub estimated_interest: rust_decimal::Decimal,
    /// Total amount to repay (principal + interest)
    pub total_repayment: rust_decimal::Decimal,
    /// Price at which collateral would be liquidated
    pub liquidation_price: rust_decimal::Decimal,
    /// Loan-to-Value ratio used
    pub loan_to_value_ratio: rust_decimal::Decimal,
    /// Annual interest rate used
    pub annual_interest_rate: rust_decimal::Decimal,
    /// Liquidation threshold percentage
    pub liquidation_threshold: rust_decimal::Decimal,
}

/// Record of a simulation stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LoanSimulationRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub loan_amount: rust_decimal::Decimal,
    pub loan_duration_days: i32,
    pub collateral_type: String,
    pub collateral_price_usd: rust_decimal::Decimal,
    pub required_collateral: rust_decimal::Decimal,
    pub collateral_quantity: rust_decimal::Decimal,
    pub estimated_interest: rust_decimal::Decimal,
    pub total_repayment: rust_decimal::Decimal,
    pub liquidation_price: rust_decimal::Decimal,
    pub loan_to_value_ratio: rust_decimal::Decimal,
    pub interest_rate: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
}

pub struct LoanSimulationService;

impl LoanSimulationService {
    /// Calculate loan simulation results
    pub fn calculate_simulation(
        req: &LoanSimulationRequest,
    ) -> Result<LoanSimulationResult, ApiError> {
        // Validate inputs
        if req.loan_amount <= rust_decimal::Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "loan_amount must be greater than 0".to_string(),
            ));
        }
        if req.loan_duration_days == 0 {
            return Err(ApiError::BadRequest(
                "loan_duration_days must be greater than 0".to_string(),
            ));
        }
        if req.collateral_price_usd <= rust_decimal::Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "collateral_price_usd must be greater than 0".to_string(),
            ));
        }

        // Parse collateral type
        let collateral_type = CollateralType::from_str(&req.collateral_type)?;

        // Get parameters based on collateral type
        let ltv_ratio = collateral_type.get_ltv_ratio();
        let annual_interest_rate = collateral_type.get_annual_interest_rate();
        let liquidation_threshold = collateral_type.get_liquidation_threshold();

        // Calculate required collateral value
        // required_collateral = loan_amount / LTV
        let required_collateral_usd = req.loan_amount / ltv_ratio;

        // Calculate collateral quantity
        // collateral_quantity = required_collateral_usd / collateral_price_usd
        let collateral_quantity = required_collateral_usd / req.collateral_price_usd;

        // Calculate interest
        // interest = loan_amount * annual_rate * (days / 365)
        let days_fraction = rust_decimal::Decimal::new(req.loan_duration_days as i64, 0)
            / rust_decimal::Decimal::new(365, 0);
        let estimated_interest = req.loan_amount * annual_interest_rate * days_fraction;

        // Calculate total repayment
        let total_repayment = req.loan_amount + estimated_interest;

        // Calculate liquidation price
        // Liquidation occurs when: collateral_quantity * price < loan_amount / liquidation_threshold
        // Solving for price: liquidation_price = (loan_amount / liquidation_threshold) / collateral_quantity
        let liquidation_price = (req.loan_amount / liquidation_threshold) / collateral_quantity;

        Ok(LoanSimulationResult {
            loan_amount: req.loan_amount,
            loan_duration_days: req.loan_duration_days,
            collateral_type: req.collateral_type.clone(),
            collateral_price_usd: req.collateral_price_usd,
            required_collateral_usd,
            collateral_quantity,
            estimated_interest,
            total_repayment,
            liquidation_price,
            loan_to_value_ratio: ltv_ratio,
            annual_interest_rate,
            liquidation_threshold,
        })
    }

    /// Create and store a loan simulation
    pub async fn create_simulation(
        db: &PgPool,
        user_id: Uuid,
        req: &LoanSimulationRequest,
    ) -> Result<LoanSimulationResult, ApiError> {
        // Calculate simulation
        let result = Self::calculate_simulation(req)?;

        // Store in database
        sqlx::query(
            r#"
            INSERT INTO loan_simulations (
                user_id, loan_amount, loan_duration_days, collateral_type, collateral_price_usd,
                required_collateral, collateral_quantity, estimated_interest, total_repayment,
                liquidation_price, loan_to_value_ratio, interest_rate
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(user_id)
        .bind(result.loan_amount)
        .bind(result.loan_duration_days as i32)
        .bind(&result.collateral_type)
        .bind(result.collateral_price_usd)
        .bind(result.required_collateral_usd)
        .bind(result.collateral_quantity)
        .bind(result.estimated_interest)
        .bind(result.total_repayment)
        .bind(result.liquidation_price)
        .bind(result.loan_to_value_ratio)
        .bind(result.annual_interest_rate)
        .execute(db)
        .await?;

        Ok(result)
    }

    /// Get simulation without storing (preview only)
    pub fn preview_simulation(
        req: &LoanSimulationRequest,
    ) -> Result<LoanSimulationResult, ApiError> {
        Self::calculate_simulation(req)
    }

    /// Get all simulations for a user
    pub async fn get_user_simulations(
        db: &PgPool,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<LoanSimulationRecord>, ApiError> {
        let records = sqlx::query_as::<_, LoanSimulationRecord>(
            r#"
            SELECT id, user_id, loan_amount, loan_duration_days, collateral_type,
                   collateral_price_usd, required_collateral, collateral_quantity,
                   estimated_interest, total_repayment, liquidation_price,
                   loan_to_value_ratio, interest_rate, created_at
            FROM loan_simulations
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(db)
        .await?;

        Ok(records)
    }

    /// Get a specific simulation by ID
    pub async fn get_simulation_by_id(
        db: &PgPool,
        simulation_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<LoanSimulationRecord>, ApiError> {
        let record = sqlx::query_as::<_, LoanSimulationRecord>(
            r#"
            SELECT id, user_id, loan_amount, loan_duration_days, collateral_type,
                   collateral_price_usd, required_collateral, collateral_quantity,
                   estimated_interest, total_repayment, liquidation_price,
                   loan_to_value_ratio, interest_rate, created_at
            FROM loan_simulations
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(simulation_id)
        .bind(user_id)
        .fetch_optional(db)
        .await?;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::{CurrencyPreference, InheritanceExecutionSafety, PlanService};
    use crate::api_error::ApiError;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn currency_preference_accepts_usdc() {
        assert_eq!(
            CurrencyPreference::from_str("USDC").unwrap(),
            CurrencyPreference::Usdc
        );
        assert_eq!(
            CurrencyPreference::from_str("usdc").unwrap(),
            CurrencyPreference::Usdc
        );
        assert_eq!(CurrencyPreference::Usdc.as_str(), "USDC");
    }

    #[test]
    fn currency_preference_accepts_fiat() {
        assert_eq!(
            CurrencyPreference::from_str("FIAT").unwrap(),
            CurrencyPreference::Fiat
        );
        assert_eq!(
            CurrencyPreference::from_str("fiat").unwrap(),
            CurrencyPreference::Fiat
        );
        assert_eq!(CurrencyPreference::Fiat.as_str(), "FIAT");
    }

    #[test]
    fn currency_preference_rejects_invalid() {
        let err = CurrencyPreference::from_str("EUR").unwrap_err();
        assert!(matches!(err, ApiError::BadRequest(_)));
        assert!(err.to_string().contains("USDC or FIAT"));
    }

    #[test]
    fn validate_beneficiary_usdc_does_not_require_bank() {
        assert!(PlanService::validate_beneficiary_for_currency(
            &CurrencyPreference::Usdc,
            None,
            None,
            None
        )
        .is_ok());
        assert!(PlanService::validate_beneficiary_for_currency(
            &CurrencyPreference::Usdc,
            Some(""),
            Some(""),
            None
        )
        .is_ok());
    }

    #[test]
    fn validate_beneficiary_fiat_requires_all_fields() {
        assert!(PlanService::validate_beneficiary_for_currency(
            &CurrencyPreference::Fiat,
            None,
            None,
            None
        )
        .is_err());
        assert!(PlanService::validate_beneficiary_for_currency(
            &CurrencyPreference::Fiat,
            Some("Jane Doe"),
            None,
            None
        )
        .is_err());
        assert!(PlanService::validate_beneficiary_for_currency(
            &CurrencyPreference::Fiat,
            Some("Jane Doe"),
            Some("Acme Bank"),
            None
        )
        .is_err());
        assert!(PlanService::validate_beneficiary_for_currency(
            &CurrencyPreference::Fiat,
            Some("Jane Doe"),
            Some("Acme Bank"),
            Some("12345678")
        )
        .is_ok());
    }

    #[test]
    fn validate_beneficiary_fiat_rejects_whitespace_only() {
        assert!(PlanService::validate_beneficiary_for_currency(
            &CurrencyPreference::Fiat,
            Some("  "),
            Some("Acme Bank"),
            Some("12345678")
        )
        .is_err());
    }

    #[test]
    fn inheritance_execution_safety_blocks_active_lending_utilization() {
        let safety = InheritanceExecutionSafety::from_plan_state(
            Decimal::new(1000, 0),
            Decimal::new(250, 0),
            false,
            false,
        );

        let reason = safety
            .blocking_reason()
            .expect("expected active lending utilization to block execution");

        assert!(reason.contains("utilized in lending"));
        assert!(reason.contains("25"));
    }

    #[test]
    fn inheritance_execution_safety_blocks_risky_plan_without_override() {
        let safety = InheritanceExecutionSafety::from_plan_state(
            Decimal::new(1000, 0),
            Decimal::ZERO,
            true,
            false,
        );

        let reason = safety
            .blocking_reason()
            .expect("expected risky plans to block execution");

        assert!(reason.contains("flagged as risky"));
    }

    #[test]
    fn inheritance_execution_safety_allows_clean_plan() {
        let safety = InheritanceExecutionSafety::from_plan_state(
            Decimal::new(1000, 0),
            Decimal::ZERO,
            false,
            false,
        );

        assert!(safety.blocking_reason().is_none());
    }

    // ========================================================================
    // Loan Simulation Tests
    // ========================================================================

    use super::{CollateralType, LoanSimulationRequest, LoanSimulationService};
    use rust_decimal_macros::dec;

    #[test]
    fn collateral_type_parsing_usdc() {
        assert_eq!(
            CollateralType::from_str("USDC").unwrap(),
            CollateralType::Usdc
        );
        assert_eq!(
            CollateralType::from_str("usdc").unwrap(),
            CollateralType::Usdc
        );
    }

    #[test]
    fn collateral_type_parsing_eth() {
        assert_eq!(
            CollateralType::from_str("ETH").unwrap(),
            CollateralType::Eth
        );
        assert_eq!(
            CollateralType::from_str("eth").unwrap(),
            CollateralType::Eth
        );
    }

    #[test]
    fn collateral_type_parsing_btc() {
        assert_eq!(
            CollateralType::from_str("BTC").unwrap(),
            CollateralType::Btc
        );
        assert_eq!(
            CollateralType::from_str("btc").unwrap(),
            CollateralType::Btc
        );
    }

    #[test]
    fn collateral_type_parsing_xlm() {
        assert_eq!(
            CollateralType::from_str("STELLAR_XLM").unwrap(),
            CollateralType::StellarXlm
        );
        assert_eq!(
            CollateralType::from_str("XLM").unwrap(),
            CollateralType::StellarXlm
        );
    }

    #[test]
    fn collateral_type_parsing_rejects_invalid() {
        let err = CollateralType::from_str("INVALID").unwrap_err();
        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[test]
    fn collateral_type_ltv_ratios() {
        // USDC should have highest LTV (0.90)
        assert_eq!(CollateralType::Usdc.get_ltv_ratio(), dec!(0.90));
        // ETH and BTC should have same LTV (0.75)
        assert_eq!(CollateralType::Eth.get_ltv_ratio(), dec!(0.75));
        assert_eq!(CollateralType::Btc.get_ltv_ratio(), dec!(0.75));
        // XLM should have lowest LTV (0.60)
        assert_eq!(CollateralType::StellarXlm.get_ltv_ratio(), dec!(0.60));
    }

    #[test]
    fn collateral_type_interest_rates() {
        // USDC should have lowest rate (5%)
        assert_eq!(CollateralType::Usdc.get_annual_interest_rate(), dec!(0.05));
        // ETH and BTC should have same rate (8%)
        assert_eq!(CollateralType::Eth.get_annual_interest_rate(), dec!(0.08));
        assert_eq!(CollateralType::Btc.get_annual_interest_rate(), dec!(0.08));
        // XLM should have highest rate (12%)
        assert_eq!(
            CollateralType::StellarXlm.get_annual_interest_rate(),
            dec!(0.12)
        );
    }

    #[test]
    fn collateral_type_liquidation_thresholds() {
        // USDC should have highest threshold (0.95)
        assert_eq!(CollateralType::Usdc.get_liquidation_threshold(), dec!(0.95));
        // ETH and BTC should have same threshold (0.85)
        assert_eq!(CollateralType::Eth.get_liquidation_threshold(), dec!(0.85));
        assert_eq!(CollateralType::Btc.get_liquidation_threshold(), dec!(0.85));
        // XLM should have lowest threshold (0.80)
        assert_eq!(
            CollateralType::StellarXlm.get_liquidation_threshold(),
            dec!(0.80)
        );
    }

    #[test]
    fn loan_simulation_calculation_usdc() {
        let req = LoanSimulationRequest {
            loan_amount: dec!(10000),
            loan_duration_days: 30,
            collateral_type: "USDC".to_string(),
            collateral_price_usd: dec!(1),
        };

        let result = LoanSimulationService::calculate_simulation(&req).unwrap();

        // Required collateral = 10000 / 0.90 = 11111.11...
        assert!(result.required_collateral_usd > dec!(11111));
        assert!(result.required_collateral_usd < dec!(11112));

        // Collateral quantity should be roughly same as USD value for USDC (price = 1)
        assert!(result.collateral_quantity > dec!(11111));

        // Interest = 10000 * 0.05 * (30/365) = ~41.09
        assert!(result.estimated_interest > dec!(41));
        assert!(result.estimated_interest < dec!(42));

        // Total repayment = 10000 + interest
        assert!(result.total_repayment > dec!(10041));
        assert!(result.total_repayment < dec!(10042));

        // LTV should be 0.90
        assert_eq!(result.loan_to_value_ratio, dec!(0.90));

        // Annual interest rate should be 0.05
        assert_eq!(result.annual_interest_rate, dec!(0.05));
    }

    #[test]
    fn loan_simulation_calculation_eth() {
        let eth_price = dec!(2000);
        let req = LoanSimulationRequest {
            loan_amount: dec!(10000),
            loan_duration_days: 90,
            collateral_type: "ETH".to_string(),
            collateral_price_usd: eth_price,
        };

        let result = LoanSimulationService::calculate_simulation(&req).unwrap();

        // Required collateral = 10000 / 0.75 = 13333.33...
        assert!(result.required_collateral_usd > dec!(13333));

        // Collateral quantity = 13333.33 / 2000 = ~6.67 ETH
        assert!(result.collateral_quantity > dec!(6));
        assert!(result.collateral_quantity < dec!(7));

        // Interest = 10000 * 0.08 * (90/365) = ~197.26
        assert!(result.estimated_interest > dec!(197));
        assert!(result.estimated_interest < dec!(198));

        // LTV should be 0.75
        assert_eq!(result.loan_to_value_ratio, dec!(0.75));
    }

    #[test]
    fn loan_simulation_calculation_btc() {
        let btc_price = dec!(50000);
        let req = LoanSimulationRequest {
            loan_amount: dec!(25000),
            loan_duration_days: 180,
            collateral_type: "BTC".to_string(),
            collateral_price_usd: btc_price,
        };

        let result = LoanSimulationService::calculate_simulation(&req).unwrap();

        // Required collateral = 25000 / 0.75 = 33333.33...
        assert!(result.required_collateral_usd > dec!(33333));

        // Collateral quantity = 33333.33 / 50000 = ~0.667 BTC
        assert!(result.collateral_quantity > dec!(0.6));
        assert!(result.collateral_quantity < dec!(0.7));

        // Interest = 25000 * 0.08 * (180/365) = ~986.30
        assert!(result.estimated_interest > dec!(986));
        assert!(result.estimated_interest < dec!(987));
    }

    #[test]
    fn loan_simulation_calculation_xlm() {
        let xlm_price = dec!(0.10);
        let req = LoanSimulationRequest {
            loan_amount: dec!(1000),
            loan_duration_days: 60,
            collateral_type: "XLM".to_string(),
            collateral_price_usd: xlm_price,
        };

        let result = LoanSimulationService::calculate_simulation(&req).unwrap();

        // Required collateral = 1000 / 0.60 = 1666.67
        assert!(result.required_collateral_usd > dec!(1666));

        // Collateral quantity = 1666.67 / 0.10 = 16666.67 XLM
        assert!(result.collateral_quantity > dec!(16666));

        // Interest = 1000 * 0.12 * (60/365) = ~19.73
        assert!(result.estimated_interest > dec!(19));
        assert!(result.estimated_interest < dec!(20));

        // LTV should be 0.60
        assert_eq!(result.loan_to_value_ratio, dec!(0.60));
    }

    #[test]
    fn loan_simulation_rejects_zero_loan_amount() {
        let req = LoanSimulationRequest {
            loan_amount: dec!(0),
            loan_duration_days: 30,
            collateral_type: "ETH".to_string(),
            collateral_price_usd: dec!(2000),
        };

        let result = LoanSimulationService::calculate_simulation(&req);
        assert!(result.is_err());
    }

    #[test]
    fn loan_simulation_rejects_zero_duration() {
        let req = LoanSimulationRequest {
            loan_amount: dec!(1000),
            loan_duration_days: 0,
            collateral_type: "ETH".to_string(),
            collateral_price_usd: dec!(2000),
        };

        let result = LoanSimulationService::calculate_simulation(&req);
        assert!(result.is_err());
    }

    #[test]
    fn loan_simulation_rejects_zero_collateral_price() {
        let req = LoanSimulationRequest {
            loan_amount: dec!(1000),
            loan_duration_days: 30,
            collateral_type: "ETH".to_string(),
            collateral_price_usd: dec!(0),
        };

        let result = LoanSimulationService::calculate_simulation(&req);
        assert!(result.is_err());
    }

    #[test]
    fn loan_simulation_rejects_invalid_collateral_type() {
        let req = LoanSimulationRequest {
            loan_amount: dec!(1000),
            loan_duration_days: 30,
            collateral_type: "INVALID".to_string(),
            collateral_price_usd: dec!(2000),
        };

        let result = LoanSimulationService::calculate_simulation(&req);
        assert!(result.is_err());
    }

    #[test]
    fn loan_simulation_liquidation_price_logic() {
        // Test that liquidation price is calculated correctly
        // For ETH: if ETH price drops below liquidation price, position gets liquidated
        let eth_price = dec!(2000);
        let req = LoanSimulationRequest {
            loan_amount: dec!(10000),
            loan_duration_days: 30,
            collateral_type: "ETH".to_string(),
            collateral_price_usd: eth_price,
        };

        let result = LoanSimulationService::calculate_simulation(&req).unwrap();

        // Liquidation price should be less than current price
        // (otherwise liquidation would happen immediately)
        assert!(result.liquidation_price < eth_price);

        // Liquidation price should be positive
        assert!(result.liquidation_price > dec!(0));

        // For ETH with 0.85 threshold, liquidation price should be around:
        // (10000 / 0.85) / 6.67 ≈ 1764
        assert!(result.liquidation_price > dec!(1500));
        assert!(result.liquidation_price < dec!(2000));
    }
}

// ── Emergency Admin Controls ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PausePlanRequest {
    pub plan_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct UnpausePlanRequest {
    pub plan_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RiskOverrideRequest {
    pub plan_id: Uuid,
    pub enabled: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmergencyContact {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub relationship: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub wallet_address: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmergencyContactRequest {
    pub name: String,
    pub relationship: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub wallet_address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmergencyContactRequest {
    pub name: String,
    pub relationship: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub wallet_address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmergencyContactDeleteResponse {
    pub success: bool,
    pub contact_id: Uuid,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmergencyAccessGrant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub emergency_contact_id: Uuid,
    pub permissions: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmergencyAccessAuditLog {
    pub id: Uuid,
    pub grant_id: Uuid,
    pub user_id: Uuid,
    pub emergency_contact_id: Uuid,
    pub action: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct EmergencyAccessAuditLogFilters {
    pub action: Option<String>,
    pub grant_id: Option<Uuid>,
    pub emergency_contact_id: Option<Uuid>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmergencyAccessGrantRequest {
    pub emergency_contact_id: Uuid,
    pub permissions: Vec<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RevokeEmergencyAccessGrantRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmergencyAccessActionResponse {
    pub success: bool,
    pub grant: EmergencyAccessGrant,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmergencyAccessRiskAlert {
    pub id: Uuid,
    pub grant_id: Uuid,
    pub user_id: Uuid,
    pub emergency_contact_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAccessDashboardItem {
    pub grant_id: Uuid,
    pub emergency_contact_id: Uuid,
    pub contact_name: String,
    pub status: String,
    pub active_access: bool,
    pub permissions: Vec<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAccessDashboardResponse {
    pub active_access_count: usize,
    pub grants: Vec<EmergencyAccessDashboardItem>,
}

// ─── Emergency Access Sessions (Issue #306) ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmergencyAccessSession {
    pub id: Uuid,
    pub grant_id: Uuid,
    pub user_id: Uuid,
    pub emergency_contact_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct StartSessionRequest {
    pub grant_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmergencyActionResponse {
    pub success: bool,
    pub plan_id: Uuid,
    pub message: String,
}

pub struct EmergencyContactService;

impl EmergencyContactService {
    fn normalize_optional(value: &Option<String>) -> Option<String> {
        value.as_ref().and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }

    fn validate_contact_details(
        name: &str,
        relationship: &str,
        email: &Option<String>,
        phone: &Option<String>,
        wallet_address: &Option<String>,
    ) -> Result<(), ApiError> {
        if name.trim().is_empty() {
            return Err(ApiError::BadRequest(
                "name is required for an emergency contact".to_string(),
            ));
        }

        if relationship.trim().is_empty() {
            return Err(ApiError::BadRequest(
                "relationship is required for an emergency contact".to_string(),
            ));
        }

        if email.is_none() && phone.is_none() && wallet_address.is_none() {
            return Err(ApiError::BadRequest(
                "at least one contact detail is required: email, phone, or wallet_address"
                    .to_string(),
            ));
        }

        Ok(())
    }

    pub async fn list_for_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<EmergencyContact>, ApiError> {
        let contacts = sqlx::query_as::<_, EmergencyContact>(
            r#"
            SELECT id, user_id, name, relationship, email, phone, wallet_address, notes,
                   created_at, updated_at
            FROM emergency_contacts
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(contacts)
    }

    pub async fn create_contact(
        pool: &PgPool,
        user_id: Uuid,
        req: &CreateEmergencyContactRequest,
    ) -> Result<EmergencyContact, ApiError> {
        let email = Self::normalize_optional(&req.email);
        let phone = Self::normalize_optional(&req.phone);
        let wallet_address = Self::normalize_optional(&req.wallet_address);
        let notes = Self::normalize_optional(&req.notes);
        let name = req.name.trim();
        let relationship = req.relationship.trim();

        Self::validate_contact_details(name, relationship, &email, &phone, &wallet_address)?;

        let contact = sqlx::query_as::<_, EmergencyContact>(
            r#"
            INSERT INTO emergency_contacts (
                user_id, name, relationship, email, phone, wallet_address, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, name, relationship, email, phone, wallet_address, notes,
                      created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(relationship)
        .bind(email)
        .bind(phone)
        .bind(wallet_address)
        .bind(notes)
        .fetch_one(pool)
        .await?;

        Ok(contact)
    }

    pub async fn update_contact(
        pool: &PgPool,
        user_id: Uuid,
        contact_id: Uuid,
        req: &UpdateEmergencyContactRequest,
    ) -> Result<EmergencyContact, ApiError> {
        let email = Self::normalize_optional(&req.email);
        let phone = Self::normalize_optional(&req.phone);
        let wallet_address = Self::normalize_optional(&req.wallet_address);
        let notes = Self::normalize_optional(&req.notes);
        let name = req.name.trim();
        let relationship = req.relationship.trim();

        Self::validate_contact_details(name, relationship, &email, &phone, &wallet_address)?;

        let updated = sqlx::query_as::<_, EmergencyContact>(
            r#"
            UPDATE emergency_contacts
            SET name = $1,
                relationship = $2,
                email = $3,
                phone = $4,
                wallet_address = $5,
                notes = $6,
                updated_at = NOW()
            WHERE id = $7 AND user_id = $8
            RETURNING id, user_id, name, relationship, email, phone, wallet_address, notes,
                      created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(relationship)
        .bind(email)
        .bind(phone)
        .bind(wallet_address)
        .bind(notes)
        .bind(contact_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        updated.ok_or_else(|| {
            ApiError::NotFound(format!("Emergency contact {} not found", contact_id))
        })
    }

    pub async fn delete_contact(
        pool: &PgPool,
        user_id: Uuid,
        contact_id: Uuid,
    ) -> Result<EmergencyContactDeleteResponse, ApiError> {
        let deleted = sqlx::query_scalar::<_, Uuid>(
            "DELETE FROM emergency_contacts WHERE id = $1 AND user_id = $2 RETURNING id",
        )
        .bind(contact_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match deleted {
            Some(contact_id) => Ok(EmergencyContactDeleteResponse {
                success: true,
                contact_id,
                message: "Emergency contact deleted successfully".to_string(),
            }),
            None => Err(ApiError::NotFound(format!(
                "Emergency contact {} not found",
                contact_id
            ))),
        }
    }
}

pub struct EmergencyAccessService;

struct RiskAlertInput<'a> {
    grant_id: Uuid,
    user_id: Uuid,
    contact_id: Uuid,
    alert_type: &'a str,
    severity: &'a str,
    message: &'a str,
    metadata: serde_json::Value,
}

impl EmergencyAccessService {
    async fn create_risk_alert(
        pool: &PgPool,
        executor: &mut sqlx::PgConnection,
        input: RiskAlertInput<'_>,
    ) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            INSERT INTO emergency_access_risk_alerts (
                grant_id, user_id, emergency_contact_id, alert_type, severity, message, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(input.grant_id)
        .bind(input.user_id)
        .bind(input.contact_id)
        .bind(input.alert_type)
        .bind(input.severity)
        .bind(input.message)
        .bind(input.metadata)
        .execute(executor)
        .await?;

        // After persisting the alert, trigger external notifications
        use crate::alert_provider::MockAlertProvider;
        use crate::notifications::EmergencyAlertService;

        let provider = MockAlertProvider;
        if let Err(e) = EmergencyAlertService::send_risk_alert(
            pool,
            &provider,
            input.user_id,
            input.contact_id,
            input.alert_type,
            input.message,
        )
        .await
        {
            tracing::error!("Failed to send emergency risk alert: {}", e);
            // We don't return error here to avoid rolling back the risk alert insertion
            // because the alert itself is already in the DB.
        }

        Ok(())
    }

    async fn evaluate_grant_risk(
        pool: &PgPool,
        executor: &mut sqlx::PgConnection,
        grant: &EmergencyAccessGrant,
    ) -> Result<(), ApiError> {
        let recent_grant_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM emergency_access_grants
            WHERE user_id = $1
              AND created_at >= NOW() - INTERVAL '1 hour'
            "#,
        )
        .bind(grant.user_id)
        .fetch_one(&mut *executor)
        .await?;

        if recent_grant_count >= 4 {
            Self::create_risk_alert(
                pool,
                executor,
                RiskAlertInput {
                    grant_id: grant.id,
                    user_id: grant.user_id,
                    contact_id: grant.emergency_contact_id,
                    alert_type: "high_frequency_grants",
                    severity: "high",
                    message:
                        "Multiple emergency access grants were created within a short time window.",
                    metadata: serde_json::json!({ "recent_grant_count": recent_grant_count }),
                },
            )
            .await?;
        }

        let long_lived_high_privilege = grant.permissions.iter().any(|permission| {
            permission == "transfer_funds" || permission == "manage_beneficiaries"
        }) && grant.expires_at
            >= Utc::now() + chrono::Duration::days(7);

        if long_lived_high_privilege {
            Self::create_risk_alert(
                pool,
                executor,
                RiskAlertInput {
                    grant_id: grant.id,
                    user_id: grant.user_id,
                    contact_id: grant.emergency_contact_id,
                    alert_type: "high_privilege_long_lived_access",
                    severity: "medium",
                    message: "High-privilege emergency access was granted with a long expiration window.",
                    metadata: serde_json::json!({
                        "permissions": grant.permissions,
                        "expires_at": grant.expires_at
                    }),
                },
            )
            .await?;
        }

        Ok(())
    }

    async fn evaluate_revoke_risk(
        pool: &PgPool,
        executor: &mut sqlx::PgConnection,
        grant: &EmergencyAccessGrant,
    ) -> Result<(), ApiError> {
        if let Some(revoked_at) = grant.revoked_at {
            let active_duration = revoked_at - grant.created_at;
            if active_duration <= chrono::Duration::minutes(10) {
                Self::create_risk_alert(
                    pool,
                    executor,
                    RiskAlertInput {
                        grant_id: grant.id,
                        user_id: grant.user_id,
                        contact_id: grant.emergency_contact_id,
                        alert_type: "rapid_grant_revoke",
                        severity: "medium",
                        message: "Emergency access was revoked shortly after it was granted.",
                        metadata: serde_json::json!({
                            "granted_at": grant.created_at,
                            "revoked_at": revoked_at,
                            "active_duration_minutes": active_duration.num_minutes()
                        }),
                    },
                )
                .await?;
            }
        }

        Ok(())
    }

    fn normalize_permissions(permissions: &[String]) -> Vec<String> {
        permissions
            .iter()
            .filter_map(|permission| {
                let trimmed = permission.trim().to_lowercase();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            })
            .collect()
    }

    fn validate_grant_input(
        permissions: &[String],
        expires_at: DateTime<Utc>,
    ) -> Result<(), ApiError> {
        if permissions.is_empty() {
            return Err(ApiError::BadRequest(
                "permissions must contain at least one emergency access capability".to_string(),
            ));
        }

        if expires_at <= Utc::now() {
            return Err(ApiError::BadRequest(
                "expires_at must be set to a future time".to_string(),
            ));
        }

        Ok(())
    }

    async fn assert_contact_belongs_to_user(
        executor: impl sqlx::PgExecutor<'_>,
        user_id: Uuid,
        contact_id: Uuid,
    ) -> Result<(), ApiError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM emergency_contacts
                WHERE id = $1 AND user_id = $2
            )
            "#,
        )
        .bind(contact_id)
        .bind(user_id)
        .fetch_one(executor)
        .await?;

        if exists {
            Ok(())
        } else {
            Err(ApiError::NotFound(format!(
                "Emergency contact {} not found",
                contact_id
            )))
        }
    }

    async fn log_action(
        executor: impl sqlx::PgExecutor<'_>,
        grant_id: Uuid,
        user_id: Uuid,
        contact_id: Uuid,
        action: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            INSERT INTO emergency_access_audit_logs (
                grant_id, user_id, emergency_contact_id, action, metadata
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(grant_id)
        .bind(user_id)
        .bind(contact_id)
        .bind(action)
        .bind(metadata)
        .execute(executor)
        .await?;

        Ok(())
    }

    pub async fn grant_access(
        pool: &PgPool,
        user_id: Uuid,
        req: &CreateEmergencyAccessGrantRequest,
    ) -> Result<EmergencyAccessActionResponse, ApiError> {
        let permissions = Self::normalize_permissions(&req.permissions);
        Self::validate_grant_input(&permissions, req.expires_at)?;

        let mut tx = pool.begin().await?;
        Self::assert_contact_belongs_to_user(&mut *tx, user_id, req.emergency_contact_id).await?;

        let grant = sqlx::query_as::<_, EmergencyAccessGrant>(
            r#"
            INSERT INTO emergency_access_grants (
                user_id, emergency_contact_id, permissions, expires_at
            )
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, emergency_contact_id, permissions, expires_at, is_active,
                      revoked_at, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(req.emergency_contact_id)
        .bind(&permissions)
        .bind(req.expires_at)
        .fetch_one(&mut *tx)
        .await?;

        Self::log_action(
            &mut *tx,
            grant.id,
            user_id,
            grant.emergency_contact_id,
            audit_action::EMERGENCY_ACCESS_GRANTED,
            serde_json::json!({
                "permissions": grant.permissions,
                "expires_at": grant.expires_at
            }),
        )
        .await?;

        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            audit_action::EMERGENCY_ACCESS_GRANTED,
            Some(grant.id),
            Some(entity_type::USER),
        )
        .await?;

        Self::evaluate_grant_risk(pool, &mut tx, &grant).await?;

        tx.commit().await?;

        Ok(EmergencyAccessActionResponse {
            success: true,
            grant,
            message: "Emergency access granted successfully".to_string(),
        })
    }

    pub async fn revoke_access(
        pool: &PgPool,
        user_id: Uuid,
        grant_id: Uuid,
        req: &RevokeEmergencyAccessGrantRequest,
    ) -> Result<EmergencyAccessActionResponse, ApiError> {
        let reason = req.reason.as_ref().and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        let mut tx = pool.begin().await?;

        let grant = sqlx::query_as::<_, EmergencyAccessGrant>(
            r#"
            SELECT id, user_id, emergency_contact_id, permissions, expires_at, is_active,
                   revoked_at, created_at, updated_at
            FROM emergency_access_grants
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(grant_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("Emergency access grant {} not found", grant_id))
        })?;

        if !grant.is_active {
            return Err(ApiError::BadRequest(
                "Emergency access grant is already inactive".to_string(),
            ));
        }

        let updated = sqlx::query_as::<_, EmergencyAccessGrant>(
            r#"
            UPDATE emergency_access_grants
            SET is_active = false,
                revoked_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, user_id, emergency_contact_id, permissions, expires_at, is_active,
                      revoked_at, created_at, updated_at
            "#,
        )
        .bind(grant_id)
        .fetch_one(&mut *tx)
        .await?;

        Self::log_action(
            &mut *tx,
            updated.id,
            user_id,
            updated.emergency_contact_id,
            audit_action::EMERGENCY_ACCESS_REVOKED,
            serde_json::json!({
                "reason": reason,
                "revoked_at": updated.revoked_at
            }),
        )
        .await?;

        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            audit_action::EMERGENCY_ACCESS_REVOKED,
            Some(updated.id),
            Some(entity_type::USER),
        )
        .await?;

        Self::evaluate_revoke_risk(pool, &mut tx, &updated).await?;

        // Auto-end all active sessions for this grant (Issue #306)
        EmergencySessionService::end_sessions_for_grant(&mut *tx, updated.id).await?;

        tx.commit().await?;

        Ok(EmergencyAccessActionResponse {
            success: true,
            grant: updated,
            message: "Emergency access revoked successfully".to_string(),
        })
    }

    pub async fn list_audit_logs(
        pool: &PgPool,
        user_id: Uuid,
        filters: &EmergencyAccessAuditLogFilters,
    ) -> Result<Vec<EmergencyAccessAuditLog>, ApiError> {
        let limit = filters.limit.unwrap_or(50).min(100) as i64;

        let mut query = QueryBuilder::<Postgres>::new(
            "SELECT id, grant_id, user_id, emergency_contact_id, action, metadata, created_at \
             FROM emergency_access_audit_logs WHERE user_id = ",
        );
        query.push_bind(user_id);

        if let Some(action) = filters
            .action
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            query.push(" AND action = ");
            query.push_bind(action);
        }

        if let Some(grant_id) = filters.grant_id {
            query.push(" AND grant_id = ");
            query.push_bind(grant_id);
        }

        if let Some(emergency_contact_id) = filters.emergency_contact_id {
            query.push(" AND emergency_contact_id = ");
            query.push_bind(emergency_contact_id);
        }

        query.push(" ORDER BY created_at DESC LIMIT ");
        query.push_bind(limit);

        let logs = query
            .build_query_as::<EmergencyAccessAuditLog>()
            .fetch_all(pool)
            .await?;

        Ok(logs)
    }

    pub async fn list_risk_alerts(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<EmergencyAccessRiskAlert>, ApiError> {
        let alerts = sqlx::query_as::<_, EmergencyAccessRiskAlert>(
            r#"
            SELECT id, grant_id, user_id, emergency_contact_id, alert_type, severity, message,
                   metadata, created_at
            FROM emergency_access_risk_alerts
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(alerts)
    }

    pub async fn get_dashboard(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<EmergencyAccessDashboardResponse, ApiError> {
        #[derive(sqlx::FromRow)]
        struct DashboardRow {
            grant_id: Uuid,
            emergency_contact_id: Uuid,
            contact_name: String,
            is_active: bool,
            expires_at: DateTime<Utc>,
            permissions: Vec<String>,
        }

        let rows = sqlx::query_as::<_, DashboardRow>(
            r#"
            SELECT g.id AS grant_id,
                   g.emergency_contact_id,
                   c.name AS contact_name,
                   g.is_active,
                   g.expires_at,
                   g.permissions
            FROM emergency_access_grants g
            INNER JOIN emergency_contacts c ON c.id = g.emergency_contact_id
            WHERE g.user_id = $1
            ORDER BY g.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let now = Utc::now();
        let grants = rows
            .into_iter()
            .map(|row| {
                let active_access = row.is_active && row.expires_at > now;
                let status = if active_access {
                    "active"
                } else if row.is_active {
                    "expired"
                } else {
                    "revoked"
                };

                EmergencyAccessDashboardItem {
                    grant_id: row.grant_id,
                    emergency_contact_id: row.emergency_contact_id,
                    contact_name: row.contact_name,
                    status: status.to_string(),
                    active_access,
                    permissions: row.permissions,
                    expires_at: row.expires_at,
                }
            })
            .collect::<Vec<_>>();

        let active_access_count = grants.iter().filter(|grant| grant.active_access).count();

        Ok(EmergencyAccessDashboardResponse {
            active_access_count,
            grants,
        })
    }
}

pub struct EmergencyAdminService;

impl EmergencyAdminService {
    /// Pause a plan - prevents claims and other operations
    pub async fn pause_plan(
        pool: &PgPool,
        admin_id: Uuid,
        req: &PausePlanRequest,
    ) -> Result<EmergencyActionResponse, ApiError> {
        let mut tx = pool.begin().await?;

        // Verify plan exists
        let plan = PlanService::get_plan_by_id_any_user(&mut *tx, req.plan_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Plan {} not found", req.plan_id)))?;

        // Check if already paused
        let is_paused: Option<bool> =
            sqlx::query_scalar("SELECT is_paused FROM plans WHERE id = $1")
                .bind(req.plan_id)
                .fetch_one(&mut *tx)
                .await?;

        if is_paused == Some(true) {
            return Err(ApiError::BadRequest("Plan is already paused".to_string()));
        }

        // Update plan to paused state
        sqlx::query(
            r#"
            UPDATE plans
            SET is_paused = true,
                paused_by = $1,
                paused_at = NOW(),
                pause_reason = $2,
                updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(admin_id)
        .bind(&req.reason)
        .bind(req.plan_id)
        .execute(&mut *tx)
        .await?;

        // Audit log
        AuditLogService::log(
            &mut *tx,
            Some(admin_id),
            audit_action::PLAN_PAUSED,
            Some(req.plan_id),
            Some(entity_type::PLAN),
        )
        .await?;

        // Notify user
        NotificationService::create(
            &mut tx,
            plan.user_id,
            notif_type::PLAN_PAUSED,
            format!(
                "Your plan '{}' has been temporarily paused by an administrator. Reason: {}",
                plan.title, req.reason
            ),
        )
        .await?;

        tx.commit().await?;

        Ok(EmergencyActionResponse {
            success: true,
            plan_id: req.plan_id,
            message: "Plan paused successfully".to_string(),
        })
    }

    /// Unpause a plan - restores normal operations
    pub async fn unpause_plan(
        pool: &PgPool,
        admin_id: Uuid,
        req: &UnpausePlanRequest,
    ) -> Result<EmergencyActionResponse, ApiError> {
        let mut tx = pool.begin().await?;

        // Verify plan exists
        let plan = PlanService::get_plan_by_id_any_user(&mut *tx, req.plan_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Plan {} not found", req.plan_id)))?;

        // Check if actually paused
        let is_paused: Option<bool> =
            sqlx::query_scalar("SELECT is_paused FROM plans WHERE id = $1")
                .bind(req.plan_id)
                .fetch_one(&mut *tx)
                .await?;

        if is_paused != Some(true) {
            return Err(ApiError::BadRequest("Plan is not paused".to_string()));
        }

        // Update plan to unpaused state
        sqlx::query(
            r#"
            UPDATE plans
            SET is_paused = false,
                paused_by = NULL,
                paused_at = NULL,
                pause_reason = NULL,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(req.plan_id)
        .execute(&mut *tx)
        .await?;

        // Audit log
        AuditLogService::log(
            &mut *tx,
            Some(admin_id),
            audit_action::PLAN_UNPAUSED,
            Some(req.plan_id),
            Some(entity_type::PLAN),
        )
        .await?;

        // Notify user
        NotificationService::create(
            &mut tx,
            plan.user_id,
            notif_type::PLAN_UNPAUSED,
            format!(
                "Your plan '{}' has been unpaused and is now active again",
                plan.title
            ),
        )
        .await?;

        tx.commit().await?;

        Ok(EmergencyActionResponse {
            success: true,
            plan_id: req.plan_id,
            message: "Plan unpaused successfully".to_string(),
        })
    }

    /// Apply or remove risk override for a plan
    pub async fn set_risk_override(
        pool: &PgPool,
        admin_id: Uuid,
        req: &RiskOverrideRequest,
    ) -> Result<EmergencyActionResponse, ApiError> {
        let mut tx = pool.begin().await?;

        // Verify plan exists
        let plan = PlanService::get_plan_by_id_any_user(&mut *tx, req.plan_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Plan {} not found", req.plan_id)))?;

        let action_type = if req.enabled {
            audit_action::RISK_OVERRIDE_APPLIED
        } else {
            audit_action::RISK_OVERRIDE_REMOVED
        };

        let notif_type = if req.enabled {
            notif_type::RISK_OVERRIDE_APPLIED
        } else {
            notif_type::RISK_OVERRIDE_REMOVED
        };

        // Update risk override settings
        if req.enabled {
            sqlx::query(
                r#"
                UPDATE plans
                SET risk_override_enabled = true,
                    risk_override_by = $1,
                    risk_override_at = NOW(),
                    risk_override_reason = $2,
                    updated_at = NOW()
                WHERE id = $3
                "#,
            )
            .bind(admin_id)
            .bind(&req.reason)
            .bind(req.plan_id)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(
                r#"
                UPDATE plans
                SET risk_override_enabled = false,
                    risk_override_by = NULL,
                    risk_override_at = NULL,
                    risk_override_reason = NULL,
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(req.plan_id)
            .execute(&mut *tx)
            .await?;
        }

        // Audit log
        AuditLogService::log(
            &mut *tx,
            Some(admin_id),
            action_type,
            Some(req.plan_id),
            Some(entity_type::PLAN),
        )
        .await?;

        // Notify user
        let message = if req.enabled {
            format!(
                "Risk monitoring override has been applied to your plan '{}'. Reason: {}",
                plan.title, req.reason
            )
        } else {
            format!(
                "Risk monitoring override has been removed from your plan '{}'",
                plan.title
            )
        };

        NotificationService::create(&mut tx, plan.user_id, notif_type, message).await?;

        tx.commit().await?;

        let action_msg = if req.enabled {
            "Risk override applied successfully"
        } else {
            "Risk override removed successfully"
        };

        Ok(EmergencyActionResponse {
            success: true,
            plan_id: req.plan_id,
            message: action_msg.to_string(),
        })
    }

    /// Get all paused plans
    pub async fn get_paused_plans(db: &PgPool) -> Result<Vec<PlanWithBeneficiary>, ApiError> {
        let rows = sqlx::query_as::<_, PlanRowFull>(
            r#"
            SELECT id, user_id, title, description, fee, net_amount, status,
                   contract_plan_id, distribution_method, is_active, is_paused, risk_override_enabled,
                   contract_created_at, beneficiary_name, bank_account_number, bank_name, currency_preference,
                   created_at, updated_at
            FROM plans
            WHERE is_paused = true
            ORDER BY paused_at DESC
            "#,
        )
        .fetch_all(db)
        .await?;

        rows.iter().map(plan_row_to_plan_with_beneficiary).collect()
    }

    /// Get all plans with risk override
    pub async fn get_risk_override_plans(
        db: &PgPool,
    ) -> Result<Vec<PlanWithBeneficiary>, ApiError> {
        let rows = sqlx::query_as::<_, PlanRowFull>(
            r#"
            SELECT id, user_id, title, description, fee, net_amount, status,
                   contract_plan_id, distribution_method, is_active, is_paused, risk_override_enabled,
                   contract_created_at, beneficiary_name, bank_account_number, bank_name, currency_preference,
                   created_at, updated_at
            FROM plans
            WHERE risk_override_enabled = true
            ORDER BY risk_override_at DESC
            "#,
        )
        .fetch_all(db)
        .await?;

        rows.iter().map(plan_row_to_plan_with_beneficiary).collect()
    }
}

// ─── Emergency Session Service (Issue #306) ─────────────────────────────────

pub struct EmergencySessionService;

impl EmergencySessionService {
    /// Start a new session for an active grant.
    pub async fn start_session(
        pool: &PgPool,
        user_id: Uuid,
        req: &StartSessionRequest,
    ) -> Result<EmergencyAccessSession, ApiError> {
        // Verify the grant exists, belongs to user, and is active
        let grant = sqlx::query_as::<_, EmergencyAccessGrant>(
            r#"
            SELECT id, user_id, emergency_contact_id, permissions, expires_at, is_active,
                   revoked_at, created_at, updated_at
            FROM emergency_access_grants
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(req.grant_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("Emergency access grant {} not found", req.grant_id))
        })?;

        if !grant.is_active {
            return Err(ApiError::BadRequest(
                "Cannot start session for an inactive grant".to_string(),
            ));
        }

        if grant.expires_at <= Utc::now() {
            return Err(ApiError::BadRequest(
                "Cannot start session for an expired grant".to_string(),
            ));
        }

        let session = sqlx::query_as::<_, EmergencyAccessSession>(
            r#"
            INSERT INTO emergency_access_sessions (
                grant_id, user_id, emergency_contact_id, ip_address, user_agent
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, grant_id, user_id, emergency_contact_id, started_at,
                      last_active_at, ended_at, ip_address, user_agent, created_at
            "#,
        )
        .bind(req.grant_id)
        .bind(user_id)
        .bind(grant.emergency_contact_id)
        .bind(&req.ip_address)
        .bind(&req.user_agent)
        .fetch_one(pool)
        .await?;

        Ok(session)
    }

    /// Update the heartbeat timestamp for an active session.
    pub async fn heartbeat(
        pool: &PgPool,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<EmergencyAccessSession, ApiError> {
        let session = sqlx::query_as::<_, EmergencyAccessSession>(
            r#"
            UPDATE emergency_access_sessions
            SET last_active_at = NOW()
            WHERE id = $1 AND user_id = $2 AND ended_at IS NULL
            RETURNING id, grant_id, user_id, emergency_contact_id, started_at,
                      last_active_at, ended_at, ip_address, user_agent, created_at
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Active session {} not found", session_id)))?;

        Ok(session)
    }

    /// End an active session.
    pub async fn end_session(
        pool: &PgPool,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<EmergencyAccessSession, ApiError> {
        let session = sqlx::query_as::<_, EmergencyAccessSession>(
            r#"
            UPDATE emergency_access_sessions
            SET ended_at = NOW(), last_active_at = NOW()
            WHERE id = $1 AND user_id = $2 AND ended_at IS NULL
            RETURNING id, grant_id, user_id, emergency_contact_id, started_at,
                      last_active_at, ended_at, ip_address, user_agent, created_at
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Active session {} not found", session_id)))?;

        Ok(session)
    }

    /// List all active sessions for a user.
    pub async fn list_active_sessions(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<EmergencyAccessSession>, ApiError> {
        let sessions = sqlx::query_as::<_, EmergencyAccessSession>(
            r#"
            SELECT id, grant_id, user_id, emergency_contact_id, started_at,
                   last_active_at, ended_at, ip_address, user_agent, created_at
            FROM emergency_access_sessions
            WHERE user_id = $1 AND ended_at IS NULL
            ORDER BY started_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(sessions)
    }

    /// End all active sessions for a specific grant (used during revocation).
    pub async fn end_sessions_for_grant(
        executor: impl sqlx::PgExecutor<'_>,
        grant_id: Uuid,
    ) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            UPDATE emergency_access_sessions
            SET ended_at = NOW(), last_active_at = NOW()
            WHERE grant_id = $1 AND ended_at IS NULL
            "#,
        )
        .bind(grant_id)
        .execute(executor)
        .await?;

        Ok(())
    }
}

// ── Emergency Access Analytics (Issue #306) ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyAccessMetric {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyAccessMetrics {
    pub total_grants: i64,
    pub active_grants: i64,
    pub total_revocations: i64,
    pub total_alerts: i64,
    pub alerts_by_severity: BTreeMap<String, i64>,
    pub grant_trend: Vec<EmergencyAccessMetric>,
}

pub struct EmergencyAccessMetricsService;

impl EmergencyAccessMetricsService {
    pub async fn get_metrics(db: &PgPool, range: &str) -> Result<EmergencyAccessMetrics, ApiError> {
        let total_grants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM emergency_access_grants")
            .fetch_one(db)
            .await?;

        let active_grants: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM emergency_access_grants WHERE is_active = true AND expires_at > NOW()",
        )
        .fetch_one(db)
        .await?;

        let total_revocations: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM emergency_access_grants WHERE revoked_at IS NOT NULL",
        )
        .fetch_one(db)
        .await?;

        let total_alerts: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM emergency_access_risk_alerts")
                .fetch_one(db)
                .await?;

        let alert_severity_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT severity, COUNT(*) FROM emergency_access_risk_alerts GROUP BY severity",
        )
        .fetch_all(db)
        .await?;

        let mut alerts_by_severity = BTreeMap::new();
        for (severity, count) in alert_severity_rows {
            alerts_by_severity.insert(severity.to_lowercase(), count);
        }

        let (interval, trunc) = match range {
            "daily" => ("30 days", "day"),
            "weekly" => ("12 weeks", "week"),
            "monthly" => ("12 months", "month"),
            _ => ("30 days", "day"), // default to daily
        };

        let trend_query = format!(
            r#"
            SELECT 
                DATE_TRUNC('{}', created_at)::DATE::TEXT as date,
                COUNT(*)::BIGINT as count
            FROM emergency_access_grants
            WHERE created_at >= NOW() - INTERVAL '{}'
            GROUP BY 1
            ORDER BY 1
            "#,
            trunc, interval
        );

        let trend_rows: Vec<(String, i64)> = sqlx::query_as(&trend_query).fetch_all(db).await?;

        let grant_trend = trend_rows
            .into_iter()
            .map(|(date, count)| EmergencyAccessMetric { date, count })
            .collect();

        Ok(EmergencyAccessMetrics {
            total_grants,
            active_grants,
            total_revocations,
            total_alerts,
            alerts_by_severity,
            grant_trend,
        })
    }
}

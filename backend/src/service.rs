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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
}

#[derive(Debug, Deserialize)]
pub struct ClaimPlanRequest {
    pub beneficiary_email: String,
    #[allow(dead_code)]
    pub claim_code: Option<u32>,
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
        contract_created_at: row.contract_created_at,
        beneficiary_name: row.beneficiary_name.clone(),
        bank_name: row.bank_name.clone(),
        bank_account_number: row.bank_account_number.clone(),
        currency_preference: row.currency_preference.clone(),
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
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
        // 1. Start Transaction
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
               contract_plan_id, distribution_method, is_active, contract_created_at,
               beneficiary_name, bank_account_number, bank_name, currency_preference,
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
               contract_plan_id, distribution_method, is_active, contract_created_at,
               beneficiary_name, bank_account_number, bank_name, currency_preference,
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
        // 1. Start the transaction
        let mut tx = pool.begin().await?;

        // 2. Use SELECT FOR UPDATE to lock the plan row and prevent concurrent claims
        let row = sqlx::query_as::<_, PlanRowFull>(
            r#"
            SELECT id, user_id, title, description, fee, net_amount, status,
                   contract_plan_id, distribution_method, is_active, contract_created_at,
                   beneficiary_name, bank_account_number, bank_name, currency_preference,
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
                    return Ok(Some(plan));
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
                    due_plans.push(plan);
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
                    due_plans.push(plan);
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

#[cfg(test)]
mod tests {
    use super::{CurrencyPreference, PlanService};
    use crate::api_error::ApiError;
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
}

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
        db: &PgPool,
        user_id: Uuid,
        req: &CreatePlanRequest,
    ) -> Result<PlanWithBeneficiary, ApiError> {
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
        .fetch_one(db)
        .await?;

        let plan = plan_row_to_plan_with_beneficiary(&row)?;

        // Audit: plan created
        AuditLogService::log(
            db,
            Some(user_id),
            audit_action::PLAN_CREATED,
            Some(plan.id),
            Some(entity_type::PLAN),
        )
        .await;

        Ok(plan)
    }

    pub async fn get_plan_by_id(
        db: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<PlanWithBeneficiary>, ApiError> {
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
        .fetch_optional(db)
        .await?;

        match row {
            Some(r) => Ok(Some(plan_row_to_plan_with_beneficiary(&r)?)),
            None => Ok(None),
        }
    }

    pub async fn claim_plan(
        db: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
        req: &ClaimPlanRequest,
    ) -> Result<PlanWithBeneficiary, ApiError> {
        let plan = Self::get_plan_by_id(db, plan_id, user_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Plan {} not found", plan_id)))?;

        let contract_plan_id = plan.contract_plan_id.unwrap_or(0_i64);

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

        if !Self::is_due_for_claim(
            plan.distribution_method.as_deref(),
            plan.contract_created_at,
        ) {
            return Err(ApiError::Forbidden(
                "Plan is not yet due for claim".to_string(),
            ));
        }

        sqlx::query(
            r#"
            INSERT INTO claims (plan_id, contract_plan_id, beneficiary_email)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(plan_id)
        .bind(contract_plan_id)
        .bind(req.beneficiary_email.trim())
        .execute(db)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.is_unique_violation() {
                    return ApiError::BadRequest(
                        "This plan has already been claimed by this beneficiary".to_string(),
                    );
                }
            }
            ApiError::from(e)
        })?;

        // Audit: plan claimed
        AuditLogService::log(
            db,
            Some(user_id),
            audit_action::PLAN_CLAIMED,
            Some(plan_id),
            Some(entity_type::PLAN),
        )
        .await;

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
        db: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
    ) -> Result<PlanWithBeneficiary, ApiError> {
        // First check if the plan exists and belongs to the user
        let plan = Self::get_plan_by_id(db, plan_id, user_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Plan {} not found", plan_id)))?;

        // Check if plan is already deactivated
        if plan.status == "deactivated" {
            return Err(ApiError::BadRequest(
                "Plan is already deactivated".to_string(),
            ));
        }

        // Check if plan has been claimed
        if plan.status == "claimed" {
            return Err(ApiError::BadRequest(
                "Cannot cancel a plan that has been claimed".to_string(),
            ));
        }

        // Update the plan to deactivated status
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
        .fetch_one(db)
        .await?;

        let updated_plan = plan_row_to_plan_with_beneficiary(&row)?;

        // Audit: plan deactivated
        AuditLogService::log(
            db,
            Some(user_id),
            audit_action::PLAN_DEACTIVATED,
            Some(plan_id),
            Some(entity_type::PLAN),
        )
        .await;

        // Notification
        NotificationService::create_silent(
            db,
            user_id,
            notif_type::PLAN_DEACTIVATED,
            format!("Plan '{}' has been deactivated", updated_plan.title),
        )
        .await;

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
    pub async fn submit_kyc(db: &PgPool, user_id: Uuid) -> Result<KycRecord, ApiError> {
        let now = Utc::now();
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
        .fetch_one(db)
        .await?;

        // Audit log
        AuditLogService::log(
            db,
            Some(user_id),
            audit_action::KYC_APPROVED, // Maybe add a KYC_SUBMITTED action
            Some(user_id),
            Some(entity_type::USER),
        )
        .await;

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
        db: &PgPool,
        admin_id: Uuid,
        user_id: Uuid,
        status: KycStatus,
    ) -> Result<KycRecord, ApiError> {
        let status_str = status.to_string();
        let now = Utc::now();

        let record = sqlx::query_as::<_, KycRecord>(
            r#"
            INSERT INTO kyc_status (user_id, status, reviewed_by, reviewed_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id) DO UPDATE 
            SET status = EXCLUDED.status, 
                reviewed_by = EXCLUDED.reviewed_by, 
                reviewed_at = EXCLUDED.reviewed_at
            RETURNING user_id, status, reviewed_by, reviewed_at, created_at
            "#,
        )
        .bind(user_id)
        .bind(status_str)
        .bind(admin_id)
        .bind(now)
        .bind(now)
        .fetch_one(db)
        .await?;

        // Fire notification to the affected user
        let (ntype, msg) = match status {
            KycStatus::Approved => (
                notif_type::KYC_APPROVED,
                "Your KYC verification has been approved.".to_string(),
            ),
            KycStatus::Rejected => (
                notif_type::KYC_REJECTED,
                "Your KYC verification has been rejected. Please contact support.".to_string(),
            ),
            KycStatus::Pending => (
                notif_type::KYC_APPROVED, // won't be hit in normal flow
                "KYC status updated.".to_string(),
            ),
        };
        NotificationService::create_silent(db, user_id, ntype, msg).await;

        // Audit log
        AuditLogService::log(
            db,
            Some(admin_id),
            match &record.status.as_str() {
                &"approved" => audit_action::KYC_APPROVED,
                _ => audit_action::KYC_REJECTED,
            },
            Some(user_id),
            Some(entity_type::USER),
        )
        .await;

        Ok(record)
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

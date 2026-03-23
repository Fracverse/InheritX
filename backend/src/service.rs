use crate::api_error::ApiError;
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

#[derive(Debug, Deserialize)]
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

        plan_row_to_plan_with_beneficiary(&row)
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

        Ok(record)
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

    // ========================================================================
    // Loan Simulation Tests
    // ========================================================================

    use super::{CollateralType, LoanSimulationRequest, LoanSimulationService};
    use rust_decimal_macros::dec;

    #[test]
    fn collateral_type_parsing_usdc() {
        assert_eq!(CollateralType::from_str("USDC").unwrap(), CollateralType::Usdc);
        assert_eq!(CollateralType::from_str("usdc").unwrap(), CollateralType::Usdc);
    }

    #[test]
    fn collateral_type_parsing_eth() {
        assert_eq!(CollateralType::from_str("ETH").unwrap(), CollateralType::Eth);
        assert_eq!(CollateralType::from_str("eth").unwrap(), CollateralType::Eth);
    }

    #[test]
    fn collateral_type_parsing_btc() {
        assert_eq!(CollateralType::from_str("BTC").unwrap(), CollateralType::Btc);
        assert_eq!(CollateralType::from_str("btc").unwrap(), CollateralType::Btc);
    }

    #[test]
    fn collateral_type_parsing_xlm() {
        assert_eq!(
            CollateralType::from_str("STELLAR_XLM").unwrap(),
            CollateralType::StellarXlm
        );
        assert_eq!(CollateralType::from_str("XLM").unwrap(), CollateralType::StellarXlm);
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
        assert_eq!(CollateralType::StellarXlm.get_annual_interest_rate(), dec!(0.12));
    }

    #[test]
    fn collateral_type_liquidation_thresholds() {
        // USDC should have highest threshold (0.95)
        assert_eq!(CollateralType::Usdc.get_liquidation_threshold(), dec!(0.95));
        // ETH and BTC should have same threshold (0.85)
        assert_eq!(CollateralType::Eth.get_liquidation_threshold(), dec!(0.85));
        assert_eq!(CollateralType::Btc.get_liquidation_threshold(), dec!(0.85));
        // XLM should have lowest threshold (0.80)
        assert_eq!(CollateralType::StellarXlm.get_liquidation_threshold(), dec!(0.80));
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

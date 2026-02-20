use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct PlanService;

impl PlanService {
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
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let plan_row = sqlx::query_as::<_, PlanRow>(
            r#"
            SELECT p.*
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
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let plan_rows = sqlx::query_as::<_, PlanRow>(
            r#"
            SELECT p.*
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
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let plan_rows = sqlx::query_as::<_, PlanRow>(
            r#"
            SELECT p.*
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

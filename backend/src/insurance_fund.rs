use crate::api_error::ApiError;
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

/// Insurance fund status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FundStatus {
    Healthy,
    Warning,
    Critical,
    Insolvent,
}

impl FundStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FundStatus::Healthy => "healthy",
            FundStatus::Warning => "warning",
            FundStatus::Critical => "critical",
            FundStatus::Insolvent => "insolvent",
        }
    }
}

impl FromStr for FundStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "healthy" => Ok(FundStatus::Healthy),
            "warning" => Ok(FundStatus::Warning),
            "critical" => Ok(FundStatus::Critical),
            "insolvent" => Ok(FundStatus::Insolvent),
            _ => Ok(FundStatus::Healthy),
        }
    }
}

/// Insurance fund details
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct InsuranceFund {
    pub id: Uuid,
    pub fund_name: String,
    pub description: Option<String>,
    pub asset_code: String,
    pub total_reserves: Decimal,
    pub available_reserves: Decimal,
    pub locked_reserves: Decimal,
    pub total_covered_liabilities: Decimal,
    pub coverage_ratio: Decimal,
    pub reserve_health_score: Decimal,
    pub min_coverage_ratio: Decimal,
    pub target_coverage_ratio: Decimal,
    pub critical_coverage_ratio: Decimal,
    pub status: String,
    pub status_changed_at: Option<DateTime<Utc>>,
    pub total_contributions: Decimal,
    pub total_payouts: Decimal,
    pub yield_earned: Decimal,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insurance fund metrics snapshot
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct InsuranceFundMetrics {
    pub fund_id: Uuid,
    pub fund_name: String,
    pub total_reserves: Decimal,
    pub available_reserves: Decimal,
    pub locked_reserves: Decimal,
    pub total_covered_liabilities: Decimal,
    pub coverage_ratio: Decimal,
    pub reserve_health_score: Decimal,
    pub status: String,
    pub coverage_ratio_percentage: Decimal,
    pub health_status_description: String,
    pub recorded_at: DateTime<Utc>,
}

/// Insurance fund transaction
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct InsuranceFundTransaction {
    pub id: Uuid,
    pub fund_id: Uuid,
    pub transaction_type: String,
    pub user_id: Option<Uuid>,
    pub plan_id: Option<Uuid>,
    pub loan_id: Option<Uuid>,
    pub asset_code: String,
    pub amount: Decimal,
    pub balance_after: Decimal,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Insurance claim
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct InsuranceClaim {
    pub id: Uuid,
    pub fund_id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub loan_id: Option<Uuid>,
    pub claim_type: String,
    pub claimed_amount: Decimal,
    pub approved_amount: Option<Decimal>,
    pub payout_amount: Option<Decimal>,
    pub status: String,
    pub rejection_reason: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create insurance claim
#[derive(Debug, Deserialize)]
pub struct CreateInsuranceClaimRequest {
    pub claim_type: String,
    pub claimed_amount: Decimal,
    pub plan_id: Option<Uuid>,
    pub loan_id: Option<Uuid>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Request to process insurance claim
#[derive(Debug, Deserialize)]
pub struct ProcessInsuranceClaimRequest {
    pub approved: bool,
    pub approved_amount: Option<Decimal>,
    pub rejection_reason: Option<String>,
}

/// Response for insurance fund dashboard
#[derive(Debug, Serialize, Deserialize)]
pub struct InsuranceFundDashboard {
    pub fund: InsuranceFundMetrics,
    pub recent_transactions: Vec<InsuranceFundTransaction>,
    pub pending_claims: Vec<InsuranceClaim>,
    pub total_claims_count: usize,
    pub total_claims_amount: Decimal,
    pub trends: FundTrends,
}

/// Fund trends data
#[derive(Debug, Serialize, Deserialize)]
pub struct FundTrends {
    pub coverage_ratio_change_24h: Option<Decimal>,
    pub reserves_change_24h: Option<Decimal>,
    pub claims_last_7_days: usize,
    pub payouts_last_7_days: Decimal,
}

/// Insurance Fund Service
pub struct InsuranceFundService {
    db: PgPool,
}

impl InsuranceFundService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Start background monitoring job
    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                if let Err(e) = self.update_fund_metrics().await {
                    error!("Insurance Fund Service error updating metrics: {}", e);
                }
            }
        });
    }

    /// Get the primary insurance fund
    pub async fn get_primary_fund(&self) -> Result<InsuranceFund, ApiError> {
        let fund = sqlx::query_as::<_, InsuranceFund>(
            "SELECT * FROM insurance_fund ORDER BY created_at LIMIT 1",
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("DB error fetching insurance fund: {}", e))
        })?
        .ok_or_else(|| ApiError::NotFound("Insurance fund not found".to_string()))?;

        Ok(fund)
    }

    /// Get all insurance funds
    pub async fn get_all_funds(&self) -> Result<Vec<InsuranceFund>, ApiError> {
        let funds =
            sqlx::query_as::<_, InsuranceFund>("SELECT * FROM insurance_fund ORDER BY created_at")
                .fetch_all(&self.db)
                .await
                .map_err(|e| {
                    ApiError::Internal(anyhow::anyhow!("DB error fetching insurance funds: {}", e))
                })?;

        Ok(funds)
    }

    /// Get fund by ID
    pub async fn get_fund_by_id(&self, fund_id: Uuid) -> Result<InsuranceFund, ApiError> {
        let fund = sqlx::query_as::<_, InsuranceFund>("SELECT * FROM insurance_fund WHERE id = $1")
            .bind(fund_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("DB error fetching insurance fund: {}", e))
            })?
            .ok_or_else(|| ApiError::NotFound(format!("Insurance fund {} not found", fund_id)))?;

        Ok(fund)
    }

    /// Calculate total covered liabilities from active loans and plans
    pub async fn calculate_covered_liabilities(&self) -> Result<Decimal, ApiError> {
        // Sum of all active loan principals + plan liabilities
        let result = sqlx::query_scalar::<_, Decimal>(
            r#"
            SELECT COALESCE(SUM(
                CASE 
                    WHEN ll.status = 'active' THEN ll.principal 
                    ELSE 0 
                END
            ), 0) as total_liabilities
            FROM loan_lifecycle ll
            WHERE ll.status IN ('active', 'overdue')
            "#,
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("DB error calculating liabilities: {}", e))
        })?;

        Ok(result)
    }

    /// Calculate coverage ratio
    pub fn calculate_coverage_ratio(reserves: Decimal, liabilities: Decimal) -> Decimal {
        if liabilities == Decimal::ZERO {
            return Decimal::new(9999, 0); // Infinite coverage when no liabilities
        }
        reserves / liabilities
    }

    /// Calculate reserve health score (0-100)
    pub fn calculate_health_score(
        coverage_ratio: Decimal,
        min_ratio: Decimal,
        target_ratio: Decimal,
    ) -> Decimal {
        if coverage_ratio >= target_ratio {
            return Decimal::new(100, 0);
        }

        if coverage_ratio <= min_ratio {
            return Decimal::ZERO;
        }

        // Linear interpolation between min and target
        let range = target_ratio - min_ratio;
        if range == Decimal::ZERO {
            return Decimal::new(50, 0);
        }

        let score = (coverage_ratio - min_ratio) / range * Decimal::new(100, 0);
        score.min(Decimal::new(100, 0)).max(Decimal::ZERO)
    }

    /// Determine fund status based on coverage ratio
    pub fn determine_status(
        coverage_ratio: Decimal,
        thresholds: (Decimal, Decimal, Decimal),
    ) -> FundStatus {
        let (critical, min, target) = thresholds;

        if coverage_ratio >= target {
            FundStatus::Healthy
        } else if coverage_ratio >= min {
            FundStatus::Warning
        } else if coverage_ratio >= critical {
            FundStatus::Critical
        } else {
            FundStatus::Insolvent
        }
    }

    /// Update fund metrics based on current state
    pub async fn update_fund_metrics(&self) -> Result<(), ApiError> {
        let funds = self.get_all_funds().await?;

        for fund in funds {
            let total_liabilities = self.calculate_covered_liabilities().await?;
            let coverage_ratio =
                Self::calculate_coverage_ratio(fund.total_reserves, total_liabilities);
            let health_score = Self::calculate_health_score(
                coverage_ratio,
                fund.min_coverage_ratio,
                fund.target_coverage_ratio,
            );

            let old_status = fund.status.parse().unwrap_or(FundStatus::Healthy);
            let new_status = Self::determine_status(
                coverage_ratio,
                (
                    fund.critical_coverage_ratio,
                    fund.min_coverage_ratio,
                    fund.target_coverage_ratio,
                ),
            );

            let status_changed = old_status != new_status;

            // Update fund
            let mut tx = self
                .db
                .begin()
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

            sqlx::query(
                r#"
                UPDATE insurance_fund
                SET total_covered_liabilities = $1,
                    coverage_ratio = $2,
                    reserve_health_score = $3,
                    status = $4,
                    status_changed_at = CASE WHEN $5 THEN CURRENT_TIMESTAMP ELSE status_changed_at END,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = $6
                "#,
            )
            .bind(total_liabilities)
            .bind(coverage_ratio)
            .bind(health_score)
            .bind(new_status.as_str())
            .bind(status_changed)
            .bind(fund.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error updating fund metrics: {}", e)))?;

            // Record metrics history
            sqlx::query(
                r#"
                INSERT INTO insurance_fund_metrics_history (
                    fund_id, total_reserves, available_reserves, locked_reserves,
                    total_covered_liabilities, coverage_ratio, reserve_health_score, status
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(fund.id)
            .bind(fund.total_reserves)
            .bind(fund.available_reserves)
            .bind(fund.locked_reserves)
            .bind(total_liabilities)
            .bind(coverage_ratio)
            .bind(health_score)
            .bind(new_status.as_str())
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("DB error recording metrics history: {}", e))
            })?;

            tx.commit()
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

            // Notify on status change
            if status_changed {
                info!(
                    "Insurance fund {} status changed: {:?} -> {:?}",
                    fund.fund_name, old_status, new_status
                );

                let mut tx =
                    self.db.begin().await.map_err(|e| {
                        ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e))
                    })?;

                // Notify admins
                NotificationService::create(
                    &mut tx,
                    fund.id, // Use fund ID as pseudo user_id for admin notifications
                    notif_type::ADMIN_ALERT,
                    format!(
                        "Insurance Fund '{}' status changed to {}. Coverage ratio: {:.2}, Health Score: {:.0}",
                        fund.fund_name,
                        new_status.as_str(),
                        coverage_ratio,
                        health_score
                    ),
                )
                .await?;

                AuditLogService::log(
                    &mut *tx,
                    None,
                    audit_action::FUND_STATUS_CHANGE,
                    Some(fund.id),
                    Some(entity_type::INSURANCE_FUND),
                )
                .await?;

                tx.commit()
                    .await
                    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;
            }
        }

        Ok(())
    }

    /// Record a transaction in the insurance fund
    #[allow(clippy::too_many_arguments)]
    pub async fn record_transaction(
        &self,
        fund_id: Uuid,
        transaction_type: &str,
        amount: Decimal,
        asset_code: &str,
        user_id: Option<Uuid>,
        plan_id: Option<Uuid>,
        loan_id: Option<Uuid>,
        description: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<InsuranceFundTransaction, ApiError> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

        // Get current balance
        let fund = self.get_fund_by_id(fund_id).await?;
        let new_balance = match transaction_type {
            "contribution" | "yield" => fund.total_reserves + amount,
            "payout" | "fee" | "penalty" => fund.total_reserves - amount,
            _ => fund.total_reserves,
        };

        // Update fund reserves
        let update_query = match transaction_type {
            "contribution" => "UPDATE insurance_fund SET total_reserves = total_reserves + $1, available_reserves = available_reserves + $1, total_contributions = total_contributions + $1 WHERE id = $2",
            "payout" => "UPDATE insurance_fund SET total_reserves = total_reserves - $1, available_reserves = available_reserves - $1, total_payouts = total_payouts + $1 WHERE id = $2",
            "yield" => "UPDATE insurance_fund SET total_reserves = total_reserves + $1, available_reserves = available_reserves + $1, yield_earned = yield_earned + $1 WHERE id = $2",
            "fee" | "penalty" => "UPDATE insurance_fund SET total_reserves = total_reserves - $1, available_reserves = available_reserves - $1 WHERE id = $2",
            _ => return Err(ApiError::BadRequest(format!("Invalid transaction type: {}", transaction_type))),
        };

        sqlx::query(update_query)
            .bind(amount)
            .bind(fund_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("DB error updating fund reserves: {}", e))
            })?;

        // Record transaction
        let transaction = sqlx::query_as::<_, InsuranceFundTransaction>(
            r#"
            INSERT INTO insurance_fund_transactions (
                fund_id, transaction_type, user_id, plan_id, loan_id,
                asset_code, amount, balance_after, description, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(fund_id)
        .bind(transaction_type)
        .bind(user_id)
        .bind(plan_id)
        .bind(loan_id)
        .bind(asset_code)
        .bind(amount)
        .bind(new_balance)
        .bind(description)
        .bind(metadata.unwrap_or_default())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("DB error recording transaction: {}", e))
        })?;

        tx.commit()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

        Ok(transaction)
    }

    /// Create insurance claim
    pub async fn create_claim(
        &self,
        fund_id: Uuid,
        user_id: Uuid,
        req: &CreateInsuranceClaimRequest,
    ) -> Result<InsuranceClaim, ApiError> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

        let claim = sqlx::query_as::<_, InsuranceClaim>(
            r#"
            INSERT INTO insurance_claims (
                fund_id, user_id, plan_id, loan_id, claim_type, claimed_amount, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(fund_id)
        .bind(user_id)
        .bind(req.plan_id)
        .bind(req.loan_id)
        .bind(&req.claim_type)
        .bind(req.claimed_amount)
        .bind(req.metadata.clone().unwrap_or_default())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("DB error creating insurance claim: {}", e))
        })?;

        AuditLogService::log(
            &mut *tx,
            Some(user_id),
            audit_action::INSURANCE_CLAIM_CREATED,
            Some(claim.id),
            Some(entity_type::INSURANCE_CLAIM),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

        Ok(claim)
    }

    /// Process insurance claim (approve/reject)
    pub async fn process_claim(
        &self,
        claim_id: Uuid,
        admin_id: Uuid,
        req: &ProcessInsuranceClaimRequest,
    ) -> Result<InsuranceClaim, ApiError> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

        let mut claim = sqlx::query_as::<_, InsuranceClaim>(
            "SELECT * FROM insurance_claims WHERE id = $1 FOR UPDATE",
        )
        .bind(claim_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching claim: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Insurance claim {} not found", claim_id)))?;

        if claim.status != "pending" {
            return Err(ApiError::BadRequest(
                "Claim has already been processed".to_string(),
            ));
        }

        let (new_status, payout_amount) = if req.approved {
            let amount = req.approved_amount.unwrap_or(claim.claimed_amount);
            ("approved".to_string(), Some(amount))
        } else {
            ("rejected".to_string(), None)
        };

        claim = sqlx::query_as::<_, InsuranceClaim>(
            r#"
            UPDATE insurance_claims
            SET status = $1,
                approved_amount = $2,
                payout_amount = $3,
                reviewed_by = $4,
                reviewed_at = CURRENT_TIMESTAMP,
                rejection_reason = $5,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $6
            RETURNING *
            "#,
        )
        .bind(&new_status)
        .bind(payout_amount)
        .bind(admin_id)
        .bind(&req.rejection_reason)
        .bind(claim_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error updating claim: {}", e)))?;

        AuditLogService::log(
            &mut *tx,
            Some(admin_id),
            audit_action::INSURANCE_CLAIM_PROCESSED,
            Some(claim.id),
            Some(entity_type::INSURANCE_CLAIM),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

        Ok(claim)
    }

    /// Pay out approved claim
    pub async fn payout_claim(&self, claim_id: Uuid) -> Result<(), ApiError> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

        let claim = sqlx::query_as::<_, InsuranceClaim>(
            "SELECT * FROM insurance_claims WHERE id = $1 FOR UPDATE",
        )
        .bind(claim_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching claim: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Insurance claim {} not found", claim_id)))?;

        if claim.status != "approved" {
            return Err(ApiError::BadRequest(
                "Claim is not approved for payout".to_string(),
            ));
        }

        if claim.payout_amount.is_none() {
            return Err(ApiError::BadRequest(
                "Claim has no payout amount set".to_string(),
            ));
        }

        let payout_amount = claim.payout_amount.unwrap();

        // Record payout transaction
        self.record_transaction(
            claim.fund_id,
            "payout",
            payout_amount,
            "USDC",
            Some(claim.user_id),
            claim.plan_id,
            claim.loan_id,
            Some(format!("Insurance claim payout for claim {}", claim_id)),
            Some(serde_json::json!({"claim_id": claim_id.to_string()})),
        )
        .await?;

        // Update claim status
        sqlx::query(
            "UPDATE insurance_claims SET status = 'paid', paid_at = CURRENT_TIMESTAMP WHERE id = $1",
        )
        .bind(claim_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error updating claim status: {}", e)))?;

        AuditLogService::log(
            &mut *tx,
            None,
            audit_action::INSURANCE_CLAIM_PAID,
            Some(claim.id),
            Some(entity_type::INSURANCE_CLAIM),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

        Ok(())
    }

    /// Get fund dashboard data
    pub async fn get_dashboard(&self, fund_id: Uuid) -> Result<InsuranceFundDashboard, ApiError> {
        let fund = self.get_fund_by_id(fund_id).await?;

        // Get recent transactions
        let recent_transactions = sqlx::query_as::<_, InsuranceFundTransaction>(
            "SELECT * FROM insurance_fund_transactions WHERE fund_id = $1 ORDER BY created_at DESC LIMIT 10",
        )
        .bind(fund_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching transactions: {}", e)))?;

        // Get pending claims
        let pending_claims = sqlx::query_as::<_, InsuranceClaim>(
            "SELECT * FROM insurance_claims WHERE fund_id = $1 AND status = 'pending' ORDER BY created_at DESC LIMIT 10",
        )
        .bind(fund_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching claims: {}", e)))?;

        // Get total claims stats
        let claims_stats: (i64, Decimal) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count, COALESCE(SUM(claimed_amount), 0) as total_amount
            FROM insurance_claims
            WHERE fund_id = $1
            "#,
        )
        .bind(fund_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("DB error fetching claims stats: {}", e))
        })?;

        let total_claims_count = claims_stats.0;
        let total_claims_amount = claims_stats.1;

        // Get trends
        let trends = self.get_fund_trends(fund_id).await?;

        let coverage_ratio_percentage = if fund.total_covered_liabilities == Decimal::ZERO {
            Decimal::new(9999, 0)
        } else {
            fund.coverage_ratio
        };

        let health_status_description = match fund.status.parse().unwrap_or(FundStatus::Healthy) {
            FundStatus::Healthy => "Fund is healthy with adequate coverage".to_string(),
            FundStatus::Warning => "Fund coverage is below target - monitor closely".to_string(),
            FundStatus::Critical => {
                "Fund coverage is critically low - immediate action required".to_string()
            }
            FundStatus::Insolvent => "Fund is insolvent - urgent intervention needed".to_string(),
        };

        Ok(InsuranceFundDashboard {
            fund: InsuranceFundMetrics {
                fund_id: fund.id,
                fund_name: fund.fund_name,
                total_reserves: fund.total_reserves,
                available_reserves: fund.available_reserves,
                locked_reserves: fund.locked_reserves,
                total_covered_liabilities: fund.total_covered_liabilities,
                coverage_ratio: fund.coverage_ratio,
                reserve_health_score: fund.reserve_health_score,
                status: fund.status,
                coverage_ratio_percentage,
                health_status_description,
                recorded_at: Utc::now(),
            },
            recent_transactions,
            pending_claims,
            total_claims_count: total_claims_count as usize,
            total_claims_amount,
            trends,
        })
    }

    /// Get fund trends
    async fn get_fund_trends(&self, fund_id: Uuid) -> Result<FundTrends, ApiError> {
        // Get metrics from 24h ago
        let old_metrics: Option<(Decimal, Decimal)> = sqlx::query_as(
            r#"
            SELECT coverage_ratio, total_reserves
            FROM insurance_fund_metrics_history
            WHERE fund_id = $1 AND recorded_at <= NOW() - INTERVAL '24 hours'
            ORDER BY recorded_at DESC
            LIMIT 1
            "#,
        )
        .bind(fund_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching old metrics: {}", e)))?;

        let current_fund = self.get_fund_by_id(fund_id).await?;

        let coverage_ratio_change_24h = if let Some(old) = old_metrics {
            Some(current_fund.coverage_ratio - old.0)
        } else {
            None
        };

        let reserves_change_24h = if let Some(old) = old_metrics {
            Some(current_fund.total_reserves - old.1)
        } else {
            None
        };

        // Get claims from last 7 days
        let claims_7d: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM insurance_claims WHERE fund_id = $1 AND created_at >= NOW() - INTERVAL '7 days'",
        )
        .bind(fund_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or(0);

        // Get payouts from last 7 days
        let payouts_7d: Decimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM insurance_fund_transactions WHERE fund_id = $1 AND transaction_type = 'payout' AND created_at >= NOW() - INTERVAL '7 days'",
        )
        .bind(fund_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or(Decimal::ZERO);

        Ok(FundTrends {
            coverage_ratio_change_24h,
            reserves_change_24h,
            claims_last_7_days: claims_7d as usize,
            payouts_last_7_days: payouts_7d,
        })
    }

    /// Get metrics history for a fund
    pub async fn get_metrics_history(
        &self,
        fund_id: Uuid,
        days: i64,
    ) -> Result<Vec<InsuranceFundMetrics>, ApiError> {
        let fund = self.get_fund_by_id(fund_id).await?;

        let history = sqlx::query_as::<_, InsuranceFundMetrics>(
            r#"
            SELECT 
                fund_id,
                $2 as fund_name,
                total_reserves,
                available_reserves,
                locked_reserves,
                total_covered_liabilities,
                coverage_ratio,
                reserve_health_score,
                status,
                coverage_ratio as coverage_ratio_percentage,
                'Historical data' as health_status_description,
                recorded_at
            FROM insurance_fund_metrics_history
            WHERE fund_id = $1 AND recorded_at >= NOW() - INTERVAL '1 day' * $3
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(fund_id)
        .bind(&fund.fund_name)
        .bind(days)
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("DB error fetching metrics history: {}", e))
        })?;

        Ok(history)
    }
}

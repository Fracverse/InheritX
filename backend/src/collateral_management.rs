//! # Collateral Management
//!
//! Provides advanced collateral management functions for active loans:
//! - Add collateral to existing loans
//! - Remove excess collateral (if health factor allows)
//! - Swap collateral types
//! - Calculate collateral values and requirements
//! - Determine safe withdrawal amounts

use crate::api_error::ApiError;
use crate::events::{EventService, DepositMetadata};
use crate::loan_lifecycle::{LoanLifecycleRecord, LoanLifecycleService};
use crate::notifications::{audit_action, entity_type, AuditLogService};
use crate::price_feed::PriceFeedService;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Request / Response Types
// ─────────────────────────────────────────────────────────────────────────────

/// Request to add collateral to an existing loan
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCollateralRequest {
    pub loan_id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub transaction_hash: Option<String>,
}

/// Request to remove collateral from an existing loan
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveCollateralRequest {
    pub loan_id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub transaction_hash: Option<String>,
}

/// Request to swap collateral type
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapCollateralRequest {
    pub loan_id: Uuid,
    pub user_id: Uuid,
    pub new_collateral_asset: String,
    pub new_collateral_amount: Decimal,
    pub transaction_hash: Option<String>,
}

/// Collateral information response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollateralInfo {
    pub loan_id: Uuid,
    pub collateral_asset: String,
    pub collateral_amount: Decimal,
    pub collateral_value_usd: Decimal,
    pub current_price: Decimal,
}

/// Collateral requirements response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollateralRequirements {
    pub loan_id: Uuid,
    pub borrow_asset: String,
    pub principal: Decimal,
    pub collateral_asset: String,
    pub required_collateral_amount: Decimal,
    pub current_collateral_amount: Decimal,
    pub collateral_surplus: Decimal,
    pub health_factor: Decimal,
    pub min_health_factor: Decimal,
}

/// Safe withdrawal calculation response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeWithdrawalInfo {
    pub loan_id: Uuid,
    pub current_collateral_amount: Decimal,
    pub max_withdrawable_amount: Decimal,
    pub health_factor_after_withdrawal: Decimal,
    pub min_health_factor: Decimal,
}

// ─────────────────────────────────────────────────────────────────────────────
// Service
// ─────────────────────────────────────────────────────────────────────────────

pub struct CollateralManagementService;

impl CollateralManagementService {
    // ── Add Collateral ────────────────────────────────────────────────────────

    /// Add additional collateral to an existing active loan.
    ///
    /// # Validation
    /// - Loan must exist and belong to the user
    /// - Loan must be in active status
    /// - Amount must be positive
    /// - Plan must not be paused
    ///
    /// # Effects
    /// - Increases collateral_amount in loan_lifecycle table
    /// - Emits a deposit event
    /// - Logs audit trail
    /// - Recalculates health factor
    pub async fn add_collateral(
        pool: &PgPool,
        req: &AddCollateralRequest,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        if req.amount <= Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "collateral amount must be greater than zero".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // Fetch the loan with row lock
        let loan = sqlx::query_as::<_, crate::loan_lifecycle::LoanLifecycleRow>(
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
        .bind(req.loan_id)
        .bind(req.user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest(
                "Loan not found or its associated plan is paused by an administrator".to_string(),
            )
        })?;

        let loan_record: LoanLifecycleRecord = loan.into();

        // Verify loan is active
        if loan_record.status != "active" {
            return Err(ApiError::BadRequest(format!(
                "cannot add collateral to a loan that is {}",
                loan_record.status
            )));
        }

        // Update collateral amount
        let updated = sqlx::query_as::<_, crate::loan_lifecycle::LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET collateral_amount = collateral_amount + $1
            WHERE id = $2
            RETURNING id, user_id, plan_id, borrow_asset, collateral_asset,
                      principal, interest_rate_bps, collateral_amount, amount_repaid,
                      status, due_date, transaction_hash,
                      created_at, updated_at, repaid_at, liquidated_at
            "#,
        )
        .bind(req.amount)
        .bind(req.loan_id)
        .fetch_one(&mut *tx)
        .await?;

        let updated_record: LoanLifecycleRecord = updated.into();

        // Emit deposit event
        EventService::emit_deposit(
            &mut tx,
            req.user_id,
            loan_record.plan_id,
            &loan_record.collateral_asset,
            req.amount,
            DepositMetadata {
                collateral_ratio: None,
                total_deposited: updated_record.collateral_amount,
            },
            req.transaction_hash.clone(),
            None,
        )
        .await?;

        // Log audit trail
        AuditLogService::log(
            &mut *tx,
            Some(req.user_id),
            None,
            audit_action::COLLATERAL_ADDED,
            Some(req.loan_id),
            Some(entity_type::LOAN),
            None,
            None,
            None,
        )
        .await?;

        tx.commit().await?;
        Ok(updated_record)
    }

    // ── Remove Collateral ─────────────────────────────────────────────────────

    /// Remove collateral from an existing active loan.
    ///
    /// # Validation
    /// - Loan must exist and belong to the user
    /// - Loan must be in active status
    /// - Amount must be positive and not exceed current collateral
    /// - Health factor must remain >= 150% (1.5) after removal
    /// - Plan must not be paused
    ///
    /// # Effects
    /// - Decreases collateral_amount in loan_lifecycle table
    /// - Emits a deposit event (negative amount)
    /// - Logs audit trail
    /// - Recalculates health factor
    pub async fn remove_collateral(
        pool: &PgPool,
        price_feed: Arc<dyn PriceFeedService>,
        req: &RemoveCollateralRequest,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        if req.amount <= Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "collateral amount must be greater than zero".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // Fetch the loan with row lock
        let loan = sqlx::query_as::<_, crate::loan_lifecycle::LoanLifecycleRow>(
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
        .bind(req.loan_id)
        .bind(req.user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest(
                "Loan not found or its associated plan is paused by an administrator".to_string(),
            )
        })?;

        let loan_record: LoanLifecycleRecord = loan.into();

        // Verify loan is active
        if loan_record.status != "active" {
            return Err(ApiError::BadRequest(format!(
                "cannot remove collateral from a loan that is {}",
                loan_record.status
            )));
        }

        // Verify amount doesn't exceed current collateral
        if req.amount > loan_record.collateral_amount {
            return Err(ApiError::BadRequest(format!(
                "cannot remove {} of collateral; only {} available",
                req.amount, loan_record.collateral_amount
            )));
        }

        // Calculate health factor after removal
        let collateral_after = loan_record.collateral_amount - req.amount;
        let health_factor_after = Self::calculate_health_factor(
            &price_feed,
            &loan_record.borrow_asset,
            &loan_record.collateral_asset,
            loan_record.principal - loan_record.amount_repaid,
            collateral_after,
        )
        .await?;

        // Verify health factor remains >= 150% (1.5)
        let min_health_factor = Decimal::from_str_exact("1.5")
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse min health factor: {e}")))?;

        if health_factor_after < min_health_factor {
            return Err(ApiError::BadRequest(format!(
                "cannot remove collateral; health factor would drop to {:.2}%, minimum is {:.2}%",
                health_factor_after * Decimal::from(100),
                min_health_factor * Decimal::from(100)
            )));
        }

        // Update collateral amount
        let updated = sqlx::query_as::<_, crate::loan_lifecycle::LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET collateral_amount = collateral_amount - $1
            WHERE id = $2
            RETURNING id, user_id, plan_id, borrow_asset, collateral_asset,
                      principal, interest_rate_bps, collateral_amount, amount_repaid,
                      status, due_date, transaction_hash,
                      created_at, updated_at, repaid_at, liquidated_at
            "#,
        )
        .bind(req.amount)
        .bind(req.loan_id)
        .fetch_one(&mut *tx)
        .await?;

        let updated_record: LoanLifecycleRecord = updated.into();

        // Emit deposit event with negative amount to represent withdrawal
        EventService::emit_deposit(
            &mut tx,
            req.user_id,
            loan_record.plan_id,
            &loan_record.collateral_asset,
            -req.amount,
            DepositMetadata {
                collateral_ratio: None,
                total_deposited: updated_record.collateral_amount,
            },
            req.transaction_hash.clone(),
            None,
        )
        .await?;

        // Log audit trail
        AuditLogService::log(
            &mut *tx,
            Some(req.user_id),
            None,
            audit_action::COLLATERAL_REMOVED,
            Some(req.loan_id),
            Some(entity_type::LOAN),
            None,
            None,
            None,
        )
        .await?;

        tx.commit().await?;
        Ok(updated_record)
    }

    // ── Swap Collateral ───────────────────────────────────────────────────────

    /// Swap collateral type for an existing active loan.
    ///
    /// # Validation
    /// - Loan must exist and belong to the user
    /// - Loan must be in active status
    /// - New collateral asset must be whitelisted
    /// - New collateral amount must maintain health factor >= 150%
    /// - Plan must not be paused
    ///
    /// # Effects
    /// - Updates collateral_asset and collateral_amount
    /// - Emits a deposit event for old collateral (negative)
    /// - Emits a deposit event for new collateral (positive)
    /// - Logs audit trail
    pub async fn swap_collateral(
        pool: &PgPool,
        price_feed: Arc<dyn PriceFeedService>,
        req: &SwapCollateralRequest,
    ) -> Result<LoanLifecycleRecord, ApiError> {
        if req.new_collateral_amount <= Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "new collateral amount must be greater than zero".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // Fetch the loan with row lock
        let loan = sqlx::query_as::<_, crate::loan_lifecycle::LoanLifecycleRow>(
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
        .bind(req.loan_id)
        .bind(req.user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest(
                "Loan not found or its associated plan is paused by an administrator".to_string(),
            )
        })?;

        let loan_record: LoanLifecycleRecord = loan.into();

        // Verify loan is active
        if loan_record.status != "active" {
            return Err(ApiError::BadRequest(format!(
                "cannot swap collateral for a loan that is {}",
                loan_record.status
            )));
        }

        // Verify new collateral asset is whitelisted (check if price exists)
        price_feed
            .get_price(&req.new_collateral_asset)
            .await
            .map_err(|_| {
                ApiError::BadRequest(format!(
                    "collateral asset {} is not supported",
                    req.new_collateral_asset
                ))
            })?;

        // Calculate health factor with new collateral
        let health_factor_after = Self::calculate_health_factor(
            &price_feed,
            &loan_record.borrow_asset,
            &req.new_collateral_asset,
            loan_record.principal - loan_record.amount_repaid,
            req.new_collateral_amount,
        )
        .await?;

        // Verify health factor remains >= 150% (1.5)
        let min_health_factor = Decimal::from_str_exact("1.5")
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse min health factor: {e}")))?;

        if health_factor_after < min_health_factor {
            return Err(ApiError::BadRequest(format!(
                "new collateral configuration would result in health factor of {:.2}%, minimum is {:.2}%",
                health_factor_after * Decimal::from(100),
                min_health_factor * Decimal::from(100)
            )));
        }

        // Update collateral asset and amount
        let updated = sqlx::query_as::<_, crate::loan_lifecycle::LoanLifecycleRow>(
            r#"
            UPDATE loan_lifecycle
            SET collateral_asset = $1, collateral_amount = $2
            WHERE id = $3
            RETURNING id, user_id, plan_id, borrow_asset, collateral_asset,
                      principal, interest_rate_bps, collateral_amount, amount_repaid,
                      status, due_date, transaction_hash,
                      created_at, updated_at, repaid_at, liquidated_at
            "#,
        )
        .bind(&req.new_collateral_asset)
        .bind(req.new_collateral_amount)
        .bind(req.loan_id)
        .fetch_one(&mut *tx)
        .await?;

        let updated_record: LoanLifecycleRecord = updated.into();

        // Emit events for old and new collateral
        EventService::emit_deposit(
            &mut tx,
            req.user_id,
            loan_record.plan_id,
            &loan_record.collateral_asset,
            -loan_record.collateral_amount,
            DepositMetadata {
                collateral_ratio: None,
                total_deposited: Decimal::ZERO,
            },
            req.transaction_hash.clone(),
            None,
        )
        .await?;

        EventService::emit_deposit(
            &mut tx,
            req.user_id,
            loan_record.plan_id,
            &req.new_collateral_asset,
            req.new_collateral_amount,
            DepositMetadata {
                collateral_ratio: None,
                total_deposited: req.new_collateral_amount,
            },
            req.transaction_hash.clone(),
            None,
        )
        .await?;

        // Log audit trail
        AuditLogService::log(
            &mut *tx,
            Some(req.user_id),
            None,
            audit_action::COLLATERAL_SWAPPED,
            Some(req.loan_id),
            Some(entity_type::LOAN),
            None,
            None,
            None,
        )
        .await?;

        tx.commit().await?;
        Ok(updated_record)
    }

    // ── Query Functions ───────────────────────────────────────────────────────

    /// Get current collateral value in USD for a loan
    pub async fn get_collateral_value(
        pool: &PgPool,
        price_feed: Arc<dyn PriceFeedService>,
        loan_id: Uuid,
    ) -> Result<CollateralInfo, ApiError> {
        let loan = LoanLifecycleService::get_loan(pool, loan_id).await?;

        let price = price_feed.get_price(&loan.collateral_asset).await?;
        let collateral_value_usd = loan.collateral_amount * price.price;

        Ok(CollateralInfo {
            loan_id,
            collateral_asset: loan.collateral_asset,
            collateral_amount: loan.collateral_amount,
            collateral_value_usd,
            current_price: price.price,
        })
    }

    /// Get maximum withdrawable collateral amount while maintaining health factor >= 150%
    pub async fn get_max_withdrawable_collateral(
        pool: &PgPool,
        price_feed: Arc<dyn PriceFeedService>,
        loan_id: Uuid,
    ) -> Result<SafeWithdrawalInfo, ApiError> {
        let loan = LoanLifecycleService::get_loan(pool, loan_id).await?;

        let borrow_price = price_feed.get_price(&loan.borrow_asset).await?;
        let collateral_price = price_feed.get_price(&loan.collateral_asset).await?;

        let remaining_debt = loan.principal - loan.amount_repaid;
        let debt_value_usd = remaining_debt * borrow_price.price;

        // Health factor = collateral_value / debt_value
        // We want: health_factor >= 1.5
        // So: collateral_value >= debt_value * 1.5
        // max_collateral_value = debt_value * 1.5
        // max_collateral_amount = max_collateral_value / collateral_price

        let min_health_factor = Decimal::from_str_exact("1.5")
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse min health factor: {e}")))?;

        let min_collateral_value = debt_value_usd * min_health_factor;
        let max_collateral_amount = min_collateral_value / collateral_price.price;

        let max_withdrawable = if loan.collateral_amount > max_collateral_amount {
            loan.collateral_amount - max_collateral_amount
        } else {
            Decimal::ZERO
        };

        let health_factor_after = if max_withdrawable > Decimal::ZERO {
            let collateral_after = loan.collateral_amount - max_withdrawable;
            let collateral_value = collateral_after * collateral_price.price;
            if debt_value_usd > Decimal::ZERO {
                collateral_value / debt_value_usd
            } else {
                Decimal::MAX
            }
        } else {
            Self::calculate_health_factor(
                &price_feed,
                &loan.borrow_asset,
                &loan.collateral_asset,
                remaining_debt,
                loan.collateral_amount,
            )
            .await?
        };

        Ok(SafeWithdrawalInfo {
            loan_id,
            current_collateral_amount: loan.collateral_amount,
            max_withdrawable_amount: max_withdrawable,
            health_factor_after_withdrawal: health_factor_after,
            min_health_factor,
        })
    }

    /// Get required collateral amount for a given loan amount
    pub async fn get_required_collateral(
        pool: &PgPool,
        price_feed: Arc<dyn PriceFeedService>,
        loan_id: Uuid,
    ) -> Result<CollateralRequirements, ApiError> {
        let loan = LoanLifecycleService::get_loan(pool, loan_id).await?;

        let borrow_price = price_feed.get_price(&loan.borrow_asset).await?;
        let collateral_price = price_feed.get_price(&loan.collateral_asset).await?;

        let remaining_debt = loan.principal - loan.amount_repaid;
        let debt_value_usd = remaining_debt * borrow_price.price;

        // Required collateral value = debt_value * 1.5 (for 150% health factor)
        let min_health_factor = Decimal::from_str_exact("1.5")
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse min health factor: {e}")))?;

        let required_collateral_value = debt_value_usd * min_health_factor;
        let required_collateral_amount = required_collateral_value / collateral_price.price;

        let collateral_surplus = loan.collateral_amount - required_collateral_amount;

        let health_factor = Self::calculate_health_factor(
            &price_feed,
            &loan.borrow_asset,
            &loan.collateral_asset,
            remaining_debt,
            loan.collateral_amount,
        )
        .await?;

        Ok(CollateralRequirements {
            loan_id,
            borrow_asset: loan.borrow_asset,
            principal: loan.principal,
            collateral_asset: loan.collateral_asset,
            required_collateral_amount,
            current_collateral_amount: loan.collateral_amount,
            collateral_surplus,
            health_factor,
            min_health_factor,
        })
    }

    // ── Helper Functions ──────────────────────────────────────────────────────

    /// Calculate health factor for a loan
    /// health_factor = collateral_value_usd / debt_value_usd
    async fn calculate_health_factor(
        price_feed: &Arc<dyn PriceFeedService>,
        borrow_asset: &str,
        collateral_asset: &str,
        debt_amount: Decimal,
        collateral_amount: Decimal,
    ) -> Result<Decimal, ApiError> {
        if debt_amount <= Decimal::ZERO {
            return Ok(Decimal::MAX);
        }

        let borrow_price = price_feed.get_price(borrow_asset).await?;
        let collateral_price = price_feed.get_price(collateral_asset).await?;

        let debt_value_usd = debt_amount * borrow_price.price;
        let collateral_value_usd = collateral_amount * collateral_price.price;

        if debt_value_usd > Decimal::ZERO {
            Ok(collateral_value_usd / debt_value_usd)
        } else {
            Ok(Decimal::MAX)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_add_collateral_request_validation() {
        let req = AddCollateralRequest {
            loan_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            amount: dec!(100.00),
            transaction_hash: None,
        };
        assert!(req.amount > Decimal::ZERO);
    }

    #[test]
    fn test_remove_collateral_request_validation() {
        let req = RemoveCollateralRequest {
            loan_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            amount: dec!(50.00),
            transaction_hash: None,
        };
        assert!(req.amount > Decimal::ZERO);
    }

    #[test]
    fn test_swap_collateral_request_validation() {
        let req = SwapCollateralRequest {
            loan_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            new_collateral_asset: "ETH".to_string(),
            new_collateral_amount: dec!(10.00),
            transaction_hash: None,
        };
        assert!(req.new_collateral_amount > Decimal::ZERO);
    }

    #[test]
    fn test_collateral_info_serialization() {
        let info = CollateralInfo {
            loan_id: Uuid::new_v4(),
            collateral_asset: "USDC".to_string(),
            collateral_amount: dec!(1000.00),
            collateral_value_usd: dec!(1000.00),
            current_price: dec!(1.00),
        };

        let json = serde_json::to_value(&info).unwrap();
        assert!(json.is_object());
        assert_eq!(json["collateralAsset"], "USDC");
    }

    #[test]
    fn test_collateral_requirements_serialization() {
        let reqs = CollateralRequirements {
            loan_id: Uuid::new_v4(),
            borrow_asset: "USDC".to_string(),
            principal: dec!(1000.00),
            collateral_asset: "ETH".to_string(),
            required_collateral_amount: dec!(0.5),
            current_collateral_amount: dec!(1.0),
            collateral_surplus: dec!(0.5),
            health_factor: dec!(2.0),
            min_health_factor: dec!(1.5),
        };

        let json = serde_json::to_value(&reqs).unwrap();
        assert!(json.is_object());
        assert_eq!(json["borrowAsset"], "USDC");
    }
}

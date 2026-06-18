use crate::api_error::ApiError;
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use crate::price_feed::PriceFeedService;
use crate::safe_math::SafeMath;
use crate::token_metadata;
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

/// Canonical scale (decimal places) for all USD valuations. Bounding the scale
/// of every monetary value keeps multiplications well within `Decimal`'s
/// 28-significant-digit budget and makes valuations of differently-scaled
/// tokens directly comparable.
const USD_VALUATION_SCALE: u32 = 8;

/// Scale of the `plans.health_factor` column (`DECIMAL(10, 4)`). The health
/// factor is rounded to this scale before *both* the risk comparison and
/// persistence so the stored value and the risk flag can never disagree at the
/// liquidation-threshold boundary.
const HEALTH_FACTOR_SCALE: u32 = 4;

/// Round a stored token amount to its native on-chain precision, dropping any
/// sub-unit dust that the flat `NUMERIC(_, 8)` storage scale may have retained.
/// Native precision is resolved from the shared [`token_metadata`] registry.
fn normalize_amount(amount: Decimal, asset_code: &str) -> Decimal {
    amount.round_dp(token_metadata::decimals_for(asset_code))
}

/// Value `amount` of `asset_code` in USD at `price`, normalizing the amount to
/// the token's native precision first and returning the result at the canonical
/// USD valuation scale. Uses checked arithmetic so high-value tokens cannot
/// overflow.
fn value_in_usd(amount: Decimal, price: Decimal, asset_code: &str) -> Result<Decimal, ApiError> {
    let normalized = normalize_amount(amount, asset_code);
    let value = SafeMath::mul(normalized, price)?;
    Ok(value.round_dp(USD_VALUATION_SCALE))
}

/// Compute a health factor (`collateral_value / debt_value`) rounded to the
/// health-factor storage scale. Errors on zero debt via `SafeMath::div`.
fn compute_health_factor(
    collateral_value: Decimal,
    debt_value: Decimal,
) -> Result<Decimal, ApiError> {
    let hf = SafeMath::div(collateral_value, debt_value)?;
    Ok(hf.round_dp(HEALTH_FACTOR_SCALE))
}

/// Outcome of evaluating a single borrowing position's collateral health.
#[derive(Debug, Clone, PartialEq)]
struct PositionAssessment {
    /// USD value of the collateral at the canonical valuation scale.
    collateral_value: Decimal,
    /// USD value of the outstanding debt at the canonical valuation scale.
    debt_value: Decimal,
    /// Collateral-to-debt health factor at the storage scale.
    health_factor: Decimal,
    /// Liquidation threshold applied for this collateral asset.
    liquidation_threshold: Decimal,
    /// Whether the position should be flagged as risky.
    is_risky: bool,
}

/// Pure, DB-free evaluation of a borrowing position.
///
/// Values both sides in USD (normalizing each amount to its token's native
/// precision via [`value_in_usd`]), derives the health factor, and decides
/// whether the position is risky against the collateral asset's liquidation
/// threshold. The comparison is strict (`<`), so a position sitting exactly at
/// its threshold is *not* flagged.
///
/// Returns `Ok(None)` when there is no positive debt to assess. Returns `Err`
/// if any valuation overflows or debt-side arithmetic is invalid, so callers
/// can skip the position instead of panicking.
#[allow(clippy::too_many_arguments)]
fn assess_position(
    collateral_amount: Decimal,
    collateral_asset: &str,
    collateral_price: Decimal,
    debt_amount: Decimal,
    borrow_asset: &str,
    borrow_price: Decimal,
    fallback_threshold: Decimal,
    risk_override_enabled: bool,
) -> Result<Option<PositionAssessment>, ApiError> {
    let collateral_value = value_in_usd(collateral_amount, collateral_price, collateral_asset)?;
    let debt_value = value_in_usd(debt_amount, borrow_price, borrow_asset)?;

    if debt_value <= Decimal::ZERO {
        return Ok(None);
    }

    let health_factor = compute_health_factor(collateral_value, debt_value)?;
    let liquidation_threshold =
        token_metadata::liquidation_threshold_for(collateral_asset, fallback_threshold);

    let is_risky = if risk_override_enabled {
        false
    } else {
        health_factor < liquidation_threshold
    };

    Ok(Some(PositionAssessment {
        collateral_value,
        debt_value,
        health_factor,
        liquidation_threshold,
        is_risky,
    }))
}

pub struct RiskEngine {
    db: PgPool,
    price_feed: Arc<dyn PriceFeedService>,
    liquidation_threshold: Decimal,
}

impl RiskEngine {
    pub fn new(
        db: PgPool,
        price_feed: Arc<dyn PriceFeedService>,
        liquidation_threshold: Decimal,
    ) -> Self {
        Self {
            db,
            price_feed,
            liquidation_threshold,
        }
    }

    pub fn start(self: Arc<Self>) {
        // Periodic full-scan (existing behavior)
        let periodic = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = periodic.check_all_loans().await {
                    error!("Risk Engine error checking loans: {}", e);
                    crate::error_tracking::capture_message(
                        &format!("RiskEngine::check_all_loans failed: {e}"),
                        sentry::Level::Error,
                    );
                }
            }
        });

        // Watch for price feed updates and trigger recalculation immediately
        let watcher = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            let mut last_seen: Option<chrono::DateTime<Utc>> = None;
            loop {
                interval.tick().await;

                let current: Result<Option<chrono::DateTime<Utc>>, sqlx::Error> =
                    sqlx::query_scalar("SELECT MAX(last_updated) FROM price_feeds")
                        .fetch_one(&watcher.db)
                        .await;

                match current {
                    Ok(ts_opt) => {
                        if ts_opt != last_seen {
                            // Update seen timestamp and trigger recalculation when non-none
                            last_seen = ts_opt;
                            if last_seen.is_some() {
                                if let Err(e) = watcher.check_all_loans().await {
                                    error!(
                                        "Risk Engine error recalculating after price update: {}",
                                        e
                                    );
                                } else {
                                    info!("Risk Engine recalculated health factors after price update.");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Risk Engine watcher DB error reading price_feeds: {}", e);
                    }
                }
            }
        });
    }

    pub async fn check_all_loans(&self) -> Result<(), ApiError> {
        #[derive(sqlx::FromRow)]
        struct LoanHealthRow {
            plan_id: uuid::Uuid,
            user_id: uuid::Uuid,
            borrow_asset: String,
            total_debt: rust_decimal::Decimal,
            collateral_asset: Option<String>,
            collateral_amount: Option<rust_decimal::Decimal>,
            is_risky: Option<bool>,
            risk_override_enabled: Option<bool>,
        }

        // Find plans that have borrowing activity by aggregating lending events.
        // Exclude paused plans from risk monitoring
        let loans_health = sqlx::query_as::<_, LoanHealthRow>(
            r#"
            SELECT ll.plan_id, ll.user_id, ll.borrow_asset, 
                   (ll.principal - ll.amount_repaid) AS total_debt,
                   ll.collateral_asset, ll.collateral_amount,
                   p.is_risky, p.risk_override_enabled
            FROM loan_lifecycle ll
            JOIN plans p ON p.id = ll.plan_id
            WHERE (ll.principal - ll.amount_repaid) > 0
              AND ll.status = 'active'
              AND (p.is_paused IS NULL OR p.is_paused = false)
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error loading loan balances: {e}")))?;

        for loan in loans_health {
            // Get prices for evaluation — skip loan if either price is stale
            let borrow_price = match self.price_feed.get_fresh_price(&loan.borrow_asset).await {
                Ok(p) => p.price,
                Err(e) => {
                    warn!(
                        "Risk Engine: Could not get fresh price for borrow asset {}: {}",
                        loan.borrow_asset, e
                    );
                    continue;
                }
            };

            let collat_asset = loan.collateral_asset.unwrap_or_else(|| "USDC".to_string());
            let collat_price = match self.price_feed.get_fresh_price(&collat_asset).await {
                Ok(p) => p.price,
                Err(e) => {
                    warn!(
                        "Risk Engine: Could not get fresh price for collateral asset {}: {}",
                        collat_asset, e
                    );
                    continue;
                }
            };

            // Evaluate the position: value both sides in USD (normalizing each
            // amount to its token's native precision via checked arithmetic),
            // derive the health factor, and decide risk against the collateral
            // asset's liquidation threshold. A `None` result means there is no
            // outstanding debt; an `Err` means valuation arithmetic failed and
            // the plan is skipped rather than risking a panic.
            let collat_amount = loan.collateral_amount.unwrap_or(Decimal::ZERO);
            let should_skip_risk_check = loan.risk_override_enabled.unwrap_or(false);
            let assessment = match assess_position(
                collat_amount,
                &collat_asset,
                collat_price,
                loan.total_debt,
                &loan.borrow_asset,
                borrow_price,
                self.liquidation_threshold,
                should_skip_risk_check,
            ) {
                Ok(Some(a)) => a,
                Ok(None) => continue, // no outstanding debt to assess
                Err(e) => {
                    warn!(
                        "Risk Engine: skipping plan {} — position assessment failed: {}",
                        loan.plan_id, e
                    );
                    continue;
                }
            };

            let health_factor = assessment.health_factor;
            let is_now_risky = assessment.is_risky;

            {
                // Update database state
                sqlx::query(
                    r#"
                    UPDATE plans
                    SET is_risky = $1, health_factor = $2, risk_flagged_at = CASE WHEN $1 AND risk_flagged_at IS NULL THEN CURRENT_TIMESTAMP WHEN NOT $1 THEN NULL ELSE risk_flagged_at END
                    WHERE id = $3
                    "#
                )
                .bind(is_now_risky)
                .bind(health_factor)
                .bind(loan.plan_id)
                .execute(&self.db)
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error updating plan risk status: {e}")))?;

                // Notify if transitioned to risky (and not overridden)
                if is_now_risky && !loan.is_risky.unwrap_or(false) && !should_skip_risk_check {
                    info!(
                        "Plan {} for User {} flagged as risky. HF: {}",
                        loan.plan_id, loan.user_id, health_factor
                    );

                    let mut tx =
                        self.db.begin().await.map_err(|e| {
                            ApiError::Internal(anyhow::anyhow!("Tx start error: {e}"))
                        })?;

                    NotificationService::create(
                        &mut tx,
                        loan.user_id,
                        notif_type::LIQUIDATION_WARNING,
                        format!("WARNING: Your loan against plan {} is at risk of liquidation. Health factor is now {:.2}. Please add collateral or repay some debt.", loan.plan_id, health_factor)
                    ).await?;

                    AuditLogService::log(
                        &mut *tx,
                        Some(loan.user_id),
                        None,
                        audit_action::LIQUIDATION_WARNING,
                        Some(loan.plan_id),
                        Some(entity_type::PLAN),
                        None,
                        None,
                        None,
                    )
                    .await?;

                    tx.commit()
                        .await
                        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {e}")))?;
                } else if !is_now_risky && loan.is_risky.unwrap_or(false) {
                    info!(
                        "Plan {} for User {} is no longer risky. HF: {}",
                        loan.plan_id, loan.user_id, health_factor
                    );
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn normalize_amount_drops_dust_below_token_precision() {
        // USDC has 6 decimals; digits beyond that are dust and must be dropped.
        assert_eq!(
            normalize_amount(dec!(100.123456789), "USDC"),
            dec!(100.123457)
        );
        // XLM has 7 decimals.
        assert_eq!(normalize_amount(dec!(50.12345678), "XLM"), dec!(50.1234568));
    }

    #[test]
    fn normalize_amount_preserves_high_precision_tokens() {
        // ETH (18 decimals) stored at scale 8 keeps all stored digits.
        assert_eq!(normalize_amount(dec!(1.23456789), "ETH"), dec!(1.23456789));
        // BTC (8 decimals) keeps its full stored precision.
        assert_eq!(normalize_amount(dec!(0.12345678), "BTC"), dec!(0.12345678));
    }

    #[test]
    fn value_in_usd_normalizes_then_prices_at_canonical_scale() {
        // 100.1234569 USDC (normalized to 100.123457) * $1.00 = 100.12345700
        let v = value_in_usd(dec!(100.1234569), dec!(1.00), "USDC").unwrap();
        assert_eq!(v, dec!(100.12345700));
        assert_eq!(v.scale(), USD_VALUATION_SCALE);
    }

    #[test]
    fn value_in_usd_handles_high_value_tokens_without_overflow() {
        // 1000 BTC at $1,000,000 — large product must not overflow.
        let v = value_in_usd(dec!(1000), dec!(1000000), "BTC").unwrap();
        assert_eq!(v, dec!(1000000000.00000000));
    }

    #[test]
    fn compute_health_factor_rounds_to_storage_scale() {
        // 150 collateral / 100 debt = 1.5, within the DECIMAL(10,4) storage scale.
        let hf = compute_health_factor(dec!(150), dec!(100)).unwrap();
        assert_eq!(hf, dec!(1.5));
    }

    #[test]
    fn compute_health_factor_is_consistent_at_threshold_boundary() {
        // A repeating ratio must round to at most the storage scale so the
        // persisted value and the risk flag agree. 1000/3000 = 0.3333... -> 0.3333.
        let hf = compute_health_factor(dec!(1000), dec!(3000)).unwrap();
        assert_eq!(hf, dec!(0.3333));
        assert!(hf.scale() <= HEALTH_FACTOR_SCALE);
    }

    #[test]
    fn compute_health_factor_zero_debt_is_error() {
        assert!(compute_health_factor(dec!(100), dec!(0)).is_err());
    }

    // --- assess_position ---

    #[test]
    fn assess_position_healthy_is_not_risky() {
        // 200 USDC collateral vs 100 USDC debt at $1 -> HF 2.0, well above 0.95.
        let a = assess_position(
            dec!(200),
            "USDC",
            dec!(1),
            dec!(100),
            "USDC",
            dec!(1),
            dec!(0.90),
            false,
        )
        .unwrap()
        .unwrap();
        assert_eq!(a.collateral_value, dec!(200.00000000));
        assert_eq!(a.debt_value, dec!(100.00000000));
        assert_eq!(a.health_factor, dec!(2));
        assert_eq!(a.liquidation_threshold, dec!(0.95));
        assert!(!a.is_risky);
    }

    #[test]
    fn assess_position_undercollateralized_is_risky() {
        // 90 USDC collateral vs 100 USDC debt -> HF 0.90 < 0.95 threshold.
        let a = assess_position(
            dec!(90),
            "USDC",
            dec!(1),
            dec!(100),
            "USDC",
            dec!(1),
            dec!(0.50),
            false,
        )
        .unwrap()
        .unwrap();
        assert_eq!(a.health_factor, dec!(0.9));
        assert!(a.is_risky);
    }

    #[test]
    fn assess_position_at_exact_threshold_is_not_risky() {
        // HF exactly equal to the threshold must NOT trip (comparison is strict <).
        // 95 USDC vs 100 USDC -> HF 0.95 == USDC threshold 0.95.
        let a = assess_position(
            dec!(95),
            "USDC",
            dec!(1),
            dec!(100),
            "USDC",
            dec!(1),
            dec!(0.50),
            false,
        )
        .unwrap()
        .unwrap();
        assert_eq!(a.health_factor, dec!(0.95));
        assert!(!a.is_risky);
    }

    #[test]
    fn assess_position_override_never_risky() {
        // Deeply undercollateralized, but override disables risk flagging.
        let a = assess_position(
            dec!(1),
            "USDC",
            dec!(1),
            dec!(100),
            "USDC",
            dec!(1),
            dec!(0.50),
            true,
        )
        .unwrap()
        .unwrap();
        assert!(a.health_factor < a.liquidation_threshold);
        assert!(!a.is_risky);
    }

    #[test]
    fn assess_position_zero_debt_returns_none() {
        let a = assess_position(
            dec!(100),
            "USDC",
            dec!(1),
            dec!(0),
            "USDC",
            dec!(1),
            dec!(0.90),
            false,
        )
        .unwrap();
        assert!(a.is_none());
    }

    #[test]
    fn assess_position_uses_collateral_asset_threshold() {
        // XLM collateral threshold is 0.80. HF 0.82 is above it -> not risky,
        // even though it would be risky under USDC's stricter 0.95.
        let a = assess_position(
            dec!(82),
            "XLM",
            dec!(1),
            dec!(100),
            "USDC",
            dec!(1),
            dec!(0.50),
            false,
        )
        .unwrap()
        .unwrap();
        assert_eq!(a.liquidation_threshold, dec!(0.80));
        assert_eq!(a.health_factor, dec!(0.82));
        assert!(!a.is_risky);
    }

    #[test]
    fn assess_position_normalizes_token_decimals() {
        // USDC dust beyond 6 decimals is dropped before valuation.
        let a = assess_position(
            dec!(100.12345678),
            "USDC",
            dec!(1),
            dec!(50),
            "USDC",
            dec!(1),
            dec!(0.90),
            false,
        )
        .unwrap()
        .unwrap();
        assert_eq!(a.collateral_value, dec!(100.12345700));
    }

    #[test]
    fn assess_position_overflow_is_error() {
        // A price that overflows when multiplied by a huge amount must error,
        // not panic.
        let huge = Decimal::MAX;
        assert!(assess_position(
            huge,
            "USDC",
            dec!(1000000),
            dec!(100),
            "USDC",
            dec!(1),
            dec!(0.90),
            false,
        )
        .is_err());
    }
}

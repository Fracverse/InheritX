use crate::api_error::ApiError;
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use crate::price_feed::PriceFeedService;
use crate::safe_math::SafeMath;
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

/// Native on-chain decimal precision for an asset, case-insensitive.
///
/// Token amounts are stored as `NUMERIC(_, 8)`, but each token has its own true
/// precision (USDC: 6, XLM: 7, BTC: 8, ETH: 18). Anything stored beyond a
/// token's native precision is dust that must not influence a valuation.
/// Unknown assets fall back to the storage scale of 8.
fn token_decimals(asset_code: &str) -> u32 {
    match asset_code.to_uppercase().as_str() {
        "USDC" | "USDT" => 6,
        "XLM" | "STELLAR_XLM" => 7,
        "BTC" | "WBTC" => 8,
        "ETH" | "WETH" => 18,
        _ => 8,
    }
}

/// Round a stored token amount to its native on-chain precision, dropping any
/// sub-unit dust that the flat `NUMERIC(_, 8)` storage scale may have retained.
fn normalize_amount(amount: Decimal, asset_code: &str) -> Decimal {
    amount.round_dp(token_decimals(asset_code))
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

            // Value collateral and debt in USD, normalizing each amount to its
            // token's native precision and using checked arithmetic so a
            // high-value position cannot overflow or silently lose precision.
            let collat_amount = loan.collateral_amount.unwrap_or(Decimal::ZERO);
            let collat_value = match value_in_usd(collat_amount, collat_price, &collat_asset) {
                Ok(v) => v,
                Err(e) => {
                    warn!(
                        "Risk Engine: skipping plan {} — collateral valuation failed: {}",
                        loan.plan_id, e
                    );
                    continue;
                }
            };
            let debt_value = match value_in_usd(loan.total_debt, borrow_price, &loan.borrow_asset) {
                Ok(v) => v,
                Err(e) => {
                    warn!(
                        "Risk Engine: skipping plan {} — debt valuation failed: {}",
                        loan.plan_id, e
                    );
                    continue;
                }
            };

            if debt_value > Decimal::ZERO {
                let health_factor = match compute_health_factor(collat_value, debt_value) {
                    Ok(hf) => hf,
                    Err(e) => {
                        warn!(
                            "Risk Engine: skipping plan {} — health factor computation failed: {}",
                            loan.plan_id, e
                        );
                        continue;
                    }
                };

                // Skip risk flagging if risk override is enabled
                let should_skip_risk_check = loan.risk_override_enabled.unwrap_or(false);

                // Determine liquidation threshold based on collateral asset when possible
                let asset_upper = collat_asset.to_uppercase();
                let liquidation_threshold_for_asset = match asset_upper.as_str() {
                    "USDC" => Decimal::new(95, 2),                // 0.95
                    "ETH" | "WETH" => Decimal::new(85, 2),        // 0.85
                    "BTC" | "WBTC" => Decimal::new(85, 2),        // 0.85
                    "XLM" | "STELLAR_XLM" => Decimal::new(80, 2), // 0.80
                    // Fallback to engine-wide threshold if unknown
                    _ => self.liquidation_threshold,
                };

                let is_now_risky = if should_skip_risk_check {
                    false // Override: never mark as risky
                } else {
                    health_factor < liquidation_threshold_for_asset
                };

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
    fn token_decimals_known_assets() {
        // Native on-chain precision per asset, case-insensitive.
        assert_eq!(token_decimals("USDC"), 6);
        assert_eq!(token_decimals("usdc"), 6);
        assert_eq!(token_decimals("USDT"), 6);
        assert_eq!(token_decimals("BTC"), 8);
        assert_eq!(token_decimals("WBTC"), 8);
        assert_eq!(token_decimals("ETH"), 18);
        assert_eq!(token_decimals("WETH"), 18);
        assert_eq!(token_decimals("XLM"), 7);
        assert_eq!(token_decimals("STELLAR_XLM"), 7);
    }

    #[test]
    fn token_decimals_unknown_asset_defaults_to_storage_scale() {
        // Unknown assets fall back to the NUMERIC(_, 8) storage scale.
        assert_eq!(token_decimals("DOGE"), 8);
    }

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
}

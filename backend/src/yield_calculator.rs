use rust_decimal::{Decimal, MathematicalOps};

/// Seconds in a standard 365-day year (ledger time basis).
pub const SECONDS_PER_YEAR: u64 = 31_536_000;

const BPS_DENOMINATOR: u32 = 10_000;

/// APY compounding configuration.
///
/// `compounding_periods_per_year` is `n` in `A = P * (1 + r/n)^(nt)`.
/// Defaults to per-second compounding so ledger elapsed seconds map directly
/// to compounding periods when `n == SECONDS_PER_YEAR`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApyConfig {
    pub compounding_periods_per_year: u32,
}

impl Default for ApyConfig {
    fn default() -> Self {
        Self {
            compounding_periods_per_year: SECONDS_PER_YEAR as u32,
        }
    }
}

impl ApyConfig {
    pub fn from_env() -> Self {
        let compounding_periods_per_year = std::env::var("YIELD_COMPOUNDING_PERIODS_PER_YEAR")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|n: &u32| *n > 0)
            .unwrap_or(SECONDS_PER_YEAR as u32);

        Self {
            compounding_periods_per_year,
        }
    }

    /// Converts basis points to the annual rate `r` used in the compound formula.
    pub fn annual_rate_from_bps(yield_rate_bps: u32) -> Decimal {
        Decimal::from(yield_rate_bps) / Decimal::from(BPS_DENOMINATOR)
    }
}

/// Projects accrued virtual yield for a locked plan over `elapsed_secs`.
///
/// Uses compound interest: `A = P * (1 + r/n)^(nt)`, returning `A - P`.
/// Returns zero when principal, rate, or elapsed time is zero.
pub fn calculate_yield(
    principal: Decimal,
    yield_rate_bps: u32,
    elapsed_secs: u64,
    config: &ApyConfig,
) -> Decimal {
    if elapsed_secs == 0 || yield_rate_bps == 0 || principal.is_zero() {
        return Decimal::ZERO;
    }

    let r = ApyConfig::annual_rate_from_bps(yield_rate_bps);
    let n = Decimal::from(config.compounding_periods_per_year);
    let t = Decimal::from(elapsed_secs) / Decimal::from(SECONDS_PER_YEAR);
    let nt = n * t;

    let rate_per_period = Decimal::ONE + (r / n);
    let growth_factor = rate_per_period.powd(nt);
    let total = principal * growth_factor;

    (total - principal).max(Decimal::ZERO)
}

/// Convenience wrapper using default APY configuration (per-second compounding).
pub fn calculate_yield_default(
    principal: Decimal,
    yield_rate_bps: u32,
    elapsed_secs: u64,
) -> Decimal {
    calculate_yield(
        principal,
        yield_rate_bps,
        elapsed_secs,
        &ApyConfig::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn dec(value: &str) -> Decimal {
        Decimal::from_str(value).expect("valid decimal literal")
    }

    fn principal(value: i64) -> Decimal {
        Decimal::from(value)
    }

    #[test]
    fn zero_elapsed_secs_returns_zero_accrual() {
        let config = ApyConfig::default();
        assert_eq!(
            calculate_yield(principal(10_000), 500, 0, &config),
            Decimal::ZERO
        );
    }

    #[test]
    fn zero_principal_returns_zero_accrual() {
        let config = ApyConfig::default();
        assert_eq!(
            calculate_yield(Decimal::ZERO, 500, SECONDS_PER_YEAR, &config),
            Decimal::ZERO
        );
    }

    #[test]
    fn zero_yield_rate_returns_zero_accrual() {
        let config = ApyConfig::default();
        assert_eq!(
            calculate_yield(principal(10_000), 0, SECONDS_PER_YEAR, &config),
            Decimal::ZERO
        );
    }

    #[test]
    fn annual_compounding_one_year() {
        let config = ApyConfig {
            compounding_periods_per_year: 1,
        };
        let accrual = calculate_yield(principal(10_000), 500, SECONDS_PER_YEAR, &config);
        assert_eq!(accrual, dec("500"));
    }

    #[test]
    fn monthly_compounding_one_year() {
        let config = ApyConfig {
            compounding_periods_per_year: 12,
        };
        let accrual = calculate_yield(principal(10_000), 500, SECONDS_PER_YEAR, &config);
        // 10000 * (1 + 0.05/12)^12 - 10000
        assert!(accrual > dec("511.5") && accrual < dec("511.7"));
    }

    #[test]
    fn per_second_compounding_one_year_exceeds_simple_interest() {
        let config = ApyConfig::default();
        let accrual = calculate_yield(principal(10_000), 500, SECONDS_PER_YEAR, &config);
        // (1 + 0.05/31536000)^31536000 - 1 ≈ 5.127%
        assert!(accrual > dec("500"));
        assert!(accrual < dec("513"));
    }

    #[test]
    fn half_year_elapsed_scales_accrual_down() {
        let config = ApyConfig {
            compounding_periods_per_year: 1,
        };
        let full_year = calculate_yield(principal(10_000), 500, SECONDS_PER_YEAR, &config);
        let half_year = calculate_yield(principal(10_000), 500, SECONDS_PER_YEAR / 2, &config);

        assert!(half_year > Decimal::ZERO);
        assert!(half_year < full_year);
    }

    #[test]
    fn ledger_elapsed_seconds_maps_to_per_second_periods() {
        let config = ApyConfig::default();
        let one_day_secs = 86_400u64;
        let one_day = calculate_yield(principal(1_000_000), 500, one_day_secs, &config);
        let two_days = calculate_yield(principal(1_000_000), 500, one_day_secs * 2, &config);

        assert!(one_day > Decimal::ZERO);
        assert!(two_days > one_day);
    }

    #[test]
    fn large_principal_retains_precision() {
        let config = ApyConfig {
            compounding_periods_per_year: 1,
        };
        let large_principal = Decimal::from_i128_with_scale(999_999_999_999_999_999, 0);
        let accrual = calculate_yield(large_principal, 100, SECONDS_PER_YEAR, &config);

        let expected = large_principal / Decimal::from(100);
        assert_eq!(accrual, expected);
    }

    #[test]
    fn apy_config_from_env_overrides_default() {
        std::env::set_var("YIELD_COMPOUNDING_PERIODS_PER_YEAR", "4");
        let config = ApyConfig::from_env();
        assert_eq!(config.compounding_periods_per_year, 4);
        std::env::remove_var("YIELD_COMPOUNDING_PERIODS_PER_YEAR");
    }

    #[test]
    fn apy_config_from_env_ignores_invalid_values() {
        std::env::set_var("YIELD_COMPOUNDING_PERIODS_PER_YEAR", "0");
        let config = ApyConfig::from_env();
        assert_eq!(config.compounding_periods_per_year, SECONDS_PER_YEAR as u32);
        std::env::remove_var("YIELD_COMPOUNDING_PERIODS_PER_YEAR");
    }

    #[test]
    fn calculate_yield_default_matches_default_config() {
        let principal = principal(25_000);
        let elapsed = 1_000_000u64;
        let bps = 350u32;

        assert_eq!(
            calculate_yield_default(principal, bps, elapsed),
            calculate_yield(principal, bps, elapsed, &ApyConfig::default())
        );
    }

    #[test]
    fn annual_rate_from_bps_conversion() {
        assert_eq!(ApyConfig::annual_rate_from_bps(500), dec("0.05"));
        assert_eq!(ApyConfig::annual_rate_from_bps(1), dec("0.0001"));
        assert_eq!(ApyConfig::annual_rate_from_bps(10_000), dec("1"));
    }
}

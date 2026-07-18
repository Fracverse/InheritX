//! Overflow-safe arithmetic and fixed-point compounding for yield accrual.
//!
//! All operations use checked i128 math (`checked_mul`, `checked_div`, ...)
//! and surface failures as contract `Error` values instead of panicking, so
//! a hostile or extreme input can never abort the VM with an unhandled trap.
//!
//! Compounding model: `yield_rate_bps` is an annual rate in basis points
//! (500 = 5% APY). Interest compounds once per whole elapsed day using a
//! fixed-point growth factor at `YIELD_SCALE` precision, raised to the
//! number of elapsed days with exponentiation by squaring (O(log n) muls).

use crate::Error;

/// Basis-point denominator: 10_000 bps == 100%.
pub const BPS_DENOMINATOR: i128 = 10_000;

/// Fixed-point scale (1e12) used for growth factors during compounding.
pub const YIELD_SCALE: i128 = 1_000_000_000_000;

/// Length of one compounding period in seconds (daily compounding).
pub const SECONDS_PER_DAY: u64 = 86_400;

/// Days per year used to derive the per-day rate from the annual rate.
pub const DAYS_PER_YEAR: i128 = 365;

/// Maximum accepted annual yield rate: 10_000 bps == 100% APY.
pub const MAX_YIELD_RATE_BPS: u32 = 10_000;

/// Checked i128 addition.
pub fn safe_add(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_add(b).ok_or(Error::MathOverflow)
}

/// Checked i128 subtraction.
pub fn safe_sub(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_sub(b).ok_or(Error::MathOverflow)
}

/// Checked i128 multiplication.
pub fn safe_mul(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_mul(b).ok_or(Error::MathOverflow)
}

/// Checked i128 division. Rejects a zero divisor explicitly and maps the
/// single overflowing case (i128::MIN / -1) to `MathOverflow`.
pub fn safe_div(a: i128, b: i128) -> Result<i128, Error> {
    if b == 0 {
        return Err(Error::DivisionByZero);
    }
    a.checked_div(b).ok_or(Error::MathOverflow)
}

/// Checked u64 addition, used for ledger-timestamp deadline arithmetic.
pub fn safe_add_u64(a: u64, b: u64) -> Result<u64, Error> {
    a.checked_add(b).ok_or(Error::MathOverflow)
}

/// Checked u64 multiplication, used for seconds/period conversions.
pub fn safe_mul_u64(a: u64, b: u64) -> Result<u64, Error> {
    a.checked_mul(b).ok_or(Error::MathOverflow)
}

/// Computes (a * b) / denominator with overflow-checked intermediate steps.
pub fn mul_div(a: i128, b: i128, denominator: i128) -> Result<i128, Error> {
    safe_div(safe_mul(a, b)?, denominator)
}

/// Applies a basis-point fraction to an amount: amount * bps / 10_000.
pub fn apply_bps(amount: i128, bps: u32) -> Result<i128, Error> {
    mul_div(amount, bps as i128, BPS_DENOMINATOR)
}

/// Validates that an annual yield rate does not exceed the protocol cap.
pub fn validate_yield_rate(rate_bps: u32) -> Result<(), Error> {
    if rate_bps > MAX_YIELD_RATE_BPS {
        return Err(Error::InvalidYieldRate);
    }
    Ok(())
}

/// Raises a fixed-point factor (scaled by `YIELD_SCALE`) to an integer power
/// using exponentiation by squaring. Every multiply is checked, so extreme
/// exponents surface `MathOverflow` instead of trapping.
pub fn pow_factor(base: i128, mut exp: u64) -> Result<i128, Error> {
    if base < 0 {
        return Err(Error::NegativeAmount);
    }
    let mut result = YIELD_SCALE;
    let mut acc = base;
    loop {
        if exp & 1 == 1 {
            result = mul_div(result, acc, YIELD_SCALE)?;
        }
        exp >>= 1;
        if exp == 0 {
            break;
        }
        acc = mul_div(acc, acc, YIELD_SCALE)?;
    }
    Ok(result)
}

/// Compounds `principal` over `periods` periods at `rate_bps_per_period`
/// per period: principal * (1 + rate)^periods, entirely in checked math.
pub fn compound_amount(
    principal: i128,
    rate_bps_per_period: u32,
    periods: u64,
) -> Result<i128, Error> {
    if principal < 0 {
        return Err(Error::NegativeAmount);
    }
    if principal == 0 || rate_bps_per_period == 0 || periods == 0 {
        return Ok(principal);
    }
    let per_period_factor = safe_add(YIELD_SCALE, apply_bps(YIELD_SCALE, rate_bps_per_period)?)?;
    let growth = pow_factor(per_period_factor, periods)?;
    mul_div(principal, growth, YIELD_SCALE)
}

/// Compounds `principal` at an annual rate of `annual_rate_bps` over
/// `elapsed_seconds`, compounding once per whole elapsed day. Partial days
/// accrue nothing until the day completes. Returns the new total value.
pub fn compound_yield(
    principal: i128,
    annual_rate_bps: u32,
    elapsed_seconds: u64,
) -> Result<i128, Error> {
    if principal < 0 {
        return Err(Error::NegativeAmount);
    }
    let periods = elapsed_seconds / SECONDS_PER_DAY;
    if principal == 0 || annual_rate_bps == 0 || periods == 0 {
        return Ok(principal);
    }
    let daily_increment = safe_div(
        safe_mul(annual_rate_bps as i128, YIELD_SCALE)?,
        safe_mul(BPS_DENOMINATOR, DAYS_PER_YEAR)?,
    )?;
    let daily_factor = safe_add(YIELD_SCALE, daily_increment)?;
    let growth = pow_factor(daily_factor, periods)?;
    mul_div(principal, growth, YIELD_SCALE)
}

/// Interest earned on top of `principal` after compounding: always >= 0.
pub fn accrued_interest(
    principal: i128,
    annual_rate_bps: u32,
    elapsed_seconds: u64,
) -> Result<i128, Error> {
    safe_sub(
        compound_yield(principal, annual_rate_bps, elapsed_seconds)?,
        principal,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- checked primitives ----

    #[test]
    fn safe_add_sums_and_detects_overflow() {
        assert_eq!(safe_add(2, 3), Ok(5));
        assert_eq!(safe_add(-2, -3), Ok(-5));
        assert_eq!(safe_add(i128::MAX, 1), Err(Error::MathOverflow));
        assert_eq!(safe_add(i128::MIN, -1), Err(Error::MathOverflow));
    }

    #[test]
    fn safe_sub_subtracts_and_detects_overflow() {
        assert_eq!(safe_sub(10, 4), Ok(6));
        assert_eq!(safe_sub(4, 10), Ok(-6));
        assert_eq!(safe_sub(i128::MIN, 1), Err(Error::MathOverflow));
        assert_eq!(safe_sub(i128::MAX, -1), Err(Error::MathOverflow));
    }

    #[test]
    fn safe_mul_multiplies_and_detects_overflow() {
        assert_eq!(safe_mul(6, 7), Ok(42));
        assert_eq!(safe_mul(-6, 7), Ok(-42));
        assert_eq!(safe_mul(0, i128::MAX), Ok(0));
        assert_eq!(safe_mul(i128::MAX, 2), Err(Error::MathOverflow));
    }

    #[test]
    fn safe_div_divides_and_rejects_zero_divisor() {
        assert_eq!(safe_div(42, 7), Ok(6));
        assert_eq!(safe_div(7, 2), Ok(3));
        assert_eq!(safe_div(42, 0), Err(Error::DivisionByZero));
        assert_eq!(safe_div(i128::MIN, -1), Err(Error::MathOverflow));
    }

    #[test]
    fn safe_u64_helpers_detect_overflow() {
        assert_eq!(safe_add_u64(1, 2), Ok(3));
        assert_eq!(safe_add_u64(u64::MAX, 1), Err(Error::MathOverflow));
        assert_eq!(safe_mul_u64(3, 4), Ok(12));
        assert_eq!(safe_mul_u64(u64::MAX, 2), Err(Error::MathOverflow));
    }

    #[test]
    fn mul_div_computes_scaled_products() {
        assert_eq!(mul_div(10, 20, 4), Ok(50));
        assert_eq!(mul_div(7, 3, 2), Ok(10));
        assert_eq!(mul_div(5, 5, 0), Err(Error::DivisionByZero));
        assert_eq!(mul_div(i128::MAX, 2, 2), Err(Error::MathOverflow));
    }

    #[test]
    fn apply_bps_takes_basis_point_fractions() {
        assert_eq!(apply_bps(1_000, 5_000), Ok(500));
        assert_eq!(apply_bps(1_000_000, 250), Ok(25_000));
        assert_eq!(apply_bps(1_000, 10_000), Ok(1_000));
        assert_eq!(apply_bps(1_000, 0), Ok(0));
        assert_eq!(apply_bps(999, 3_333), Ok(332));
        assert_eq!(apply_bps(i128::MAX, 2), Err(Error::MathOverflow));
    }

    #[test]
    fn validate_yield_rate_enforces_cap() {
        assert_eq!(validate_yield_rate(0), Ok(()));
        assert_eq!(validate_yield_rate(500), Ok(()));
        assert_eq!(validate_yield_rate(MAX_YIELD_RATE_BPS), Ok(()));
        assert_eq!(
            validate_yield_rate(MAX_YIELD_RATE_BPS + 1),
            Err(Error::InvalidYieldRate)
        );
    }

    // ---- fixed-point exponentiation ----

    #[test]
    fn pow_factor_zero_exponent_is_identity() {
        assert_eq!(pow_factor(3 * YIELD_SCALE, 0), Ok(YIELD_SCALE));
        assert_eq!(pow_factor(0, 0), Ok(YIELD_SCALE));
    }

    #[test]
    fn pow_factor_first_power_returns_base() {
        assert_eq!(pow_factor(YIELD_SCALE, 1), Ok(YIELD_SCALE));
        assert_eq!(pow_factor(2 * YIELD_SCALE, 1), Ok(2 * YIELD_SCALE));
    }

    #[test]
    fn pow_factor_computes_integer_powers() {
        // 2^10 = 1024 at fixed-point scale
        assert_eq!(pow_factor(2 * YIELD_SCALE, 10), Ok(1024 * YIELD_SCALE));
        // 1.1^2 = 1.21 exactly representable at 1e12 scale
        assert_eq!(pow_factor(1_100_000_000_000, 2), Ok(1_210_000_000_000));
    }

    #[test]
    fn pow_factor_rejects_negative_base_and_overflow() {
        assert_eq!(pow_factor(-YIELD_SCALE, 2), Err(Error::NegativeAmount));
        assert_eq!(pow_factor(2 * YIELD_SCALE, 200), Err(Error::MathOverflow));
    }

    // ---- per-period compounding ----

    #[test]
    fn compound_amount_identity_cases() {
        assert_eq!(compound_amount(10_000, 1_000, 0), Ok(10_000));
        assert_eq!(compound_amount(10_000, 0, 5), Ok(10_000));
        assert_eq!(compound_amount(0, 1_000, 5), Ok(0));
    }

    #[test]
    fn compound_amount_compounds_per_period() {
        // 10% per period on 10_000: 11_000, 12_100, 13_310
        assert_eq!(compound_amount(10_000, 1_000, 1), Ok(11_000));
        assert_eq!(compound_amount(10_000, 1_000, 2), Ok(12_100));
        assert_eq!(compound_amount(10_000, 1_000, 3), Ok(13_310));
    }

    #[test]
    fn compound_amount_rejects_negative_principal() {
        assert_eq!(compound_amount(-1, 1_000, 1), Err(Error::NegativeAmount));
    }

    #[test]
    fn compound_amount_overflows_cleanly() {
        assert_eq!(
            compound_amount(i128::MAX / 2, 10_000, 2),
            Err(Error::MathOverflow)
        );
    }

    // ---- time-based compounding ----

    #[test]
    fn compound_yield_identity_cases() {
        assert_eq!(compound_yield(1_000, 500, 0), Ok(1_000));
        // Partial day accrues nothing
        assert_eq!(compound_yield(1_000, 500, SECONDS_PER_DAY - 1), Ok(1_000));
        assert_eq!(compound_yield(1_000, 0, 10 * SECONDS_PER_DAY), Ok(1_000));
        assert_eq!(compound_yield(0, 500, 10 * SECONDS_PER_DAY), Ok(0));
    }

    #[test]
    fn compound_yield_rejects_negative_principal() {
        assert_eq!(
            compound_yield(-5, 500, SECONDS_PER_DAY),
            Err(Error::NegativeAmount)
        );
    }

    #[test]
    fn compound_yield_one_year_beats_simple_interest() {
        let principal: i128 = 1_000_000_000;
        let one_year = 365 * SECONDS_PER_DAY;
        let value = compound_yield(principal, 500, one_year).unwrap();
        // Daily compounding at 5% APY lands between simple 5% and 5.2%
        assert!(
            value > 1_050_000_000,
            "compounding must beat simple interest"
        );
        assert!(value < 1_052_000_000, "5% APY cannot exceed 5.2% in a year");
    }

    #[test]
    fn compound_yield_is_monotonic_in_time() {
        let principal: i128 = 1_000_000_000;
        let y1 = compound_yield(principal, 500, 365 * SECONDS_PER_DAY).unwrap();
        let y2 = compound_yield(principal, 500, 730 * SECONDS_PER_DAY).unwrap();
        assert!(y2 > y1);
        assert!(y1 > principal);
    }

    #[test]
    fn compound_yield_overflow_surfaces_error() {
        // 100% APY over 100 years: growth ~e^100, far beyond i128 range
        let hundred_years = 36_500 * SECONDS_PER_DAY;
        assert_eq!(
            compound_yield(1_000_000, 10_000, hundred_years),
            Err(Error::MathOverflow)
        );
    }

    #[test]
    fn accrued_interest_is_value_minus_principal() {
        let principal: i128 = 1_000_000_000;
        let one_year = 365 * SECONDS_PER_DAY;
        let total = compound_yield(principal, 500, one_year).unwrap();
        let interest = accrued_interest(principal, 500, one_year).unwrap();
        assert_eq!(interest, total - principal);
        assert!(interest > 0);
        assert_eq!(accrued_interest(principal, 500, 0), Ok(0));
    }

    // ---- additional boundary and consistency coverage ----

    #[test]
    fn apply_bps_at_max_rate_is_exact_amount() {
        assert_eq!(apply_bps(123_456, MAX_YIELD_RATE_BPS), Ok(123_456));
    }

    #[test]
    fn apply_bps_handles_negative_amounts_symmetrically() {
        // Bridge fees always operate on non-negative shares, but the helper
        // itself must stay sign-correct since it's pure checked math.
        assert_eq!(apply_bps(-1_000, 5_000), Ok(-500));
    }

    #[test]
    fn mul_div_rounds_toward_zero_like_integer_division() {
        assert_eq!(mul_div(1, 1, 3), Ok(0));
        assert_eq!(mul_div(-1, 1, 3), Ok(0));
        assert_eq!(mul_div(2, 1, 3), Ok(0));
        assert_eq!(mul_div(4, 1, 3), Ok(1));
    }

    #[test]
    fn pow_factor_handles_odd_and_even_exponents_consistently() {
        // x^7 == x^6 * x == (x^3)^2 * x, exercised via the squaring ladder
        let base = 1_050_000_000_000; // 1.05 at YIELD_SCALE
        let seven = pow_factor(base, 7).unwrap();
        let six = pow_factor(base, 6).unwrap();
        let manual_seven = mul_div(six, base, YIELD_SCALE).unwrap();
        assert_eq!(seven, manual_seven);
    }

    #[test]
    fn pow_factor_large_exponent_stays_within_bounds_at_low_base() {
        // A base just above 1.0 with many periods must not overflow even
        // though the loop runs through every bit of a large exponent.
        let base = YIELD_SCALE + 1; // smallest possible daily growth tick
        let result = pow_factor(base, 100_000).unwrap();
        assert!(result >= YIELD_SCALE);
    }

    #[test]
    fn compound_amount_zero_periods_ignores_rate() {
        assert_eq!(compound_amount(10_000, MAX_YIELD_RATE_BPS, 0), Ok(10_000));
    }

    #[test]
    fn compound_amount_single_period_matches_apply_bps_plus_principal() {
        let principal: i128 = 50_000;
        let rate_bps = 750u32;
        let expected = principal + apply_bps(principal, rate_bps).unwrap();
        assert_eq!(compound_amount(principal, rate_bps, 1), Ok(expected));
    }

    #[test]
    fn compound_yield_zero_rate_is_identity_across_long_horizons() {
        let principal: i128 = 987_654_321;
        assert_eq!(
            compound_yield(principal, 0, 10_000 * SECONDS_PER_DAY),
            Ok(principal)
        );
    }

    #[test]
    fn compound_yield_small_principal_does_not_underflow_to_negative() {
        // A tiny principal at a low rate over a short horizon may round its
        // interest down to zero, but must never go negative.
        let value = compound_yield(1, 1, SECONDS_PER_DAY).unwrap();
        assert!(value >= 1);
    }

    #[test]
    fn safe_div_negative_operands_round_toward_zero() {
        assert_eq!(safe_div(-7, 2), Ok(-3));
        assert_eq!(safe_div(7, -2), Ok(-3));
        assert_eq!(safe_div(-7, -2), Ok(3));
    }

    #[test]
    fn safe_mul_u64_zero_and_one_are_identities() {
        assert_eq!(safe_mul_u64(0, 12345), Ok(0));
        assert_eq!(safe_mul_u64(1, 12345), Ok(12345));
    }
}

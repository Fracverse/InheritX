//! Overflow-safe arithmetic for yield accrual, projection, and fee splitting.
//!
//! Every operation in this module is total: it either returns a value or an
//! `InheritanceError`, and it never panics, traps, or wraps. That matters more
//! here than in ordinary application code — a trap inside a harvest aborts the
//! whole transaction, and a silent wraparound would mint or destroy value in a
//! plan's locked balance.
//!
//! ## Conventions
//!
//! - **Rates** are annual, in basis points: `500` means 5.00% per year, and
//!   `BPS_DENOMINATOR` (10_000) is 100%.
//! - **Amounts** are `u64` stroops, matching `InheritancePlan::total_amount`.
//!   Intermediates widen to `u128` so a multiply cannot overflow before the
//!   divide that brings the value back into range.
//! - **Fixed point** growth factors are scaled by `YIELD_SCALE` (1e12). A
//!   factor of `1.05` is stored as `1_050_000_000_000`.
//!
//! ## Error mapping
//!
//! `InheritanceError` sits at the 50-variant ceiling `#[contracterror]`
//! permits, so this module reuses existing variants rather than adding its
//! own:
//!
//! - `InvalidTotalAmount` — an arithmetic result would overflow `u64`/`i128`
//! - `InvalidAllocation` — a rate or basis-point argument is out of range
//! - `InvalidBeneficiaryData` — a divisor was zero

use crate::InheritanceError;

/// Basis-point denominator: 10_000 bps == 100%.
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Fixed-point scale (1e12) used for compounding growth factors.
pub const YIELD_SCALE: u128 = 1_000_000_000_000;

/// Seconds in one compounding period (daily compounding).
pub const SECONDS_PER_DAY: u64 = 86_400;

/// Days per year used to derive a per-day rate from an annual rate.
pub const DAYS_PER_YEAR: u64 = 365;

/// Seconds per year, used for simple pro-rata interest.
pub const SECONDS_PER_YEAR: u64 = 31_536_000;

/// Highest annual rate this module will accept: 10_000 bps == 100% APY.
///
/// Rates above this are rejected rather than clamped, so a fat-fingered
/// configuration surfaces as an error at write time instead of quietly
/// accruing an absurd amount later.
pub const MAX_YIELD_RATE_BPS: u32 = 10_000;

/// Highest performance fee this module will accept: 5_000 bps == 50%.
pub const MAX_PERFORMANCE_FEE_BPS: u32 = 5_000;

/// Cap on compounding periods per projection call.
///
/// Bounds the exponentiation-by-squaring loop and keeps projections inside a
/// predictable gas envelope. 36_500 days is a hundred years — far past any
/// realistic inheritance horizon.
pub const MAX_COMPOUND_PERIODS: u64 = 36_500;

// ─────────────────────────────────────────────────
// Checked primitives
// ─────────────────────────────────────────────────

/// Checked `u64` addition.
pub fn safe_add(a: u64, b: u64) -> Result<u64, InheritanceError> {
    a.checked_add(b).ok_or(InheritanceError::MathOverflow)
}

/// Checked `u64` subtraction. Underflow is an error, not a wrap to `u64::MAX`.
pub fn safe_sub(a: u64, b: u64) -> Result<u64, InheritanceError> {
    a.checked_sub(b).ok_or(InheritanceError::MathOverflow)
}

/// Checked `u64` multiplication.
pub fn safe_mul(a: u64, b: u64) -> Result<u64, InheritanceError> {
    a.checked_mul(b).ok_or(InheritanceError::MathOverflow)
}

/// Checked `u64` division. A zero divisor is rejected explicitly.
pub fn safe_div(a: u64, b: u64) -> Result<u64, InheritanceError> {
    if b == 0 {
        return Err(InheritanceError::InvalidBeneficiaryData);
    }
    a.checked_div(b).ok_or(InheritanceError::MathOverflow)
}

/// Checked `u128` multiplication, for fixed-point intermediates.
pub fn safe_mul_u128(a: u128, b: u128) -> Result<u128, InheritanceError> {
    a.checked_mul(b).ok_or(InheritanceError::MathOverflow)
}

/// Checked `u128` division.
pub fn safe_div_u128(a: u128, b: u128) -> Result<u128, InheritanceError> {
    if b == 0 {
        return Err(InheritanceError::InvalidBeneficiaryData);
    }
    a.checked_div(b).ok_or(InheritanceError::MathOverflow)
}

/// Narrow a `u128` back to `u64`, erroring rather than truncating.
pub fn to_u64(value: u128) -> Result<u64, InheritanceError> {
    u64::try_from(value).map_err(|_| InheritanceError::MathOverflow)
}

/// Computes `a * b / denominator` with a `u128` intermediate, so the multiply
/// cannot overflow before the divide brings the value back into `u64` range.
///
/// Rounds toward zero, which always favours the pool over the claimant — the
/// vault never credits a fraction of a stroop it was not paid.
pub fn mul_div(a: u64, b: u64, denominator: u64) -> Result<u64, InheritanceError> {
    if denominator == 0 {
        return Err(InheritanceError::InvalidBeneficiaryData);
    }
    let product = safe_mul_u128(a as u128, b as u128)?;
    to_u64(safe_div_u128(product, denominator as u128)?)
}

/// Applies a basis-point fraction to an amount: `amount * bps / 10_000`.
pub fn apply_bps(amount: u64, bps: u32) -> Result<u64, InheritanceError> {
    mul_div(amount, bps as u64, BPS_DENOMINATOR)
}

// ─────────────────────────────────────────────────
// Validation
// ─────────────────────────────────────────────────

/// Rejects an annual rate above [`MAX_YIELD_RATE_BPS`].
pub fn validate_yield_rate(rate_bps: u32) -> Result<(), InheritanceError> {
    if rate_bps > MAX_YIELD_RATE_BPS {
        return Err(InheritanceError::InvalidAllocation);
    }
    Ok(())
}

/// Rejects a performance fee above [`MAX_PERFORMANCE_FEE_BPS`].
pub fn validate_performance_fee(fee_bps: u32) -> Result<(), InheritanceError> {
    if fee_bps > MAX_PERFORMANCE_FEE_BPS {
        return Err(InheritanceError::InvalidAllocation);
    }
    Ok(())
}

/// Rejects a period count above [`MAX_COMPOUND_PERIODS`].
pub fn validate_periods(periods: u64) -> Result<(), InheritanceError> {
    if periods > MAX_COMPOUND_PERIODS {
        return Err(InheritanceError::InvalidAllocation);
    }
    Ok(())
}

// ─────────────────────────────────────────────────
// Interest
// ─────────────────────────────────────────────────

/// Simple (non-compounding) pro-rata interest over an elapsed span:
/// `principal * rate_bps * elapsed / (10_000 * seconds_per_year)`.
///
/// This mirrors what the lending pool charges between harvests, so a vault's
/// projection lines up with what it will actually be paid.
pub fn simple_interest(
    principal: u64,
    annual_rate_bps: u32,
    elapsed_secs: u64,
) -> Result<u64, InheritanceError> {
    validate_yield_rate(annual_rate_bps)?;
    if principal == 0 || annual_rate_bps == 0 || elapsed_secs == 0 {
        return Ok(0);
    }

    let numerator = safe_mul_u128(
        safe_mul_u128(principal as u128, annual_rate_bps as u128)?,
        elapsed_secs as u128,
    )?;
    let denominator = safe_mul_u128(BPS_DENOMINATOR as u128, SECONDS_PER_YEAR as u128)?;

    to_u64(safe_div_u128(numerator, denominator)?)
}

/// The per-day rate, in basis points, implied by an annual rate.
///
/// Integer division means a rate below 365 bps floors to 0 per day. Callers
/// that need sub-basis-point daily precision should use [`daily_growth_factor`]
/// instead, which keeps the remainder at `YIELD_SCALE` precision.
pub fn daily_rate_bps(annual_rate_bps: u32) -> Result<u32, InheritanceError> {
    validate_yield_rate(annual_rate_bps)?;
    Ok((annual_rate_bps as u64 / DAYS_PER_YEAR) as u32)
}

/// Number of whole compounding periods in an elapsed span.
pub fn periods_elapsed(elapsed_secs: u64, period_secs: u64) -> Result<u64, InheritanceError> {
    if period_secs == 0 {
        return Err(InheritanceError::InvalidBeneficiaryData);
    }
    Ok(elapsed_secs / period_secs)
}

// ─────────────────────────────────────────────────
// Compounding
// ─────────────────────────────────────────────────

/// The daily growth factor for an annual rate, at [`YIELD_SCALE`] precision.
///
/// Returns `1 + annual_rate / 365` scaled by 1e12, keeping the fractional part
/// that [`daily_rate_bps`] would discard.
pub fn daily_growth_factor(annual_rate_bps: u32) -> Result<u128, InheritanceError> {
    validate_yield_rate(annual_rate_bps)?;

    let daily_numerator = safe_mul_u128(YIELD_SCALE, annual_rate_bps as u128)?;
    let daily_increment = safe_div_u128(
        daily_numerator,
        safe_mul_u128(BPS_DENOMINATOR as u128, DAYS_PER_YEAR as u128)?,
    )?;

    YIELD_SCALE
        .checked_add(daily_increment)
        .ok_or(InheritanceError::MathOverflow)
}

/// Multiplies two [`YIELD_SCALE`] fixed-point factors, returning a factor at
/// the same scale.
pub fn mul_factor(a: u128, b: u128) -> Result<u128, InheritanceError> {
    safe_div_u128(safe_mul_u128(a, b)?, YIELD_SCALE)
}

/// Raises a fixed-point factor to an integer power by exponentiation by
/// squaring — O(log n) multiplies rather than O(n).
///
/// Every multiply is checked, so an extreme exponent surfaces as
/// `InvalidTotalAmount` instead of trapping the VM.
pub fn pow_factor(base: u128, exponent: u64) -> Result<u128, InheritanceError> {
    validate_periods(exponent)?;

    let mut result = YIELD_SCALE;
    let mut accumulator = base;
    let mut remaining = exponent;

    if remaining == 0 {
        return Ok(result);
    }

    loop {
        if remaining & 1 == 1 {
            result = mul_factor(result, accumulator)?;
        }
        remaining >>= 1;
        if remaining == 0 {
            break;
        }
        accumulator = mul_factor(accumulator, accumulator)?;
    }

    Ok(result)
}

/// Compounds `principal` forward over `periods` periods at a per-period rate.
///
/// Returns the *total* balance after compounding, not the interest earned —
/// see [`compound_interest`] for the delta.
pub fn compound_amount(
    principal: u64,
    rate_bps_per_period: u32,
    periods: u64,
) -> Result<u64, InheritanceError> {
    validate_yield_rate(rate_bps_per_period)?;
    validate_periods(periods)?;

    if principal == 0 || rate_bps_per_period == 0 || periods == 0 {
        return Ok(principal);
    }

    let per_period_increment = safe_div_u128(
        safe_mul_u128(YIELD_SCALE, rate_bps_per_period as u128)?,
        BPS_DENOMINATOR as u128,
    )?;
    let per_period_factor = YIELD_SCALE
        .checked_add(per_period_increment)
        .ok_or(InheritanceError::InvalidTotalAmount)?;

    let growth = pow_factor(per_period_factor, periods)?;
    to_u64(safe_div_u128(
        safe_mul_u128(principal as u128, growth)?,
        YIELD_SCALE,
    )?)
}

/// Interest earned by compounding `principal` over `periods` periods —
/// [`compound_amount`] minus the original principal.
pub fn compound_interest(
    principal: u64,
    rate_bps_per_period: u32,
    periods: u64,
) -> Result<u64, InheritanceError> {
    let total = compound_amount(principal, rate_bps_per_period, periods)?;
    safe_sub(total, principal)
}

/// Projects the balance of a position after `days` of daily compounding at an
/// annual rate, keeping sub-basis-point daily precision.
pub fn project_daily_compound(
    principal: u64,
    annual_rate_bps: u32,
    days: u64,
) -> Result<u64, InheritanceError> {
    validate_yield_rate(annual_rate_bps)?;
    validate_periods(days)?;

    if principal == 0 || annual_rate_bps == 0 || days == 0 {
        return Ok(principal);
    }

    let growth = pow_factor(daily_growth_factor(annual_rate_bps)?, days)?;
    to_u64(safe_div_u128(
        safe_mul_u128(principal as u128, growth)?,
        YIELD_SCALE,
    )?)
}

/// Interest a position would earn over `days` of daily compounding.
pub fn project_daily_interest(
    principal: u64,
    annual_rate_bps: u32,
    days: u64,
) -> Result<u64, InheritanceError> {
    let total = project_daily_compound(principal, annual_rate_bps, days)?;
    safe_sub(total, principal)
}

/// The effective APY, in basis points, of a nominal annual rate compounded
/// daily. Always at or above the nominal rate.
pub fn effective_apy_bps(nominal_annual_rate_bps: u32) -> Result<u32, InheritanceError> {
    validate_yield_rate(nominal_annual_rate_bps)?;
    if nominal_annual_rate_bps == 0 {
        return Ok(0);
    }

    let growth = pow_factor(daily_growth_factor(nominal_annual_rate_bps)?, DAYS_PER_YEAR)?;
    let gain = growth
        .checked_sub(YIELD_SCALE)
        .ok_or(InheritanceError::MathOverflow)?;
    let bps = safe_div_u128(safe_mul_u128(gain, BPS_DENOMINATOR as u128)?, YIELD_SCALE)?;

    u32::try_from(bps).map_err(|_| InheritanceError::MathOverflow)
}

// ─────────────────────────────────────────────────
// Fee splitting
// ─────────────────────────────────────────────────

/// Splits a harvested amount into `(net_to_plan, protocol_fee)`.
///
/// The fee is computed first and the net is the remainder, so the two halves
/// always sum back to exactly `amount` — rounding never creates or loses a
/// stroop.
pub fn split_performance_fee(amount: u64, fee_bps: u32) -> Result<(u64, u64), InheritanceError> {
    validate_performance_fee(fee_bps)?;
    if amount == 0 || fee_bps == 0 {
        return Ok((amount, 0));
    }

    let fee = apply_bps(amount, fee_bps)?;
    let net = safe_sub(amount, fee)?;
    Ok((net, fee))
}

/// Weighted average of two rates by their principal, in basis points.
///
/// Used when a plan's yield-bearing balance is split across positions and the
/// vault needs one headline rate to report.
pub fn blended_rate_bps(
    principal_a: u64,
    rate_a_bps: u32,
    principal_b: u64,
    rate_b_bps: u32,
) -> Result<u32, InheritanceError> {
    validate_yield_rate(rate_a_bps)?;
    validate_yield_rate(rate_b_bps)?;

    let total = safe_add(principal_a, principal_b)?;
    if total == 0 {
        return Ok(0);
    }

    let weighted = safe_mul_u128(principal_a as u128, rate_a_bps as u128)?
        .checked_add(safe_mul_u128(principal_b as u128, rate_b_bps as u128)?)
        .ok_or(InheritanceError::MathOverflow)?;

    let blended = safe_div_u128(weighted, total as u128)?;
    u32::try_from(blended).map_err(|_| InheritanceError::MathOverflow)
}

/// Whether a harvest is due: the cooldown has elapsed and the pending amount
/// clears the configured floor.
///
/// Both conditions must hold. A zero interval or zero minimum disables that
/// half of the check.
pub fn is_harvest_due(
    now: u64,
    last_harvest_at: u64,
    interval_secs: u64,
    pending: u64,
    minimum: u64,
) -> bool {
    let cooldown_elapsed =
        interval_secs == 0 || now >= last_harvest_at.saturating_add(interval_secs);
    let clears_floor = pending >= minimum;
    cooldown_elapsed && clears_floor && pending > 0
}

/// The earliest timestamp at which the next harvest becomes eligible.
pub fn next_harvest_at(last_harvest_at: u64, interval_secs: u64) -> u64 {
    last_harvest_at.saturating_add(interval_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Checked primitives ──────────────────────

    #[test]
    fn safe_add_rejects_overflow() {
        assert_eq!(safe_add(1, 2), Ok(3));
        assert_eq!(safe_add(u64::MAX, 0), Ok(u64::MAX));
        assert_eq!(
            safe_add(u64::MAX, 1),
            Err(InheritanceError::InvalidTotalAmount)
        );
    }

    #[test]
    fn safe_sub_rejects_underflow() {
        assert_eq!(safe_sub(5, 3), Ok(2));
        assert_eq!(safe_sub(5, 5), Ok(0));
        assert_eq!(safe_sub(3, 5), Err(InheritanceError::InvalidTotalAmount));
    }

    #[test]
    fn safe_mul_rejects_overflow() {
        assert_eq!(safe_mul(1_000, 1_000), Ok(1_000_000));
        assert_eq!(safe_mul(u64::MAX, 1), Ok(u64::MAX));
        assert_eq!(
            safe_mul(u64::MAX, 2),
            Err(InheritanceError::InvalidTotalAmount)
        );
    }

    #[test]
    fn safe_div_rejects_zero_divisor() {
        assert_eq!(safe_div(10, 2), Ok(5));
        assert_eq!(safe_div(7, 2), Ok(3)); // rounds toward zero
        assert_eq!(
            safe_div(1, 0),
            Err(InheritanceError::InvalidBeneficiaryData)
        );
    }

    #[test]
    fn to_u64_rejects_values_above_range() {
        assert_eq!(to_u64(42), Ok(42));
        assert_eq!(to_u64(u64::MAX as u128), Ok(u64::MAX));
        assert_eq!(
            to_u64(u64::MAX as u128 + 1),
            Err(InheritanceError::InvalidTotalAmount)
        );
    }

    #[test]
    fn mul_div_survives_intermediate_overflow() {
        // u64::MAX * 2 would overflow u64, but the u128 intermediate holds it
        // and the divide brings the result back into range.
        assert_eq!(mul_div(u64::MAX, 2, 2), Ok(u64::MAX));
        assert_eq!(mul_div(1_000_000, 5_000, 10_000), Ok(500_000));
        assert_eq!(
            mul_div(1, 1, 0),
            Err(InheritanceError::InvalidBeneficiaryData)
        );
    }

    #[test]
    fn mul_div_result_beyond_u64_errors() {
        assert_eq!(
            mul_div(u64::MAX, 4, 2),
            Err(InheritanceError::InvalidTotalAmount)
        );
    }

    #[test]
    fn apply_bps_computes_percentages() {
        assert_eq!(apply_bps(1_000_000, 10_000), Ok(1_000_000)); // 100%
        assert_eq!(apply_bps(1_000_000, 5_000), Ok(500_000)); // 50%
        assert_eq!(apply_bps(1_000_000, 100), Ok(10_000)); // 1%
        assert_eq!(apply_bps(1_000_000, 1), Ok(100)); // 0.01%
        assert_eq!(apply_bps(1_000_000, 0), Ok(0));
        assert_eq!(apply_bps(0, 5_000), Ok(0));
    }

    #[test]
    fn apply_bps_rounds_toward_zero() {
        // 99 * 1bps = 0.0099, floors to 0 rather than rounding up.
        assert_eq!(apply_bps(99, 1), Ok(0));
        assert_eq!(apply_bps(10_001, 1), Ok(1));
    }

    // ─── Validation ──────────────────────────────

    #[test]
    fn yield_rate_validation_bounds() {
        assert_eq!(validate_yield_rate(0), Ok(()));
        assert_eq!(validate_yield_rate(MAX_YIELD_RATE_BPS), Ok(()));
        assert_eq!(
            validate_yield_rate(MAX_YIELD_RATE_BPS + 1),
            Err(InheritanceError::InvalidAllocation)
        );
    }

    #[test]
    fn performance_fee_validation_bounds() {
        assert_eq!(validate_performance_fee(0), Ok(()));
        assert_eq!(validate_performance_fee(MAX_PERFORMANCE_FEE_BPS), Ok(()));
        assert_eq!(
            validate_performance_fee(MAX_PERFORMANCE_FEE_BPS + 1),
            Err(InheritanceError::InvalidAllocation)
        );
    }

    #[test]
    fn period_validation_bounds() {
        assert_eq!(validate_periods(0), Ok(()));
        assert_eq!(validate_periods(MAX_COMPOUND_PERIODS), Ok(()));
        assert_eq!(
            validate_periods(MAX_COMPOUND_PERIODS + 1),
            Err(InheritanceError::InvalidAllocation)
        );
    }

    // ─── Simple interest ─────────────────────────

    #[test]
    fn simple_interest_over_one_year() {
        // 5% of 1_000_000 over a full year.
        assert_eq!(
            simple_interest(1_000_000, 500, SECONDS_PER_YEAR),
            Ok(50_000)
        );
    }

    #[test]
    fn simple_interest_is_pro_rata() {
        let full_year = simple_interest(1_000_000, 500, SECONDS_PER_YEAR).unwrap();
        let half_year = simple_interest(1_000_000, 500, SECONDS_PER_YEAR / 2).unwrap();
        assert_eq!(half_year, full_year / 2);
    }

    #[test]
    fn simple_interest_zero_cases() {
        assert_eq!(simple_interest(0, 500, SECONDS_PER_YEAR), Ok(0));
        assert_eq!(simple_interest(1_000_000, 0, SECONDS_PER_YEAR), Ok(0));
        assert_eq!(simple_interest(1_000_000, 500, 0), Ok(0));
    }

    #[test]
    fn simple_interest_rejects_absurd_rate() {
        assert_eq!(
            simple_interest(1_000_000, MAX_YIELD_RATE_BPS + 1, SECONDS_PER_YEAR),
            Err(InheritanceError::InvalidAllocation)
        );
    }

    #[test]
    fn simple_interest_handles_large_principal() {
        // A principal near u64::MAX must not overflow the intermediate.
        let result = simple_interest(u64::MAX / 2, 100, SECONDS_PER_YEAR);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (u64::MAX / 2) / 100);
    }

    #[test]
    fn daily_rate_floors_small_annual_rates() {
        assert_eq!(daily_rate_bps(365), Ok(1));
        assert_eq!(daily_rate_bps(364), Ok(0));
        assert_eq!(daily_rate_bps(3_650), Ok(10));
    }

    #[test]
    fn periods_elapsed_counts_whole_periods() {
        assert_eq!(periods_elapsed(SECONDS_PER_DAY * 3, SECONDS_PER_DAY), Ok(3));
        assert_eq!(periods_elapsed(SECONDS_PER_DAY - 1, SECONDS_PER_DAY), Ok(0));
        assert_eq!(
            periods_elapsed(100, 0),
            Err(InheritanceError::InvalidBeneficiaryData)
        );
    }

    // ─── Compounding ─────────────────────────────

    #[test]
    fn pow_factor_identity_cases() {
        assert_eq!(pow_factor(YIELD_SCALE, 0), Ok(YIELD_SCALE));
        assert_eq!(pow_factor(YIELD_SCALE, 1), Ok(YIELD_SCALE));
        assert_eq!(pow_factor(YIELD_SCALE, 1_000), Ok(YIELD_SCALE));
    }

    #[test]
    fn pow_factor_squares_correctly() {
        let two = YIELD_SCALE * 2;
        assert_eq!(pow_factor(two, 2), Ok(YIELD_SCALE * 4));
        assert_eq!(pow_factor(two, 3), Ok(YIELD_SCALE * 8));
        assert_eq!(pow_factor(two, 10), Ok(YIELD_SCALE * 1024));
    }

    #[test]
    fn pow_factor_rejects_excessive_exponent() {
        assert_eq!(
            pow_factor(YIELD_SCALE, MAX_COMPOUND_PERIODS + 1),
            Err(InheritanceError::InvalidAllocation)
        );
    }

    #[test]
    fn pow_factor_rejects_runaway_growth() {
        // 2^36500 cannot fit in u128 — this must error, not wrap.
        assert_eq!(
            pow_factor(YIELD_SCALE * 2, MAX_COMPOUND_PERIODS),
            Err(InheritanceError::InvalidTotalAmount)
        );
    }

    #[test]
    fn compound_amount_beats_simple_interest() {
        // 10% compounded over 10 periods > 10% simple over 10 periods.
        let compounded = compound_amount(1_000_000, 1_000, 10).unwrap();
        assert!(compounded > 2_000_000, "got {}", compounded);
        // (1.1)^10 ≈ 2.5937
        assert_eq!(compounded, 2_593_742);
    }

    #[test]
    fn compound_amount_zero_cases_return_principal() {
        assert_eq!(compound_amount(1_000_000, 500, 0), Ok(1_000_000));
        assert_eq!(compound_amount(1_000_000, 0, 10), Ok(1_000_000));
        assert_eq!(compound_amount(0, 500, 10), Ok(0));
    }

    #[test]
    fn compound_interest_is_total_minus_principal() {
        let principal = 1_000_000;
        let total = compound_amount(principal, 1_000, 10).unwrap();
        let interest = compound_interest(principal, 1_000, 10).unwrap();
        assert_eq!(interest, total - principal);
    }

    #[test]
    fn compound_amount_rejects_bad_rate() {
        assert_eq!(
            compound_amount(1_000, MAX_YIELD_RATE_BPS + 1, 5),
            Err(InheritanceError::InvalidAllocation)
        );
    }

    #[test]
    fn daily_growth_factor_is_just_above_one() {
        let factor = daily_growth_factor(365).unwrap();
        assert!(factor > YIELD_SCALE);
        // 365bps / 365 days = 1bps per day = 1.0001
        assert_eq!(factor, YIELD_SCALE + YIELD_SCALE / 10_000);
    }

    #[test]
    fn daily_growth_factor_of_zero_rate_is_unity() {
        assert_eq!(daily_growth_factor(0), Ok(YIELD_SCALE));
    }

    #[test]
    fn projection_grows_monotonically_with_time() {
        let thirty = project_daily_compound(1_000_000, 500, 30).unwrap();
        let sixty = project_daily_compound(1_000_000, 500, 60).unwrap();
        let year = project_daily_compound(1_000_000, 500, 365).unwrap();

        assert!(thirty > 1_000_000);
        assert!(sixty > thirty);
        assert!(year > sixty);
    }

    #[test]
    fn projection_over_a_year_approximates_nominal_rate() {
        // 5% compounded daily lands slightly above 5% simple.
        let projected = project_daily_compound(1_000_000, 500, 365).unwrap();
        assert!(projected > 1_050_000, "got {}", projected);
        assert!(projected < 1_052_000, "got {}", projected);
    }

    #[test]
    fn projection_zero_cases_return_principal() {
        assert_eq!(project_daily_compound(1_000_000, 500, 0), Ok(1_000_000));
        assert_eq!(project_daily_compound(1_000_000, 0, 365), Ok(1_000_000));
        assert_eq!(project_daily_compound(0, 500, 365), Ok(0));
    }

    #[test]
    fn projected_interest_excludes_principal() {
        let total = project_daily_compound(1_000_000, 500, 365).unwrap();
        let interest = project_daily_interest(1_000_000, 500, 365).unwrap();
        assert_eq!(interest, total - 1_000_000);
    }

    #[test]
    fn effective_apy_exceeds_nominal_rate() {
        assert!(effective_apy_bps(500).unwrap() > 500);
        assert!(effective_apy_bps(1_000).unwrap() > 1_000);
        assert_eq!(effective_apy_bps(0), Ok(0));
    }

    #[test]
    fn effective_apy_of_five_percent_is_about_five_point_one() {
        let apy = effective_apy_bps(500).unwrap();
        assert!((512..=513).contains(&apy), "got {}", apy);
    }

    // ─── Fee splitting ───────────────────────────

    #[test]
    fn fee_split_halves_sum_to_the_whole() {
        for amount in [0u64, 1, 7, 99, 1_000, 1_000_000, u64::MAX / 2] {
            for fee_bps in [0u32, 1, 250, 1_000, 5_000] {
                let (net, fee) = split_performance_fee(amount, fee_bps).unwrap();
                assert_eq!(
                    net + fee,
                    amount,
                    "amount {} at {}bps split to {}+{}",
                    amount,
                    fee_bps,
                    net,
                    fee
                );
            }
        }
    }

    #[test]
    fn fee_split_takes_the_configured_cut() {
        assert_eq!(
            split_performance_fee(1_000_000, 1_000),
            Ok((900_000, 100_000))
        );
        assert_eq!(split_performance_fee(1_000_000, 0), Ok((1_000_000, 0)));
        assert_eq!(split_performance_fee(0, 1_000), Ok((0, 0)));
    }

    #[test]
    fn fee_split_rejects_excessive_fee() {
        assert_eq!(
            split_performance_fee(1_000, MAX_PERFORMANCE_FEE_BPS + 1),
            Err(InheritanceError::InvalidAllocation)
        );
    }

    #[test]
    fn fee_split_of_dust_favours_the_plan() {
        // 1 stroop at 10% rounds the fee to 0, so the plan keeps it.
        assert_eq!(split_performance_fee(1, 1_000), Ok((1, 0)));
    }

    // ─── Blended rates ───────────────────────────

    #[test]
    fn blended_rate_weights_by_principal() {
        // Equal principals at 400 and 600 bps average to 500.
        assert_eq!(blended_rate_bps(1_000, 400, 1_000, 600), Ok(500));
        // Three-to-one weighting pulls toward the larger position.
        assert_eq!(blended_rate_bps(3_000, 400, 1_000, 800), Ok(500));
    }

    #[test]
    fn blended_rate_of_empty_positions_is_zero() {
        assert_eq!(blended_rate_bps(0, 500, 0, 900), Ok(0));
    }

    #[test]
    fn blended_rate_with_one_empty_side_returns_the_other() {
        assert_eq!(blended_rate_bps(1_000, 700, 0, 100), Ok(700));
        assert_eq!(blended_rate_bps(0, 700, 1_000, 100), Ok(100));
    }

    #[test]
    fn blended_rate_rejects_bad_inputs() {
        assert_eq!(
            blended_rate_bps(1_000, MAX_YIELD_RATE_BPS + 1, 1_000, 500),
            Err(InheritanceError::InvalidAllocation)
        );
    }

    // ─── Harvest scheduling ──────────────────────

    #[test]
    fn harvest_is_due_once_cooldown_elapses() {
        // 100s cooldown, last harvest at t=1000.
        assert!(!is_harvest_due(1_050, 1_000, 100, 500, 0));
        assert!(is_harvest_due(1_100, 1_000, 100, 500, 0));
        assert!(is_harvest_due(1_500, 1_000, 100, 500, 0));
    }

    #[test]
    fn harvest_is_blocked_below_the_minimum() {
        assert!(!is_harvest_due(2_000, 1_000, 0, 99, 100));
        assert!(is_harvest_due(2_000, 1_000, 0, 100, 100));
    }

    #[test]
    fn harvest_needs_something_to_claim() {
        assert!(!is_harvest_due(2_000, 1_000, 0, 0, 0));
    }

    #[test]
    fn zero_interval_disables_the_cooldown() {
        assert!(is_harvest_due(1_000, 1_000, 0, 1, 0));
    }

    #[test]
    fn next_harvest_saturates_rather_than_wrapping() {
        assert_eq!(next_harvest_at(1_000, 500), 1_500);
        assert_eq!(next_harvest_at(u64::MAX, 500), u64::MAX);
    }
}

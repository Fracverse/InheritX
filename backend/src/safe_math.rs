use crate::api_error::ApiError;
use rust_decimal::Decimal;

/// Safe arithmetic operations for Decimal types that prevent overflow/underflow
pub struct SafeMath;

impl SafeMath {
    /// Safely add two Decimal values, returning an error on overflow
    pub fn add(a: Decimal, b: Decimal) -> Result<Decimal, ApiError> {
        a.checked_add(b).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "Arithmetic overflow: {} + {} exceeds maximum value",
                a, b
            ))
        })
    }

    /// Safely subtract two Decimal values, returning an error on underflow
    pub fn sub(a: Decimal, b: Decimal) -> Result<Decimal, ApiError> {
        let result = a.checked_sub(b).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "Arithmetic underflow: {} - {} results in overflow",
                a, b
            ))
        })?;

        // Also check if result is negative (which we consider underflow for financial operations)
        if result.is_sign_negative() {
            return Err(ApiError::BadRequest(format!(
                "Arithmetic underflow: {} - {} results in negative value {}",
                a, b, result
            )));
        }

        Ok(result)
    }

    /// Safely multiply two Decimal values, returning an error on overflow
    pub fn mul(a: Decimal, b: Decimal) -> Result<Decimal, ApiError> {
        a.checked_mul(b).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "Arithmetic overflow: {} * {} exceeds maximum value",
                a, b
            ))
        })
    }

    /// Safely divide two Decimal values, returning an error on division by zero or overflow
    pub fn div(a: Decimal, b: Decimal) -> Result<Decimal, ApiError> {
        if b.is_zero() {
            return Err(ApiError::BadRequest(
                "Division by zero is not allowed".to_string(),
            ));
        }

        a.checked_div(b).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "Arithmetic overflow: {} / {} exceeds maximum value",
                a, b
            ))
        })
    }

    /// Calculate percentage safely: (value * percentage) / 100
    /// percentage should be a whole number (e.g., 2 for 2%)
    pub fn percentage(value: Decimal, percentage: Decimal) -> Result<Decimal, ApiError> {
        let hundred = Decimal::new(100, 0);
        let product = Self::mul(value, percentage)?;
        Self::div(product, hundred)
    }

    /// Calculate fee as a percentage of the amount
    /// Returns (fee, net_amount) where net_amount = amount - fee
    pub fn calculate_fee(
        amount: Decimal,
        fee_percentage: Decimal,
    ) -> Result<(Decimal, Decimal), ApiError> {
        let fee = Self::percentage(amount, fee_percentage)?;
        let net_amount = Self::sub(amount, fee)?;
        Ok((fee, net_amount))
    }

    /// Ensure a value is non-negative
    pub fn ensure_non_negative(value: Decimal, field_name: &str) -> Result<Decimal, ApiError> {
        if value.is_sign_negative() {
            return Err(ApiError::BadRequest(format!(
                "{} cannot be negative: {}",
                field_name, value
            )));
        }
        Ok(value)
    }

    /// Ensure a value is positive (greater than zero)
    pub fn ensure_positive(value: Decimal, field_name: &str) -> Result<Decimal, ApiError> {
        if value.is_zero() || value.is_sign_negative() {
            return Err(ApiError::BadRequest(format!(
                "{} must be positive: {}",
                field_name, value
            )));
        }
        Ok(value)
    }

    /// Safely calculate collateral ratio: (collateral_value / debt_value) * 100
    pub fn collateral_ratio(
        collateral_value: Decimal,
        debt_value: Decimal,
    ) -> Result<Decimal, ApiError> {
        if debt_value.is_zero() {
            return Err(ApiError::BadRequest(
                "Cannot calculate collateral ratio with zero debt".to_string(),
            ));
        }

        let ratio = Self::div(collateral_value, debt_value)?;
        Self::mul(ratio, Decimal::new(100, 0))
    }

    /// Safely calculate loan-to-value ratio: (loan_amount / collateral_value) * 100
    pub fn loan_to_value(
        loan_amount: Decimal,
        collateral_value: Decimal,
    ) -> Result<Decimal, ApiError> {
        if collateral_value.is_zero() {
            return Err(ApiError::BadRequest(
                "Cannot calculate LTV with zero collateral".to_string(),
            ));
        }

        let ratio = Self::div(loan_amount, collateral_value)?;
        Self::mul(ratio, Decimal::new(100, 0))
    }

    /// Calculate interest: principal * rate * time
    /// rate should be annual rate as decimal (e.g., 0.05 for 5%)
    /// time should be in years (e.g., 0.5 for 6 months)
    pub fn calculate_interest(
        principal: Decimal,
        annual_rate: Decimal,
        time_in_years: Decimal,
    ) -> Result<Decimal, ApiError> {
        let rate_times_time = Self::mul(annual_rate, time_in_years)?;
        Self::mul(principal, rate_times_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_safe_add() {
        assert_eq!(SafeMath::add(dec!(100), dec!(50)).unwrap(), dec!(150));
        assert_eq!(SafeMath::add(dec!(0), dec!(0)).unwrap(), dec!(0));
        assert_eq!(
            SafeMath::add(dec!(999999999), dec!(1)).unwrap(),
            dec!(1000000000)
        );
    }

    #[test]
    fn test_safe_add_overflow() {
        let max = Decimal::MAX;
        let result = SafeMath::add(max, dec!(1));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Arithmetic overflow"));
    }

    #[test]
    fn test_safe_sub() {
        assert_eq!(SafeMath::sub(dec!(100), dec!(50)).unwrap(), dec!(50));
        assert_eq!(SafeMath::sub(dec!(100), dec!(100)).unwrap(), dec!(0));
    }

    #[test]
    fn test_safe_sub_underflow() {
        let result = SafeMath::sub(dec!(50), dec!(100));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Arithmetic underflow"));
    }

    #[test]
    fn test_safe_mul() {
        assert_eq!(SafeMath::mul(dec!(10), dec!(5)).unwrap(), dec!(50));
        assert_eq!(SafeMath::mul(dec!(0), dec!(100)).unwrap(), dec!(0));
        assert_eq!(SafeMath::mul(dec!(2.5), dec!(4)).unwrap(), dec!(10));
    }

    #[test]
    fn test_safe_mul_overflow() {
        let large = Decimal::MAX / dec!(2);
        let result = SafeMath::mul(large, dec!(3));
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_div() {
        assert_eq!(SafeMath::div(dec!(100), dec!(5)).unwrap(), dec!(20));
        assert_eq!(SafeMath::div(dec!(10), dec!(4)).unwrap(), dec!(2.5));
    }

    #[test]
    fn test_safe_div_by_zero() {
        let result = SafeMath::div(dec!(100), dec!(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }

    #[test]
    fn test_percentage() {
        assert_eq!(SafeMath::percentage(dec!(1000), dec!(2)).unwrap(), dec!(20));
        assert_eq!(SafeMath::percentage(dec!(500), dec!(10)).unwrap(), dec!(50));
        assert_eq!(
            SafeMath::percentage(dec!(1000), dec!(0.5)).unwrap(),
            dec!(5)
        );
    }

    #[test]
    fn test_calculate_fee() {
        let (fee, net) = SafeMath::calculate_fee(dec!(1000), dec!(2)).unwrap();
        assert_eq!(fee, dec!(20));
        assert_eq!(net, dec!(980));

        let (fee, net) = SafeMath::calculate_fee(dec!(500), dec!(5)).unwrap();
        assert_eq!(fee, dec!(25));
        assert_eq!(net, dec!(475));
    }

    #[test]
    fn test_ensure_non_negative() {
        assert_eq!(
            SafeMath::ensure_non_negative(dec!(100), "amount").unwrap(),
            dec!(100)
        );
        assert_eq!(
            SafeMath::ensure_non_negative(dec!(0), "amount").unwrap(),
            dec!(0)
        );

        let result = SafeMath::ensure_non_negative(dec!(-10), "amount");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot be negative"));
    }

    #[test]
    fn test_ensure_positive() {
        assert_eq!(
            SafeMath::ensure_positive(dec!(100), "amount").unwrap(),
            dec!(100)
        );

        let result = SafeMath::ensure_positive(dec!(0), "amount");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be positive"));

        let result = SafeMath::ensure_positive(dec!(-10), "amount");
        assert!(result.is_err());
    }

    #[test]
    fn test_collateral_ratio() {
        let ratio = SafeMath::collateral_ratio(dec!(1500), dec!(1000)).unwrap();
        assert_eq!(ratio, dec!(150));

        let ratio = SafeMath::collateral_ratio(dec!(2000), dec!(1000)).unwrap();
        assert_eq!(ratio, dec!(200));
    }

    #[test]
    fn test_collateral_ratio_zero_debt() {
        let result = SafeMath::collateral_ratio(dec!(1000), dec!(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("zero debt"));
    }

    #[test]
    fn test_loan_to_value() {
        let ltv = SafeMath::loan_to_value(dec!(750), dec!(1000)).unwrap();
        assert_eq!(ltv, dec!(75));

        let ltv = SafeMath::loan_to_value(dec!(500), dec!(1000)).unwrap();
        assert_eq!(ltv, dec!(50));
    }

    #[test]
    fn test_loan_to_value_zero_collateral() {
        let result = SafeMath::loan_to_value(dec!(1000), dec!(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("zero collateral"));
    }

    #[test]
    fn test_calculate_interest() {
        // 5% annual rate for 1 year on $1000
        let interest = SafeMath::calculate_interest(dec!(1000), dec!(0.05), dec!(1)).unwrap();
        assert_eq!(interest, dec!(50));

        // 5% annual rate for 6 months on $1000
        let interest = SafeMath::calculate_interest(dec!(1000), dec!(0.05), dec!(0.5)).unwrap();
        assert_eq!(interest, dec!(25));

        // 10% annual rate for 2 years on $500
        let interest = SafeMath::calculate_interest(dec!(500), dec!(0.10), dec!(2)).unwrap();
        assert_eq!(interest, dec!(100));
    }
}

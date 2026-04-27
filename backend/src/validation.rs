/// Input validation utilities for API request bodies.
///
/// Provides field-level validation with structured error responses.
/// All POST/PUT endpoints should call the relevant `validate_*` function
/// before processing the request body.
use crate::api_error::ApiError;
use regex::Regex;
use std::collections::HashMap;

/// Collects field-level validation errors.
#[derive(Debug, Default)]
pub struct ValidationErrors {
    pub fields: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, field: &str, message: &str) {
        self.fields
            .entry(field.to_string())
            .or_default()
            .push(message.to_string());
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Converts to an `ApiError::BadRequest` with JSON-serialised field details.
    pub fn into_api_error(self) -> ApiError {
        let detail =
            serde_json::to_string(&self.fields).unwrap_or_else(|_| "Validation failed".to_string());
        ApiError::BadRequest(detail)
    }
}

// ── Sanitisation ──────────────────────────────────────────────────────────────

/// Strips leading/trailing whitespace and removes common SQL injection patterns.
pub fn sanitize_string(input: &str) -> String {
    // Remove null bytes and common SQL injection tokens
    let dangerous = Regex::new(r"(?i)(\x00|--|;|/\*|\*/|xp_|UNION\s+SELECT|DROP\s+TABLE|INSERT\s+INTO|DELETE\s+FROM|UPDATE\s+\w+\s+SET)")
        .expect("static regex");
    dangerous.replace_all(input.trim(), "").to_string()
}

// ── Field validators ──────────────────────────────────────────────────────────

pub fn validate_non_empty(errors: &mut ValidationErrors, field: &str, value: &str) {
    if value.trim().is_empty() {
        errors.add(field, "must not be empty");
    }
}

pub fn validate_max_length(errors: &mut ValidationErrors, field: &str, value: &str, max: usize) {
    if value.len() > max {
        errors.add(field, &format!("must not exceed {max} characters"));
    }
}

pub fn validate_min_length(errors: &mut ValidationErrors, field: &str, value: &str, min: usize) {
    if value.len() < min {
        errors.add(field, &format!("must be at least {min} characters"));
    }
}

pub fn validate_email(errors: &mut ValidationErrors, field: &str, value: &str) {
    let re = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("static regex");
    if !re.is_match(value) {
        errors.add(field, "must be a valid email address");
    }
}

pub fn validate_uuid(errors: &mut ValidationErrors, field: &str, value: &str) {
    if uuid::Uuid::parse_str(value).is_err() {
        errors.add(field, "must be a valid UUID");
    }
}

pub fn validate_positive_decimal(
    errors: &mut ValidationErrors,
    field: &str,
    value: rust_decimal::Decimal,
) {
    if value <= rust_decimal::Decimal::ZERO {
        errors.add(field, "must be greater than zero");
    }
}

pub fn validate_percentage(
    errors: &mut ValidationErrors,
    field: &str,
    value: rust_decimal::Decimal,
) {
    if value < rust_decimal::Decimal::ZERO || value > rust_decimal::Decimal::ONE_HUNDRED {
        errors.add(field, "must be between 0 and 100");
    }
}

/// Validates that a string does not contain SQL injection patterns.
pub fn validate_no_injection(errors: &mut ValidationErrors, field: &str, value: &str) {
    let sanitized = sanitize_string(value);
    if sanitized != value.trim() {
        errors.add(field, "contains invalid characters or patterns");
    }
}

// ── Convenience macro ─────────────────────────────────────────────────────────

/// Returns an `ApiError::BadRequest` if `$errors` is non-empty.
#[macro_export]
macro_rules! bail_if_invalid {
    ($errors:expr) => {
        if !$errors.is_empty() {
            return Err($errors.into_api_error());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_strips_sql_injection() {
        let input = "hello'; DROP TABLE users; --";
        let result = sanitize_string(input);
        assert!(!result.contains("DROP TABLE"));
        assert!(!result.contains("--"));
    }

    #[test]
    fn test_validate_email_valid() {
        let mut errors = ValidationErrors::new();
        validate_email(&mut errors, "email", "user@example.com");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_email_invalid() {
        let mut errors = ValidationErrors::new();
        validate_email(&mut errors, "email", "not-an-email");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validate_non_empty() {
        let mut errors = ValidationErrors::new();
        validate_non_empty(&mut errors, "name", "  ");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validate_no_injection_clean() {
        let mut errors = ValidationErrors::new();
        validate_no_injection(&mut errors, "field", "normal input");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_no_injection_dirty() {
        let mut errors = ValidationErrors::new();
        validate_no_injection(&mut errors, "field", "value; DROP TABLE users");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_into_api_error_is_bad_request() {
        let mut errors = ValidationErrors::new();
        errors.add("field", "required");
        let err = errors.into_api_error();
        assert!(matches!(err, crate::api_error::ApiError::BadRequest(_)));
    }
}

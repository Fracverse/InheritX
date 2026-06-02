/// Input validation utilities for API request bodies.
///
/// Provides field-level validation with structured error responses.
/// All POST/PUT endpoints should call the relevant `validate_*` function
/// before processing the request body.
use crate::api_error::ApiError;
use regex::Regex;
use std::collections::HashMap;
use serde_json::Value as JsonValue;
use once_cell::sync::Lazy;

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
    static EMAIL_RE: Lazy<Regex> = Lazy::new(|| {
        // This pattern is a pragmatic RFC 5322 compatible (local@domain) validator.
        // It supports quoted local-parts, dots, and IPv4/IPv6 literals in domains.
        Regex::new(r"(?xi)^(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|\"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x00-\x7f])*\")@(?:(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4]\d|[01]?\d?\d)(?:\.(?:25[0-5]|2[0-4]\d|[01]?\d?\d)){3}|[a-f0-9:\.]+)\])$").expect("email regex")
    });

    let s = value.trim();
    // Per RFCs, the maximum total length for an email address is 254 characters.
    if s.len() == 0 || s.len() > 254 {
        errors.add(field, "must be a valid email address");
        return;
    }

    if !EMAIL_RE.is_match(s) {
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

// ── Length constants and JSON validators ───────────────────────────────────

/// Default maximum length for individual string fields (characters).
pub const DEFAULT_MAX_FIELD_LENGTH: usize = 1024;

/// Maximum allowed request body size (bytes) — used by middleware checks.
pub const DEFAULT_MAX_BODY_BYTES: usize = 16 * 1024; // 16 KiB

/// Recursively validate that no string in the provided JSON value exceeds `max`.
///
/// `path` is the JSON path used for error messages (e.g. `$.user.name`).
pub fn validate_json_string_lengths(
    errors: &mut ValidationErrors,
    value: &JsonValue,
    path: &str,
    max: usize,
) {
    match value {
        JsonValue::String(s) => {
            if s.len() > max {
                errors.add(path, &format!("must not exceed {max} characters"));
            }
        }
        JsonValue::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let child_path = format!("{}[{}]", path, i);
                validate_json_string_lengths(errors, v, &child_path, max);
            }
        }
        JsonValue::Object(map) => {
            for (k, v) in map.iter() {
                let child_path = if path == "$" {
                    format!("$.{}", k)
                } else {
                    format!("{}.{}", path, k)
                };
                validate_json_string_lengths(errors, v, &child_path, max);
            }
        }
        _ => {}
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

// ── Custom Path Extractor for Input Validation ───────────────────────────────

/// A wrapper around `axum::extract::Path` that converts deserialisation rejections
/// into structured `ApiError::BadRequest` responses.
#[derive(Debug)]
pub struct Path<T>(pub T);

#[axum::async_trait]
impl<S, T> axum::extract::FromRequestParts<S> for Path<T>
where
    T: serde::de::DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(axum::extract::Path(value)) => Ok(Path(value)),
            Err(err) => {
                Err(ApiError::BadRequest(format!("Invalid path parameter: {}", err)))
            }
        }
    }
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

    #[test]
    fn test_validate_email_various_valid() {
        let valids = [
            "simple@example.com",
            "very.common@example.com",
            "disposable.style.email.with+symbol@example.com",
            "other.email-with-hyphen@example.com",
            "fully-qualified-domain@example.com",
            "user.name+tag+sorting@example.com",
            "x@example.com",
            "example-indeed@strange-example.com",
            "\"much.more unusual\"@example.com",
            "\"very.unusual.@.unusual.com\"@example.com",
            "user@[192.168.2.1]",
            "user@[IPv6:2001:db8::1]",
        ];

        for a in valids.iter() {
            let mut errors = ValidationErrors::new();
            validate_email(&mut errors, "email", a);
            assert!(errors.is_empty(), "valid email rejected: {}", a);
        }
    }

    #[test]
    fn test_validate_email_various_invalid() {
        let invalids = [
            "Abc.example.com",
            "A@b@c@example.com",
            "a\"b(c)d,e:f;g<h>i[j\\k]l@example.com",
            "just\"not\"right@example.com",
            "this is\"not\\allowed@example.com",
            "this\\ still\\\"not\\\\allowed@example.com",
            "john..doe@example.com",
            ".john@example.com",
            "john.@example.com",
            "john@.example.com",
            "",
            "   ",
        ];

        for a in invalids.iter() {
            let mut errors = ValidationErrors::new();
            validate_email(&mut errors, "email", a);
            assert!(!errors.is_empty(), "invalid email accepted: {}", a);
        }
    }
}

/// Input validation utilities for API request bodies.
///
/// Provides field-level validation with structured error responses.
/// All POST/PUT endpoints should call the relevant `validate_*` function
/// before processing the request body.
use crate::api_error::ApiError;
use regex::Regex;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::OnceLock;

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
    sql_injection_pattern()
        .replace_all(input.trim(), "")
        .to_string()
}

fn sql_injection_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();

    PATTERN.get_or_init(|| {
        Regex::new(
            r"(?i)(\x00|--|/\*|\*/|;|\bunion\s+(?:all\s+)?select\b|\bdrop\s+table\b|\binsert\s+into\b|\bdelete\s+from\b|\bupdate\s+\w+\s+set\b|\bor\s+1\s*=\s*1\b|\band\s+1\s*=\s*1\b|\bexec(?:ute)?\b|\bxp_\w+\b|\bsleep\s*\(|\bbenchmark\s*\()",
        )
        .expect("static regex")
    })
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
    let s = value.trim();
    if s.is_empty() || s.len() > 254 {
        errors.add(field, "must be a valid email address");
        return;
    }

    let parts: Vec<&str> = s.rsplitn(2, '@').collect();
    if parts.len() != 2 {
        errors.add(field, "must be a valid email address");
        return;
    }

    let domain = parts[0];
    let local = parts[1];

    if local.is_empty() || domain.is_empty() {
        errors.add(field, "must be a valid email address");
        return;
    }

    if local.starts_with('.') || local.ends_with('.') || local.contains("..") {
        errors.add(field, "must be a valid email address");
        return;
    }

    if domain.starts_with('.') || domain.ends_with('.') || domain.contains("..") {
        errors.add(field, "must be a valid email address");
        return;
    }

    let valid_local = if local.starts_with('"') && local.ends_with('"') {
        validate_quoted_local_part(local)
    } else {
        is_valid_unquoted_local_part(local)
    };

    if !valid_local {
        errors.add(field, "must be a valid email address");
        return;
    }

    if !is_valid_domain(domain) {
        errors.add(field, "must be a valid email address");
    }
}

fn is_valid_unquoted_local_part(local: &str) -> bool {
    if local.is_empty() {
        return false;
    }

    local.as_bytes().iter().all(|&b| match b {
        b'a'..=b'z'
        | b'A'..=b'Z'
        | b'0'..=b'9'
        | b'!'
        | b'#'
        | b'$'
        | b'%'
        | b'&'
        | b'\''
        | b'*'
        | b'+'
        | b'-'
        | b'/'
        | b'='
        | b'?'
        | b'^'
        | b'_'
        | b'`'
        | b'{'
        | b'|'
        | b'}'
        | b'~'
        | b'.' => true,
        _ => false,
    })
}

fn is_valid_domain(domain: &str) -> bool {
    if domain.starts_with('[') && domain.ends_with(']') {
        let literal = &domain[1..domain.len() - 1];
        return is_valid_ipv4(literal) || is_valid_ipv6_literal(literal);
    }

    if domain.len() > 255 {
        return false;
    }

    for label in domain.split('.') {
        if label.is_empty() || label.len() > 63 {
            return false;
        }
        if label.starts_with('-') || label.ends_with('-') {
            return false;
        }
        if !label.as_bytes().iter().all(|&b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' => true,
            _ => false,
        }) {
            return false;
        }
    }

    true
}

fn is_valid_ipv4(literal: &str) -> bool {
    let octets: Vec<&str> = literal.split('.').collect();
    if octets.len() != 4 {
        return false;
    }
    for octet in octets {
        if octet.is_empty() || octet.len() > 3 {
            return false;
        }
        if octet.starts_with('0') && octet.len() > 1 {
            return false;
        }
        let value = octet.parse::<u8>();
        if value.is_err() {
            return false;
        }
    }
    true
}

fn is_valid_ipv6_literal(literal: &str) -> bool {
    literal
        .strip_prefix("IPv6:")
        .map(|v| {
            v.chars()
                .all(|ch| ch.is_ascii_hexdigit() || ch == ':' || ch == '.')
        })
        .unwrap_or(false)
}

fn validate_quoted_local_part(local: &str) -> bool {
    let inner = &local[1..local.len() - 1];
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if chars.next().is_none() {
                return false;
            }
            continue;
        }

        if ch == '"' || ch == '\r' || ch == '\n' || ch.is_control() {
            return false;
        }
    }

    true
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

impl<S, T> axum::extract::FromRequestParts<S> for Path<T>
where
    T: serde::de::DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            match axum::extract::Path::<T>::from_request_parts(parts, state).await {
                Ok(axum::extract::Path(value)) => Ok(Path(value)),
                Err(err) => Err(ApiError::BadRequest(format!(
                    "Invalid path parameter: {}",
                    err
                ))),
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
    fn test_sanitize_strips_common_injection_payloads() {
        let input = "' OR 1=1; UNION ALL SELECT * FROM secrets /*";
        let result = sanitize_string(input);
        assert!(!result.contains("OR 1=1"));
        assert!(!result.contains("UNION ALL SELECT"));
        assert!(!result.contains("/*"));
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

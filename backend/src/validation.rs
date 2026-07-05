use regex::Regex;
use std::collections::HashMap;
use serde_json::Value as JsonValue;

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

    pub fn into_error_body(self) -> serde_json::Value {
        serde_json::json!({ "error": "Validation failed", "details": self.fields })
    }
}

pub fn sanitize_string(input: &str) -> String {
    let dangerous = Regex::new(r"(?i)(\x00|--|;|/\*|\*/|xp_|UNION\s+SELECT|DROP\s+TABLE|INSERT\s+INTO|DELETE\s+FROM|UPDATE\s+\w+\s+SET)")
        .expect("static regex");
    dangerous.replace_all(input.trim(), "").to_string()
}

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

pub fn validate_no_injection(errors: &mut ValidationErrors, field: &str, value: &str) {
    let sanitized = sanitize_string(value);
    if sanitized != value.trim() {
        errors.add(field, "contains invalid characters or patterns");
    }
}

pub const DEFAULT_MAX_FIELD_LENGTH: usize = 1024;

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
    fn test_validate_uuid_valid() {
        let mut errors = ValidationErrors::new();
        validate_uuid(&mut errors, "id", "550e8400-e29b-41d4-a716-446655440000");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        let mut errors = ValidationErrors::new();
        validate_uuid(&mut errors, "id", "not-a-uuid");
        assert!(!errors.is_empty());
    }
}

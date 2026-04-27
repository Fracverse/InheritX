//! # Will Legal Compliance Validation
//!
//! Validates that generated wills meet basic jurisdiction-specific legal
//! requirements including witness counts, required fields, and formatting rules.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::will_pdf::{BeneficiaryEntry, WillDocumentInput};

// --- Jurisdiction Rules ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionRules {
    pub jurisdiction: String,
    pub min_witnesses: u32,
    pub require_notarization: bool,
    pub min_beneficiaries: u32,
    pub require_execution_rules: bool,
    pub min_testator_age: Option<u32>,
    pub require_relationship: bool,
}

fn us_rules() -> JurisdictionRules {
    JurisdictionRules {
        jurisdiction: "US".to_string(),
        min_witnesses: 2,
        require_notarization: false,
        min_beneficiaries: 1,
        require_execution_rules: false,
        min_testator_age: Some(18),
        require_relationship: false,
    }
}

fn uk_rules() -> JurisdictionRules {
    JurisdictionRules {
        jurisdiction: "UK".to_string(),
        min_witnesses: 2,
        require_notarization: false,
        min_beneficiaries: 1,
        require_execution_rules: false,
        min_testator_age: Some(18),
        require_relationship: true,
    }
}

fn eu_rules() -> JurisdictionRules {
    JurisdictionRules {
        jurisdiction: "EU".to_string(),
        min_witnesses: 2,
        require_notarization: true,
        min_beneficiaries: 1,
        require_execution_rules: false,
        min_testator_age: Some(18),
        require_relationship: false,
    }
}

fn global_rules() -> JurisdictionRules {
    JurisdictionRules {
        jurisdiction: "GLOBAL".to_string(),
        min_witnesses: 1,
        require_notarization: false,
        min_beneficiaries: 1,
        require_execution_rules: false,
        min_testator_age: None,
        require_relationship: false,
    }
}

// --- Validation Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub jurisdiction: String,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: String,
}

// --- Service ---

pub struct WillComplianceService;

impl WillComplianceService {
    pub fn get_jurisdiction_rules(jurisdiction: &str) -> JurisdictionRules {
        match jurisdiction.to_uppercase().as_str() {
            "US" => us_rules(),
            "UK" => uk_rules(),
            "EU" => eu_rules(),
            _ => global_rules(),
        }
    }

    pub fn list_supported_jurisdictions() -> Vec<String> {
        vec![
            "US".to_string(),
            "UK".to_string(),
            "EU".to_string(),
            "GLOBAL".to_string(),
        ]
    }

    pub fn validate(input: &WillDocumentInput, witness_count: u32) -> ValidationResult {
        let jurisdiction_key = input.jurisdiction.as_deref().unwrap_or("GLOBAL");
        let rules = Self::get_jurisdiction_rules(jurisdiction_key);
        let mut errors: Vec<ValidationError> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        // Owner name required
        if input.owner_name.trim().is_empty() {
            errors.push(ValidationError {
                field: "owner_name".to_string(),
                message: "Owner name is required".to_string(),
                severity: "error".to_string(),
            });
        }

        // Owner wallet required
        if input.owner_wallet.trim().is_empty() {
            errors.push(ValidationError {
                field: "owner_wallet".to_string(),
                message: "Owner wallet address is required".to_string(),
                severity: "error".to_string(),
            });
        }

        // Minimum beneficiaries
        if input.beneficiaries.len() < rules.min_beneficiaries as usize {
            errors.push(ValidationError {
                field: "beneficiaries".to_string(),
                message: format!(
                    "At least {} beneficiary required for {} jurisdiction",
                    rules.min_beneficiaries, rules.jurisdiction
                ),
                severity: "error".to_string(),
            });
        }

        // Validate each beneficiary
        validate_beneficiaries(&input.beneficiaries, &rules, &mut errors);

        // Allocation sum must equal 100%
        let total: Decimal = input
            .beneficiaries
            .iter()
            .map(|b| b.allocation_percent)
            .sum();
        if total != Decimal::new(100, 0) {
            errors.push(ValidationError {
                field: "beneficiaries.allocation_percent".to_string(),
                message: format!("Beneficiary allocations must sum to 100%, got {total}%"),
                severity: "error".to_string(),
            });
        }

        // Witness count
        if witness_count < rules.min_witnesses {
            errors.push(ValidationError {
                field: "witness_count".to_string(),
                message: format!(
                    "{} jurisdiction requires at least {} witnesses, got {}",
                    rules.jurisdiction, rules.min_witnesses, witness_count
                ),
                severity: "error".to_string(),
            });
        }

        // Execution rules required
        if rules.require_execution_rules
            && input
                .execution_rules
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
        {
            errors.push(ValidationError {
                field: "execution_rules".to_string(),
                message: format!(
                    "Execution rules are required for {} jurisdiction",
                    rules.jurisdiction
                ),
                severity: "error".to_string(),
            });
        }

        // Notarization warning
        if rules.require_notarization {
            warnings.push(format!(
                "{} jurisdiction requires notarization. Ensure the document is notarized before execution.",
                rules.jurisdiction
            ));
        }

        let is_valid = errors.is_empty();

        ValidationResult {
            is_valid,
            jurisdiction: rules.jurisdiction,
            errors,
            warnings,
        }
    }
}

fn validate_beneficiaries(
    beneficiaries: &[BeneficiaryEntry],
    rules: &JurisdictionRules,
    errors: &mut Vec<ValidationError>,
) {
    for (i, b) in beneficiaries.iter().enumerate() {
        let idx = i + 1;

        if b.name.trim().is_empty() {
            errors.push(ValidationError {
                field: format!("beneficiaries[{idx}].name"),
                message: format!("Beneficiary {idx} name is required"),
                severity: "error".to_string(),
            });
        }

        if b.wallet_address.trim().is_empty() {
            errors.push(ValidationError {
                field: format!("beneficiaries[{idx}].wallet_address"),
                message: format!("Beneficiary {idx} wallet address is required"),
                severity: "error".to_string(),
            });
        }

        if rules.require_relationship && b.relationship.as_deref().unwrap_or("").trim().is_empty() {
            errors.push(ValidationError {
                field: format!("beneficiaries[{idx}].relationship"),
                message: format!(
                    "Beneficiary {idx} relationship is required for {} jurisdiction",
                    rules.jurisdiction
                ),
                severity: "error".to_string(),
            });
        }
    }
}

// --- Unit Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::will_pdf::WillTemplate;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn valid_input() -> WillDocumentInput {
        WillDocumentInput {
            plan_id: Uuid::new_v4(),
            owner_name: "Alice Testator".to_string(),
            owner_wallet: "GABC1234567890ABCDEF".to_string(),
            vault_id: "vault-001".to_string(),
            beneficiaries: vec![BeneficiaryEntry {
                name: "Bob Beneficiary".to_string(),
                wallet_address: "GBOB1234567890ABCDEF".to_string(),
                allocation_percent: dec!(100),
                relationship: Some("Son".to_string()),
            }],
            execution_rules: Some("Distribute after 90-day inactivity".to_string()),
            template: WillTemplate::Formal,
            jurisdiction: Some("US".to_string()),
            will_hash_reference: None,
        }
    }

    #[test]
    fn test_valid_document_passes() {
        let input = valid_input();
        let result = WillComplianceService::validate(&input, 2);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert_eq!(result.jurisdiction, "US");
    }

    #[test]
    fn test_missing_owner_name() {
        let mut input = valid_input();
        input.owner_name = "".to_string();
        let result = WillComplianceService::validate(&input, 2);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "owner_name"));
    }

    #[test]
    fn test_missing_owner_wallet() {
        let mut input = valid_input();
        input.owner_wallet = "  ".to_string();
        let result = WillComplianceService::validate(&input, 2);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "owner_wallet"));
    }

    #[test]
    fn test_insufficient_witnesses_us() {
        let input = valid_input();
        let result = WillComplianceService::validate(&input, 1);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "witness_count"));
    }

    #[test]
    fn test_allocation_not_100() {
        let mut input = valid_input();
        input.beneficiaries[0].allocation_percent = dec!(50);
        let result = WillComplianceService::validate(&input, 2);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.field == "beneficiaries.allocation_percent"));
    }

    #[test]
    fn test_empty_beneficiary_name() {
        let mut input = valid_input();
        input.beneficiaries[0].name = "".to_string();
        let result = WillComplianceService::validate(&input, 2);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.field == "beneficiaries[1].name"));
    }

    #[test]
    fn test_empty_beneficiary_wallet() {
        let mut input = valid_input();
        input.beneficiaries[0].wallet_address = "".to_string();
        let result = WillComplianceService::validate(&input, 2);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.field == "beneficiaries[1].wallet_address"));
    }

    #[test]
    fn test_uk_requires_relationship() {
        let mut input = valid_input();
        input.jurisdiction = Some("UK".to_string());
        input.beneficiaries[0].relationship = None;
        let result = WillComplianceService::validate(&input, 2);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.field == "beneficiaries[1].relationship"));
    }

    #[test]
    fn test_uk_valid_with_relationship() {
        let mut input = valid_input();
        input.jurisdiction = Some("UK".to_string());
        let result = WillComplianceService::validate(&input, 2);
        assert!(result.is_valid);
    }

    #[test]
    fn test_eu_notarization_warning() {
        let mut input = valid_input();
        input.jurisdiction = Some("EU".to_string());
        let result = WillComplianceService::validate(&input, 2);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("notarization"));
    }

    #[test]
    fn test_global_fallback_one_witness() {
        let mut input = valid_input();
        input.jurisdiction = None;
        let result = WillComplianceService::validate(&input, 1);
        assert!(result.is_valid);
        assert_eq!(result.jurisdiction, "GLOBAL");
    }

    #[test]
    fn test_no_beneficiaries() {
        let mut input = valid_input();
        input.beneficiaries.clear();
        let result = WillComplianceService::validate(&input, 2);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.field == "beneficiaries"));
    }

    #[test]
    fn test_list_supported_jurisdictions() {
        let jurisdictions = WillComplianceService::list_supported_jurisdictions();
        assert_eq!(jurisdictions.len(), 4);
        assert!(jurisdictions.contains(&"US".to_string()));
        assert!(jurisdictions.contains(&"UK".to_string()));
        assert!(jurisdictions.contains(&"EU".to_string()));
        assert!(jurisdictions.contains(&"GLOBAL".to_string()));
    }

    #[test]
    fn test_get_jurisdiction_rules_us() {
        let rules = WillComplianceService::get_jurisdiction_rules("US");
        assert_eq!(rules.min_witnesses, 2);
        assert_eq!(rules.jurisdiction, "US");
    }

    #[test]
    fn test_get_jurisdiction_rules_unknown_falls_back_to_global() {
        let rules = WillComplianceService::get_jurisdiction_rules("JP");
        assert_eq!(rules.jurisdiction, "GLOBAL");
        assert_eq!(rules.min_witnesses, 1);
    }

    #[test]
    fn test_multiple_errors_reported() {
        let mut input = valid_input();
        input.owner_name = "".to_string();
        input.owner_wallet = "".to_string();
        input.beneficiaries[0].allocation_percent = dec!(50);
        let result = WillComplianceService::validate(&input, 0);
        assert!(!result.is_valid);
        assert!(result.errors.len() >= 3);
    }
}

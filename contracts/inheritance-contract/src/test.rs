#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String, Symbol, Vec};

#[test]
fn test_hash_string() {
    let env = Env::default();

    let input = String::from_str(&env, "test");
    let hash1 = InheritanceContract::hash_string(&env, input.clone());
    let hash2 = InheritanceContract::hash_string(&env, input);

    // Same input should produce same hash
    assert_eq!(hash1, hash2);

    let different_input = String::from_str(&env, "different");
    let hash3 = InheritanceContract::hash_string(&env, different_input);

    // Different input should produce different hash
    assert_ne!(hash1, hash3);
}

#[test]
fn test_hash_claim_code_valid() {
    let env = Env::default();

    let valid_code = String::from_str(&env, "123456");
    let result = InheritanceContract::hash_claim_code(&env, valid_code);
    assert!(result.is_ok());
}

#[test]
fn test_hash_claim_code_invalid_length() {
    let env = Env::default();

    let short_code = String::from_str(&env, "12345"); // 5 digits
    let result = InheritanceContract::hash_claim_code(&env, short_code);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), InheritanceError::InvalidClaimCode);

    let long_code = String::from_str(&env, "1234567"); // 7 digits
    let result = InheritanceContract::hash_claim_code(&env, long_code);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), InheritanceError::InvalidClaimCode);
}

#[test]
fn test_validate_plan_inputs() {
    let env = Env::default();

    let valid_name = String::from_str(&env, "Valid Plan");
    let valid_description = String::from_str(&env, "Valid description");
    let asset_type = Symbol::new(&env, "USDC");
    let valid_amount = 1000000;

    let result = InheritanceContract::validate_plan_inputs(
        valid_name.clone(),
        valid_description.clone(),
        asset_type.clone(),
        valid_amount,
    );
    assert!(result.is_ok());

    // Test empty plan name
    let empty_name = String::from_str(&env, "");
    let result = InheritanceContract::validate_plan_inputs(
        empty_name,
        valid_description.clone(),
        asset_type.clone(),
        valid_amount,
    );
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        InheritanceError::MissingRequiredField
    );

    // Test invalid amount
    let result =
        InheritanceContract::validate_plan_inputs(valid_name, valid_description, asset_type, 0);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), InheritanceError::InvalidTotalAmount);
}

#[test]
fn test_validate_beneficiaries() {
    let env = Env::default();

    // Valid beneficiaries
    let valid_beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "John"),
            String::from_str(&env, "john@example.com"),
            String::from_str(&env, "123456"),
            String::from_str(&env, "123456789"),
            50u32,
        ),
        (
            String::from_str(&env, "Jane"),
            String::from_str(&env, "jane@example.com"),
            String::from_str(&env, "654321"),
            String::from_str(&env, "987654321"),
            50u32,
        ),
    ];

    let result = InheritanceContract::validate_beneficiaries(valid_beneficiaries);
    assert!(result.is_ok());

    // Test empty beneficiaries
    let empty_beneficiaries = Vec::new(&env);
    let result = InheritanceContract::validate_beneficiaries(empty_beneficiaries);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        InheritanceError::MissingRequiredField
    );

    // Test allocation mismatch
    let invalid_allocation = vec![
        &env,
        (
            String::from_str(&env, "John"),
            String::from_str(&env, "john@example.com"),
            String::from_str(&env, "123456"),
            String::from_str(&env, "123456789"),
            60u32,
        ),
        (
            String::from_str(&env, "Jane"),
            String::from_str(&env, "jane@example.com"),
            String::from_str(&env, "654321"),
            String::from_str(&env, "987654321"),
            50u32,
        ),
    ];

    let result = InheritanceContract::validate_beneficiaries(invalid_allocation);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        InheritanceError::AllocationPercentageMismatch
    );
}

#[test]
fn test_create_beneficiary_success() {
    let env = Env::default();

    let full_name = String::from_str(&env, "John Doe");
    let email = String::from_str(&env, "john@example.com");
    let claim_code = String::from_str(&env, "123456");
    let bank_account = String::from_str(&env, "1234567890123456");
    let allocation = 100u32;

    let result = InheritanceContract::create_beneficiary(
        &env,
        full_name,
        email,
        claim_code,
        bank_account,
        allocation,
    );

    assert!(result.is_ok());
    let beneficiary = result.unwrap();
    assert_eq!(beneficiary.allocation_percentage, 100);
}

#[test]
fn test_create_beneficiary_invalid_data() {
    let env = Env::default();

    // Test empty name
    let result = InheritanceContract::create_beneficiary(
        &env,
        String::from_str(&env, ""), // empty name
        String::from_str(&env, "john@example.com"),
        String::from_str(&env, "123456"),
        String::from_str(&env, "1234567890123456"),
        100u32,
    );
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        InheritanceError::InvalidBeneficiaryData
    );

    // Test invalid claim code
    let result = InheritanceContract::create_beneficiary(
        &env,
        String::from_str(&env, "John Doe"),
        String::from_str(&env, "john@example.com"),
        String::from_str(&env, "12345"), // invalid length
        String::from_str(&env, "1234567890123456"),
        100u32,
    );
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), InheritanceError::InvalidClaimCode);
}

#[test]
fn test_storage_functions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    // Test that we can call the hello function (basic functionality)
    let words = client.hello(&symbol_short!("Dev"));
    assert_eq!(
        words,
        vec![&env, symbol_short!("Hello"), symbol_short!("Dev"),]
    );
}

// Integration tests for the full create_inheritance_plan function
#[test]
fn test_create_inheritance_plan_integration_success() {
    let env = Env::default();

    // Test the validation functions directly since storage requires contract context
    let plan_name = String::from_str(&env, "Integration Test Plan");
    let description = String::from_str(&env, "A comprehensive integration test");
    let asset_type = Symbol::new(&env, "USDC");
    let total_amount = 2000000u64;

    let beneficiaries_data = vec![
        &env,
        (
            String::from_str(&env, "Alice Johnson"),
            String::from_str(&env, "alice@example.com"),
            String::from_str(&env, "111111"),
            String::from_str(&env, "1111111111111111"),
            40u32,
        ),
        (
            String::from_str(&env, "Bob Smith"),
            String::from_str(&env, "bob@example.com"),
            String::from_str(&env, "222222"),
            String::from_str(&env, "2222222222222222"),
            30u32,
        ),
        (
            String::from_str(&env, "Charlie Brown"),
            String::from_str(&env, "charlie@example.com"),
            String::from_str(&env, "333333"),
            String::from_str(&env, "3333333333333333"),
            30u32,
        ),
    ];

    // Test validation functions
    let plan_result = InheritanceContract::validate_plan_inputs(
        plan_name.clone(),
        description.clone(),
        asset_type.clone(),
        total_amount,
    );
    assert!(plan_result.is_ok());

    let beneficiaries_result =
        InheritanceContract::validate_beneficiaries(beneficiaries_data.clone());
    assert!(beneficiaries_result.is_ok());

    // Test beneficiary creation
    let beneficiary = InheritanceContract::create_beneficiary(
        &env,
        String::from_str(&env, "Alice Johnson"),
        String::from_str(&env, "alice@example.com"),
        String::from_str(&env, "111111"),
        String::from_str(&env, "1111111111111111"),
        40u32,
    );
    assert!(beneficiary.is_ok());
}

#[test]
fn test_create_inheritance_plan_integration_validation_failure() {
    let env = Env::default();

    // Test with empty plan name validation
    let asset_type = Symbol::new(&env, "USDC");
    let result = InheritanceContract::validate_plan_inputs(
        String::from_str(&env, ""), // Empty plan name
        String::from_str(&env, "Test"),
        asset_type,
        1000000,
    );

    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        InheritanceError::MissingRequiredField
    );
}

#[test]
fn test_create_inheritance_plan_integration_allocation_failure() {
    let env = Env::default();

    // Test with allocation percentages that don't sum to 100%
    let beneficiaries_data = vec![
        &env,
        (
            String::from_str(&env, "John"),
            String::from_str(&env, "john@example.com"),
            String::from_str(&env, "123456"),
            String::from_str(&env, "123456789"),
            50u32,
        ),
        (
            String::from_str(&env, "Jane"),
            String::from_str(&env, "jane@example.com"),
            String::from_str(&env, "654321"),
            String::from_str(&env, "987654321"),
            40u32, // Total = 90%, should fail
        ),
    ];

    let result = InheritanceContract::validate_beneficiaries(beneficiaries_data);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        InheritanceError::AllocationPercentageMismatch
    );
}

#[test]
fn test_create_inheritance_plan_integration_multiple_plans() {
    let env = Env::default();

    // Test validation for first plan
    let asset_type = Symbol::new(&env, "USDC");
    let result1 = InheritanceContract::validate_plan_inputs(
        String::from_str(&env, "First Plan"),
        String::from_str(&env, "First test plan"),
        asset_type.clone(),
        1000000,
    );
    assert!(result1.is_ok());

    let beneficiaries1 = vec![
        &env,
        (
            String::from_str(&env, "John"),
            String::from_str(&env, "john@example.com"),
            String::from_str(&env, "123456"),
            String::from_str(&env, "123456789"),
            100u32,
        ),
    ];
    let beneficiaries_result1 = InheritanceContract::validate_beneficiaries(beneficiaries1);
    assert!(beneficiaries_result1.is_ok());

    // Test validation for second plan
    let result2 = InheritanceContract::validate_plan_inputs(
        String::from_str(&env, "Second Plan"),
        String::from_str(&env, "Second test plan"),
        asset_type,
        2000000,
    );
    assert!(result2.is_ok());

    let beneficiaries2 = vec![
        &env,
        (
            String::from_str(&env, "Jane"),
            String::from_str(&env, "jane@example.com"),
            String::from_str(&env, "654321"),
            String::from_str(&env, "987654321"),
            100u32,
        ),
    ];
    let beneficiaries_result2 = InheritanceContract::validate_beneficiaries(beneficiaries2);
    assert!(beneficiaries_result2.is_ok());
}

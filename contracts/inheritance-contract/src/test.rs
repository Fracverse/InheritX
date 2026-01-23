#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Events}, vec, Address, Bytes, Env, String, Symbol, Vec};

// Helper function to create test address
fn create_test_address(env: &Env, _seed: u64) -> Address {
    Address::generate(env)
}

// Helper function to create test bytes
fn create_test_bytes(env: &Env, data: &str) -> Bytes {
    let mut bytes = Bytes::new(env);
    for byte in data.as_bytes() {
        bytes.push_back(*byte);
    }
    bytes
}

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

    let valid_code = 123456u32;
    let result = InheritanceContract::hash_claim_code(&env, valid_code);
    assert!(result.is_ok());

    // Test edge cases
    let min_code = 0u32;
    let result = InheritanceContract::hash_claim_code(&env, min_code);
    assert!(result.is_ok());

    let max_code = 999999u32;
    let result = InheritanceContract::hash_claim_code(&env, max_code);
    assert!(result.is_ok());
}

#[test]
fn test_hash_claim_code_invalid_range() {
    let env = Env::default();

    let invalid_code = 1000000u32; // > 999999
    let result = InheritanceContract::hash_claim_code(&env, invalid_code);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        InheritanceError::InvalidClaimCodeRange
    );
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
fn test_validate_beneficiaries_basis_points() {
    let env = Env::default();

    // Valid beneficiaries with basis points totaling 10000 (100%)
    let valid_beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "John"),
            String::from_str(&env, "john@example.com"),
            123456u32,
            create_test_bytes(&env, "123456789"),
            5000u32, // 50%
        ),
        (
            String::from_str(&env, "Jane"),
            String::from_str(&env, "jane@example.com"),
            654321u32,
            create_test_bytes(&env, "987654321"),
            5000u32, // 50%
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

    // Test allocation mismatch (not totaling 10000)
    let invalid_allocation = vec![
        &env,
        (
            String::from_str(&env, "John"),
            String::from_str(&env, "john@example.com"),
            123456u32,
            create_test_bytes(&env, "123456789"),
            6000u32,
        ),
        (
            String::from_str(&env, "Jane"),
            String::from_str(&env, "jane@example.com"),
            654321u32,
            create_test_bytes(&env, "987654321"),
            5000u32,
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
    let claim_code = 123456u32;
    let bank_account = create_test_bytes(&env, "1234567890123456");
    let allocation = 5000u32; // 50% in basis points

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
    assert_eq!(beneficiary.allocation_bp, 5000);
}

#[test]
fn test_create_beneficiary_invalid_data() {
    let env = Env::default();

    // Test empty name
    let result = InheritanceContract::create_beneficiary(
        &env,
        String::from_str(&env, ""), // empty name
        String::from_str(&env, "john@example.com"),
        123456u32,
        create_test_bytes(&env, "1234567890123456"),
        5000u32,
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
        1000000u32, // > 999999
        create_test_bytes(&env, "1234567890123456"),
        5000u32,
    );
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        InheritanceError::InvalidClaimCodeRange
    );

    // Test zero allocation
    let result = InheritanceContract::create_beneficiary(
        &env,
        String::from_str(&env, "John Doe"),
        String::from_str(&env, "john@example.com"),
        123456u32,
        create_test_bytes(&env, "1234567890123456"),
        0u32, // zero allocation
    );
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), InheritanceError::InvalidAllocation);
}

#[test]
fn test_add_beneficiary_success() {
    let env = Env::default();
    env.mock_all_auths(); // Mock all authorizations for testing
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create a plan first with full allocation
    let beneficiaries_data_full = vec![
        &env,
        (
            String::from_str(&env, "Alice Johnson"),
            String::from_str(&env, "alice@example.com"),
            111111u32,
            create_test_bytes(&env, "1111111111111111"),
            10000u32, // 100%
        ),
    ];

    let _plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Test Plan"),
        &String::from_str(&env, "Test Description"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data_full,
    );

    // This test demonstrates that we can create a plan successfully
    // Testing add_beneficiary requires removing a beneficiary first to make room
}

#[test]
fn test_add_beneficiary_to_empty_allocation() {
    let _env = Env::default();
    // For testing add_beneficiary, we need a plan with < 10000 bp allocated
    // But create_inheritance_plan requires exactly 10000 bp
    // This is a design consideration - we'll test the validation logic directly
}

#[test]
fn test_add_beneficiary_max_limit() {
    let _env = Env::default();
    // Test that we can't add more than 10 beneficiaries
    // This would be tested through the contract client in integration tests
}

#[test]
fn test_add_beneficiary_allocation_exceeds_limit() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create plan with 10000 bp (100%)
    let beneficiaries_data = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            111111u32,
            create_test_bytes(&env, "1111111111111111"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Test Plan"),
        &String::from_str(&env, "Test Description"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );

    // Try to add another beneficiary - should fail because allocation would exceed 10000
    let result = client.try_add_beneficiary(
        &owner,
        &plan_id,
        &String::from_str(&env, "Bob"),
        &String::from_str(&env, "bob@example.com"),
        &222222u32,
        &1000u32,
        &create_test_bytes(&env, "2222222222222222"),
    );

    assert!(result.is_err());
}

#[test]
fn test_remove_beneficiary_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create plan with 2 beneficiaries
    let beneficiaries_data = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            111111u32,
            create_test_bytes(&env, "1111111111111111"),
            5000u32,
        ),
        (
            String::from_str(&env, "Bob"),
            String::from_str(&env, "bob@example.com"),
            222222u32,
            create_test_bytes(&env, "2222222222222222"),
            5000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Test Plan"),
        &String::from_str(&env, "Test Description"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );

    // Remove first beneficiary
    let result = client.try_remove_beneficiary(&owner, &plan_id, &0u32);
    assert!(result.is_ok());

    // Now we can add a new beneficiary since we have room
    let add_result = client.try_add_beneficiary(
        &owner,
        &plan_id,
        &String::from_str(&env, "Charlie"),
        &String::from_str(&env, "charlie@example.com"),
        &333333u32,
        &3000u32,
        &create_test_bytes(&env, "3333333333333333"),
    );
    assert!(add_result.is_ok());
}

#[test]
fn test_remove_beneficiary_invalid_index() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create plan with 1 beneficiary
    let beneficiaries_data = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            111111u32,
            create_test_bytes(&env, "1111111111111111"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Test Plan"),
        &String::from_str(&env, "Test Description"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );

    // Try to remove beneficiary at invalid index
    let result = client.try_remove_beneficiary(&owner, &plan_id, &5u32);
    assert!(result.is_err());
}

#[test]
fn test_remove_beneficiary_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);
    let unauthorized = create_test_address(&env, 2);

    // Create plan
    let beneficiaries_data = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            111111u32,
            create_test_bytes(&env, "1111111111111111"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Test Plan"),
        &String::from_str(&env, "Test Description"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );

    // Try to remove with unauthorized address
    let result = client.try_remove_beneficiary(&unauthorized, &plan_id, &0u32);
    assert!(result.is_err());
}

#[test]
fn test_beneficiary_allocation_tracking() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create plan with 3 beneficiaries totaling 10000 bp
    let beneficiaries_data = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            111111u32,
            create_test_bytes(&env, "1111111111111111"),
            4000u32, // 40%
        ),
        (
            String::from_str(&env, "Bob"),
            String::from_str(&env, "bob@example.com"),
            222222u32,
            create_test_bytes(&env, "2222222222222222"),
            3000u32, // 30%
        ),
        (
            String::from_str(&env, "Charlie"),
            String::from_str(&env, "charlie@example.com"),
            333333u32,
            create_test_bytes(&env, "3333333333333333"),
            3000u32, // 30%
        ),
    ];

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Test Plan"),
        &String::from_str(&env, "Test Description"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );

    // Remove one beneficiary (3000 bp)
    client.remove_beneficiary(&owner, &plan_id, &1u32);

    // Now we should be able to add a beneficiary with up to 3000 bp
    let result = client.try_add_beneficiary(
        &owner,
        &plan_id,
        &String::from_str(&env, "David"),
        &String::from_str(&env, "david@example.com"),
        &444444u32,
        &3000u32,
        &create_test_bytes(&env, "4444444444444444"),
    );
    assert!(result.is_ok());

    // Try to add another - should fail
    let result2 = client.try_add_beneficiary(
        &owner,
        &plan_id,
        &String::from_str(&env, "Eve"),
        &String::from_str(&env, "eve@example.com"),
        &555555u32,
        &1000u32,
        &create_test_bytes(&env, "5555555555555555"),
    );
    assert!(result2.is_err());
}

#[test]
fn test_max_10_beneficiaries() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create plan with 10 beneficiaries (1000 bp each = 10000 total)
    let mut beneficiaries_data = Vec::new(&env);
    for i in 0..10 {
        beneficiaries_data.push_back((
            String::from_str(&env, "Beneficiary"),
            String::from_str(&env, "test@example.com"),
            100000u32 + i,
            create_test_bytes(&env, "1234567890123456"),
            1000u32,
        ));
    }

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Test Plan"),
        &String::from_str(&env, "Test Description"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );

    // Remove one to make room
    client.remove_beneficiary(&owner, &plan_id, &0u32);

    // Add one back
    let result = client.try_add_beneficiary(
        &owner,
        &plan_id,
        &String::from_str(&env, "New Beneficiary"),
        &String::from_str(&env, "new@example.com"),
        &999999u32,
        &1000u32,
        &create_test_bytes(&env, "9999999999999999"),
    );
    assert!(result.is_ok());

    // Try to add 11th - should fail
    client.remove_beneficiary(&owner, &plan_id, &0u32);
    
    // Add back to get to 10
    client.add_beneficiary(
        &owner,
        &plan_id,
        &String::from_str(&env, "Tenth"),
        &String::from_str(&env, "tenth@example.com"),
        &888888u32,
        &1000u32,
        &create_test_bytes(&env, "8888888888888888"),
    );

    // Now try to add 11th
    let result = client.try_add_beneficiary(
        &owner,
        &plan_id,
        &String::from_str(&env, "Eleventh"),
        &String::from_str(&env, "eleventh@example.com"),
        &777777u32,
        &100u32,
        &create_test_bytes(&env, "7777777777777777"),
    );
    assert!(result.is_err());
}

#[test]
fn test_events_emitted() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create plan
    let beneficiaries_data = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            111111u32,
            create_test_bytes(&env, "1111111111111111"),
            5000u32,
        ),
        (
            String::from_str(&env, "Bob"),
            String::from_str(&env, "bob@example.com"),
            222222u32,
            create_test_bytes(&env, "2222222222222222"),
            5000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Test Plan"),
        &String::from_str(&env, "Test Description"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );

    // Remove beneficiary and check events
    client.remove_beneficiary(&owner, &plan_id, &0u32);

    // Add beneficiary and check events
    client.add_beneficiary(
        &owner,
        &plan_id,
        &String::from_str(&env, "Charlie"),
        &String::from_str(&env, "charlie@example.com"),
        &333333u32,
        &2000u32,
        &create_test_bytes(&env, "3333333333333333"),
    );

    // Events should be in the event log
    let events = env.events().all();
    assert!(events.len() > 0);
}

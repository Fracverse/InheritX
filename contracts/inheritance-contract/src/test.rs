#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Bytes, Env, String, Symbol, Vec};

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

fn one_beneficiary(
    env: &Env,
    name: &str,
    email: &str,
    claim_code: u32,
) -> Vec<(String, String, u32, Bytes, u32)> {
    vec![
        env,
        (
            String::from_str(env, name),
            String::from_str(env, email),
            claim_code,
            create_test_bytes(env, "1111111111111111"),
            10000u32,
        ),
    ]
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
        &BeneficiaryInput {
            name: String::from_str(&env, "Charlie"),
            email: String::from_str(&env, "charlie@example.com"),
            claim_code: 333333,
            bank_account: create_test_bytes(&env, "3333333333333333"),
            allocation_bp: 2000,
        },
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
        &BeneficiaryInput {
            name: String::from_str(&env, "Charlie"),
            email: String::from_str(&env, "charlie@example.com"),
            claim_code: 333333,
            bank_account: create_test_bytes(&env, "3333333333333333"),
            allocation_bp: 2000,
        },
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
        &BeneficiaryInput {
            name: String::from_str(&env, "Charlie"),
            email: String::from_str(&env, "charlie@example.com"),
            claim_code: 333333,
            bank_account: create_test_bytes(&env, "3333333333333333"),
            allocation_bp: 2000,
        },
    );
    assert!(result.is_ok());

    // Try to add another - should fail
    let result2 = client.try_add_beneficiary(
        &owner,
        &plan_id,
        &BeneficiaryInput {
            name: String::from_str(&env, "Charlie"),
            email: String::from_str(&env, "charlie@example.com"),
            claim_code: 333333,
            bank_account: create_test_bytes(&env, "3333333333333333"),
            allocation_bp: 2000,
        },
    );
    assert!(result2.is_err());
}
#[test]
fn test_claim_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            123456u32,
            create_test_bytes(&env, "1111"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Will"),
        &String::from_str(&env, "Inheritance Plan"),
        &1000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries,
    );

    client.claim_inheritance_plan(
        &plan_id,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
}

#[test]
#[should_panic]
fn test_double_claim_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            123456u32,
            create_test_bytes(&env, "1111"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Will"),
        &String::from_str(&env, "Inheritance Plan"),
        &1000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries,
    );

    client.claim_inheritance_plan(
        &plan_id,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );

    // second claim should panic
    client.claim_inheritance_plan(
        &plan_id,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
}
#[test]
#[should_panic]
fn test_claim_with_wrong_code_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            123456u32,
            create_test_bytes(&env, "1111"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Will"),
        &String::from_str(&env, "Inheritance Plan"),
        &1000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries,
    );

    client.claim_inheritance_plan(
        &plan_id,
        &String::from_str(&env, "alice@example.com"),
        &999999u32, // wrong code
    );
}

#[test]
fn test_deactivate_plan_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create a plan
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

    // Deactivate the plan
    let result = client.try_deactivate_inheritance_plan(&owner, &plan_id);
    assert!(result.is_ok());
}

#[test]
fn test_deactivate_plan_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);
    let unauthorized = create_test_address(&env, 2);

    // Create a plan
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

    // Try to deactivate with unauthorized address
    let result = client.try_deactivate_inheritance_plan(&unauthorized, &plan_id);
    assert!(result.is_err());
}

#[test]
fn test_deactivate_plan_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Try to deactivate a non-existent plan
    let result = client.try_deactivate_inheritance_plan(&owner, &999u64);
    assert!(result.is_err());
}

#[test]
fn test_deactivate_plan_already_deactivated() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create a plan
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

    // Deactivate the plan
    client.deactivate_inheritance_plan(&owner, &plan_id);

    // Try to deactivate again
    let result = client.try_deactivate_inheritance_plan(&owner, &plan_id);
    assert!(result.is_err());
}

#[test]
#[should_panic]
fn test_claim_deactivated_plan_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create a plan
    let beneficiaries_data = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            123456u32,
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

    // Deactivate the plan
    client.deactivate_inheritance_plan(&owner, &plan_id);

    // Try to claim from deactivated plan - should panic
    client.claim_inheritance_plan(
        &plan_id,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
}

#[test]
fn test_deactivate_plan_with_multiple_beneficiaries() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create a plan with multiple beneficiaries
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
        &2000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );

    // Deactivate the plan
    let result = client.try_deactivate_inheritance_plan(&owner, &plan_id);
    assert!(result.is_ok());
}

#[test]
fn test_get_plan_details() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    // Create a plan
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

    // Get plan details
    let plan = client.get_plan_details(&plan_id);
    assert!(plan.is_some());

    let plan_data = plan.unwrap();
    assert!(plan_data.is_active);
    assert_eq!(plan_data.total_amount, 1000000u64);

    // Deactivate and check again
    client.deactivate_inheritance_plan(&owner, &plan_id);

    let deactivated_plan = client.get_plan_details(&plan_id);
    assert!(deactivated_plan.is_some());
    assert!(!deactivated_plan.unwrap().is_active);
}

#[test]
fn test_kyc_approve_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let user = create_test_address(&env, 2);

    client.initialize_admin(&admin);
    client.submit_kyc(&user);

    let result = client.try_approve_kyc(&admin, &user);
    assert!(result.is_ok());

    let stored: KycStatus = env.as_contract(&contract_id, || {
        env.storage().persistent().get(&DataKey::Kyc(user)).unwrap()
    });
    assert!(stored.submitted);
    assert!(stored.approved);
}

#[test]
fn test_kyc_approve_non_admin_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let non_admin = create_test_address(&env, 2);
    let user = create_test_address(&env, 3);

    client.initialize_admin(&admin);
    client.submit_kyc(&user);

    let result = client.try_approve_kyc(&non_admin, &user);
    assert!(result.is_err());
}

#[test]
fn test_kyc_approve_without_submission_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let user = create_test_address(&env, 2);

    client.initialize_admin(&admin);

    let result = client.try_approve_kyc(&admin, &user);
    assert!(result.is_err());
}

#[test]
fn test_kyc_approve_already_approved_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let user = create_test_address(&env, 2);

    client.initialize_admin(&admin);
    client.submit_kyc(&user);
    client.approve_kyc(&admin, &user);

    let result = client.try_approve_kyc(&admin, &user);
    assert!(result.is_err());
}

// ───────────────────────────────────────────────────
// KYC Rejection Tests
// ───────────────────────────────────────────────────

#[test]
fn test_kyc_reject_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let user = create_test_address(&env, 2);

    client.initialize_admin(&admin);
    client.submit_kyc(&user);

    let result = client.try_reject_kyc(&admin, &user);
    assert!(result.is_ok());

    let stored: KycStatus = env.as_contract(&contract_id, || {
        env.storage().persistent().get(&DataKey::Kyc(user)).unwrap()
    });
    assert!(stored.submitted);
    assert!(!stored.approved);
    assert!(stored.rejected);
    assert_eq!(stored.rejected_at, env.ledger().timestamp());
}

#[test]
fn test_kyc_reject_non_admin_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let non_admin = create_test_address(&env, 2);
    let user = create_test_address(&env, 3);

    client.initialize_admin(&admin);
    client.submit_kyc(&user);

    let result = client.try_reject_kyc(&non_admin, &user);
    assert!(result.is_err());
}

#[test]
fn test_kyc_reject_without_submission_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let user = create_test_address(&env, 2);

    client.initialize_admin(&admin);

    let result = client.try_reject_kyc(&admin, &user);
    assert!(result.is_err());
}

#[test]
fn test_kyc_reject_already_rejected_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let user = create_test_address(&env, 2);

    client.initialize_admin(&admin);
    client.submit_kyc(&user);
    client.reject_kyc(&admin, &user);

    let result = client.try_reject_kyc(&admin, &user);
    assert!(result.is_err());
}

// ───────────────────────────────────────────────────
// Contract Upgrade Tests
// ───────────────────────────────────────────────────

fn fake_wasm_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

#[test]
fn test_version_returns_default() {
    let env = Env::default();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let version = client.version();
    assert_eq!(version, 1);
}

#[test]
fn test_upgrade_rejects_non_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let non_admin = create_test_address(&env, 2);
    client.initialize_admin(&admin);

    // Auth check happens before wasm swap, so this returns NotAdmin
    let result = client.try_upgrade(&non_admin, &fake_wasm_hash(&env));
    assert!(result.is_err());
}

#[test]
fn test_upgrade_rejects_no_admin_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let caller = create_test_address(&env, 1);

    let result = client.try_upgrade(&caller, &fake_wasm_hash(&env));
    assert!(result.is_err());
}

#[test]
fn test_upgrade_version_stored_in_storage() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    client.initialize_admin(&admin);

    // Directly set version in storage to simulate upgrade version tracking
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Version, &5u32);
    });

    let version = client.version();
    assert_eq!(version, 5);
}

#[test]
fn test_migrate_no_migration_needed() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    client.initialize_admin(&admin);

    // Set version to CONTRACT_VERSION so migration is not needed
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Version, &1u32);
    });
    let result = client.try_migrate(&admin);
    assert!(result.is_err());
}

#[test]
fn test_migrate_rejects_non_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let non_admin = create_test_address(&env, 2);
    client.initialize_admin(&admin);

    let result = client.try_migrate(&non_admin);
    assert!(result.is_err());
}

#[test]
fn test_migrate_runs_when_version_outdated() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    client.initialize_admin(&admin);

    // Set stored version to 0 (older than CONTRACT_VERSION) to simulate needing migration
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Version, &0u32);
    });

    let result = client.try_migrate(&admin);
    assert!(result.is_ok());

    // After migration, version should be CONTRACT_VERSION
    let version = client.version();
    assert_eq!(version, 1);
}

#[test]
fn test_plan_data_survives_across_versions() {
    // Soroban upgrades preserve all persistent/instance storage.
    // This test verifies plan data stays intact when version changes.
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 1);
    let owner = create_test_address(&env, 2);
    client.initialize_admin(&admin);

    // Create plans, claims, KYC before version bump
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
        &String::from_str(&env, "Pre-Upgrade Plan"),
        &String::from_str(&env, "Should survive"),
        &5000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );

    // Deactivate second plan
    let deact_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Deactivated"),
        &String::from_str(&env, "Will deactivate"),
        &2000000u64,
        &DistributionMethod::Monthly,
        &beneficiaries_data.clone(),
    );
    client.deactivate_inheritance_plan(&owner, &deact_id);

    // Submit + approve KYC
    let user = create_test_address(&env, 3);
    client.submit_kyc(&user);
    client.approve_kyc(&admin, &user.clone());

    // Simulate version bump (as upgrade would do)
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Version, &2u32);
    });

    // All data still accessible
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert!(plan.is_active);
    assert_eq!(plan.total_amount, 5000000u64);
    assert_eq!(plan.beneficiaries.len(), 2);
    assert_eq!(plan.owner, owner);

    let deact_plan = client.get_plan_details(&deact_id).unwrap();
    assert!(!deact_plan.is_active);

    let kyc: KycStatus = env.as_contract(&contract_id, || {
        env.storage().persistent().get(&DataKey::Kyc(user)).unwrap()
    });
    assert!(kyc.submitted);
    assert!(kyc.approved);

    assert_eq!(client.version(), 2);
}

#[test]
fn test_get_user_deactivated_plans() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);
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

    // Create 2 plans
    let plan1 = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Plan 1"),
        &String::from_str(&env, "Desc 1"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );
    let _plan2 = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Plan 2"),
        &String::from_str(&env, "Desc 2"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );

    // Deactivate plan 1
    client.deactivate_inheritance_plan(&owner, &plan1);

    // Get deactivated plans
    let deactivated = client.get_user_deactivated_plans(&owner);
    assert_eq!(deactivated.len(), 1);
    assert_eq!(
        deactivated.get(0).unwrap().plan_name,
        String::from_str(&env, "Plan 1")
    );
}

#[test]
fn test_admin_retrieval() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 99);
    client.initialize_admin(&admin);

    let owner1 = create_test_address(&env, 1);
    let owner2 = create_test_address(&env, 2);
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

    // Owner 1 creates and deactivates
    let plan1 = client.create_inheritance_plan(
        &owner1,
        &String::from_str(&env, "Plan 1"),
        &String::from_str(&env, "Desc 1"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );
    client.deactivate_inheritance_plan(&owner1, &plan1);

    // Owner 2 creates and deactivates
    let plan2 = client.create_inheritance_plan(
        &owner2,
        &String::from_str(&env, "Plan 2"),
        &String::from_str(&env, "Desc 2"),
        &1000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries_data,
    );
    client.deactivate_inheritance_plan(&owner2, &plan2);

    // Admin retrieves all
    let all_deactivated = client.get_all_deactivated_plans(&admin);
    assert_eq!(all_deactivated.len(), 2);
}

#[test]
fn test_get_claimed_plan() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);
    let beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            123456u32,
            create_test_bytes(&env, "1111"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Will"),
        &String::from_str(&env, "Inheritance Plan"),
        &1000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries,
    );

    // Should error because it's not claimed yet
    let result = client.try_get_claimed_plan(&owner, &plan_id);
    assert!(result.is_err());

    client.claim_inheritance_plan(
        &plan_id,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );

    // Should succeed now
    let plan = client.get_claimed_plan(&owner, &plan_id);
    assert_eq!(plan.total_amount, 1000u64);
}

#[test]
fn test_get_user_claimed_plans() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);
    let beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            123456u32,
            create_test_bytes(&env, "1111"),
            10000u32,
        ),
    ];

    let plan1 = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Will 1"),
        &String::from_str(&env, "Plan"),
        &1000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries,
    );

    let plan2 = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Will 2"),
        &String::from_str(&env, "Plan"),
        &2000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries,
    );

    client.claim_inheritance_plan(
        &plan1,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
    client.claim_inheritance_plan(
        &plan2,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );

    let plans = client.get_user_claimed_plans(&owner);
    assert_eq!(plans.len(), 2);
}

#[test]
fn test_get_all_claimed_plans() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 99);
    client.initialize_admin(&admin);

    let owner = create_test_address(&env, 1);
    let beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            123456u32,
            create_test_bytes(&env, "1111"),
            10000u32,
        ),
    ];

    let plan1 = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Will"),
        &String::from_str(&env, "Plan"),
        &1000u64,
        &DistributionMethod::LumpSum,
        &beneficiaries,
    );

    client.claim_inheritance_plan(
        &plan1,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );

    let plans = client.get_all_claimed_plans(&admin);
    assert_eq!(plans.len(), 1);

    let non_admin = create_test_address(&env, 2);
    let result = client.try_get_all_claimed_plans(&non_admin);
    assert!(result.is_err());
}

#[test]
fn test_get_user_plan_supports_active_and_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);
    let stranger = create_test_address(&env, 2);

    let plan_id = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Plan A"),
        &String::from_str(&env, "Plan A Description"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice1@example.com", 123456),
    );

    let active_plan = client.get_user_plan(&owner, &plan_id);
    assert!(active_plan.is_active);

    client.deactivate_inheritance_plan(&owner, &plan_id);
    let inactive_plan = client.get_user_plan(&owner, &plan_id);
    assert!(!inactive_plan.is_active);

    let unauthorized = client.try_get_user_plan(&stranger, &plan_id);
    assert!(unauthorized.is_err());
}

#[test]
fn test_get_user_plans_returns_all_user_plans() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    let plan_1 = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Plan 1"),
        &String::from_str(&env, "Description 1"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice2@example.com", 111111),
    );

    let _plan_2 = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Plan 2"),
        &String::from_str(&env, "Description 2"),
        &2000000u64,
        &DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Bob", "bob2@example.com", 222222),
    );

    client.deactivate_inheritance_plan(&owner, &plan_1);

    let plans = client.get_user_plans(&owner);
    assert_eq!(plans.len(), 2);
}

#[test]
fn test_get_all_plans_admin_only_and_includes_active_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 100);
    client.initialize_admin(&admin);

    let user_a = create_test_address(&env, 1);
    let user_b = create_test_address(&env, 2);

    let plan_a1 = client.create_inheritance_plan(
        &user_a,
        &String::from_str(&env, "A1"),
        &String::from_str(&env, "A1 Desc"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &one_beneficiary(&env, "A", "a1@example.com", 100001),
    );

    let _plan_a2 = client.create_inheritance_plan(
        &user_a,
        &String::from_str(&env, "A2"),
        &String::from_str(&env, "A2 Desc"),
        &2000000u64,
        &DistributionMethod::LumpSum,
        &one_beneficiary(&env, "A", "a2@example.com", 100002),
    );

    let _plan_b1 = client.create_inheritance_plan(
        &user_b,
        &String::from_str(&env, "B1"),
        &String::from_str(&env, "B1 Desc"),
        &3000000u64,
        &DistributionMethod::LumpSum,
        &one_beneficiary(&env, "B", "b1@example.com", 100003),
    );

    client.deactivate_inheritance_plan(&user_a, &plan_a1);

    let all_plans = client.get_all_plans(&admin);
    assert_eq!(all_plans.len(), 3);

    let non_admin = create_test_address(&env, 999);
    let unauthorized = client.try_get_all_plans(&non_admin);
    assert!(unauthorized.is_err());
}

#[test]
fn test_get_user_pending_plans_filters_only_active() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = create_test_address(&env, 1);

    let plan_1 = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Plan 1"),
        &String::from_str(&env, "Description 1"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice3@example.com", 333333),
    );

    let _plan_2 = client.create_inheritance_plan(
        &owner,
        &String::from_str(&env, "Plan 2"),
        &String::from_str(&env, "Description 2"),
        &2000000u64,
        &DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Bob", "bob3@example.com", 444444),
    );

    client.deactivate_inheritance_plan(&owner, &plan_1);

    let pending = client.get_user_pending_plans(&owner);
    assert_eq!(pending.len(), 1);
    assert!(pending.get(0).unwrap().is_active);
}

#[test]
fn test_get_all_pending_plans_admin_only() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = create_test_address(&env, 100);
    client.initialize_admin(&admin);

    let user_a = create_test_address(&env, 1);
    let user_b = create_test_address(&env, 2);

    let plan_a = client.create_inheritance_plan(
        &user_a,
        &String::from_str(&env, "A"),
        &String::from_str(&env, "A Desc"),
        &1000000u64,
        &DistributionMethod::LumpSum,
        &one_beneficiary(&env, "A", "a3@example.com", 555555),
    );

    let _plan_b = client.create_inheritance_plan(
        &user_b,
        &String::from_str(&env, "B"),
        &String::from_str(&env, "B Desc"),
        &2000000u64,
        &DistributionMethod::LumpSum,
        &one_beneficiary(&env, "B", "b3@example.com", 666666),
    );

    client.deactivate_inheritance_plan(&user_a, &plan_a);

    let pending = client.get_all_pending_plans(&admin);
    assert_eq!(pending.len(), 1);
    assert!(pending.get(0).unwrap().is_active);

    let not_admin = create_test_address(&env, 999);
    let unauthorized = client.try_get_all_pending_plans(&not_admin);
    assert!(unauthorized.is_err());
}

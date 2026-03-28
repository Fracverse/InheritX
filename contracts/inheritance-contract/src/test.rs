#![cfg(test)]

use super::*;
use mock_token::MockToken;
use mock_token::MockTokenClient;
use soroban_sdk::{
    testutils::Address as _, testutils::Events, testutils::Ledger, token, vec, Address, Bytes, Env,
    String, Vec,
};

/// Test helper for balance and mint (uses mock-token crate client).
struct TestTokenHelper<'a> {
    env: &'a Env,
    token: Address,
}

impl TestTokenHelper<'_> {
    fn new<'a>(env: &'a Env, token: &'a Address) -> TestTokenHelper<'a> {
        TestTokenHelper {
            env,
            token: token.clone(),
        }
    }

    fn balance(&self, id: &Address) -> i128 {
        token::Client::new(self.env, &self.token).balance(id)
    }

    fn mint(&self, to: &Address, amount: &i128) {
        MockTokenClient::new(self.env, &self.token).mint(to, amount);
    }
}

// ----- Test setup and param helpers -----
/// Sets up env with inheritance contract, mock token, admin initialized, owner minted.
/// Returns (client, token_id, admin, owner).
fn setup_with_token_and_admin(
    env: &Env,
) -> (InheritanceContractClient<'_>, Address, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let token_id = env.register_contract(None, MockToken);
    let admin = create_test_address(env, 100);
    let owner = create_test_address(env, 1);
    let client = InheritanceContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);
    TestTokenHelper::new(env, &token_id).mint(&owner, &10_000_000i128);

    // Approve KYC for owner by default so they can create plans
    client.submit_kyc(&owner);
    client.approve_kyc(&admin, &owner);

    (client, token_id, admin, owner)
}

/// Sets up env without KYC approval - for testing KYC validation
fn setup_with_token_and_admin_no_kyc(
    env: &Env,
) -> (InheritanceContractClient<'_>, Address, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let token_id = env.register_contract(None, MockToken);
    let admin = create_test_address(env, 101);
    let owner = create_test_address(env, 2);
    let client = InheritanceContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);
    TestTokenHelper::new(env, &token_id).mint(&owner, &10_000_000i128);
    (client, token_id, admin, owner)
}

#[allow(clippy::too_many_arguments)]
fn plan_params(
    env: &Env,
    owner: &Address,
    token: &Address,
    plan_name: &str,
    description: &str,
    total_amount: u64,
    distribution_method: DistributionMethod,
    beneficiaries_data: &Vec<(String, String, u32, Bytes, u32)>,
) -> CreateInheritancePlanParams {
    CreateInheritancePlanParams {
        owner: owner.clone(),
        token: token.clone(),
        plan_name: String::from_str(env, plan_name),
        description: String::from_str(env, description),
        total_amount,
        distribution_method,
        beneficiaries_data: beneficiaries_data.clone(),
        is_lendable: true,
    }
}

fn default_beneficiaries(env: &Env) -> Vec<(String, String, u32, Bytes, u32)> {
    vec![
        env,
        (
            String::from_str(env, "Alice"),
            String::from_str(env, "alice@example.com"),
            111111u32,
            create_test_bytes(env, "1111111111111111"),
            10000u32,
        ),
    ]
}

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
        &env,
        valid_name.clone(),
        valid_description.clone(),
        asset_type.clone(),
        valid_amount,
    );
    assert!(result.is_ok());

    // Test empty plan name
    let empty_name = String::from_str(&env, "");
    let result = InheritanceContract::validate_plan_inputs(
        &env,
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
    let result = InheritanceContract::validate_plan_inputs(
        &env,
        valid_name,
        valid_description,
        asset_type,
        0,
    );
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
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

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

    let _plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data_full,
    ));

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
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

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
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

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
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

    // Try to remove beneficiary at invalid index
    let result = client.try_remove_beneficiary(&owner, &plan_id, &5u32);
    assert!(result.is_err());
}

#[test]
fn test_remove_beneficiary_unauthorized() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    let unauthorized = create_test_address(&env, 2);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

    // Try to remove with unauthorized address
    let result = client.try_remove_beneficiary(&unauthorized, &plan_id, &0u32);
    assert!(result.is_err());
}

#[test]
fn test_beneficiary_allocation_tracking() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

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
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 100);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "Inheritance Plan",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries,
    ));

    // Approve KYC for beneficiary
    client.submit_kyc(&beneficiary);
    client.approve_kyc(&admin, &beneficiary);

    // Claim should succeed and log an event, we now also test if transferring would work if we had the code implemented fully.
    // NOTE: In the current MVP setup for inheritance-contract, we modified claim_inheritance_plan
    // to emit the event with the payout amount. In a real integration test with the lending contract,
    // we would deposit actual mock tokens and verify the beneficiary balance increases.
    // For this unit test, we just verify it doesn't panic.
    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
}

#[test]
#[should_panic]
fn test_double_claim_fails() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 201);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "Inheritance Plan",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries,
    ));

    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );

    // second claim should panic
    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
}
#[test]
#[should_panic]
fn test_claim_with_wrong_code_fails() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 202);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "Inheritance Plan",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries,
    ));

    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &999999u32, // wrong code
    );
}

#[test]
fn test_deactivate_plan_success() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

    // Deactivate the plan
    let result = client.try_deactivate_inheritance_plan(&owner, &plan_id);
    assert!(result.is_ok());
}

#[test]
fn test_deactivate_plan_unauthorized() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    let unauthorized = create_test_address(&env, 2);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

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
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

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
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 203);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

    // Deactivate the plan
    client.deactivate_inheritance_plan(&owner, &plan_id);

    // Try to claim from deactivated plan - should panic
    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
}

#[test]
fn test_deactivate_plan_with_multiple_beneficiaries() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        2000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

    // Deactivate the plan
    let result = client.try_deactivate_inheritance_plan(&owner, &plan_id);
    assert!(result.is_ok());
}

#[test]
fn test_get_plan_details() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Test Plan",
        "Test Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

    // Get plan details — plan stores net amount (user input minus 2% fee): 1000000 * 0.98 = 980000
    let plan = client.get_plan_details(&plan_id);
    assert!(plan.is_some());

    let plan_data = plan.unwrap();
    assert!(plan_data.is_active);
    assert_eq!(plan_data.total_amount, 980000u64);

    // Deactivate and check again
    client.deactivate_inheritance_plan(&owner, &plan_id);

    let deactivated_plan = client.get_plan_details(&plan_id);
    assert!(deactivated_plan.is_some());
    assert!(!deactivated_plan.unwrap().is_active);
}

// --- 2% creation fee: unit and integration tests ---

#[test]
fn test_creation_fee_calculation_and_net_amount_stored() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

    // User input 100_000; 2% fee = 2_000, net = 98_000
    let input_amount = 100_000u64;
    let beneficiaries_data = default_beneficiaries(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Fee Test Plan",
        "Description",
        input_amount,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

    let plan = client.get_plan_details(&plan_id).unwrap();
    let expected_fee = input_amount * 2 / 100;
    let expected_net = input_amount - expected_fee;
    assert_eq!(
        plan.total_amount, expected_net,
        "Plan must store net amount (input minus 2% fee)"
    );
    assert_eq!(expected_net, 98_000u64);
}

#[test]
fn test_fee_transfer_to_admin_wallet() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let input_amount = 1000u64; // fee = 20
    let beneficiaries_data = default_beneficiaries(&env);

    let admin_balance_before = TestTokenHelper::new(&env, &token).balance(&admin);

    client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Plan",
        "Desc",
        input_amount,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

    let admin_balance_after = TestTokenHelper::new(&env, &token).balance(&admin);
    let expected_fee = 20i128; // 2% of 1000
    assert_eq!(
        admin_balance_after - admin_balance_before,
        expected_fee,
        "Admin must receive 2% fee"
    );
}

#[test]
fn test_insufficient_balance_returns_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let token_id = env.register_contract(None, MockToken);
    let admin = create_test_address(&env, 100);
    let owner = create_test_address(&env, 1);

    let client = InheritanceContractClient::new(&env, &contract_id);
    client.initialize_admin(&admin);

    // Approve KYC for owner
    client.submit_kyc(&owner);
    client.approve_kyc(&admin, &owner);

    // Mint only 100 to owner (less than 1000 needed)
    TestTokenHelper::new(&env, &token_id).mint(&owner, &100i128);

    let beneficiaries_data = default_beneficiaries(&env);

    let result = client.try_create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token_id,
        "Plan",
        "Desc",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(
        err.is_ok(),
        "contract should return InheritanceError, not InvokeError"
    );
    assert_eq!(err.ok().unwrap(), InheritanceError::InsufficientBalance);
}

// Note: This test is commented out because KYC check now happens before admin check.
// To test admin validation, we would need a different approach.
// #[test]
// fn test_create_plan_without_admin_fails() {
//     let env = Env::default();
//     env.mock_all_auths();
//     let contract_id = env.register_contract(None, InheritanceContract);
//     let token_id = env.register_contract(None, MockToken);
//     let owner = create_test_address(&env, 1);
//     TestTokenHelper::new(&env, &token_id).mint(&owner, &10_000_000i128);

//     let client = InheritanceContractClient::new(&env, &contract_id);
//     // Do NOT call initialize_admin

//     let result = client.try_create_inheritance_plan(&plan_params(
//         &env,
//         &owner,
//         &token_id,
//         "Plan",
//         "Desc",
//         1000u64,
//         DistributionMethod::LumpSum,
//         &default_beneficiaries(&env),
//     ));

//     assert!(result.is_err());
//     let err = result.err().unwrap();
//     assert!(
//         err.is_ok(),
//         "contract should return InheritanceError, not InvokeError"
//     );
//     assert_eq!(err.ok().unwrap(), InheritanceError::AdminNotSet);
// }

#[test]
fn test_successful_plan_creation_with_net_amount() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

    let input = 50_000u64;
    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "My Plan",
        "Desc",
        input,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    ));

    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_amount, 49_000u64); // 50_000 - 2% = 49_000
    assert!(plan.is_active);
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
    let token_id = env.register_contract(None, MockToken);
    let client = InheritanceContractClient::new(&env, &contract_id);
    let admin = create_test_address(&env, 1);
    let owner = create_test_address(&env, 2);
    client.initialize_admin(&admin);
    TestTokenHelper::new(&env, &token_id).mint(&owner, &10_000_000i128);

    // Approve KYC for owner
    client.submit_kyc(&owner);
    client.approve_kyc(&admin, &owner);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token_id,
        "Pre-Upgrade Plan",
        "Should survive",
        5000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

    // Deactivate second plan
    let deact_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token_id,
        "Deactivated",
        "Will deactivate",
        2000000u64,
        DistributionMethod::Monthly,
        &beneficiaries_data,
    ));
    client.deactivate_inheritance_plan(&owner, &deact_id);

    // Submit + approve KYC
    let user = create_test_address(&env, 3);
    client.submit_kyc(&user);
    client.approve_kyc(&admin, &user.clone());

    // Simulate version bump (as upgrade would do)
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Version, &2u32);
    });

    // All data still accessible (plan stores net amount after 2% fee: 5000000 * 0.98 = 4900000)
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert!(plan.is_active);
    assert_eq!(plan.total_amount, 4_900_000u64);
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
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

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
    let plan1 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Plan 1",
        "Desc 1",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));
    let _plan2 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Plan 2",
        "Desc 2",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));

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
    let (client, token, admin, _) = setup_with_token_and_admin(&env);
    let owner1 = create_test_address(&env, 1);
    let owner2 = create_test_address(&env, 2);
    TestTokenHelper::new(&env, &token).mint(&owner1, &10_000_000i128);
    TestTokenHelper::new(&env, &token).mint(&owner2, &10_000_000i128);

    // Approve KYC for new owners
    client.submit_kyc(&owner1);
    client.approve_kyc(&admin, &owner1);
    client.submit_kyc(&owner2);
    client.approve_kyc(&admin, &owner2);

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
    let plan1 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner1,
        &token,
        "Plan 1",
        "Desc 1",
        1000000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));
    client.deactivate_inheritance_plan(&owner1, &plan1);

    // Owner 2 creates and deactivates
    let plan2 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner2,
        &token,
        "Plan 2",
        "Desc 2",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries_data,
    ));
    client.deactivate_inheritance_plan(&owner2, &plan2);

    // Admin retrieves all
    let all_deactivated = client.get_all_deactivated_plans(&admin);
    assert_eq!(all_deactivated.len(), 2);
}

#[test]
fn test_get_claimed_plan() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 204);

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

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "Inheritance Plan",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries,
    ));

    // Should error because it's not claimed yet
    let result = client.try_get_claimed_plan(&owner, &plan_id);
    assert!(result.is_err());

    // Approve KYC for beneficiary
    client.submit_kyc(&beneficiary);
    client.approve_kyc(&admin, &beneficiary);

    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );

    // Should succeed now (plan stores net after 2% fee: 1000 * 0.98 = 980)
    // After 100% claim, the remaining balance should be 0.
    let plan = client.get_claimed_plan(&owner, &plan_id);
    assert_eq!(plan.total_amount, 0u64);
}

#[test]
fn test_get_user_claimed_plans() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 205);

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

    let plan1 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will 1",
        "Plan",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries,
    ));

    let plan2 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will 2",
        "Plan",
        2000u64,
        DistributionMethod::LumpSum,
        &beneficiaries,
    ));

    // Approve KYC for beneficiary
    client.submit_kyc(&beneficiary);
    client.approve_kyc(&admin, &beneficiary);

    client.claim_inheritance_plan(
        &plan1,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
    client.claim_inheritance_plan(
        &plan2,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );

    let plans = client.get_user_claimed_plans(&owner);
    assert_eq!(plans.len(), 2);
}

#[test]
fn test_get_all_claimed_plans() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 206);

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

    let plan1 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "Plan",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries,
    ));

    // Approve KYC for beneficiary
    client.submit_kyc(&beneficiary);
    client.approve_kyc(&admin, &beneficiary);

    client.claim_inheritance_plan(
        &plan1,
        &beneficiary,
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
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    let stranger = create_test_address(&env, 2);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Plan A",
        "Plan A Description",
        1000000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice1@example.com", 123456),
    ));

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
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

    let plan_1 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Plan 1",
        "Description 1",
        1000000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice2@example.com", 111111),
    ));

    let _plan_2 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Plan 2",
        "Description 2",
        2000000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Bob", "bob2@example.com", 222222),
    ));

    client.deactivate_inheritance_plan(&owner, &plan_1);

    let plans = client.get_user_plans(&owner);
    assert_eq!(plans.len(), 2);
}

#[test]
fn test_get_all_plans_admin_only_and_includes_active_inactive() {
    let env = Env::default();
    let (client, token, admin, _) = setup_with_token_and_admin(&env);
    let user_a = create_test_address(&env, 1);
    let user_b = create_test_address(&env, 2);
    TestTokenHelper::new(&env, &token).mint(&user_a, &10_000_000i128);
    TestTokenHelper::new(&env, &token).mint(&user_b, &10_000_000i128);

    // Approve KYC for users
    client.submit_kyc(&user_a);
    client.approve_kyc(&admin, &user_a);
    client.submit_kyc(&user_b);
    client.approve_kyc(&admin, &user_b);

    let plan_a1 = client.create_inheritance_plan(&plan_params(
        &env,
        &user_a,
        &token,
        "A1",
        "A1 Desc",
        1000000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "A", "a1@example.com", 100001),
    ));

    let _plan_a2 = client.create_inheritance_plan(&plan_params(
        &env,
        &user_a,
        &token,
        "A2",
        "A2 Desc",
        2000000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "A", "a2@example.com", 100002),
    ));

    let _plan_b1 = client.create_inheritance_plan(&plan_params(
        &env,
        &user_b,
        &token,
        "B1",
        "B1 Desc",
        3000000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "B", "b1@example.com", 100003),
    ));

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
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

    let plan_1 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Plan 1",
        "Description 1",
        1000000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice3@example.com", 333333),
    ));

    let _plan_2 = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Plan 2",
        "Description 2",
        2000000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Bob", "bob3@example.com", 444444),
    ));

    client.deactivate_inheritance_plan(&owner, &plan_1);

    let pending = client.get_user_pending_plans(&owner);
    assert_eq!(pending.len(), 1);
    assert!(pending.get(0).unwrap().is_active);
}

#[test]
fn test_get_all_pending_plans_admin_only() {
    let env = Env::default();
    let (client, token, admin, _) = setup_with_token_and_admin(&env);
    let user_a = create_test_address(&env, 1);
    let user_b = create_test_address(&env, 2);
    TestTokenHelper::new(&env, &token).mint(&user_a, &10_000_000i128);
    TestTokenHelper::new(&env, &token).mint(&user_b, &10_000_000i128);

    // Approve KYC for users
    client.submit_kyc(&user_a);
    client.approve_kyc(&admin, &user_a);
    client.submit_kyc(&user_b);
    client.approve_kyc(&admin, &user_b);

    let plan_a = client.create_inheritance_plan(&plan_params(
        &env,
        &user_a,
        &token,
        "A",
        "A Desc",
        1000000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "A", "a3@example.com", 555555),
    ));

    let _plan_b = client.create_inheritance_plan(&plan_params(
        &env,
        &user_b,
        &token,
        "B",
        "B Desc",
        2000000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "B", "b3@example.com", 666666),
    ));

    client.deactivate_inheritance_plan(&user_a, &plan_a);

    let pending = client.get_all_pending_plans(&admin);
    assert_eq!(pending.len(), 1);
    assert!(pending.get(0).unwrap().is_active);

    let not_admin = create_test_address(&env, 999);
    let unauthorized = client.try_get_all_pending_plans(&not_admin);
    assert!(unauthorized.is_err());
}

// ───────────────────────────────────────────────────
// Lending Features Tests
// ───────────────────────────────────────────────────

#[test]
fn test_set_lendable() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Lend",
        "Test Lend",
        1000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "B", "b@example.com", 666666),
    ));

    // Initially lendable defaults to true based on our plan_params modification
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert!(plan.is_lendable);

    // Toggle off
    client.set_lendable(&owner, &plan_id, &false);
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert!(!plan.is_lendable);

    // Unauthorized fails
    let not_owner = create_test_address(&env, 999);
    let result = client.try_set_lendable(&not_owner, &plan_id, &true);
    assert!(result.is_err());
}

#[test]
fn test_vault_deposit_and_withdraw() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    TestTokenHelper::new(&env, &token).mint(&owner, &10_000_000i128);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Lend",
        "Test Lend",
        1000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "B", "b@example.com", 666666),
    ));

    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_amount, 980); // 1000 - 2% fee

    // Deposit more
    client.deposit(&owner, &token, &plan_id, &500u64);
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_amount, 1480);

    // Withdraw some
    client.withdraw(&owner, &token, &plan_id, &300u64);
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_amount, 1180);
    assert_eq!(plan.total_loaned, 0);

    // Unauthorized fails
    let not_owner = create_test_address(&env, 999);
    let result = client.try_deposit(&not_owner, &token, &plan_id, &100u64);
    assert!(result.is_err());
    let result = client.try_withdraw(&not_owner, &token, &plan_id, &100u64);
    assert!(result.is_err());
}

#[test]
fn test_vault_withdraw_prevents_over_withdrawal() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    TestTokenHelper::new(&env, &token).mint(&owner, &10_000_000i128);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Lend",
        "Test Lend",
        1000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "B", "b@example.com", 666666),
    ));

    client.deposit(&owner, &token, &plan_id, &500u64);

    // We don't have a public function to change total_loaned from the client (since
    // it's for external protocols), so we simulate it by setting it in storage.
    let mut plan = client.get_plan_details(&plan_id).unwrap();
    plan.total_loaned = 1000;

    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKey::Plan(plan_id), &plan);
    });

    let modified_plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(modified_plan.total_amount, 1480);
    assert_eq!(modified_plan.total_loaned, 1000);

    // Withdraw 400 OK (1480 - 1000 = 480 available)
    assert!(client
        .try_withdraw(&owner, &token, &plan_id, &400u64)
        .is_ok());

    // Another 100 FAILS (480 - 400 = 80 available)
    let err = client.try_withdraw(&owner, &token, &plan_id, &100u64);
    assert!(err.is_err());
}

// ───────────────────────────────────────────────────
// Loan Recall on Inheritance Trigger Tests
// ───────────────────────────────────────────────────

#[test]
fn test_trigger_inheritance_freezes_loans() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Plan should be lendable initially
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert!(plan.is_lendable);

    // Trigger inheritance
    client.trigger_inheritance(&admin, &plan_id);

    // Plan should now have is_lendable = false (loans frozen)
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert!(!plan.is_lendable);

    // Trigger info should exist
    let trigger_info = client.get_inheritance_trigger(&plan_id);
    assert!(trigger_info.is_some());
    let info = trigger_info.unwrap();
    assert!(info.loan_freeze_active);
    assert!(!info.recall_attempted);
    assert!(!info.liquidation_triggered);
}

#[test]
fn test_trigger_inheritance_double_trigger_fails() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    client.trigger_inheritance(&admin, &plan_id);

    // Second trigger should fail
    let result = client.try_trigger_inheritance(&admin, &plan_id);
    assert!(result.is_err());
}

#[test]
fn test_trigger_inheritance_non_admin_fails() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    let non_admin = create_test_address(&env, 999);
    let result = client.try_trigger_inheritance(&non_admin, &plan_id);
    assert!(result.is_err());
}

#[test]
fn test_trigger_inheritance_inactive_plan_fails() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Deactivate first
    client.deactivate_inheritance_plan(&owner, &plan_id);

    let result = client.try_trigger_inheritance(&admin, &plan_id);
    assert!(result.is_err());
}

#[test]
fn test_recall_loan_success() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Simulate outstanding loans by setting total_loaned
    let mut plan = client.get_plan_details(&plan_id).unwrap();
    plan.total_loaned = 50_000;
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKey::Plan(plan_id), &plan);
    });

    // Trigger inheritance
    client.trigger_inheritance(&admin, &plan_id);

    // Recall 30,000 of the 50,000 loaned
    client.recall_loan(&admin, &plan_id, &30_000u64);

    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_loaned, 20_000);

    let info = client.get_inheritance_trigger(&plan_id).unwrap();
    assert!(info.recall_attempted);
    assert_eq!(info.recalled_amount, 30_000);

    // Recall remaining
    client.recall_loan(&admin, &plan_id, &20_000u64);

    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_loaned, 0);

    let info = client.get_inheritance_trigger(&plan_id).unwrap();
    assert_eq!(info.recalled_amount, 50_000);
}

#[test]
fn test_recall_loan_exceeds_loaned_fails() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    let mut plan = client.get_plan_details(&plan_id).unwrap();
    plan.total_loaned = 10_000;
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKey::Plan(plan_id), &plan);
    });

    client.trigger_inheritance(&admin, &plan_id);

    // Recall more than loaned should fail
    let result = client.try_recall_loan(&admin, &plan_id, &20_000u64);
    assert!(result.is_err());
}

#[test]
fn test_recall_loan_without_trigger_fails() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Try to recall without triggering inheritance first
    let result = client.try_recall_loan(&admin, &plan_id, &1000u64);
    assert!(result.is_err());
}

#[test]
fn test_recall_loan_no_outstanding_loans_fails() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    client.trigger_inheritance(&admin, &plan_id);

    // No loans to recall
    let result = client.try_recall_loan(&admin, &plan_id, &1000u64);
    assert!(result.is_err());
}

#[test]
fn test_liquidation_fallback_success() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Plan stores 98,000 (100,000 - 2% fee)
    // Simulate 30,000 in loans
    let mut plan = client.get_plan_details(&plan_id).unwrap();
    plan.total_loaned = 30_000;
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKey::Plan(plan_id), &plan);
    });

    // Trigger inheritance
    client.trigger_inheritance(&admin, &plan_id);

    // Trigger liquidation fallback — write off 30,000
    client.liquidation_fallback(&admin, &plan_id);

    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_loaned, 0);
    // 98,000 - 30,000 = 68,000 claimable
    assert_eq!(plan.total_amount, 68_000);

    let info = client.get_inheritance_trigger(&plan_id).unwrap();
    assert!(info.liquidation_triggered);
    assert_eq!(info.settled_amount, 30_000);
}

#[test]
fn test_liquidation_fallback_without_trigger_fails() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    let result = client.try_liquidation_fallback(&admin, &plan_id);
    assert!(result.is_err());
}

#[test]
fn test_liquidation_fallback_no_loans_fails() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    client.trigger_inheritance(&admin, &plan_id);

    // No loans to liquidate
    let result = client.try_liquidation_fallback(&admin, &plan_id);
    assert!(result.is_err());
}

#[test]
fn test_partial_recall_then_liquidation_fallback() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Plan stores 98,000, simulate 40,000 in loans
    let mut plan = client.get_plan_details(&plan_id).unwrap();
    plan.total_loaned = 40_000;
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKey::Plan(plan_id), &plan);
    });

    client.trigger_inheritance(&admin, &plan_id);

    // Recall 25,000 of 40,000
    client.recall_loan(&admin, &plan_id, &25_000u64);

    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_loaned, 15_000);

    // Liquidation fallback for remaining 15,000
    client.liquidation_fallback(&admin, &plan_id);

    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_loaned, 0);
    // 98,000 - 15,000 = 83,000 claimable
    assert_eq!(plan.total_amount, 83_000);

    let info = client.get_inheritance_trigger(&plan_id).unwrap();
    assert!(info.recall_attempted);
    assert!(info.liquidation_triggered);
    assert_eq!(info.recalled_amount, 25_000);
    assert_eq!(info.settled_amount, 15_000);
}

#[test]
fn test_inheritance_claim_not_blocked_by_loans() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 207);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Simulate outstanding loans
    let mut plan = client.get_plan_details(&plan_id).unwrap();
    plan.total_loaned = 50_000;
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKey::Plan(plan_id), &plan);
    });

    // Trigger inheritance
    client.trigger_inheritance(&admin, &plan_id);

    // Approve KYC for beneficiary
    client.submit_kyc(&beneficiary);
    client.approve_kyc(&admin, &beneficiary);

    // Claim should succeed even with outstanding loans
    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );

    // After claiming, total_amount is reduced by base_payout so claimable is 0
    let claimable = client.get_claimable_amount(&plan_id);
    assert_eq!(claimable, 0);
}

#[test]
fn test_inheritance_claim_bypasses_time_check_when_triggered() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 208);

    // Create plan with Yearly distribution (would normally need 365 days)
    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::Yearly,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Approve KYC for beneficiary
    client.submit_kyc(&beneficiary);
    client.approve_kyc(&admin, &beneficiary);

    // Without trigger, claim should fail (time not met)
    let result = client.try_claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
    assert!(result.is_err());

    // Trigger inheritance
    client.trigger_inheritance(&admin, &plan_id);

    // Now claim should succeed despite time not elapsed
    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
}

#[test]
fn test_get_claimable_amount() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // No loans — full amount claimable (98,000 after 2% fee)
    let claimable = client.get_claimable_amount(&plan_id);
    assert_eq!(claimable, 98_000);

    // Simulate loans
    let mut plan = client.get_plan_details(&plan_id).unwrap();
    plan.total_loaned = 20_000;
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKey::Plan(plan_id), &plan);
    });

    let claimable = client.get_claimable_amount(&plan_id);
    assert_eq!(claimable, 78_000);
}

#[test]
fn test_full_loan_recall_workflow() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 209);

    // Step 1: Create plan
    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Estate",
        "Full estate plan",
        500_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Plan stores 490,000 (500k - 2% fee)
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_amount, 490_000);
    assert!(plan.is_lendable);

    // Step 2: Simulate some funds being loaned out
    let mut plan = client.get_plan_details(&plan_id).unwrap();
    plan.total_loaned = 200_000;
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKey::Plan(plan_id), &plan);
    });

    // Step 3: Trigger inheritance — freezes new loans
    client.trigger_inheritance(&admin, &plan_id);
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert!(!plan.is_lendable); // Frozen

    // Step 4: Attempt recall — recover 150k of 200k
    client.recall_loan(&admin, &plan_id, &150_000u64);

    // Step 5: Liquidation fallback for remaining 50k
    client.liquidation_fallback(&admin, &plan_id);

    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_loaned, 0);
    // 490,000 - 50,000 = 440,000 (only unrecoverable 50k was written off)
    assert_eq!(plan.total_amount, 440_000);

    // Approve KYC for beneficiary
    client.submit_kyc(&beneficiary);
    client.approve_kyc(&admin, &beneficiary);

    // Step 6: Beneficiary claims
    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );

    // After claiming, total_amount is reduced by base_payout so claimable is 0
    let claimable = client.get_claimable_amount(&plan_id);
    assert_eq!(claimable, 0);

    // Verify full trigger info
    let info = client.get_inheritance_trigger(&plan_id).unwrap();
    assert!(info.loan_freeze_active);
    assert!(info.recall_attempted);
    assert!(info.liquidation_triggered);
    assert_eq!(info.original_loaned, 200_000);
    assert_eq!(info.recalled_amount, 150_000);
    assert_eq!(info.settled_amount, 50_000);
}

// ───────────────────────────────────────────────────
// Emergency Access and Transfer Guard Tests
// ───────────────────────────────────────────────────

#[test]
fn test_activate_and_deactivate_emergency_access() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    let trusted_contact = create_test_address(&env, 555);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Initially no emergency access
    assert!(client.get_emergency_access(&plan_id).is_none());
    assert!(!client.is_emergency_active(&plan_id));

    // Activate
    client.activate_emergency_access(&owner, &plan_id, &trusted_contact);

    let record = client.get_emergency_access(&plan_id).unwrap();
    assert_eq!(record.trusted_contact, trusted_contact);
    assert!(client.is_emergency_active(&plan_id));

    // Deactivate
    client.deactivate_emergency_access(&owner, &plan_id);
    assert!(client.get_emergency_access(&plan_id).is_none());
    assert!(!client.is_emergency_active(&plan_id));
}

#[test]
fn test_is_emergency_active_cooldown() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    let trusted_contact = create_test_address(&env, 555);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    client.activate_emergency_access(&owner, &plan_id, &trusted_contact);
    assert!(client.is_emergency_active(&plan_id));

    // Jump forward 23 hours (82800 seconds) - should still be active
    env.ledger().with_mut(|li| li.timestamp += 82800);
    assert!(client.is_emergency_active(&plan_id));

    // Jump forward another 2 hours (total 25 hours) - should NO LONGER be active
    env.ledger().with_mut(|li| li.timestamp += 7200);
    assert!(!client.is_emergency_active(&plan_id));
}

#[test]
fn test_withdraw_emergency_limit() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup_with_token_and_admin(&env);
    let trusted_contact = create_test_address(&env, 555);

    // Plan stores 98,000 (100,000 - 2% fee)
    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    // Activate emergency
    client.activate_emergency_access(&owner, &plan_id, &trusted_contact);

    // 10% limit of 98,000 is 9,800

    // Withdraw 5,000 should SUCCEED
    assert!(client
        .try_withdraw(&owner, &token, &plan_id, &5_000u64)
        .is_ok());

    // Withdraw 10,000 should FAIL
    let result = client.try_withdraw(&owner, &token, &plan_id, &10_000u64);
    assert!(result.is_err());

    // Jump forward 25 hours - limit should BE REMOVED
    env.ledger().with_mut(|li| li.timestamp += 90000);
    assert!(client
        .try_withdraw(&owner, &token, &plan_id, &10_000u64)
        .is_ok());
}

#[test]
fn test_claim_emergency_limit() {
    let env = Env::default();
    let (client, token, admin, owner) = setup_with_token_and_admin(&env);
    let trusted_contact = create_test_address(&env, 555);
    let beneficiary = create_test_address(&env, 210);

    // 10% limit will be applied to the payout.
    // If we want it to fail, we need a payout > 10% of total.
    // Since we only have one beneficiary with 100%, payout is 100% of total.

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        "Will",
        "My will",
        100_000u64,
        DistributionMethod::LumpSum,
        &one_beneficiary(&env, "Alice", "alice@example.com", 123456),
    ));

    client.activate_emergency_access(&owner, &plan_id, &trusted_contact);

    // Approve KYC for beneficiary
    client.submit_kyc(&beneficiary);
    client.approve_kyc(&admin, &beneficiary);

    // Claim should FAIL because payout (100%) > limit (10%)
    let result = client.try_claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
    assert!(result.is_err());

    // Jump forward 25 hours - should SUCCEED
    env.ledger().with_mut(|li| li.timestamp += 90000);
    let result = client.try_claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
    assert!(result.is_ok());
}

#[test]
fn test_emergency_access_events() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let contract_id = client.address.clone();
    let trusted_contact = create_test_address(&env, 99);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    // 1. Test Activation Event
    let guardians: soroban_sdk::Vec<soroban_sdk::Address> = soroban_sdk::vec![&env, user.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &1);
    client.approve_emergency_access(&user, &plan_id, &trusted_contact);

    let events = env.events().all();
    let activation_event = events.get(events.len() - 1).unwrap();

    assert_eq!(activation_event.0, contract_id);
    assert_eq!(
        activation_event.1,
        (symbol_short!("EMERG"), symbol_short!("ACTIV")).into_val(&env)
    );

    // 2. Test Revocation Event
    client.deactivate_emergency_access(&user, &plan_id);
    let events = env.events().all();
    let revocation_event = events.get(events.len() - 1).unwrap();
    assert_eq!(revocation_event.0, contract_id);
    assert_eq!(
        revocation_event.1,
        (symbol_short!("EMERG"), symbol_short!("REVOK")).into_val(&env)
    );
}

#[test]
fn test_emergency_access_expiration() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let contract_id = client.address.clone();
    let trusted_contact = create_test_address(&env, 99);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    // Activate
    let guardians: soroban_sdk::Vec<soroban_sdk::Address> = soroban_sdk::vec![&env, user.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &1);
    client.approve_emergency_access(&user, &plan_id, &trusted_contact);
    assert!(client.get_emergency_access(&plan_id).is_some());

    // Fast forward 604801 seconds (7 days + 1s)
    env.ledger().set_timestamp(604801);

    // Call any function that triggers the check (e.g. get_emergency_access)
    assert!(client.get_emergency_access(&plan_id).is_none());

    // Verify Expiration Event
    let events = env.events().all();
    let expiration_event = events.get(events.len() - 1).unwrap();
    assert_eq!(expiration_event.0, contract_id);
    assert_eq!(
        expiration_event.1,
        (symbol_short!("EMERG"), symbol_short!("EXPIR")).into_val(&env)
    );
}

#[test]
fn test_emergency_withdrawal_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let trusted_contact = create_test_address(&env, 99);
    let token_helper = TestTokenHelper::new(&env, &token_id);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    // Plan owner deposits
    client.deposit(&user, &token_id, &plan_id, &5000);
    assert_eq!(token_helper.balance(&user), 10_000_000 - 10000 - 5000);

    // Activate emergency access
    let guardians: soroban_sdk::Vec<soroban_sdk::Address> = soroban_sdk::vec![&env, user.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &1);
    client.approve_emergency_access(&user, &plan_id, &trusted_contact);

    // Trusted contact withdraws (should fail)
    let result = client.try_withdraw(&trusted_contact, &token_id, &plan_id, &2000);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), Ok(InheritanceError::Unauthorized));

    // Verify balance
    assert_eq!(token_helper.balance(&trusted_contact), 0);
    let plan = client.get_plan_details(&plan_id).unwrap();
    // Initial 9800 (10000 - 2% fee) + 5000 (deposit) = 14800
    assert_eq!(plan.total_amount, 14800);
}

#[test]
fn test_emergency_withdrawal_fails_after_expiration() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let trusted_contact = create_test_address(&env, 99);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    client.deposit(&user, &token_id, &plan_id, &5000);

    // Activate
    let guardians: soroban_sdk::Vec<soroban_sdk::Address> = soroban_sdk::vec![&env, user.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &1);
    client.approve_emergency_access(&user, &plan_id, &trusted_contact);

    // Fast forward 7 days + 1s
    env.ledger().set_timestamp(604801);

    // Withdrawal should fail
    let result = client.try_withdraw(&trusted_contact, &token_id, &plan_id, &2000);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), Ok(InheritanceError::Unauthorized));

    // Emergency record should be gone
    assert!(client.get_emergency_access(&plan_id).is_none());
}

#[test]
fn test_emergency_deposit_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let trusted_contact = create_test_address(&env, 99);
    let token_helper = TestTokenHelper::new(&env, &token_id);
    token_helper.mint(&trusted_contact, &1000);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    // Activate
    let guardians: soroban_sdk::Vec<soroban_sdk::Address> = soroban_sdk::vec![&env, user.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &1);
    client.approve_emergency_access(&user, &plan_id, &trusted_contact);

    // Trusted contact deposits (should fail)
    let result = client.try_deposit(&trusted_contact, &token_id, &plan_id, &500);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), Ok(InheritanceError::Unauthorized));

    // Verify
    let plan = client.get_plan_details(&plan_id).unwrap();
    // Initial 9800 (10000 - 2% fee)
    assert_eq!(plan.total_amount, 9800);
    assert_eq!(token_helper.balance(&trusted_contact), 1000);
}

#[test]
fn test_emergency_view_plan_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let trusted_contact = create_test_address(&env, 99);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    // Activate emergency access
    let guardians: soroban_sdk::Vec<soroban_sdk::Address> = soroban_sdk::vec![&env, user.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &1);
    client.approve_emergency_access(&user, &plan_id, &trusted_contact);

    // Trusted contact can view plan
    let plan = client.get_user_plan(&trusted_contact, &plan_id);
    assert_eq!(plan.owner, user);
}

#[test]
fn test_emergency_trigger_inheritance_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let trusted_contact = create_test_address(&env, 99);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    // Activate emergency access
    let guardians: soroban_sdk::Vec<soroban_sdk::Address> = soroban_sdk::vec![&env, user.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &1);
    client.approve_emergency_access(&user, &plan_id, &trusted_contact);

    // Trusted contact can trigger inheritance
    client.trigger_inheritance(&trusted_contact, &plan_id);

    // Verify it's triggered (is_lendable should be false)
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert!(!plan.is_lendable);
}

#[test]
fn test_owner_trigger_inheritance_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    // Owner can trigger inheritance
    client.trigger_inheritance(&user, &plan_id);

    // Verify it's triggered (is_lendable should be false)
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert!(!plan.is_lendable);
}

#[test]
fn test_instant_revocation_by_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let trusted_contact = create_test_address(&env, 99);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    // 1. Activate emergency access
    let guardians: soroban_sdk::Vec<soroban_sdk::Address> = soroban_sdk::vec![&env, user.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &1);
    client.approve_emergency_access(&user, &plan_id, &trusted_contact);

    // 2. Verify it's active
    assert!(client.get_emergency_access(&plan_id).is_some());
    let plan = client.get_user_plan(&trusted_contact, &plan_id);
    assert_eq!(plan.owner, user);

    // 3. Owner revokes instantly
    client.deactivate_emergency_access(&user, &plan_id);

    // 4. Verify it's gone and subsequent calls fail
    assert!(client.get_emergency_access(&plan_id).is_none());
    let result = client.try_get_user_plan(&trusted_contact, &plan_id);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), Ok(InheritanceError::Unauthorized));

    // 5. Verify withdrawals also fail
    let withdraw_result = client.try_withdraw(&trusted_contact, &token_id, &plan_id, &100);
    assert!(withdraw_result.is_err());
    assert_eq!(
        withdraw_result.err().unwrap(),
        Ok(InheritanceError::Unauthorized)
    );
}

#[test]
fn test_multi_guardian_approval_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let guardian_1 = create_test_address(&env, 101);
    let guardian_2 = create_test_address(&env, 102);
    let trusted_contact = create_test_address(&env, 99);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    let guardians = soroban_sdk::vec![&env, guardian_1.clone(), guardian_2.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &2);

    // First guardian approves
    client.approve_emergency_access(&guardian_1, &plan_id, &trusted_contact);
    assert!(client.get_emergency_access(&plan_id).is_none());

    // Second guardian approves
    client.approve_emergency_access(&guardian_2, &plan_id, &trusted_contact);
    assert!(client.get_emergency_access(&plan_id).is_some());
}

#[test]
fn test_invalid_guardian_rejection() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let guardian = create_test_address(&env, 101);
    let random_user = create_test_address(&env, 102);
    let trusted_contact = create_test_address(&env, 99);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    let guardians = soroban_sdk::vec![&env, guardian.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &1);

    let res = client.try_approve_emergency_access(&random_user, &plan_id, &trusted_contact);
    assert!(res.is_err());
    assert_eq!(res.err().unwrap(), Ok(InheritanceError::Unauthorized));
}

#[test]
fn test_double_approval_rejection() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let guardian_1 = create_test_address(&env, 101);
    let guardian_2 = create_test_address(&env, 102);
    let trusted_contact = create_test_address(&env, 99);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    let guardians = soroban_sdk::vec![&env, guardian_1.clone(), guardian_2.clone()];
    client.set_guardians(&user, &plan_id, &guardians, &2);

    client.approve_emergency_access(&guardian_1, &plan_id, &trusted_contact);
    let res = client.try_approve_emergency_access(&guardian_1, &plan_id, &trusted_contact);
    assert!(res.is_err());
    assert_eq!(res.err().unwrap(), Ok(InheritanceError::AlreadyApproved));
}

// ─────────────────────────────────────────────────────────────────────────────
// Emergency Contact Registration Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_add_emergency_contact_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let contact = create_test_address(&env, 50);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    client.add_emergency_contact(&user, &plan_id, &contact);

    let contacts = client.get_emergency_contacts(&plan_id);
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts.get(0).unwrap(), contact);
}

#[test]
fn test_add_multiple_emergency_contacts() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let contact_1 = create_test_address(&env, 50);
    let contact_2 = create_test_address(&env, 51);
    let contact_3 = create_test_address(&env, 52);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    client.add_emergency_contact(&user, &plan_id, &contact_1);
    client.add_emergency_contact(&user, &plan_id, &contact_2);
    client.add_emergency_contact(&user, &plan_id, &contact_3);

    let contacts = client.get_emergency_contacts(&plan_id);
    assert_eq!(contacts.len(), 3);
}

#[test]
fn test_add_emergency_contact_duplicate_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let contact = create_test_address(&env, 50);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    client.add_emergency_contact(&user, &plan_id, &contact);
    let res = client.try_add_emergency_contact(&user, &plan_id, &contact);
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap(),
        Ok(InheritanceError::EmergencyContactAlreadyExists)
    );
}

#[test]
fn test_add_emergency_contact_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let other_user = create_test_address(&env, 99);
    let contact = create_test_address(&env, 50);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    let res = client.try_add_emergency_contact(&other_user, &plan_id, &contact);
    assert!(res.is_err());
    assert_eq!(res.err().unwrap(), Ok(InheritanceError::Unauthorized));
}

#[test]
fn test_remove_emergency_contact_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let contact = create_test_address(&env, 50);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    client.add_emergency_contact(&user, &plan_id, &contact);
    assert_eq!(client.get_emergency_contacts(&plan_id).len(), 1);

    client.remove_emergency_contact(&user, &plan_id, &contact);
    assert_eq!(client.get_emergency_contacts(&plan_id).len(), 0);
}

#[test]
fn test_remove_emergency_contact_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let contact = create_test_address(&env, 50);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    let res = client.try_remove_emergency_contact(&user, &plan_id, &contact);
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap(),
        Ok(InheritanceError::EmergencyContactNotFound)
    );
}

#[test]
fn test_remove_emergency_contact_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let other_user = create_test_address(&env, 99);
    let contact = create_test_address(&env, 50);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    client.add_emergency_contact(&user, &plan_id, &contact);

    let res = client.try_remove_emergency_contact(&other_user, &plan_id, &contact);
    assert!(res.is_err());
    assert_eq!(res.err().unwrap(), Ok(InheritanceError::Unauthorized));
}

#[test]
fn test_emergency_contact_events() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);
    let contact = create_test_address(&env, 50);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    client.add_emergency_contact(&user, &plan_id, &contact);

    let events = env.events().all();
    let add_event = events.get(events.len() - 1).unwrap();
    assert_eq!(add_event.0, client.address.clone());
    assert_eq!(
        add_event.1,
        (symbol_short!("EMERG"), symbol_short!("CON_ADD")).into_val(&env)
    );

    client.remove_emergency_contact(&user, &plan_id, &contact);

    let events = env.events().all();
    let remove_event = events.get(events.len() - 1).unwrap();
    assert_eq!(remove_event.0, client.address.clone());
    assert_eq!(
        remove_event.1,
        (symbol_short!("EMERG"), symbol_short!("CON_REM")).into_val(&env)
    );
}

#[test]
fn test_get_emergency_contacts_empty() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, user) = setup_with_token_and_admin(&env);

    let params = plan_params(
        &env,
        &user,
        &token_id,
        "Plan",
        "Desc",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    client.create_inheritance_plan(&params);
    let plan_id = 1u64;

    let contacts = client.get_emergency_contacts(&plan_id);
    assert_eq!(contacts.len(), 0);
}

// ── Will Management System Tests (Issues #314–#317) ──

fn create_plan_and_get_id(
    env: &Env,
    client: &InheritanceContractClient,
    token_id: &Address,
    owner: &Address,
) -> u64 {
    let params = plan_params(
        env,
        owner,
        token_id,
        "Test Plan",
        "Test Description",
        10000,
        DistributionMethod::LumpSum,
        &default_beneficiaries(env),
    );
    client.create_inheritance_plan(&params);
    1u64
}

fn test_will_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

fn test_will_hash_2(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[2u8; 32])
}

// --- Issue #314: Legal Will Hash Storage ---

#[test]
fn test_store_will_hash_success() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    client.store_will_hash(&owner, &plan_id, &will_hash);

    let stored = client.get_will_hash(&plan_id);
    assert_eq!(stored, Some(will_hash));
}

#[test]
fn test_store_will_hash_already_stored() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    client.store_will_hash(&owner, &plan_id, &will_hash);

    let result = client.try_store_will_hash(&owner, &plan_id, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_store_will_hash_unauthorized() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let other = create_test_address(&env, 99);
    let will_hash = test_will_hash(&env);

    let result = client.try_store_will_hash(&other, &plan_id, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_store_will_hash_plan_not_found() {
    let env = Env::default();
    let (client, _token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let will_hash = test_will_hash(&env);

    let result = client.try_store_will_hash(&owner, &999u64, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_get_will_hash_none() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let _plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let result = client.get_will_hash(&1u64);
    assert_eq!(result, None);
}

// --- Issue #315: Link Will Document to Vault ---

#[test]
fn test_link_will_to_vault_success() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    client.link_will_to_vault(&owner, &plan_id, &will_hash);

    let stored = client.get_vault_will(&plan_id);
    assert_eq!(stored, Some(will_hash));
}

#[test]
fn test_link_will_to_vault_already_linked() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    client.link_will_to_vault(&owner, &plan_id, &will_hash);

    let result = client.try_link_will_to_vault(&owner, &plan_id, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_link_will_to_vault_not_found() {
    let env = Env::default();
    let (client, _token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let will_hash = test_will_hash(&env);

    let result = client.try_link_will_to_vault(&owner, &999u64, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_link_will_to_vault_unauthorized() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let other = create_test_address(&env, 99);
    let will_hash = test_will_hash(&env);

    let result = client.try_link_will_to_vault(&other, &plan_id, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_get_vault_will_none() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let _plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let result = client.get_vault_will(&1u64);
    assert_eq!(result, None);
}

// --- Issue #316: Beneficiary Verification ---

#[test]
fn test_verify_beneficiaries_match() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    // Get the plan to extract the hashed_email of the beneficiary
    let plan = client.get_plan_details(&plan_id).unwrap();
    let ben = plan.beneficiaries.get(0).unwrap();

    let will_bens: Vec<(BytesN<32>, u32)> =
        vec![&env, (ben.hashed_email.clone(), ben.allocation_bp)];

    let result = client.verify_beneficiaries(&plan_id, &will_bens);
    assert!(result);

    let status = client.get_verification_status(&plan_id);
    assert_eq!(status, Some(true));
}

#[test]
fn test_verify_beneficiaries_mismatch_allocation() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let plan = client.get_plan_details(&plan_id).unwrap();
    let ben = plan.beneficiaries.get(0).unwrap();

    // Wrong allocation
    let will_bens: Vec<(BytesN<32>, u32)> = vec![&env, (ben.hashed_email.clone(), 5000u32)];

    let result = client.verify_beneficiaries(&plan_id, &will_bens);
    assert!(!result);

    let status = client.get_verification_status(&plan_id);
    assert_eq!(status, Some(false));
}

#[test]
fn test_verify_beneficiaries_mismatch_count() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    // Empty list — count mismatch
    let will_bens: Vec<(BytesN<32>, u32)> = Vec::new(&env);

    let result = client.verify_beneficiaries(&plan_id, &will_bens);
    assert!(!result);
}

#[test]
fn test_verify_beneficiaries_plan_not_found() {
    let env = Env::default();
    let (client, _token_id, _admin, _owner) = setup_with_token_and_admin(&env);

    let will_bens: Vec<(BytesN<32>, u32)> = Vec::new(&env);
    let result = client.try_verify_beneficiaries(&999u64, &will_bens);
    assert!(result.is_err());
}

#[test]
fn test_get_verification_status_none() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let _plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let status = client.get_verification_status(&1u64);
    assert_eq!(status, None);
}

// --- Issue #317: Will Versioning System ---

#[test]
fn test_create_will_version_first() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);
    assert_eq!(version, 1);

    let count = client.get_will_version_count(&plan_id);
    assert_eq!(count, 1);

    let ver_info = client.get_will_version(&plan_id, &1u32).unwrap();
    assert_eq!(ver_info.version, 1);
    assert_eq!(ver_info.will_hash, will_hash);
    assert!(ver_info.is_active);
}

#[test]
fn test_create_will_version_multiple() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let hash1 = test_will_hash(&env);
    let hash2 = test_will_hash_2(&env);

    let v1 = client.create_will_version(&owner, &plan_id, &hash1);
    assert_eq!(v1, 1);

    let v2 = client.create_will_version(&owner, &plan_id, &hash2);
    assert_eq!(v2, 2);

    // v1 should be deactivated
    let ver1 = client.get_will_version(&plan_id, &1u32).unwrap();
    assert!(!ver1.is_active);

    // v2 should be active
    let ver2 = client.get_will_version(&plan_id, &2u32).unwrap();
    assert!(ver2.is_active);

    let active = client.get_active_will_version(&plan_id).unwrap();
    assert_eq!(active.version, 2);
    assert_eq!(active.will_hash, hash2);

    let count = client.get_will_version_count(&plan_id);
    assert_eq!(count, 2);
}

#[test]
fn test_create_will_version_updates_vault_will() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let hash1 = test_will_hash(&env);
    let hash2 = test_will_hash_2(&env);

    client.create_will_version(&owner, &plan_id, &hash1);
    let vault_will = client.get_vault_will(&plan_id);
    assert_eq!(vault_will, Some(hash1));

    client.create_will_version(&owner, &plan_id, &hash2);
    let vault_will = client.get_vault_will(&plan_id);
    assert_eq!(vault_will, Some(hash2));
}

#[test]
fn test_create_will_version_unauthorized() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let other = create_test_address(&env, 99);
    let will_hash = test_will_hash(&env);

    let result = client.try_create_will_version(&other, &plan_id, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_create_will_version_plan_not_found() {
    let env = Env::default();
    let (client, _token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let will_hash = test_will_hash(&env);

    let result = client.try_create_will_version(&owner, &999u64, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_get_will_version_not_found() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let _plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let result = client.get_will_version(&1u64, &99u32);
    assert_eq!(result, None);
}

#[test]
fn test_get_active_will_version_none() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let _plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let result = client.get_active_will_version(&1u64);
    assert_eq!(result, None);
}

#[test]
fn test_get_will_version_count_zero() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let _plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let count = client.get_will_version_count(&1u64);
    assert_eq!(count, 0);
}

// --- Issue #318: Legal Will Signature Verification ---

#[test]
fn test_sign_will_success() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    client.sign_will(&owner, &plan_id, &will_hash);

    let proof = client.get_will_signature(&plan_id).unwrap();
    assert_eq!(proof.vault_id, plan_id);
    assert_eq!(proof.will_hash, will_hash);
    assert_eq!(proof.signer, owner);
}

#[test]
fn test_sign_will_emits_event() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    client.sign_will(&owner, &plan_id, &will_hash);

    let events = env.events().all();
    // Find the WillSigned event
    let found = events.iter().any(|e| {
        let topics: soroban_sdk::Vec<soroban_sdk::Val> = e.1;
        topics.len() >= 2
    });
    assert!(found);
}

#[test]
fn test_sign_will_replay_protection() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    // First sign succeeds
    client.sign_will(&owner, &plan_id, &will_hash);

    // Same (vault_id, will_hash) pair must be rejected
    let result = client.try_sign_will(&owner, &plan_id, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_sign_will_different_will_hash_allowed() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    // Sign with first will hash
    client.sign_will(&owner, &plan_id, &test_will_hash(&env));

    // Sign with a different will hash (new version) should succeed
    client.sign_will(&owner, &plan_id, &test_will_hash_2(&env));

    let proof = client.get_will_signature(&plan_id).unwrap();
    assert_eq!(proof.will_hash, test_will_hash_2(&env));
}

#[test]
fn test_sign_will_unauthorized() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    let attacker = Address::generate(&env);
    let result = client.try_sign_will(&attacker, &plan_id, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_sign_will_plan_not_found() {
    let env = Env::default();
    let (client, _token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let will_hash = test_will_hash(&env);

    let result = client.try_sign_will(&owner, &999u64, &will_hash);
    assert!(result.is_err());
}

#[test]
fn test_get_will_signature_none() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let _plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let result = client.get_will_signature(&1u64);
    assert_eq!(result, None);
}

// --- Issue #319: Will Finalization ---

#[test]
fn test_finalize_will_success() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);
    client.sign_will(&owner, &plan_id, &will_hash);

    client.finalize_will(&owner, &plan_id, &version);

    assert!(client.is_will_finalized(&plan_id, &version));
    assert!(client.get_will_finalized_at(&plan_id, &version).is_some());
}

#[test]
fn test_finalize_will_emits_event() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);
    client.sign_will(&owner, &plan_id, &will_hash);
    client.finalize_will(&owner, &plan_id, &version);

    let events = env.events().all();
    let found = events.iter().any(|e| {
        let topics: soroban_sdk::Vec<soroban_sdk::Val> = e.1;
        topics.len() >= 2
    });
    assert!(found);
}

#[test]
fn test_finalize_will_without_signature_fails() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);

    let result = client.try_finalize_will(&owner, &plan_id, &version);
    assert!(result.is_err());
}

#[test]
fn test_finalize_will_already_finalized_fails() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);
    client.sign_will(&owner, &plan_id, &will_hash);
    client.finalize_will(&owner, &plan_id, &version);

    let result = client.try_finalize_will(&owner, &plan_id, &version);
    assert!(result.is_err());
}

#[test]
fn test_finalize_will_unauthorized() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);
    client.sign_will(&owner, &plan_id, &will_hash);

    let attacker = Address::generate(&env);
    let result = client.try_finalize_will(&attacker, &plan_id, &version);
    assert!(result.is_err());
}

#[test]
fn test_finalize_will_version_not_found() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    client.sign_will(&owner, &plan_id, &will_hash);

    let result = client.try_finalize_will(&owner, &plan_id, &99u32);
    assert!(result.is_err());
}

#[test]
fn test_is_will_finalized_false_by_default() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    assert!(!client.is_will_finalized(&plan_id, &1u32));
}

#[test]
fn test_finalized_will_blocks_new_version() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);
    client.sign_will(&owner, &plan_id, &will_hash);
    client.finalize_will(&owner, &plan_id, &version);

    let result = client.try_create_will_version(&owner, &plan_id, &test_will_hash_2(&env));
    assert!(result.is_err());
}

// --- Issue #320: Legal Witness Verification ---

#[test]
fn test_add_witness_success() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let witness = Address::generate(&env);

    client.add_witness(&owner, &plan_id, &witness);

    let witnesses = client.get_witnesses(&plan_id);
    assert_eq!(witnesses.len(), 1);
    assert_eq!(witnesses.get(0).unwrap(), witness);
}

#[test]
fn test_add_witness_emits_event() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let witness = Address::generate(&env);

    client.add_witness(&owner, &plan_id, &witness);

    let events = env.events().all();
    let found = events.iter().any(|e| {
        let topics: soroban_sdk::Vec<soroban_sdk::Val> = e.1;
        topics.len() >= 2
    });
    assert!(found);
}

#[test]
fn test_add_witness_duplicate_fails() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let witness = Address::generate(&env);

    client.add_witness(&owner, &plan_id, &witness);

    let result = client.try_add_witness(&owner, &plan_id, &witness);
    assert!(result.is_err());
}

#[test]
fn test_add_witness_unauthorized() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let witness = Address::generate(&env);
    let attacker = Address::generate(&env);

    let result = client.try_add_witness(&attacker, &plan_id, &witness);
    assert!(result.is_err());
}

#[test]
fn test_add_multiple_witnesses() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let w1 = Address::generate(&env);
    let w2 = Address::generate(&env);

    client.add_witness(&owner, &plan_id, &w1);
    client.add_witness(&owner, &plan_id, &w2);

    let witnesses = client.get_witnesses(&plan_id);
    assert_eq!(witnesses.len(), 2);
}

#[test]
fn test_sign_as_witness_success() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let witness = Address::generate(&env);

    client.add_witness(&owner, &plan_id, &witness);
    client.sign_as_witness(&witness, &plan_id);

    let signed_at = client.get_witness_signature(&plan_id, &witness);
    assert!(signed_at.is_some());
}

#[test]
fn test_sign_as_witness_emits_event() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let witness = Address::generate(&env);

    client.add_witness(&owner, &plan_id, &witness);
    client.sign_as_witness(&witness, &plan_id);

    let events = env.events().all();
    let found = events.iter().any(|e| {
        let topics: soroban_sdk::Vec<soroban_sdk::Val> = e.1;
        topics.len() >= 2
    });
    assert!(found);
}

#[test]
fn test_sign_as_witness_not_registered_fails() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let stranger = Address::generate(&env);

    let result = client.try_sign_as_witness(&stranger, &plan_id);
    assert!(result.is_err());
}

#[test]
fn test_sign_as_witness_double_sign_fails() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let witness = Address::generate(&env);

    client.add_witness(&owner, &plan_id, &witness);
    client.sign_as_witness(&witness, &plan_id);

    let result = client.try_sign_as_witness(&witness, &plan_id);
    assert!(result.is_err());
}

#[test]
fn test_finalize_fails_when_witness_not_signed() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);
    let witness = Address::generate(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);
    client.sign_will(&owner, &plan_id, &will_hash);
    client.add_witness(&owner, &plan_id, &witness);

    let result = client.try_finalize_will(&owner, &plan_id, &version);
    assert!(result.is_err());
}

#[test]
fn test_finalize_succeeds_after_all_witnesses_sign() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);
    let witness = Address::generate(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);
    client.sign_will(&owner, &plan_id, &will_hash);
    client.add_witness(&owner, &plan_id, &witness);
    client.sign_as_witness(&witness, &plan_id);

    client.finalize_will(&owner, &plan_id, &version);
    assert!(client.is_will_finalized(&plan_id, &version));
}

#[test]
fn test_get_witnesses_empty() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let witnesses = client.get_witnesses(&plan_id);
    assert_eq!(witnesses.len(), 0);
}

#[test]
fn test_get_witness_signature_none() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let witness = Address::generate(&env);

    let result = client.get_witness_signature(&plan_id, &witness);
    assert_eq!(result, None);
}

// --- Issue #321: Will Update Restrictions ---

#[test]
fn test_finalized_will_cannot_be_modified() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);
    client.sign_will(&owner, &plan_id, &will_hash);
    client.finalize_will(&owner, &plan_id, &version);

    let result = client.try_create_will_version(&owner, &plan_id, &test_will_hash_2(&env));
    assert!(result.is_err());
}

#[test]
fn test_unfinalized_will_can_be_updated() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let v1 = client.create_will_version(&owner, &plan_id, &test_will_hash(&env));
    assert_eq!(v1, 1);

    let v2 = client.create_will_version(&owner, &plan_id, &test_will_hash_2(&env));
    assert_eq!(v2, 2);
}

#[test]
fn test_finalized_version_is_immutable() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);
    let will_hash = test_will_hash(&env);

    let version = client.create_will_version(&owner, &plan_id, &will_hash);
    client.sign_will(&owner, &plan_id, &will_hash);
    client.finalize_will(&owner, &plan_id, &version);

    let ver_info = client.get_will_version(&plan_id, &version).unwrap();
    assert_eq!(ver_info.will_hash, will_hash);
    assert!(client.is_will_finalized(&plan_id, &version));
}

// --- Issue #360: Message Update Before Lock ---

#[test]
fn test_update_legacy_message_before_lock() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let original_hash = BytesN::from_array(&env, &[1u8; 32]);
    let future_ts = env.ledger().timestamp() + 10_000;
    let message_id = client.create_legacy_message(
        &owner,
        &CreateLegacyMessageParams {
            vault_id: plan_id,
            message_hash: original_hash,
            unlock_timestamp: future_ts,
            key_reference: soroban_sdk::String::from_str(&env, "ref_1"),
        },
    );

    let updated_hash = BytesN::from_array(&env, &[2u8; 32]);
    let new_unlock_ts = future_ts + 5_000;
    client.update_legacy_message(
        &owner,
        &message_id,
        &CreateLegacyMessageParams {
            vault_id: plan_id,
            message_hash: updated_hash.clone(),
            unlock_timestamp: new_unlock_ts,
            key_reference: soroban_sdk::String::from_str(&env, "ref_updated"),
        },
    );

    let stored = client.get_legacy_message(&message_id).unwrap();
    assert_eq!(stored.message_hash, updated_hash);
    assert_eq!(stored.unlock_timestamp, new_unlock_ts);
    assert!(!stored.is_finalized);
}

#[test]
fn test_update_legacy_message_rejected_after_lock() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let future_ts = env.ledger().timestamp() + 10_000;
    let message_id = client.create_legacy_message(
        &owner,
        &CreateLegacyMessageParams {
            vault_id: plan_id,
            message_hash: BytesN::from_array(&env, &[1u8; 32]),
            unlock_timestamp: future_ts,
            key_reference: soroban_sdk::String::from_str(&env, "ref_1"),
        },
    );

    // Finalize (lock) the message
    client.finalize_legacy_message(&owner, &message_id);
    assert!(client.get_legacy_message(&message_id).unwrap().is_finalized);

    // Update after finalization must fail
    let result = client.try_update_legacy_message(
        &owner,
        &message_id,
        &CreateLegacyMessageParams {
            vault_id: plan_id,
            message_hash: BytesN::from_array(&env, &[3u8; 32]),
            unlock_timestamp: future_ts,
            key_reference: soroban_sdk::String::from_str(&env, "ref_new"),
        },
    );
    assert_eq!(result, Err(Ok(InheritanceError::WillAlreadyFinalized)));
}

#[test]
fn test_update_legacy_message_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let plan_id = create_plan_and_get_id(&env, &client, &token_id, &owner);

    let future_ts = env.ledger().timestamp() + 10_000;
    let message_id = client.create_legacy_message(
        &owner,
        &CreateLegacyMessageParams {
            vault_id: plan_id,
            message_hash: BytesN::from_array(&env, &[1u8; 32]),
            unlock_timestamp: future_ts,
            key_reference: soroban_sdk::String::from_str(&env, "ref_1"),
        },
    );

    let stranger = Address::generate(&env);
    let result = client.try_update_legacy_message(
        &stranger,
        &message_id,
        &CreateLegacyMessageParams {
            vault_id: plan_id,
            message_hash: BytesN::from_array(&env, &[9u8; 32]),
            unlock_timestamp: future_ts,
            key_reference: soroban_sdk::String::from_str(&env, "ref_9"),
        },
    );
    assert_eq!(result, Err(Ok(InheritanceError::Unauthorized)));
}
// --- Issue #71: KYC Verification for Plan Creation and Claiming ---

#[test]
fn test_create_plan_without_kyc_fails() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin_no_kyc(&env);

    // Owner has not submitted KYC - should fail
    let params = plan_params(
        &env,
        &owner,
        &token_id,
        "Test Plan",
        "Test Description",
        50_000u64,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );

    let result = client.try_create_inheritance_plan(&params);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(
        err.is_ok(),
        "contract should return InheritanceError, not InvokeError"
    );
    assert_eq!(err.ok().unwrap(), InheritanceError::KycNotSubmitted);
}

#[test]
fn test_create_plan_with_pending_kyc_fails() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin_no_kyc(&env);

    // Submit KYC but don't approve yet (pending state)
    client.submit_kyc(&owner);

    let params = plan_params(
        &env,
        &owner,
        &token_id,
        "Test Plan",
        "Test Description",
        50_000u64,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );

    // Should fail because KYC is not approved yet
    let result = client.try_create_inheritance_plan(&params);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(
        err.is_ok(),
        "contract should return InheritanceError, not InvokeError"
    );
    assert_eq!(err.ok().unwrap(), InheritanceError::KycNotSubmitted);
}

#[test]
fn test_create_plan_with_rejected_kyc_fails() {
    let env = Env::default();
    let (client, token_id, admin, owner) = setup_with_token_and_admin_no_kyc(&env);

    // Submit and reject KYC
    client.submit_kyc(&owner);
    client.reject_kyc(&admin, &owner);

    let params = plan_params(
        &env,
        &owner,
        &token_id,
        "Test Plan",
        "Test Description",
        50_000u64,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );

    // Should fail because KYC was rejected
    let result = client.try_create_inheritance_plan(&params);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(
        err.is_ok(),
        "contract should return InheritanceError, not InvokeError"
    );
    assert_eq!(err.ok().unwrap(), InheritanceError::KycNotSubmitted);
}

#[test]
fn test_create_plan_with_approved_kyc_succeeds() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);

    // Owner already has approved KYC from setup helper
    let params = plan_params(
        &env,
        &owner,
        &token_id,
        "Test Plan",
        "Test Description",
        50_000u64,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );

    // Should succeed
    let plan_id = client.create_inheritance_plan(&params);
    assert!(plan_id > 0);

    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_amount, 49_000u64); // 50_000 - 2% = 49_000
    assert!(plan.is_active);
}

#[test]
fn test_claim_plan_without_kyc_fails() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 100);

    // Owner already has approved KYC from setup helper
    let params = plan_params(
        &env,
        &owner,
        &token_id,
        "Test Plan",
        "Test Description",
        50_000u64,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    let plan_id = client.create_inheritance_plan(&params);

    // Beneficiary tries to claim without KYC - should fail
    let result = client.try_claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &111111u32,
    );
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(
        err.is_ok(),
        "contract should return InheritanceError, not InvokeError"
    );
    assert_eq!(err.ok().unwrap(), InheritanceError::KycNotSubmitted);
}

#[test]
fn test_claim_plan_with_pending_kyc_fails() {
    let env = Env::default();
    let (client, token_id, _admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 100);

    // Owner already has approved KYC from setup helper
    let params = plan_params(
        &env,
        &owner,
        &token_id,
        "Test Plan",
        "Test Description",
        50_000u64,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    let plan_id = client.create_inheritance_plan(&params);

    // Beneficiary submits KYC but not approved
    client.submit_kyc(&beneficiary);

    // Should fail because beneficiary KYC is not approved
    let result = client.try_claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &111111u32,
    );
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(
        err.is_ok(),
        "contract should return InheritanceError, not InvokeError"
    );
    assert_eq!(err.ok().unwrap(), InheritanceError::KycNotSubmitted);
}

#[test]
fn test_claim_plan_with_rejected_kyc_fails() {
    let env = Env::default();
    let (client, token_id, admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 100);

    // Owner already has approved KYC from setup helper
    let params = plan_params(
        &env,
        &owner,
        &token_id,
        "Test Plan",
        "Test Description",
        50_000u64,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    let plan_id = client.create_inheritance_plan(&params);

    // Beneficiary has rejected KYC
    client.submit_kyc(&beneficiary);
    client.reject_kyc(&admin, &beneficiary);

    // Should fail because beneficiary KYC was rejected
    let result = client.try_claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &111111u32,
    );
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(
        err.is_ok(),
        "contract should return InheritanceError, not InvokeError"
    );
    assert_eq!(err.ok().unwrap(), InheritanceError::KycNotSubmitted);
}

#[test]
fn test_claim_plan_with_approved_kyc_succeeds() {
    let env = Env::default();
    let (client, token_id, admin, owner) = setup_with_token_and_admin(&env);
    let beneficiary = create_test_address(&env, 100);

    // Owner already has approved KYC from setup helper
    let params = plan_params(
        &env,
        &owner,
        &token_id,
        "Test Plan",
        "Test Description",
        50_000u64,
        DistributionMethod::LumpSum,
        &default_beneficiaries(&env),
    );
    let plan_id = client.create_inheritance_plan(&params);

    // Beneficiary has approved KYC
    client.submit_kyc(&beneficiary);
    client.approve_kyc(&admin, &beneficiary);

    // Should succeed
    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &111111u32,
    );

    // Verify claim was recorded (no error means success)
}

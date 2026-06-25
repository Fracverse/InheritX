#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String, Vec};

#[test]
fn test_contract_compilation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let _client = InheritanceContractClient::new(&env, &contract_id);
}

#[test]
fn test_create_plan_success() {
    let env = Env::default();
    env.mock_all_auths();

    // Register our contract
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    // Register mock token contract
    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary_address = Address::generate(&env);

    // Mint tokens to owner
    token_client.mint(&owner, &2000);

    let beneficiary = Beneficiary {
        address: beneficiary_address.clone(),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "NGN_BANK"),
    };

    client.create_plan(
        &owner,
        &token_id,
        &1500,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
    );

    // Verify balances
    assert_eq!(token_client.balance(&owner), 500);
    assert_eq!(token_client.balance(&contract_id), 1500);

    // Verify stored plan
    let plan = client.get_plan(&owner);
    assert_eq!(plan.owner, owner);
    assert_eq!(plan.token, token_id);
    assert_eq!(plan.amount, 1500);
    assert_eq!(plan.grace_period, 3600);
    assert!(plan.earn_yield);
    assert_eq!(plan.yield_rate_bps, 500);
    assert!(plan.is_active);
    assert_eq!(plan.beneficiaries.len(), 1);
    assert_eq!(
        plan.beneficiaries.get(0).unwrap().address,
        beneficiary_address
    );
    assert_eq!(plan.beneficiaries.get(0).unwrap().allocation_bps, 10000);
}

#[test]
fn test_create_plan_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    token_client.mint(&owner, &1000);

    let beneficiary = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "NGN_BANK"),
    };

    // Attempting to create plan for 1500 (owner only has 1000)
    let result = client.try_create_plan(
        &owner,
        &token_id,
        &1500,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
    );

    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_create_plan_negative_or_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    token_client.mint(&owner, &1000);

    let beneficiary = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "NGN_BANK"),
    };

    // Amount = 0
    let result_zero = client.try_create_plan(
        &owner,
        &token_id,
        &0,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
    );
    assert_eq!(result_zero, Err(Ok(Error::NegativeAmount)));

    // Amount = -10
    let result_neg = client.try_create_plan(
        &owner,
        &token_id,
        &-10,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
    );
    assert_eq!(result_neg, Err(Ok(Error::NegativeAmount)));
}

#[test]
fn test_create_plan_invalid_basis_points() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    token_client.mint(&owner, &1000);

    let beneficiary1 = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 4000,
        fiat_anchor_info: String::from_str(&env, "NGN_BANK"),
    };

    let beneficiary2 = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 5000, // Total = 9000 BPS (less than 10000)
        fiat_anchor_info: String::from_str(&env, "NGN_BANK"),
    };

    let result = client.try_create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [beneficiary1, beneficiary2]),
        &3600,
        &true,
        &500,
    );

    assert_eq!(result, Err(Ok(Error::InvalidBasisPoints)));
}

#[test]
fn test_create_plan_already_exists() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    token_client.mint(&owner, &2000);

    let beneficiary = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "NGN_BANK"),
    };

    // First creation
    client.create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
    );

    // Second creation on same owner
    let result2 = client.try_create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
    );
    assert_eq!(result2, Err(Ok(Error::PlanAlreadyExists)));
}

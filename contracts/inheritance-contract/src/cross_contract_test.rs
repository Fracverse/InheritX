#![cfg(test)]

use crate::{InheritanceContract, InheritanceContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_set_and_get_contracts() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    let lending_contract = Address::generate(&env);
    let governance_contract = Address::generate(&env);

    client.set_lending_contract(&admin, &lending_contract);
    assert_eq!(client.get_lending_contract(), Some(lending_contract.clone()));

    client.set_governance_contract(&admin, &governance_contract);
    assert_eq!(client.get_governance_contract(), Some(governance_contract.clone()));
}

#[test]
fn test_set_lending_contract_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Try setting with a non-admin
    let non_admin = Address::generate(&env);
    let lending_contract = Address::generate(&env);
    let result = client.try_set_lending_contract(&non_admin, &lending_contract);
    assert!(result.is_err());
}

#[test]
fn test_verify_plan_ownership_no_plan() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // No plan created, should return false
    let any_user = Address::generate(&env);
    assert!(!client.verify_plan_ownership(&999_u64, &any_user));
}

#[test]
fn test_get_lending_contract_returns_none_when_unset() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Not set yet
    assert_eq!(client.get_lending_contract(), None);
    assert_eq!(client.get_governance_contract(), None);
}

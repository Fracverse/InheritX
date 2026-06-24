#![cfg(test)]

use super::*;
use soroban_sdk::{Address, Env};

#[test]
fn test_contract_compilation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let _client = InheritanceContractClient::new(&env, &contract_id);
}

#[test]
fn test_plan_storage() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);
    
    let owner = Address::generate(&env);
    let token = Address::generate(&env);
    let beneficiary = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "test"),
    };
    
    let result = client.create_plan(
        &owner,
        &token,
        &1000,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &100,
        &true,
        &500,
    );
    
    assert!(result.is_ok());
    
    let plan = client.get_plan(&owner);
    assert!(plan.is_ok());
}

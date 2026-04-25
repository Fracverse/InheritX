#![cfg(test)]

use crate::{LendingContract, LendingContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_set_and_get_contracts() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    client.initialize(&admin, &token, &1000, &1500, &15000, &8000);

    let inheritance_contract = Address::generate(&env);
    let governance_contract = Address::generate(&env);

    client.set_inheritance_contract(&admin, &inheritance_contract);
    assert_eq!(
        client.get_inheritance_contract(),
        Some(inheritance_contract.clone())
    );

    client.set_governance_contract(&admin, &governance_contract);
    assert_eq!(
        client.get_governance_contract(),
        Some(governance_contract.clone())
    );
}

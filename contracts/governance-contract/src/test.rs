#![cfg(test)]
use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

#[test]
fn test_governance_flow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // Initialize
    client.initialize(&admin, &500, &15000, &500);

    assert_eq!(client.get_interest_rate(), 500);
    assert_eq!(client.get_collateral_ratio(), 15000);
    assert_eq!(client.get_liquidation_bonus(), 500);
    assert_eq!(client.get_admin(), admin);

    // Update parameters
    env.mock_all_auths();

    client.update_interest_rate(&600);
    assert_eq!(client.get_interest_rate(), 600);

    client.update_collateral_ratio(&16000);
    assert_eq!(client.get_collateral_ratio(), 16000);

    client.update_liquidation_bonus(&700);
    assert_eq!(client.get_liquidation_bonus(), 700);
}

#[test]
#[should_panic]
fn test_unauthorized_update() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin, &500, &15000, &500);

    // Call without mock_all_auths and without providing admin auth
    // This should panic due to require_auth failing
    client.update_interest_rate(&600);
}

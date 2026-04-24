#![cfg(test)]
use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

#[test]
fn test_delegation_flow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);
    client.set_token_balance(&delegate, &500);

    assert_eq!(client.get_delegate(&delegator), None);
    assert_eq!(client.get_delegators(&delegate).len(), 0);

    env.mock_all_auths();
    client.delegate_votes(&delegator, &delegate);

    assert_eq!(client.get_delegate(&delegator), Some(delegate.clone()));
    assert_eq!(client.get_delegators(&delegate).len(), 1);
    assert_eq!(client.get_delegators(&delegate).get(0).unwrap(), delegator);

    assert_eq!(client.get_voting_power(&delegate), 1500);
    assert_eq!(client.get_voting_power(&delegator), 0);
}

#[test]
fn test_undelegation_flow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);
    client.set_token_balance(&delegate, &500);

    env.mock_all_auths();
    client.delegate_votes(&delegator, &delegate);

    assert_eq!(client.get_delegate(&delegator), Some(delegate.clone()));
    assert_eq!(client.get_voting_power(&delegate), 1500);

    client.undelegate_votes(&delegator);

    assert_eq!(client.get_delegate(&delegator), None);
    assert_eq!(client.get_delegators(&delegate).len(), 0);

    assert_eq!(client.get_voting_power(&delegate), 500);
    assert_eq!(client.get_voting_power(&delegator), 1000);
}

#[test]
fn test_delegate_votes_with_aggregated_power() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator1 = Address::generate(&env);
    let delegator2 = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator1, &1000);
    client.set_token_balance(&delegator2, &2000);
    client.set_token_balance(&delegate, &500);

    env.mock_all_auths();
    client.delegate_votes(&delegator1, &delegate);
    client.delegate_votes(&delegator2, &delegate);

    assert_eq!(client.get_delegators(&delegate).len(), 2);
    assert_eq!(client.get_voting_power(&delegate), 3500);

    let proposal_id = 1u32;
    client.vote(&delegate, &proposal_id, &3500);

    assert_eq!(client.get_proposal_votes(&proposal_id), 3500);
}

#[test]
fn test_self_delegation_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let user = Address::generate(&env);
    client.set_token_balance(&user, &1000);

    env.mock_all_auths();
    let result = client.try_delegate_votes(&user, &user);
    assert!(result.is_err());
}

#[test]
fn test_circular_delegation_prevention() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let user_c = Address::generate(&env);

    client.set_token_balance(&user_a, &1000);
    client.set_token_balance(&user_b, &1000);
    client.set_token_balance(&user_c, &1000);

    env.mock_all_auths();

    client.delegate_votes(&user_a, &user_b);
    client.delegate_votes(&user_b, &user_c);

    let result = client.try_delegate_votes(&user_c, &user_a);
    assert!(result.is_err());
}

#[test]
fn test_circular_delegation_direct() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);

    client.set_token_balance(&user_a, &1000);
    client.set_token_balance(&user_b, &1000);

    env.mock_all_auths();

    client.delegate_votes(&user_a, &user_b);

    let result = client.try_delegate_votes(&user_b, &user_a);
    assert!(result.is_err());
}

#[test]
fn test_multiple_delegators_to_one_delegate() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator1 = Address::generate(&env);
    let delegator2 = Address::generate(&env);
    let delegator3 = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator1, &1000);
    client.set_token_balance(&delegator2, &2000);
    client.set_token_balance(&delegator3, &3000);
    client.set_token_balance(&delegate, &500);

    env.mock_all_auths();
    client.delegate_votes(&delegator1, &delegate);
    client.delegate_votes(&delegator2, &delegate);
    client.delegate_votes(&delegator3, &delegate);

    let delegators = client.get_delegators(&delegate);
    assert_eq!(delegators.len(), 3);

    assert_eq!(client.get_voting_power(&delegate), 6500);

    assert_eq!(client.get_voting_power(&delegator1), 0);
    assert_eq!(client.get_voting_power(&delegator2), 0);
    assert_eq!(client.get_voting_power(&delegator3), 0);
}

#[test]
fn test_delegation_overwrite() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator = Address::generate(&env);
    let delegate1 = Address::generate(&env);
    let delegate2 = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);
    client.set_token_balance(&delegate1, &500);
    client.set_token_balance(&delegate2, &600);

    env.mock_all_auths();
    client.delegate_votes(&delegator, &delegate1);

    assert_eq!(client.get_delegate(&delegator), Some(delegate1.clone()));
    assert_eq!(client.get_delegators(&delegate1).len(), 1);
    assert_eq!(client.get_delegators(&delegate2).len(), 0);
    assert_eq!(client.get_voting_power(&delegate1), 1500);
    assert_eq!(client.get_voting_power(&delegator), 0);

    client.delegate_votes(&delegator, &delegate2);

    assert_eq!(client.get_delegate(&delegator), Some(delegate2.clone()));
    assert_eq!(client.get_delegators(&delegate1).len(), 0);
    assert_eq!(client.get_delegators(&delegate2).len(), 1);
    assert_eq!(client.get_voting_power(&delegate2), 1600);
    assert_eq!(client.get_voting_power(&delegate1), 500);
}

#[test]
fn test_delegator_cannot_vote_when_delegated() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);
    client.set_token_balance(&delegate, &500);

    env.mock_all_auths();
    client.delegate_votes(&delegator, &delegate);

    let result = client.try_vote(&delegator, &1u32, &1000);
    assert!(result.is_err());
}

#[test]
fn test_delegation_history_tracking() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator = Address::generate(&env);
    let delegate1 = Address::generate(&env);
    let delegate2 = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);

    env.mock_all_auths();
    client.delegate_votes(&delegator, &delegate1);

    let history = client.get_delegation_history();
    assert_eq!(history.len(), 1);

    client.delegate_votes(&delegator, &delegate2);

    let history = client.get_delegation_history();
    assert_eq!(history.len(), 2);

    client.undelegate_votes(&delegator);

    let history = client.get_delegation_history();
    assert_eq!(history.len(), 3);
}

#[test]
fn test_voting_integrity_no_double_counting() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator1 = Address::generate(&env);
    let delegator2 = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator1, &1000);
    client.set_token_balance(&delegator2, &2000);
    client.set_token_balance(&delegate, &500);

    env.mock_all_auths();
    client.delegate_votes(&delegator1, &delegate);
    client.delegate_votes(&delegator2, &delegate);

    let total_voting_power = client.get_voting_power(&delegate);
    assert_eq!(total_voting_power, 3500);

    let delegator1_power = client.get_voting_power(&delegator1);
    let delegator2_power = client.get_voting_power(&delegator2);
    let delegate_power = client.get_voting_power(&delegate);

    let sum_of_all_powers = delegator1_power + delegator2_power + delegate_power;
    assert_eq!(sum_of_all_powers, 3500);

    let proposal_id = 1u32;
    client.vote(&delegate, &proposal_id, &3500);

    let total_proposal_votes = client.get_proposal_votes(&proposal_id);
    assert_eq!(total_proposal_votes, 3500);
}

#[test]
fn test_undelegate_then_vote() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);

    env.mock_all_auths();
    client.delegate_votes(&delegator, &delegate);

    assert_eq!(client.get_voting_power(&delegator), 0);

    client.undelegate_votes(&delegator);

    client.vote(&delegator, &1u32, &1000);

    assert_eq!(client.get_proposal_votes(&1u32), 1000);
}

#[test]
fn test_no_double_voting() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let voter = Address::generate(&env);

    client.set_token_balance(&voter, &1000);

    env.mock_all_auths();
    client.vote(&voter, &1u32, &500);

    let result = client.try_vote(&voter, &1u32, &300);
    assert!(result.is_err());

    assert!(client.has_voted(&voter, &1u32));
}

#[test]
fn test_vote_with_exact_voting_power() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);
    client.set_token_balance(&delegate, &500);

    env.mock_all_auths();
    client.delegate_votes(&delegator, &delegate);

    client.vote(&delegate, &1u32, &1500);

    assert_eq!(client.get_proposal_votes(&1u32), 1500);
}

#[test]
fn test_vote_exceeds_voting_power_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let voter = Address::generate(&env);

    client.set_token_balance(&voter, &1000);

    env.mock_all_auths();
    let result = client.try_vote(&voter, &1u32, &1500);
    assert!(result.is_err());
}

#[test]
fn test_undelegate_without_delegation_fails() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let user = Address::generate(&env);

    env.mock_all_auths();
    let result = client.try_undelegate_votes(&user);
    assert!(result.is_err());
}

#[test]
fn test_delegate_without_balance() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegate, &500);

    env.mock_all_auths();
    client.delegate_votes(&delegator, &delegate);

    assert_eq!(client.get_voting_power(&delegate), 500);
    assert_eq!(client.get_voting_power(&delegator), 0);
}

#[test]
fn test_chain_delegation_depth_prevention() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &500, &15000, &500);

    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let user_c = Address::generate(&env);
    let user_d = Address::generate(&env);

    client.set_token_balance(&user_a, &100);
    client.set_token_balance(&user_b, &100);
    client.set_token_balance(&user_c, &100);
    client.set_token_balance(&user_d, &100);

    env.mock_all_auths();

    client.delegate_votes(&user_a, &user_b);
    client.delegate_votes(&user_b, &user_c);
    client.delegate_votes(&user_c, &user_d);

    let result = client.try_delegate_votes(&user_d, &user_a);
    assert!(result.is_err());
}

#[test]
fn test_governance_flow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin, &500, &15000, &500);

    assert_eq!(client.get_interest_rate(), 500);
    assert_eq!(client.get_collateral_ratio(), 15000);
    assert_eq!(client.get_liquidation_bonus(), 500);
    assert_eq!(client.get_admin(), admin);

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

    client.update_interest_rate(&600);
}

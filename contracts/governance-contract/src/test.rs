#![cfg(test)]
use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Env, String};

// ─────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────

fn setup_contract(env: &Env) -> (GovernanceContractClient<'_>, Address) {
    let contract_id = env.register_contract(None, GovernanceContract);
    let client = GovernanceContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin, &500, &15000, &500);
    (client, admin)
}

fn make_proposal(env: &Env, client: &GovernanceContractClient, proposer: &Address) -> u32 {
    client.create_proposal(
        proposer,
        &String::from_str(env, "Test Proposal"),
        &String::from_str(env, "A test governance proposal"),
    )
}

// ─────────────────────────────────────────────────
// Delegation Tests
// ─────────────────────────────────────────────────

#[test]
fn test_delegation_flow() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

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
    let (client, _admin) = setup_contract(&env);

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
    let (client, _admin) = setup_contract(&env);

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

    let proposal_id = make_proposal(&env, &client, &delegate);
    client.vote(&delegate, &proposal_id, &VoteChoice::Yes);

    assert_eq!(client.get_proposal_votes(&proposal_id), 3500);
}

#[test]
fn test_self_delegation_fails() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let user = Address::generate(&env);
    client.set_token_balance(&user, &1000);

    env.mock_all_auths();
    let result = client.try_delegate_votes(&user, &user);
    assert!(result.is_err());
}

#[test]
fn test_circular_delegation_prevention() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

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
    let (client, _admin) = setup_contract(&env);

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
    let (client, _admin) = setup_contract(&env);

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
    let (client, _admin) = setup_contract(&env);

    let delegator = Address::generate(&env);
    let delegate1 = Address::generate(&env);
    let delegate2 = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);
    client.set_token_balance(&delegate1, &500);

    env.mock_all_auths();
    client.delegate_votes(&delegator, &delegate1);
    assert_eq!(client.get_delegators(&delegate1).len(), 1);

    client.delegate_votes(&delegator, &delegate2);
    assert_eq!(client.get_delegators(&delegate1).len(), 0);
    assert_eq!(client.get_delegators(&delegate2).len(), 1);
}

#[test]
fn test_delegator_cannot_vote_when_delegated() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);
    client.set_token_balance(&delegate, &500);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);
    client.delegate_votes(&delegator, &delegate);

    // Delegator has given away their vote — they cannot vote directly
    let result = client.try_vote(&delegator, &proposal_id, &VoteChoice::Yes);
    assert!(result.is_err());
}

#[test]
fn test_delegation_history_tracking() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

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
    let (client, _admin) = setup_contract(&env);

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

    // Total power is consolidated in delegate; delegators show 0
    let sum_of_all_powers = delegator1_power + delegator2_power + delegate_power;
    assert_eq!(sum_of_all_powers, 3500);

    let proposal_id = make_proposal(&env, &client, &delegate);
    client.vote(&delegate, &proposal_id, &VoteChoice::Yes);

    let total_proposal_votes = client.get_proposal_votes(&proposal_id);
    assert_eq!(total_proposal_votes, 3500);
}

#[test]
fn test_undelegate_then_vote() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &delegator);
    client.delegate_votes(&delegator, &delegate);

    assert_eq!(client.get_voting_power(&delegator), 0);

    client.undelegate_votes(&delegator);

    // After undelegation, voting power is restored and vote can be cast
    client.vote(&delegator, &proposal_id, &VoteChoice::Yes);

    assert_eq!(client.get_proposal_votes(&proposal_id), 1000);
}

#[test]
fn test_no_double_voting() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let voter = Address::generate(&env);

    client.set_token_balance(&voter, &1000);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &voter);
    client.vote(&voter, &proposal_id, &VoteChoice::Yes);

    // Second vote on the same proposal must fail
    let result = client.try_vote(&voter, &proposal_id, &VoteChoice::No);
    assert!(result.is_err());

    assert!(client.has_voted(&voter, &proposal_id));
}

#[test]
fn test_vote_with_exact_voting_power() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    client.set_token_balance(&delegator, &1000);
    client.set_token_balance(&delegate, &500);

    env.mock_all_auths();
    client.delegate_votes(&delegator, &delegate);

    let proposal_id = make_proposal(&env, &client, &delegate);
    client.vote(&delegate, &proposal_id, &VoteChoice::Yes);

    assert_eq!(client.get_proposal_votes(&proposal_id), 1500);
}

#[test]
fn test_vote_with_zero_power_fails() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    // voter has no token balance and no delegated votes
    let voter_no_power = Address::generate(&env);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);

    let result = client.try_vote(&voter_no_power, &proposal_id, &VoteChoice::Yes);
    assert!(result.is_err());
}

#[test]
fn test_undelegate_without_delegation_fails() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let user = Address::generate(&env);

    env.mock_all_auths();
    let result = client.try_undelegate_votes(&user);
    assert!(result.is_err());
}

#[test]
fn test_delegate_without_balance() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

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
    let (client, _admin) = setup_contract(&env);

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
    let (client, admin) = setup_contract(&env);

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
    let (client, _admin) = setup_contract(&env);
    client.update_interest_rate(&600);
}

// ─────────────────────────────────────────────────
// Proposal Governance Tests
// ─────────────────────────────────────────────────

#[test]
fn test_create_proposal() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    env.mock_all_auths();
    let proposal_id = client.create_proposal(
        &admin,
        &String::from_str(&env, "Adjust Interest Rate"),
        &String::from_str(&env, "Proposal to change the base interest rate to 800 bps"),
    );

    assert_eq!(proposal_id, 1u32);

    let proposal = client
        .get_proposal(&proposal_id)
        .expect("Proposal must exist");
    assert_eq!(proposal.id, 1u32);
    assert_eq!(proposal.proposer, admin);
    assert_eq!(proposal.yes_votes, 0);
    assert_eq!(proposal.no_votes, 0);
    assert_eq!(proposal.abstain_votes, 0);
    assert_eq!(proposal.status, ProposalStatus::Active);
}

#[test]
fn test_proposal_ids_increment() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    env.mock_all_auths();
    let id1 = make_proposal(&env, &client, &admin);
    let id2 = make_proposal(&env, &client, &admin);
    let id3 = make_proposal(&env, &client, &admin);

    assert_eq!(id1, 1u32);
    assert_eq!(id2, 2u32);
    assert_eq!(id3, 3u32);
}

#[test]
fn test_vote_yes_no_abstain() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    let voter_yes = Address::generate(&env);
    let voter_no = Address::generate(&env);
    let voter_abstain = Address::generate(&env);

    client.set_token_balance(&voter_yes, &1000);
    client.set_token_balance(&voter_no, &500);
    client.set_token_balance(&voter_abstain, &250);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);

    client.vote(&voter_yes, &proposal_id, &VoteChoice::Yes);
    client.vote(&voter_no, &proposal_id, &VoteChoice::No);
    client.vote(&voter_abstain, &proposal_id, &VoteChoice::Abstain);

    let counts = client.get_vote_count(&proposal_id);
    assert_eq!(counts.yes_votes, 1000);
    assert_eq!(counts.no_votes, 500);
    assert_eq!(counts.abstain_votes, 250);
}

#[test]
fn test_get_user_vote() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    let voter = Address::generate(&env);
    client.set_token_balance(&voter, &1000);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);

    // Before voting — no vote recorded
    assert_eq!(client.get_user_vote(&voter, &proposal_id), None);

    client.vote(&voter, &proposal_id, &VoteChoice::No);

    assert_eq!(
        client.get_user_vote(&voter, &proposal_id),
        Some(VoteChoice::No)
    );
}

#[test]
fn test_get_proposal_status_active() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);

    // Voting period has not ended
    let status = client.get_proposal_status(&proposal_id);
    assert_eq!(status, ProposalStatus::Active);
}

#[test]
fn test_execute_proposal_after_voting_period() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    let voter = Address::generate(&env);
    client.set_token_balance(&voter, &1000);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);
    client.vote(&voter, &proposal_id, &VoteChoice::Yes);

    // Advance ledger timestamp past the proposal expiry (7 days + 1 second)
    env.ledger().with_mut(|li| {
        li.timestamp = 604_801;
    });

    let status = client.get_proposal_status(&proposal_id);
    assert_eq!(status, ProposalStatus::Passed);

    client.execute_proposal(&admin, &proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Executed);
}

#[test]
fn test_execute_rejected_proposal_fails() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    let voter = Address::generate(&env);
    client.set_token_balance(&voter, &1000);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);
    client.vote(&voter, &proposal_id, &VoteChoice::No);

    // Advance past expiry
    env.ledger().with_mut(|li| {
        li.timestamp = 604_801;
    });

    let status = client.get_proposal_status(&proposal_id);
    assert_eq!(status, ProposalStatus::Rejected);

    let result = client.try_execute_proposal(&admin, &proposal_id);
    assert!(result.is_err());
}

#[test]
fn test_cancel_proposal() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);

    client.cancel_proposal(&admin, &proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Cancelled);
}

#[test]
fn test_cancel_proposal_non_proposer_fails() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    let other = Address::generate(&env);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);

    let result = client.try_cancel_proposal(&other, &proposal_id);
    assert!(result.is_err());
}

#[test]
fn test_vote_on_cancelled_proposal_fails() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    let voter = Address::generate(&env);
    client.set_token_balance(&voter, &1000);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);
    client.cancel_proposal(&admin, &proposal_id);

    let result = client.try_vote(&voter, &proposal_id, &VoteChoice::Yes);
    assert!(result.is_err());
}

#[test]
fn test_quorum_not_met_rejects_proposal() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);

    // No votes cast — quorum not met
    env.ledger().with_mut(|li| {
        li.timestamp = 604_801;
    });

    let status = client.get_proposal_status(&proposal_id);
    assert_eq!(status, ProposalStatus::Rejected);
}

#[test]
fn test_vote_on_nonexistent_proposal_fails() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let voter = Address::generate(&env);
    client.set_token_balance(&voter, &1000);

    env.mock_all_auths();
    let result = client.try_vote(&voter, &99u32, &VoteChoice::Yes);
    assert!(result.is_err());
}

#[test]
fn test_has_voted_reflects_state() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    let voter = Address::generate(&env);
    client.set_token_balance(&voter, &500);

    env.mock_all_auths();
    let proposal_id = make_proposal(&env, &client, &admin);

    assert!(!client.has_voted(&voter, &proposal_id));

    client.vote(&voter, &proposal_id, &VoteChoice::Abstain);

    assert!(client.has_voted(&voter, &proposal_id));
}

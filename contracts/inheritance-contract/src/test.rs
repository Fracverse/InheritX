#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::testutils::Ledger;
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

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary_address = Address::generate(&env);

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
        &Vec::new(&env),
        &0,
    );

    assert_eq!(token_client.balance(&owner), 500);
    assert_eq!(token_client.balance(&contract_id), 1500);

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

    let result = client.try_create_plan(
        &owner,
        &token_id,
        &1500,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
        &Vec::new(&env),
        &0,
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

    let result_zero = client.try_create_plan(
        &owner,
        &token_id,
        &0,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
        &Vec::new(&env),
        &0,
    );
    assert_eq!(result_zero, Err(Ok(Error::NegativeAmount)));

    let result_neg = client.try_create_plan(
        &owner,
        &token_id,
        &-10,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
        &Vec::new(&env),
        &0,
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
        allocation_bps: 5000,
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
        &Vec::new(&env),
        &0,
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

    client.create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
        &Vec::new(&env),
        &0,
    );

    let result2 = client.try_create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
        &Vec::new(&env),
        &0,
    );
    assert_eq!(result2, Err(Ok(Error::PlanAlreadyExists)));
}

#[test]
fn test_trigger_payout_single_beneficiary() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    token_client.mint(&owner, &2000);

    let b = Beneficiary {
        address: beneficiary.clone(),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
    };

    let start = 1_000_000;
    env.ledger().set_timestamp(start);

    client.create_plan(
        &owner,
        &token_id,
        &1500,
        &Vec::from_array(&env, [b]),
        &3600,
        &true,
        &500,
        &Vec::new(&env),
        &0,
    );

    client.close_plan(&owner);
    env.ledger().set_timestamp(start + 4000);

    client.trigger_payout(&owner);

    assert_eq!(token_client.balance(&beneficiary), 1500);
    assert_eq!(token_client.balance(&contract_id), 0);

    let result = client.try_get_plan(&owner);
    assert_eq!(result, Err(Ok(Error::PlanNotFound)));
}

#[test]
fn test_trigger_payout_multiple_beneficiaries() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    token_client.mint(&owner, &5000);

    let alice_bene = Beneficiary {
        address: alice.clone(),
        allocation_bps: 5000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
    };
    let bob_bene = Beneficiary {
        address: bob.clone(),
        allocation_bps: 3000,
        fiat_anchor_info: String::from_str(&env, "EUR_BANK"),
    };
    let charlie_bene = Beneficiary {
        address: charlie.clone(),
        allocation_bps: 2000,
        fiat_anchor_info: String::from_str(&env, "GBP_BANK"),
    };

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &1000,
        &Vec::from_array(&env, [alice_bene, bob_bene, charlie_bene]),
        &3600,
        &true,
        &500,
        &Vec::new(&env),
        &0,
    );

    client.close_plan(&owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    client.trigger_payout(&owner);

    assert_eq!(token_client.balance(&alice), 500);
    assert_eq!(token_client.balance(&bob), 300);
    assert_eq!(token_client.balance(&charlie), 200);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_trigger_payout_dust_goes_to_last_beneficiary() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let a = Address::generate(&env);
    let b = Address::generate(&env);

    token_client.mint(&owner, &100);

    let bene_a = Beneficiary {
        address: a.clone(),
        allocation_bps: 3333,
        fiat_anchor_info: String::from_str(&env, ""),
    };
    let bene_b = Beneficiary {
        address: b.clone(),
        allocation_bps: 6667,
        fiat_anchor_info: String::from_str(&env, ""),
    };

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &100,
        &Vec::from_array(&env, [bene_a, bene_b]),
        &3600,
        &false,
        &0,
        &Vec::new(&env),
        &0,
    );

    client.close_plan(&owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    client.trigger_payout(&owner);

    assert_eq!(token_client.balance(&a), 33);
    assert_eq!(token_client.balance(&b), 67);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_trigger_payout_plan_still_active() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    token_client.mint(&owner, &2000);

    let b = Beneficiary {
        address: beneficiary.clone(),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, ""),
    };

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [b]),
        &3600,
        &false,
        &0,
        &Vec::new(&env),
        &0,
    );

    env.ledger().set_timestamp(1_000_000 + 4000);

    let result = client.try_trigger_payout(&owner);
    assert_eq!(result, Err(Ok(Error::InactivityPeriodNotMet)));
}

#[test]
fn test_trigger_payout_grace_period_not_met() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    token_client.mint(&owner, &2000);

    let b = Beneficiary {
        address: beneficiary.clone(),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, ""),
    };

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [b]),
        &3600,
        &false,
        &0,
        &Vec::new(&env),
        &0,
    );

    client.close_plan(&owner);
    env.ledger().set_timestamp(1_000_000 + 1000);

    let result = client.try_trigger_payout(&owner);
    assert_eq!(result, Err(Ok(Error::InactivityPeriodNotMet)));
}

#[test]
fn test_trigger_payout_double_payout_prevented() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    token_client.mint(&owner, &2000);

    let b = Beneficiary {
        address: beneficiary.clone(),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, ""),
    };

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [b]),
        &3600,
        &false,
        &0,
        &Vec::new(&env),
        &0,
    );

    client.close_plan(&owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    client.trigger_payout(&owner);
    assert_eq!(token_client.balance(&beneficiary), 500);

    let result = client.try_trigger_payout(&owner);
    assert_eq!(result, Err(Ok(Error::PlanNotFound)));
}

#[test]
fn test_trigger_payout_no_plan() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let result = client.try_trigger_payout(&owner);
    assert_eq!(result, Err(Ok(Error::PlanNotFound)));
}

// ── Guardian multi-sig tests ──────────────────────────────────────────────────

#[test]
fn test_guardian_approve_and_trigger_payout() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let guardian1 = Address::generate(&env);
    let guardian2 = Address::generate(&env);
    let guardian3 = Address::generate(&env);

    token_client.mint(&owner, &1000);

    let b = Beneficiary {
        address: beneficiary.clone(),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
    };

    env.ledger().set_timestamp(1_000_000);

    // 2-of-3 guardian threshold
    client.create_plan(
        &owner,
        &token_id,
        &1000,
        &Vec::from_array(&env, [b]),
        &3600,
        &false,
        &0,
        &Vec::from_array(&env, [guardian1.clone(), guardian2.clone(), guardian3.clone()]),
        &2,
    );

    client.close_plan(&owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    // Only 1 approval — payout should fail
    client.approve_payout(&guardian1, &owner);
    let result = client.try_trigger_payout(&owner);
    assert_eq!(result, Err(Ok(Error::GuardianThresholdNotMet)));

    // Second approval — threshold met, payout succeeds
    client.approve_payout(&guardian2, &owner);
    client.trigger_payout(&owner);

    assert_eq!(token_client.balance(&beneficiary), 1000);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_guardian_duplicate_approval_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let guardian = Address::generate(&env);

    token_client.mint(&owner, &500);

    let b = Beneficiary {
        address: beneficiary.clone(),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, ""),
    };

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [b]),
        &3600,
        &false,
        &0,
        &Vec::from_array(&env, [guardian.clone()]),
        &1,
    );

    client.approve_payout(&guardian, &owner);

    // Second approval by same guardian should fail
    let result = client.try_approve_payout(&guardian, &owner);
    assert_eq!(result, Err(Ok(Error::AlreadyApproved)));
}

#[test]
fn test_non_guardian_approval_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let guardian = Address::generate(&env);
    let stranger = Address::generate(&env);

    token_client.mint(&owner, &500);

    let b = Beneficiary {
        address: beneficiary.clone(),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, ""),
    };

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [b]),
        &3600,
        &false,
        &0,
        &Vec::from_array(&env, [guardian.clone()]),
        &1,
    );

    let result = client.try_approve_payout(&stranger, &owner);
    assert_eq!(result, Err(Ok(Error::NotAGuardian)));
}

#[test]
fn test_no_guardians_payout_skips_threshold_check() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    token_client.mint(&owner, &500);

    let b = Beneficiary {
        address: beneficiary.clone(),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, ""),
    };

    env.ledger().set_timestamp(1_000_000);

    // No guardians, threshold = 0
    client.create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [b]),
        &3600,
        &false,
        &0,
        &Vec::new(&env),
        &0,
    );

    client.close_plan(&owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    // Should succeed without any guardian approvals
    client.trigger_payout(&owner);
    assert_eq!(token_client.balance(&beneficiary), 500);
}

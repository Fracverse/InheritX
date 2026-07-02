use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::testutils::{Events, Ledger};
use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, String, Vec};

// Helper function to deactivate a plan for grace period testing
fn deactivate_plan_for_testing(env: &Env, contract_id: &Address, owner: &Address) {
    let key = DataKey::Plan(owner.clone());
    let plan_option: Option<Plan> =
        env.as_contract(contract_id, || env.storage().persistent().get(&key));

    if let Some(mut plan) = plan_option {
        plan.is_active = false;
        env.as_contract(contract_id, || {
            env.storage().persistent().set(&key, &plan);
        });
    }
}

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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    client.create_plan(
        &owner,
        &token_id,
        &1500,
        &Vec::from_array(&env, [beneficiary.clone()]),
        &3600,
        &true,
        &500,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
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
fn test_ping_updates_last_ping_and_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "NGN_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    token_client.mint(&owner, &2000);

    let start = 1_000_000;
    env.ledger().set_timestamp(start);

    client.create_plan(
        &owner,
        &token_id,
        &1500,
        &Vec::from_array(&env, [beneficiary]),
        &3600,
        &true,
        &500,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );
    assert_eq!(client.get_plan(&owner).last_ping, start);

    let ping_timestamp = start + 1234;
    env.ledger().set_timestamp(ping_timestamp);

    client.ping(&owner);

    let plan = client.get_plan(&owner);
    assert_eq!(plan.last_ping, ping_timestamp);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id,
                (symbol_short!("ping"), owner).into_val(&env),
                ping_timestamp.into_val(&env),
            ),
        ]
    );
}

#[test]
#[should_panic]
fn test_ping_requires_owner_auth() {
    let env = Env::default();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let key = DataKey::Plan(owner.clone());
    let plan = Plan {
        owner: owner.clone(),
        token: Address::generate(&env),
        amount: 1,
        beneficiaries: Vec::new(&env),
        last_ping: env.ledger().timestamp(),
        grace_period: 3600,
        earn_yield: false,
        yield_rate_bps: 0,
        is_active: true,
        timelock_duration: 86400,
        source_chain: String::from_str(&env, "Stellar"),
        source_tx_hash: String::from_str(&env, "SRC_TX_HASH"),
    };

    env.as_contract(&contract_id, || {
        env.storage().persistent().set(&key, &plan);
    });

    client.ping(&owner);
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    let beneficiary2 = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 5000, // Total = 9000 BPS (less than 10000)
        fiat_anchor_info: String::from_str(&env, "NGN_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    let result = client.try_create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [beneficiary1, beneficiary2]),
        &3600,
        &true,
        &500,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Deactivate plan to start grace period
    deactivate_plan_for_testing(&env, &contract_id, &owner);

    // Jump past grace period
    env.ledger().set_timestamp(start + 4000);

    // Trigger payout
    client.claim(&owner);
    env.ledger().set_timestamp(env.ledger().timestamp() + 86400);
    client.trigger_payout(&owner);

    // Beneficiary receives full amount, contract emptied
    assert_eq!(token_client.balance(&beneficiary), 1500);
    assert_eq!(token_client.balance(&contract_id), 0);

    // Plan removed from storage
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let bob_bene = Beneficiary {
        address: bob.clone(),
        allocation_bps: 3000,
        fiat_anchor_info: String::from_str(&env, "EUR_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let charlie_bene = Beneficiary {
        address: charlie.clone(),
        allocation_bps: 2000,
        fiat_anchor_info: String::from_str(&env, "GBP_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Deactivate plan to start grace period
    deactivate_plan_for_testing(&env, &contract_id, &owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    client.claim(&owner);
    env.ledger().set_timestamp(env.ledger().timestamp() + 86400);
    client.trigger_payout(&owner);

    // Alice: 1000 * 5000 / 10000 = 500
    assert_eq!(token_client.balance(&alice), 500);
    // Bob: 1000 * 3000 / 10000 = 300
    assert_eq!(token_client.balance(&bob), 300);
    // Charlie: remaining = 1000 - 500 - 300 = 200
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let bene_b = Beneficiary {
        address: b.clone(),
        allocation_bps: 6667,
        fiat_anchor_info: String::from_str(&env, ""),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Deactivate plan to start grace period
    deactivate_plan_for_testing(&env, &contract_id, &owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    client.claim(&owner);
    env.ledger().set_timestamp(env.ledger().timestamp() + 86400);
    client.trigger_payout(&owner);

    // A: 100 * 3333 / 10000 = 33 (integer truncation)
    assert_eq!(token_client.balance(&a), 33);
    // B: remaining = 100 - 33 = 67 (not 66, so dust is captured)
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Plan is still active — deactivate_plan_for_testing was never called
    env.ledger().set_timestamp(1_000_000 + 4000);

    let result = client.try_claim(&owner);
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Deactivate plan to start grace period
    deactivate_plan_for_testing(&env, &contract_id, &owner);

    // Only 1000 seconds passed — need 3600
    env.ledger().set_timestamp(1_000_000 + 1000);

    let result = client.try_claim(&owner);
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
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
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Deactivate plan to start grace period
    deactivate_plan_for_testing(&env, &contract_id, &owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    // First payout succeeds
    client.claim(&owner);
    env.ledger().set_timestamp(env.ledger().timestamp() + 86400);
    client.trigger_payout(&owner);
    assert_eq!(token_client.balance(&beneficiary), 500);

    // Second payout fails — plan already removed
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

#[test]
fn test_cancel_claim_success() {
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    let start = 1_000_000;
    env.ledger().set_timestamp(start);

    client.create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [b]),
        &3600,
        &false,
        &0,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Deactivate plan to start grace period
    deactivate_plan_for_testing(&env, &contract_id, &owner);
    env.ledger().set_timestamp(start + 4000);

    // Trigger payout
    client.claim(&owner);

    // Cancel payout
    client.cancel_claim(&owner);

    // Attempting trigger_payout should now fail since the payout has been cancelled
    env.ledger().set_timestamp(env.ledger().timestamp() + 86400);
    let result = client.try_trigger_payout(&owner);
    assert_eq!(result, Err(Ok(Error::PayoutNotTriggered)));
}

#[test]
fn test_reclaim_success() {
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
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    let start = 1_000_000;
    env.ledger().set_timestamp(start);

    client.create_plan(
        &owner,
        &token_id,
        &500,
        &Vec::from_array(&env, [b]),
        &3600,
        &false,
        &0,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Owner reclaims before claim
    client.reclaim(&owner);

    assert_eq!(token_client.balance(&owner), 2000);
    assert_eq!(token_client.balance(&contract_id), 0);

    let result = client.try_get_plan(&owner);
    assert_eq!(result, Err(Ok(Error::PlanNotFound)));
}

// ============================================================================
// Issue #843: Unit Tests for Keep-Alive Ping and Close_Plan
// ============================================================================

#[test]
fn test_ping_success_from_owner_updates_timestamp() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    token_client.mint(&owner, &5000);

    let start = 1_000_000;
    env.ledger().set_timestamp(start);

    client.create_plan(
        &owner,
        &token_id,
        &3000,
        &Vec::from_array(&env, [beneficiary]),
        &7200,
        &true,
        &500,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Verify initial ping timestamp
    let plan = client.get_plan(&owner);
    assert_eq!(plan.last_ping, start);

    // Owner pings at a later time
    let ping_time = start + 5000;
    env.ledger().set_timestamp(ping_time);
    client.ping(&owner);

    // Verify timestamp is updated
    let updated_plan = client.get_plan(&owner);
    assert_eq!(updated_plan.last_ping, ping_time);

    // Owner is still within grace period
    let timeout_deadline = client.try_get_timeout_deadline(&owner);
    assert_eq!(timeout_deadline, Ok(Ok(ping_time + 7200)));
}

#[test]
fn test_ping_from_third_party_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let third_party = Address::generate(&env);
    let beneficiary = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    token_client.mint(&owner, &5000);

    let start = 1_000_000;
    env.ledger().set_timestamp(start);

    client.create_plan(
        &owner,
        &token_id,
        &2000,
        &Vec::from_array(&env, [beneficiary]),
        &3600,
        &true,
        &500,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Try to ping as third party without auth
    env.mock_auths(&[]);
    let result = client.try_ping(&third_party);

    // Should fail due to authorization check
    assert!(result.is_err());
}

#[test]
fn test_ping_nonexistent_plan_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let result = client.try_ping(&owner);
    assert_eq!(result, Err(Ok(Error::PlanNotFound)));
}

#[test]
fn test_close_plan_refunds_all_tokens_and_deletes_storage() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary1 = Address::generate(&env);
    let beneficiary2 = Address::generate(&env);

    let initial_balance = 10000;
    token_client.mint(&owner, &initial_balance);

    let plan_amount = 6000;
    let bene1 = Beneficiary {
        address: beneficiary1,
        allocation_bps: 5000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let bene2 = Beneficiary {
        address: beneficiary2,
        allocation_bps: 5000,
        fiat_anchor_info: String::from_str(&env, "EUR_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &plan_amount,
        &Vec::from_array(&env, [bene1, bene2]),
        &3600,
        &false,
        &0,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Verify tokens are transferred to contract
    assert_eq!(token_client.balance(&owner), initial_balance - plan_amount);
    assert_eq!(token_client.balance(&contract_id), plan_amount);

    // Close plan early - should refund all tokens and delete plan
    client.close_plan(&owner);

    // Verify tokens are refunded to owner
    assert_eq!(token_client.balance(&owner), initial_balance);
    assert_eq!(token_client.balance(&contract_id), 0);

    // Verify plan is deleted from storage
    let result = client.try_get_plan(&owner);
    assert_eq!(result, Err(Ok(Error::PlanNotFound)));
}

#[test]
fn test_close_plan_requires_owner_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);
    let beneficiary = Beneficiary {
        address: Address::generate(&env),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    token_client.mint(&owner, &5000);

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &2000,
        &Vec::from_array(&env, [beneficiary]),
        &3600,
        &false,
        &0,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Try to close plan as unauthorized user
    env.mock_auths(&[]);
    let result = client.try_close_plan(&unauthorized_user);

    // Should fail due to authorization check
    assert!(result.is_err());
}

#[test]
fn test_close_plan_nonexistent_plan_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let result = client.try_close_plan(&owner);
    assert_eq!(result, Err(Ok(Error::PlanNotFound)));
}

// ============================================================================
// Issue #845: Unit Tests for Multi-Beneficiary Payout with Various Edge Cases
// ============================================================================

#[test]
fn test_trigger_payout_5_beneficiaries_with_equal_allocations() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let b1 = Address::generate(&env);
    let b2 = Address::generate(&env);
    let b3 = Address::generate(&env);
    let b4 = Address::generate(&env);
    let b5 = Address::generate(&env);

    token_client.mint(&owner, &100000);

    // Each beneficiary gets 2000 BPS (20%)
    let bene1 = Beneficiary {
        address: b1.clone(),
        allocation_bps: 2000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let bene2 = Beneficiary {
        address: b2.clone(),
        allocation_bps: 2000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let bene3 = Beneficiary {
        address: b3.clone(),
        allocation_bps: 2000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let bene4 = Beneficiary {
        address: b4.clone(),
        allocation_bps: 2000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let bene5 = Beneficiary {
        address: b5.clone(),
        allocation_bps: 2000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &10000,
        &Vec::from_array(&env, [bene1, bene2, bene3, bene4, bene5]),
        &3600,
        &false,
        &0,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Deactivate, claim, and payout
    deactivate_plan_for_testing(&env, &contract_id, &owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    client.claim(&owner);
    env.ledger().set_timestamp(env.ledger().timestamp() + 86400);
    client.trigger_payout(&owner);

    // Each gets exactly 2000 (10000 * 2000 / 10000)
    assert_eq!(token_client.balance(&b1), 2000);
    assert_eq!(token_client.balance(&b2), 2000);
    assert_eq!(token_client.balance(&b3), 2000);
    assert_eq!(token_client.balance(&b4), 2000);
    assert_eq!(token_client.balance(&b5), 2000);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_trigger_payout_10_beneficiaries_unequal_allocations() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiaries = [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];

    token_client.mint(&owner, &500000);

    // Create beneficiaries with varying allocations (1000, 1000, ..., 1000 = 10000 BPS)
    let mut bene_array = Vec::new(&env);
    for beneficiary in beneficiaries.iter() {
        let b = Beneficiary {
            address: beneficiary.clone(),
            allocation_bps: 1000,
            fiat_anchor_info: String::from_str(&env, "USD_BANK"),
            destination_chain: String::from_str(&env, "Stellar"),
            destination_address: String::from_str(&env, "GDESTADDR"),
        };
        bene_array.push_back(b);
    }

    let plan_amount = 50000;
    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &plan_amount,
        &bene_array,
        &3600,
        &false,
        &0,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Deactivate, claim, and payout
    deactivate_plan_for_testing(&env, &contract_id, &owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    client.claim(&owner);
    env.ledger().set_timestamp(env.ledger().timestamp() + 86400);
    client.trigger_payout(&owner);

    // Each gets exactly 5000 (50000 * 1000 / 10000)
    for beneficiary in beneficiaries.iter() {
        assert_eq!(token_client.balance(beneficiary), 5000);
    }
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_trigger_payout_rounding_with_3_beneficiaries() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let bene1 = Address::generate(&env);
    let bene2 = Address::generate(&env);
    let bene3 = Address::generate(&env);

    token_client.mint(&owner, &100000);

    // Allocations: 3333, 3333, 3334 BPS to test rounding
    let b1 = Beneficiary {
        address: bene1.clone(),
        allocation_bps: 3333,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let b2 = Beneficiary {
        address: bene2.clone(),
        allocation_bps: 3333,
        fiat_anchor_info: String::from_str(&env, "EUR_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let b3 = Beneficiary {
        address: bene3.clone(),
        allocation_bps: 3334,
        fiat_anchor_info: String::from_str(&env, "GBP_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &1000,
        &Vec::from_array(&env, [b1, b2, b3]),
        &3600,
        &false,
        &0,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    deactivate_plan_for_testing(&env, &contract_id, &owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    client.claim(&owner);
    env.ledger().set_timestamp(env.ledger().timestamp() + 86400);
    client.trigger_payout(&owner);

    // bene1: 1000 * 3333 / 10000 = 333 (truncated)
    // bene2: 1000 * 3333 / 10000 = 333 (truncated)
    // bene3: 1000 - 333 - 333 = 334 (gets the remainder/dust)
    assert_eq!(token_client.balance(&bene1), 333);
    assert_eq!(token_client.balance(&bene2), 333);
    assert_eq!(token_client.balance(&bene3), 334);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_trigger_payout_after_grace_period_and_timelock_expiry() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    token_client.mint(&owner, &50000);

    let alice_bene = Beneficiary {
        address: alice.clone(),
        allocation_bps: 6000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let bob_bene = Beneficiary {
        address: bob.clone(),
        allocation_bps: 4000,
        fiat_anchor_info: String::from_str(&env, "EUR_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    let grace_period = 7200; // 2 hours
    let timelock_duration = 86400; // 1 day

    let start = 1_000_000;
    env.ledger().set_timestamp(start);

    client.create_plan(
        &owner,
        &token_id,
        &20000,
        &Vec::from_array(&env, [alice_bene, bob_bene]),
        &grace_period,
        &false,
        &0,
        &timelock_duration,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Deactivate plan
    deactivate_plan_for_testing(&env, &contract_id, &owner);

    // Jump to just before grace period ends - claim should fail
    env.ledger().set_timestamp(start + grace_period - 100);
    let too_early = client.try_claim(&owner);
    assert_eq!(too_early, Err(Ok(Error::InactivityPeriodNotMet)));

    // Jump past grace period - now claim should succeed
    env.ledger().set_timestamp(start + grace_period + 100);
    client.claim(&owner);

    // Jump to before timelock ends - trigger should fail
    env.ledger()
        .set_timestamp(start + grace_period + timelock_duration - 100);
    let trigger_too_early = client.try_trigger_payout(&owner);
    assert_eq!(trigger_too_early, Err(Ok(Error::TimelockNotExpired)));

    // Jump past timelock - now trigger should succeed
    env.ledger()
        .set_timestamp(start + grace_period + timelock_duration + 100);
    client.trigger_payout(&owner);

    // Verify payouts
    assert_eq!(token_client.balance(&alice), 12000); // 20000 * 6000 / 10000
    assert_eq!(token_client.balance(&bob), 8000); // 20000 * 4000 / 10000
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_trigger_payout_with_single_beneficiary_receives_all() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let sole_beneficiary = Address::generate(&env);

    token_client.mint(&owner, &100000);

    let sole_bene = Beneficiary {
        address: sole_beneficiary.clone(),
        allocation_bps: 10000, // 100%
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    let plan_amount = 55555;
    env.ledger().set_timestamp(1_000_000);

    client.create_plan(
        &owner,
        &token_id,
        &plan_amount,
        &Vec::from_array(&env, [sole_bene]),
        &3600,
        &false,
        &0,
        &86400,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    deactivate_plan_for_testing(&env, &contract_id, &owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    client.claim(&owner);
    env.ledger().set_timestamp(env.ledger().timestamp() + 86400);
    client.trigger_payout(&owner);

    // Sole beneficiary gets all
    assert_eq!(token_client.balance(&sole_beneficiary), plan_amount);
    assert_eq!(token_client.balance(&contract_id), 0);
}

// ============================================================================
// Unit Tests for create_plan and get_plan
// ============================================================================

/// Verifies that create_plan correctly stores all plan fields when multiple
/// beneficiaries with split allocations are provided.
#[test]
fn test_create_plan_stores_all_fields_with_multiple_beneficiaries() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    token_client.mint(&owner, &10000);

    let alice_bene = Beneficiary {
        address: alice.clone(),
        allocation_bps: 7000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };
    let bob_bene = Beneficiary {
        address: bob.clone(),
        allocation_bps: 3000,
        fiat_anchor_info: String::from_str(&env, "EUR_BANK"),
        destination_chain: String::from_str(&env, "Stellar"),
        destination_address: String::from_str(&env, "GDESTADDR"),
    };

    let start = 2_000_000u64;
    env.ledger().set_timestamp(start);

    client.create_plan(
        &owner,
        &token_id,
        &5000,
        &Vec::from_array(&env, [alice_bene.clone(), bob_bene.clone()]),
        &7200,
        &true,
        &300,
        &172800,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "SRC_TX_HASH"),
    );

    // Tokens are transferred: owner balance reduced, contract holds the amount
    assert_eq!(token_client.balance(&owner), 5000);
    assert_eq!(token_client.balance(&contract_id), 5000);

    // All stored plan fields match what was passed in
    let plan = client.get_plan(&owner);
    assert_eq!(plan.owner, owner);
    assert_eq!(plan.token, token_id);
    assert_eq!(plan.amount, 5000);
    assert_eq!(plan.grace_period, 7200);
    assert!(plan.earn_yield);
    assert_eq!(plan.yield_rate_bps, 300);
    assert_eq!(plan.timelock_duration, 172800);
    assert!(plan.is_active);
    assert_eq!(plan.last_ping, start);

    // Beneficiary details are preserved in order
    assert_eq!(plan.beneficiaries.len(), 2);
    let stored_alice = plan.beneficiaries.get(0).unwrap();
    assert_eq!(stored_alice.address, alice);
    assert_eq!(stored_alice.allocation_bps, 7000);
    let stored_bob = plan.beneficiaries.get(1).unwrap();
    assert_eq!(stored_bob.address, bob);
    assert_eq!(stored_bob.allocation_bps, 3000);
}

/// Verifies that get_plan returns PlanNotFound when no plan exists for the
/// given owner address.
#[test]
fn test_get_plan_returns_not_found_for_unknown_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let unknown = Address::generate(&env);

    let result = client.try_get_plan(&unknown);
    assert_eq!(result, Err(Ok(Error::PlanNotFound)));
}

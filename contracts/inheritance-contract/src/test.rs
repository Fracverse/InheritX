#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::testutils::Ledger;
use soroban_sdk::{Address, BytesN, Env, String, Vec};

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
    );

    // Deactivate plan
    client.close_plan(&owner);

    // Jump past grace period
    env.ledger().set_timestamp(start + 4000);

    // Trigger payout
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
    );

    client.close_plan(&owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

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
    );

    client.close_plan(&owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

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
    );

    // Plan is still active — close_plan was never called
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
    );

    client.close_plan(&owner);

    // Only 1000 seconds passed — need 3600
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
    );

    client.close_plan(&owner);
    env.ledger().set_timestamp(1_000_000 + 4000);

    // First payout succeeds
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

// --- Upgradeable contract pattern tests -----------------------------

/// Helper that uploads the pre-built `inheritance_contract.wasm` and returns
/// its on-chain wasm hash. Soroban 21.x's `upload_contract_wasm` validates the
/// wasm magic header, so we embed a real build rather than a synthetic byte
/// buffer. The fixture is regenerated by:
///   `cargo build --target wasm32-unknown-unknown --release -p inheritance-contract`
/// and copied into `testdata/`.
fn upload_inheritance_wasm(env: &Env) -> BytesN<32> {
    const WASM: &[u8] = include_bytes!("../testdata/inheritance_contract.wasm");
    let bytes_obj = soroban_sdk::Bytes::from_slice(env, WASM);
    env.deployer().upload_contract_wasm(bytes_obj)
}

#[test]
fn test_version_returns_current_version() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    assert_eq!(client.version(), CONTRACT_VERSION);
}

#[test]
fn test_get_admin_uninitialized_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let result = client.try_get_admin();
    assert_eq!(result, Err(Ok(Error::NotInitialized)));
}

#[test]
fn test_initialize_succeeds() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let other = Address::generate(&env);
    let result = client.try_initialize(&other);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));

    // Existing admin must be untouched by the rejected retry.
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_set_admin_rotates_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let new_admin = Address::generate(&env);
    client.set_admin(&new_admin);

    assert_eq!(client.get_admin(), new_admin);
}

#[test]
#[should_panic]
fn test_set_admin_unauthorized_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);

    // Strip every subsequent authorization so set_admin's admin require_auth
    // call fails just like an imposter invocation would in production.
    env.set_auths(&[]);

    let impostor = Address::generate(&env);
    client.set_admin(&impostor);
}

#[test]
fn test_upgrade_uninitialized_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let new_hash = upload_inheritance_wasm(&env);
    let result = client.try_upgrade(&new_hash);
    assert_eq!(result, Err(Ok(Error::NotInitialized)));
}

#[test]
#[should_panic]
fn test_upgrade_with_impostor_address_panics_without_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    // Authorize only the admin's initialize call; upgrade must still panic
    // because the caller below has no authorization.
    env.mock_all_auths();
    client.initialize(&admin);
    // Disable mocking-by-authorizing-only-admin via selective auth.
    env.set_auths(&[]);

    let new_hash = upload_inheritance_wasm(&env);
    client.upgrade(&new_hash); // panics: require_auth on stored admin
}

#[test]
fn test_upgrade_succeeds_when_admin_authorizes() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let new_hash = upload_inheritance_wasm(&env);
    client.upgrade(&new_hash);
    // If we reach here, the host wasm swap succeeded under admin authorization.
}

#[test]
fn test_upgrade_preserves_persistent_storage() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    // Stand up the admin and a token contract.
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_id = env.register_contract(None, mock_token::MockToken);
    let token_client = mock_token::MockTokenClient::new(&env, &token_id);

    let owner = Address::generate(&env);
    let beneficiary_addr = Address::generate(&env);
    token_client.mint(&owner, &2000);

    let beneficiary = Beneficiary {
        address: beneficiary_addr.clone(),
        allocation_bps: 10000,
        fiat_anchor_info: String::from_str(&env, "USD_BANK"),
    };

    client.create_plan(
        &owner,
        &token_id,
        &1500,
        &Vec::from_array(&env, [beneficiary]),
        &3600,
        &false,
        &0,
    );

    // Capture the plan as persisted *before* the wasm swap; the host persists
    // it under the contract address key, independent of which code runs.
    let plan_key = DataKey::Plan(owner.clone());
    let plan_before: Plan = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get(&plan_key)
            .expect("plan should exist before upgrade")
    });

    // Swap the wasm to a dummy build.
    let new_hash = upload_inheritance_wasm(&env);
    client.upgrade(&new_hash);

    // The host did not modify persistent storage during `update_current_contract_wasm`;
    // reading it directly from the contract's storage context proves retention.
    let plan_after: Plan = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get(&plan_key)
            .expect("plan must be retained across upgrade")
    });
    assert_eq!(plan_before, plan_after);

    // The token balance held by the contract account is similarly untouched.
    assert_eq!(token_client.balance(&contract_id), 1500);
}

#[test]
fn test_upgrade_preserves_admin_in_instance_storage() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let client = InheritanceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let new_hash = upload_inheritance_wasm(&env);
    client.upgrade(&new_hash);

    // After swapping the wasm the admin record still lives in instance storage.
    let stored_admin: Address = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get(&InstanceDataKey::Admin)
            .expect("admin must persist across upgrade")
    });
    assert_eq!(stored_admin, admin);
}

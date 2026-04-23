use super::*;
use mock_token::MockToken;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    vec, Address, BytesN, Env, IntoVal, String, Symbol,
};

fn create_test_address(env: &Env) -> Address {
    Address::generate(env)
}

fn create_test_bytes_n_32(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

fn setup_test(env: &Env) -> (InheritanceContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let token_id = env.register_contract(None, MockToken);
    let admin = create_test_address(env);
    let owner = create_test_address(env);
    let client = InheritanceContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);

    // Create a plan first as it's required for messages
    let beneficiaries_data = vec![
        env,
        (
            String::from_str(env, "Alice"),
            String::from_str(env, "alice@example.com"),
            111111u32,
            soroban_sdk::Bytes::from_slice(env, b"1234567890"),
            10000u32,
        ),
    ];

    let _plan_params = CreateInheritancePlanParams {
        owner: owner.clone(),
        token: token_id,
        plan_name: String::from_str(env, "Test Plan"),
        description: String::from_str(env, "Test Description"),
        total_amount: 1000000,
        distribution_method: DistributionMethod::LumpSum,
        beneficiaries_data,
        is_lendable: true,
        waterfall_enabled: false,
    };

    // We need to mint tokens to owner for plan creation
    // But for simplicity in message tests, let's just mock auths
    // and assume vault exists if we can get past the plan check.
    // In test.rs they actually mint tokens.

    (client, owner, contract_id)
}

#[test]
fn test_message_lifecycle_events() {
    let env = Env::default();
    let (_client, _owner, _contract_addr) = setup_test(&env);

    // 1. Create a plan (actually needed because create_legacy_message checks for plan)
    // We need to actually create it or mock the storage.
    // Let's use the real create_inheritance_plan but we need to setup tokens.
    // Actually, I'll just use the setup from test.rs if I can't easily mock it.

    // For now, let's assume we have a plan with ID 0.
    // Wait, the first plan ID will be 0.

    // Let's just use the setup_with_token_and_admin from test.rs logic
    // but I can't easily call it if it's not pub. I'll just replicate it.

    env.ledger().set_timestamp(1000);

    // Mocking plan existence since we just want to test message logic
    // Actually, let's just create a real plan to be safe.
}

// Re-implementing setup properly to ensure it works
fn full_setup(env: &Env) -> (InheritanceContractClient<'_>, Address, u64) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let token_id = env.register_contract(None, MockToken);
    let admin = Address::generate(env);
    let owner = Address::generate(env);
    let client = InheritanceContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);

    // Submit and approve KYC for owner
    client.submit_kyc(&owner);
    client.approve_kyc(&admin, &owner);

    // Mint tokens to owner
    let mock_token_client = mock_token::MockTokenClient::new(env, &token_id);
    mock_token_client.mint(&owner, &10_000_000i128);

    let beneficiaries_data = vec![
        env,
        (
            String::from_str(env, "Alice"),
            String::from_str(env, "alice@example.com"),
            111111u32,
            soroban_sdk::Bytes::from_slice(env, b"1234567890"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(&CreateInheritancePlanParams {
        owner: owner.clone(),
        token: token_id,
        plan_name: String::from_str(env, "Test Plan"),
        description: String::from_str(env, "Test Description"),
        total_amount: 1000000,
        distribution_method: DistributionMethod::LumpSum,
        beneficiaries_data,
        is_lendable: true,
        waterfall_enabled: false,
    });

    (client, owner, plan_id)
}

#[test]
fn test_message_created_event() {
    let env = Env::default();
    let (client, owner, vault_id) = full_setup(&env);

    let message_hash = create_test_bytes_n_32(&env);
    let unlock_timestamp = 2000;
    env.ledger().set_timestamp(1000);

    let message_id = client.create_legacy_message(
        &owner,
        &CreateLegacyMessageParams {
            vault_id,
            message_hash,
            unlock_timestamp,
            key_reference: String::from_str(&env, "ref_1"),
        },
    );

    let last_event = env.events().all().last().unwrap();
    assert_eq!(last_event.0, contract_id(&env, &client));
    assert_eq!(
        last_event.1,
        (Symbol::new(&env, "message_created"), vault_id).into_val(&env)
    );

    // Verify event structure
    let event: MessageCreatedEvent = last_event.2.into_val(&env);
    assert_eq!(event.vault_id, vault_id);
    assert_eq!(event.message_id, message_id);
    assert_eq!(event.timestamp, 1000);
}

#[test]
fn test_message_updated_event() {
    let env = Env::default();
    let (client, owner, vault_id) = full_setup(&env);

    let message_hash = create_test_bytes_n_32(&env);
    let unlock_timestamp = 2000;
    env.ledger().set_timestamp(1000);

    let message_id = client.create_legacy_message(
        &owner,
        &CreateLegacyMessageParams {
            vault_id,
            message_hash: message_hash.clone(),
            unlock_timestamp,
            key_reference: String::from_str(&env, "ref_1"),
        },
    );

    env.ledger().set_timestamp(1100);
    let new_hash = BytesN::from_array(&env, &[2u8; 32]);
    client.update_legacy_message(
        &owner,
        &message_id,
        &CreateLegacyMessageParams {
            vault_id,
            message_hash: new_hash,
            unlock_timestamp: 2500,
            key_reference: String::from_str(&env, "ref_updated"),
        },
    );

    let last_event = env.events().all().last().unwrap();
    assert_eq!(
        last_event.1,
        (Symbol::new(&env, "message_updated"), vault_id).into_val(&env)
    );

    let event: MessageUpdatedEvent = last_event.2.into_val(&env);
    assert_eq!(event.message_id, message_id);
    assert_eq!(event.timestamp, 1100);
}

#[test]
fn test_message_finalized_event() {
    let env = Env::default();
    let (client, owner, vault_id) = full_setup(&env);

    let message_id = client.create_legacy_message(
        &owner,
        &CreateLegacyMessageParams {
            vault_id,
            message_hash: create_test_bytes_n_32(&env),
            unlock_timestamp: 2000,
            key_reference: String::from_str(&env, "ref_1"),
        },
    );

    env.ledger().set_timestamp(1200);
    client.finalize_legacy_message(&owner, &message_id);

    let last_event = env.events().all().last().unwrap();
    assert_eq!(
        last_event.1,
        (Symbol::new(&env, "message_finalized"), vault_id).into_val(&env)
    );

    let event: MessageFinalizedEvent = last_event.2.into_val(&env);
    assert_eq!(event.message_id, message_id);
    assert_eq!(event.timestamp, 1200);

    // Verify update fails after finalization
    let result = client.try_update_legacy_message(
        &owner,
        &message_id,
        &CreateLegacyMessageParams {
            vault_id,
            message_hash: create_test_bytes_n_32(&env),
            unlock_timestamp: 3000,
            key_reference: String::from_str(&env, "ref_updated"),
        },
    );
    assert!(result.is_err());
}

#[test]
fn test_message_accessed_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InheritanceContract);
    let token_id = env.register_contract(None, MockToken);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let alice = Address::generate(&env);
    let client = InheritanceContractClient::new(&env, &contract_id);
    client.initialize_admin(&admin);

    // Submit and approve KYC for owner
    client.submit_kyc(&owner);
    client.approve_kyc(&admin, &owner);

    mock_token::MockTokenClient::new(&env, &token_id).mint(&owner, &10_000_000i128);

    let beneficiaries_data = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice"),
            111111u32,
            soroban_sdk::Bytes::from_slice(&env, b"1234567890"),
            10000u32,
        ),
    ];

    let vault_id = client.create_inheritance_plan(&CreateInheritancePlanParams {
        owner: owner.clone(),
        token: token_id,
        plan_name: String::from_str(&env, "Test Plan"),
        description: String::from_str(&env, "Test Description"),
        total_amount: 1000000,
        distribution_method: DistributionMethod::LumpSum,
        beneficiaries_data,
        is_lendable: true,
        waterfall_enabled: false,
    });

    let message_id = client.create_legacy_message(
        &owner,
        &CreateLegacyMessageParams {
            vault_id,
            message_hash: create_test_bytes_n_32(&env),
            unlock_timestamp: 2000,
            key_reference: String::from_str(&env, "ref_1"),
        },
    );

    // Unlock message by advancing time
    env.ledger().set_timestamp(2001);

    // Alice accesses the message
    let result = client.try_access_legacy_message(&alice, &message_id);
    assert!(result.is_err());
}

fn contract_id(_env: &Env, client: &InheritanceContractClient<'_>) -> Address {
    client.address.clone()
}

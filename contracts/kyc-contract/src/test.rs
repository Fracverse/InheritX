#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KycContract);
    let client = KycContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);
}

#[test]
#[should_panic]
fn test_initialize_twice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KycContract);
    let client = KycContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);
    client.initialize(&admin); // Should panic
}

#[test]
fn test_set_and_get_status() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KycContract);
    let client = KycContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);

    // Default status is Pending
    assert_eq!(client.get_status(&user), KycStatus::Pending);

    // Set to Approved
    client.set_status(&admin, &user, &KycStatus::Approved);
    assert_eq!(client.get_status(&user), KycStatus::Approved);

    // Set to Rejected
    client.set_status(&admin, &user, &KycStatus::Rejected);
    assert_eq!(client.get_status(&user), KycStatus::Rejected);
}

#[test]
fn test_require_approved() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KycContract);
    let client = KycContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let approved_user = Address::generate(&env);
    let pending_user = Address::generate(&env);
    let rejected_user = Address::generate(&env);

    env.mock_all_auths();

    client.initialize(&admin);
    client.set_status(&admin, &approved_user, &KycStatus::Approved);
    client.set_status(&admin, &rejected_user, &KycStatus::Rejected);
    // pending_user stays at default Pending

    // Approved user should pass
    assert!(client.require_approved(&approved_user).is_ok());

    // Pending user should fail
    assert!(client.require_approved(&pending_user).is_err());

    // Rejected user should fail
    assert!(client.require_approved(&rejected_user).is_err());
}

#[test]
#[should_panic]
fn test_unauthorized_set_status() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KycContract);
    let client = KycContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let user = Address::generate(&env);

    env.mock_all_auths();

    client.initialize(&admin);

    // Unauthorized user should not be able to set status
    client.set_status(&unauthorized, &user, &KycStatus::Approved);
}

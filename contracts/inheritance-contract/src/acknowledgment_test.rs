#![cfg(test)]

use crate::{InheritanceContract, InheritanceContractClient};
use mock_token::MockToken;
use mock_token::MockTokenClient;
use soroban_sdk::{
    testutils::Address as _, testutils::Events, testutils::Ledger, token, Address, Bytes, Env,
    String, Vec, vec
};
use crate::{DistributionMethod, BeneficiaryInput, BeneficiaryAcknowledgment, InheritanceError};

// Helper function to create test address
fn create_test_address(env: &Env, _seed: u64) -> Address {
    Address::generate(env)
}

// Helper function to create test bytes
fn create_test_bytes(env: &Env, data: &str) -> Bytes {
    let mut bytes = Bytes::new(env);
    for byte in data.as_bytes() {
        bytes.push_back(*byte);
    }
    bytes
}

/// Sets up env with inheritance contract, mock token, admin initialized, owner minted.
/// Returns (client, token_id, admin, owner).
fn setup_with_token_and_admin(
    env: &Env,
) -> (InheritanceContractClient<'_>, Address, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, InheritanceContract);
    let token_id = env.register_contract(None, MockToken);
    let admin = create_test_address(env, 100);
    let owner = create_test_address(env, 1);
    let client = InheritanceContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);
    MockTokenClient::new(env, &token_id).mint(&owner, &10_000_000i128);

    // Approve KYC for owner by default so they can create plans
    client.submit_kyc(&owner);
    client.approve_kyc(&admin, &owner);

    (client, token_id, admin, owner)
}

fn plan_params(
    env: &Env,
    owner: &Address,
    token: &Address,
    plan_name: &str,
    description: &str,
    total_amount: u64,
    distribution_method: DistributionMethod,
    beneficiaries_data: &Vec<(String, String, u32, Bytes, u32)>,
) -> crate::CreateInheritancePlanParams {
    crate::CreateInheritancePlanParams {
        owner: owner.clone(),
        token: token.clone(),
        plan_name: String::from_str(env, plan_name),
        description: String::from_str(env, description),
        total_amount,
        distribution_method,
        beneficiaries_data: beneficiaries_data.clone(),
        is_lendable: true,
    }
}

#[test]
fn test_beneficiary_acknowledgment_flow() {
    let env = Env::default();
    env.ledger().set_timestamp(100);
    let (client, token_id, admin, owner) = setup_with_token_and_admin(&env);

    let beneficiary = create_test_address(&env, 100);

    let beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            123456u32,
            create_test_bytes(&env, "1111"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token_id,
        "Will",
        "Inheritance Plan",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries,
    ));

    // Initially, there are unacknowledged beneficiaries
    let unacknowledged = client.get_unacknowledged_beneficiaries(&plan_id);
    assert_eq!(unacknowledged.len(), 1);
    assert_eq!(unacknowledged.get(0).unwrap(), 0);

    let initial_ack = client.get_beneficiary_acknowledgment(&plan_id, &0);
    assert!(initial_ack.is_none());

    // Owner sets requirement
    client.set_require_acknowledgment(&owner, &plan_id, &true);

    // Owner notifies beneficiary
    client.notify_beneficiary(&owner, &plan_id, &0);

    let notified_ack = client.get_beneficiary_acknowledgment(&plan_id, &0).unwrap();
    assert_ne!(notified_ack.notification_sent_at, 0);
    assert_eq!(notified_ack.acknowledged_at, 0);

    // Beneficiary attempts to claim but hasn't acknowledged
    client.submit_kyc(&beneficiary);
    client.approve_kyc(&admin, &beneficiary);

    let claim_result = client.try_claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
    assert_eq!(claim_result, Err(Ok(InheritanceError::VerificationFailed)));

    // Beneficiary acknowledges
    client.acknowledge_beneficiary_status(
        &beneficiary,
        &plan_id,
        &0,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );

    let final_ack = client.get_beneficiary_acknowledgment(&plan_id, &0).unwrap();
    assert_ne!(final_ack.acknowledged_at, 0);

    // Now unacknowledged list is empty
    let unacknowledged_after = client.get_unacknowledged_beneficiaries(&plan_id);
    assert_eq!(unacknowledged_after.len(), 0);

    // Claim should now succeed
    client.claim_inheritance_plan(
        &plan_id,
        &beneficiary,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
}

#[test]
fn test_unauthorized_notify() {
    let env = Env::default();
    let (client, token_id, admin, owner) = setup_with_token_and_admin(&env);

    let stranger = create_test_address(&env, 999);

    let beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            123456u32,
            create_test_bytes(&env, "1111"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token_id,
        "Will",
        "Inheritance Plan",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries,
    ));

    let res = client.try_notify_beneficiary(&stranger, &plan_id, &0);
    assert_eq!(res, Err(Ok(InheritanceError::Unauthorized)));
}

#[test]
fn test_verification_failed_acknowledgment() {
    let env = Env::default();
    let (client, token_id, admin, owner) = setup_with_token_and_admin(&env);

    let beneficiary = create_test_address(&env, 100);

    let beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "Alice"),
            String::from_str(&env, "alice@example.com"),
            123456u32,
            create_test_bytes(&env, "1111"),
            10000u32,
        ),
    ];

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token_id,
        "Will",
        "Inheritance Plan",
        1000u64,
        DistributionMethod::LumpSum,
        &beneficiaries,
    ));

    // wrong email
    let res = client.try_acknowledge_beneficiary_status(
        &beneficiary,
        &plan_id,
        &0,
        &String::from_str(&env, "wrong_length_email@example.com"),
        &123456u32,
    );
    assert_eq!(res, Err(Ok(InheritanceError::VerificationFailed)));
}

#![cfg(test)]
#![allow(clippy::all)]

//! Tests for the closed-plan storage clean-up (#1170) and the zero-knowledge
//! genetic-proof hook (#1176).

use super::*;
use mock_token::{MockToken, MockTokenClient};
use soroban_sdk::{testutils::Address as _, token, vec, Address, Bytes, Env, String, Vec};

// ─── Mock Groth16 verifier ────────────────────────

/// A stand-in for a real Groth16 verifier contract.
///
/// It accepts a proof iff the first public input equals the expected commitment
/// that the test set up, which is enough to exercise both the accept and reject
/// paths of the hook without doing pairing checks on-chain.
#[contract]
pub struct MockZkVerifier;

#[contracttype]
#[derive(Clone)]
pub enum VerifierKey {
    Expected,
    Reject,
}

#[contractimpl]
impl MockZkVerifier {
    /// Set the commitment the verifier will accept.
    pub fn set_expected(env: Env, commitment: soroban_sdk::BytesN<32>) {
        env.storage().instance().set(&VerifierKey::Expected, &commitment);
    }

    /// Make the verifier reject everything, standing in for a reverting or
    /// misbehaving verifier.
    pub fn set_reject_all(env: Env, reject: bool) {
        env.storage().instance().set(&VerifierKey::Reject, &reject);
    }

    pub fn verify(
        env: Env,
        _proof: Bytes,
        public_inputs: Vec<soroban_sdk::BytesN<32>>,
    ) -> bool {
        if env
            .storage()
            .instance()
            .get::<VerifierKey, bool>(&VerifierKey::Reject)
            .unwrap_or(false)
        {
            return false;
        }
        let expected: soroban_sdk::BytesN<32> = env
            .storage()
            .instance()
            .get(&VerifierKey::Expected)
            .unwrap_or(soroban_sdk::BytesN::<32>::from_array(&env, &[0u8; 32]));
        match public_inputs.first() {
            Some(first) => first == expected,
            None => false,
        }
    }
}

// ─── Setup helpers ────────────────────────────────

fn setup(env: &Env) -> (InheritanceContractClient<'_>, Address, Address, Address) {
    env.mock_all_auths_allowing_non_root_auth();
    let contract_id = env.register_contract(None, InheritanceContract);
    let token_id = env.register_contract(None, MockToken);
    let admin = Address::generate(env);
    let owner = Address::generate(env);
    let client = InheritanceContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);
    MockTokenClient::new(env, &token_id).mint(&owner, &10_000_000i128);
    client.submit_kyc(&owner);
    client.approve_kyc(&admin, &owner);
    (client, token_id, admin, owner)
}

fn test_bytes(env: &Env, s: &str) -> Bytes {
    Bytes::from_slice(env, s.as_bytes())
}

fn one_beneficiary(env: &Env, code: u32) -> Vec<(String, String, u32, Bytes, u32, u32)> {
    vec![
        env,
        (
            String::from_str(env, "Alice"),
            String::from_str(env, "alice@example.com"),
            code,
            test_bytes(env, "1111111111111111"),
            10000u32,
            1u32,
        ),
    ]
}

fn plan_params(
    env: &Env,
    owner: &Address,
    token: &Address,
    amount: u64,
    bens: &Vec<(String, String, u32, Bytes, u32, u32)>,
) -> CreateInheritancePlanParams {
    CreateInheritancePlanParams {
        owner: owner.clone(),
        token: token.clone(),
        plan_name: String::from_str(env, "Plan"),
        description: String::from_str(env, "Desc"),
        total_amount: amount,
        distribution_method: DistributionMethod::LumpSum,
        beneficiaries_data: bens.clone(),
        is_lendable: true,
        guardians: Vec::new(env),
        guardian_threshold: 0,
    }
}

// ─── #1170 closed-plan storage clean-up ───────────

#[test]
fn cleanup_releases_storage_and_refunds_rent_to_creator() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup(&env);
    let token_helper = token::Client::new(&env, &token);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));

    // Give the plan a real footprint beyond the plan body, so the sweep is
    // observable rather than just removing one key.
    client.set_guardians(&owner, &plan_id, &vec![&env, owner.clone()], &1);
    let before = client.plan_storage_footprint(&plan_id);
    assert!(before > 1, "plan should occupy more than just its body");

    let owner_balance_before = token_helper.balance(&owner);

    // Close the plan: deactivate, then withdraw the remaining balance.
    client.deactivate_inheritance_plan(&owner, &plan_id);
    let plan = client.get_plan_details(&plan_id).unwrap();
    assert_eq!(plan.total_amount, 9800);
    client.withdraw(&owner, &token, &plan_id, &9800);
    assert_eq!(client.get_plan_details(&plan_id).unwrap().total_amount, 0);

    client.cleanup_closed_plan(&owner, &plan_id);

    // The plan body is gone from the ledger.
    assert!(client.get_plan_details(&plan_id).is_none());
    assert_eq!(client.plan_storage_footprint(&plan_id), 0);

    // And the creator got the vault residue back rather than it being stranded.
    assert!(token_helper.balance(&owner) > owner_balance_before);
}

#[test]
fn cleanup_rejects_plan_that_is_not_fully_closed() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));

    // Active and fully funded: cleaning up now would delete a live plan.
    let result = client.try_cleanup_closed_plan(&owner, &plan_id);
    assert_eq!(result, Err(Ok(InheritanceError::PlanNotClaimed)));
    assert!(client.get_plan_details(&plan_id).is_some());

    // Deactivated but still holding a balance is also not closed: the creator
    // can still withdraw, so the vault key must survive.
    client.deactivate_inheritance_plan(&owner, &plan_id);
    let result = client.try_cleanup_closed_plan(&owner, &plan_id);
    assert_eq!(result, Err(Ok(InheritanceError::PlanNotClaimed)));
    assert!(client.get_plan_details(&plan_id).is_some());
    assert!(client.get_plan_vault_address(&plan_id).is_some());
}

#[test]
fn cleanup_rejects_non_owner_non_admin() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup(&env);
    let stranger = Address::generate(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));
    client.deactivate_inheritance_plan(&owner, &plan_id);
    client.withdraw(&owner, &token, &plan_id, &9800);

    let result = client.try_cleanup_closed_plan(&stranger, &plan_id);
    assert_eq!(result, Err(Ok(InheritanceError::Unauthorized)));
    assert!(client.get_plan_details(&plan_id).is_some());
}

#[test]
fn cleanup_by_admin_refunds_the_owner_not_the_caller() {
    let env = Env::default();
    let (client, token, admin, owner) = setup(&env);
    let token_helper = token::Client::new(&env, &token);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));
    client.deactivate_inheritance_plan(&owner, &plan_id);
    // Settle the plan fully, then have someone send tokens straight to the
    // plan's vault. The plan is closed and its tracked balance is zero, so the
    // clean-up is entitled to sweep that residue back to the owner.
    client.withdraw(&owner, &token, &plan_id, &9_800);
    let vault = client.get_plan_vault_address(&plan_id).unwrap();
    MockTokenClient::new(&env, &token).mint(&vault, &100);

    let owner_before = token_helper.balance(&owner);
    let admin_before = token_helper.balance(&admin);

    client.cleanup_closed_plan(&admin, &plan_id);

    // The refund follows the plan's recorded owner, never the caller.
    assert_eq!(token_helper.balance(&owner), owner_before + 100);
    assert_eq!(token_helper.balance(&admin), admin_before);
}

#[test]
fn cleanup_removes_plan_from_owner_and_global_indexes() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));
    assert!(client.get_user_plan(&owner, &1).total_amount > 0);
    assert_eq!(client.get_user_plans(&owner).len(), 1);

    client.deactivate_inheritance_plan(&owner, &plan_id);
    client.withdraw(&owner, &token, &plan_id, &9800);
    client.cleanup_closed_plan(&owner, &plan_id);

    assert!(client.get_user_plans(&owner).is_empty());
    assert!(client.get_user_deactivated_plans(&owner).is_empty());
    // The plan body is gone, so reading it now reverts rather than returning
    // stale state.
    assert_eq!(
        client.try_get_user_plan(&owner, &1),
        Err(Ok(InheritanceError::PlanNotFound))
    );
}

#[test]
fn cleanup_is_not_repeatable() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));
    client.deactivate_inheritance_plan(&owner, &plan_id);
    client.withdraw(&owner, &token, &plan_id, &9800);

    client.cleanup_closed_plan(&owner, &plan_id);
    // Second call finds no plan at all.
    let result = client.try_cleanup_closed_plan(&owner, &plan_id);
    assert_eq!(result, Err(Ok(InheritanceError::PlanNotFound)));
}

// ─── #1176 zero-knowledge genetic proof ──────────

fn setup_verifier(env: &Env) -> (InheritanceContractClient<'_>, Address, Address) {
    let (client, token, admin, owner) = setup(env);
    let verifier_id = env.register_contract(None, MockZkVerifier);
    client.set_zk_verifier(&admin, &verifier_id);
    (client, token, owner)
}

#[test]
fn mock_verifier_responds_to_a_direct_call() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, MockZkVerifier);
    let c = MockZkVerifierClient::new(&env, &id);
    let expected: soroban_sdk::BytesN<32> = env.crypto().sha256(&Bytes::from_slice(&env, b"x")).into();
    c.set_expected(&expected);
    assert!(c.verify(&Bytes::from_slice(&env, b"p"), &vec![&env, expected.clone()]));
    assert!(!c.verify(&Bytes::from_slice(&env, b"p"), &Vec::new(&env)));
}

#[test]
fn zk_verification_fails_closed_without_a_configured_verifier() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup(&env);

    // No verifier registered: the hook must not approve anything.
    assert!(client.get_zk_verifier().is_none());
    assert!(!client.verify_zk_genetic_proof(
        &Bytes::from_slice(&env, b"proof"),
        &vec![&env, env.crypto().sha256(&Bytes::from_slice(&env, b"x")).into()]
    ));

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));
    let result = client.try_verify_genetic_kin_claim(
        &owner,
        &plan_id,
        &Bytes::from_slice(&env, b"proof"),
        &Vec::new(&env),
    );
    assert_eq!(result, Err(Ok(InheritanceError::ZkProofRequired)));
}

#[test]
fn zk_verification_survives_a_rejecting_verifier() {
    let env = Env::default();
    let (client, _token, _owner) = setup_verifier(&env);
    let verifier_id = client.get_zk_verifier().unwrap();

    // Stand in for a verifier that reverts or misbehaves.
    MockZkVerifierClient::new(&env, &verifier_id).set_reject_all(&true);

    assert!(!client.verify_zk_genetic_proof(
        &Bytes::from_slice(&env, b"proof"),
        &vec![&env, env.crypto().sha256(&Bytes::from_slice(&env, b"x")).into()]
    ));
}

#[test]
fn valid_proof_records_claimant_and_binds_to_the_plan() {
    let env = Env::default();
    let (client, token, owner) = setup_verifier(&env);
    let verifier_id = client.get_zk_verifier().unwrap();

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));

    let claimant = Address::generate(&env);
    let commitment = client.genetic_proof_commitment(&plan_id, &claimant);
    MockZkVerifierClient::new(&env, &verifier_id).set_expected(&commitment);

    assert!(!client.has_verified_genetic_proof(&plan_id, &claimant));

    client.verify_genetic_kin_claim(
        &claimant,
        &plan_id,
        &Bytes::from_slice(&env, b"proof"),
        &vec![&env, commitment.clone()],
    );

    assert!(client.has_verified_genetic_proof(&plan_id, &claimant));
    // The proof is bound to this plan, so it does not carry to another.
    assert!(!client.has_verified_genetic_proof(&(plan_id + 1), &claimant));
}

#[test]
fn invalid_proof_records_nothing() {
    let env = Env::default();
    let (client, token, owner) = setup_verifier(&env);
    let verifier_id = client.get_zk_verifier().unwrap();

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));
    let claimant = Address::generate(&env);
    // Verifier expects a different commitment than the one supplied.
    MockZkVerifierClient::new(&env, &verifier_id)
        .set_expected(&env.crypto().sha256(&Bytes::from_slice(&env, b"other")).into());

    let result = client.try_verify_genetic_kin_claim(
        &claimant,
        &plan_id,
        &Bytes::from_slice(&env, b"proof"),
        &vec![&env, client.genetic_proof_commitment(&plan_id, &claimant)],
    );
    assert_eq!(result, Err(Ok(InheritanceError::ZkProofRequired)));
    assert!(!client.has_verified_genetic_proof(&plan_id, &claimant));
}

#[test]
fn genetic_kin_requirement_is_owner_only_and_index_bounded() {
    let env = Env::default();
    let (client, token, _admin, owner) = setup(&env);
    let stranger = Address::generate(&env);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));

    // A non-owner cannot set the flag.
    let result = client.try_set_genetic_kin_requirement(&stranger, &plan_id, &0, &true);
    assert_eq!(result, Err(Ok(InheritanceError::Unauthorized)));

    // Nor can the owner set it for an index the plan does not have.
    let result = client.try_set_genetic_kin_requirement(&owner, &plan_id, &7, &true);
    assert_eq!(result, Err(Ok(InheritanceError::InvalidBeneficiaryIndex)));

    assert!(!client.is_genetic_kin_required(&plan_id, &0));
    client.set_genetic_kin_requirement(&owner, &plan_id, &0, &true);
    assert!(client.is_genetic_kin_required(&plan_id, &0));

    // And it can be cleared again.
    client.set_genetic_kin_requirement(&owner, &plan_id, &0, &false);
    assert!(!client.is_genetic_kin_required(&plan_id, &0));
}

#[test]
fn genetic_kin_claim_requires_a_verified_proof() {
    let env = Env::default();
    let (client, token, admin, owner) = setup(&env);
    let verifier_id = env.register_contract(None, MockZkVerifier);
    client.set_zk_verifier(&admin, &verifier_id);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));
    client.set_genetic_kin_requirement(&owner, &plan_id, &0, &true);

    let claimant = Address::generate(&env);
    client.submit_kyc(&claimant);
    client.approve_kyc(&admin, &claimant);

    // Without a proof on file the claim is refused.
    let result = client.try_claim_inheritance_plan(
        &plan_id,
        &claimant,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
    assert_eq!(result, Err(Ok(InheritanceError::ZkProofRequired)));

    // With one, it goes through.
    let commitment = client.genetic_proof_commitment(&plan_id, &claimant);
    MockZkVerifierClient::new(&env, &verifier_id).set_expected(&commitment);
    client.verify_genetic_kin_claim(
        &claimant,
        &plan_id,
        &Bytes::from_slice(&env, b"proof"),
        &vec![&env, commitment],
    );
    client.claim_inheritance_plan(
        &plan_id,
        &claimant,
        &String::from_str(&env, "alice@example.com"),
        &123456u32,
    );
}

#[test]
fn proof_verification_rejects_a_closed_plan() {
    let env = Env::default();
    let (client, token, owner) = setup_verifier(&env);
    let claimant = Address::generate(&env);
    let commitment = client.genetic_proof_commitment(&1, &claimant);
    MockZkVerifierClient::new(&env, &client.get_zk_verifier().unwrap()).set_expected(&commitment);

    let plan_id = client.create_inheritance_plan(&plan_params(
        &env,
        &owner,
        &token,
        10_000,
        &one_beneficiary(&env, 123456),
    ));
    client.deactivate_inheritance_plan(&owner, &plan_id);

    let result = client.try_verify_genetic_kin_claim(
        &claimant,
        &plan_id,
        &Bytes::from_slice(&env, b"proof"),
        &vec![&env, commitment],
    );
    assert_eq!(result, Err(Ok(InheritanceError::PlanNotActive)));
}

#[test]
fn proof_verification_rejects_a_nonexistent_plan() {
    let env = Env::default();
    let (client, _token, _owner) = setup_verifier(&env);
    let claimant = Address::generate(&env);

    let result = client.try_verify_genetic_kin_claim(
        &claimant,
        &999u64,
        &Bytes::from_slice(&env, b"proof"),
        &Vec::new(&env),
    );
    assert_eq!(result, Err(Ok(InheritanceError::PlanNotFound)));
}

#[test]
fn set_zk_verifier_is_admin_only() {
    let env = Env::default();
    let (client, _token, _admin, _owner) = setup(&env);
    let stranger = Address::generate(&env);
    let verifier_id = env.register_contract(None, MockZkVerifier);

    let result = client.try_set_zk_verifier(&stranger, &verifier_id);
    assert!(result.is_err());
    assert!(client.get_zk_verifier().is_none());
}

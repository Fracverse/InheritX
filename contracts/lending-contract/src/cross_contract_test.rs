#![cfg(test)]
#![allow(clippy::all)]

use crate::{LendingContract, LendingContractClient};
use inheritance_contract::{
    CreateInheritancePlanParams, DistributionMethod, InheritanceContract,
    InheritanceContractClient, InheritanceError,
};
use loan_nft::{LoanNFT, LoanNFTClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, vec, Address, Bytes, Env, String, Vec,
};

const YEAR: u64 = 365 * 24 * 60 * 60;
const INACTIVITY_PERIOD: u64 = 60 * 60;
const PLAN_AMOUNT: u64 = 10_000;
const PLAN_NET_AMOUNT: u64 = 9_800;
const LP_DEPOSIT: u64 = 100_000;
const LOAN_AMOUNT: u64 = 50_000;
const COLLATERAL_AMOUNT: u64 = 75_000;

#[test]
fn test_inheritance_lending_nft_lifecycle_settles_and_reconciles_balances() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    env.ledger().set_timestamp(1_000_000);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let borrower = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let collateral_admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let collateral_id = env
        .register_stellar_asset_contract_v2(collateral_admin)
        .address();
    let token = token::Client::new(&env, &token_id);
    let token_issuer = token::StellarAssetClient::new(&env, &token_id);
    let collateral = token::Client::new(&env, &collateral_id);
    let collateral_issuer = token::StellarAssetClient::new(&env, &collateral_id);

    token_issuer.mint(&owner, &150_000);
    token_issuer.mint(&borrower, &100_000);
    collateral_issuer.mint(&borrower, &150_000);
    let owner_starting_balance = token.balance(&owner);
    let initial_token_supply = token.balance(&owner) + token.balance(&borrower);

    let inheritance_id = env.register_contract(None, InheritanceContract);
    let inheritance = InheritanceContractClient::new(&env, &inheritance_id);
    inheritance.initialize_admin(&admin);
    inheritance.submit_kyc(&owner);
    inheritance.approve_kyc(&admin, &owner);
    inheritance.submit_kyc(&beneficiary);
    inheritance.approve_kyc(&admin, &beneficiary);

    let lending_id = env.register_contract(None, LendingContract);
    let lending = LendingContractClient::new(&env, &lending_id);
    lending.initialize(&admin, &token_id, &500, &2_000, &15_000, &10_000);
    lending.whitelist_collateral(&admin, &collateral_id);

    let nft_id = env.register_contract(None, LoanNFT);
    let nft = LoanNFTClient::new(&env, &nft_id);
    nft.initialize(&lending_id);
    lending.set_nft_token(&admin, &nft_id);
    lending.set_inheritance_contract(&admin, &inheritance_id);
    inheritance.set_lending_contract(&admin, &lending_id);
    assert_eq!(
        lending.get_inheritance_contract(),
        Some(inheritance_id.clone())
    );
    assert_eq!(inheritance.get_lending_contract(), Some(lending_id.clone()));

    let email = String::from_str(&env, "beneficiary@example.com");
    let mut bank_account = Bytes::new(&env);
    bank_account.extend_from_slice(b"test-bank-account");
    let beneficiaries = vec![
        &env,
        (
            String::from_str(&env, "Beneficiary"),
            email.clone(),
            123_456u32,
            bank_account,
            10_000u32,
            1u32,
            None,
        ),
    ];
    let plan_id = inheritance.create_inheritance_plan(&CreateInheritancePlanParams {
        owner: owner.clone(),
        token: token_id.clone(),
        plan_name: String::from_str(&env, "Cross-contract lifecycle"),
        description: String::from_str(&env, "Lifecycle settlement test"),
        total_amount: PLAN_AMOUNT,
        distribution_method: DistributionMethod::Monthly,
        beneficiaries_data: beneficiaries.clone(),
        is_lendable: true,
        guardians: Vec::new(&env),
        guardian_threshold: 0,
    });

    let plan = inheritance
        .get_plan_details(&plan_id)
        .expect("created inheritance plan");
    assert_eq!(plan.owner, owner);
    assert!(inheritance.verify_plan_ownership(&plan_id, &owner));
    assert!(!inheritance.verify_plan_ownership(&plan_id, &beneficiary));
    assert_eq!(plan.token, token_id);
    assert_eq!(plan.total_amount, PLAN_NET_AMOUNT);
    assert!(plan.is_active);
    assert_eq!(plan.beneficiaries.len(), 1);
    assert!(!plan.beneficiaries.get(0).expect("beneficiary").is_claimed);
    let plan_vault = inheritance
        .get_plan_vault_address(&plan_id)
        .expect("created plan vault address");
    assert_eq!(token.balance(&plan_vault), PLAN_NET_AMOUNT as i128);
    assert_eq!(
        token.balance(&admin),
        (PLAN_AMOUNT - PLAN_NET_AMOUNT) as i128
    );
    assert_eq!(
        inheritance.try_claim_inheritance_plan(&plan_id, &beneficiary, &email, &123_456),
        Err(Ok(InheritanceError::PlanNotExpired))
    );
    assert_eq!(token.balance(&beneficiary), 0);

    inheritance.update_plan(&owner, &plan_id, &beneficiaries, &604_800, &true);
    lending.deposit(&owner, &token_id, &LP_DEPOSIT);

    let pool_after_deposit = lending.get_pool_state(&token_id);
    assert_eq!(pool_after_deposit.total_deposits, LP_DEPOSIT);
    assert_eq!(pool_after_deposit.total_borrowed, 0);
    assert_eq!(pool_after_deposit.total_shares, LP_DEPOSIT);
    assert_eq!(lending.get_shares_of(&token_id, &owner), LP_DEPOSIT - 1_000);
    assert_eq!(token.balance(&lending_id), LP_DEPOSIT as i128);

    let first_loan_id = lending.borrow(
        &borrower,
        &token_id,
        &LOAN_AMOUNT,
        &collateral_id,
        &COLLATERAL_AMOUNT,
        &YEAR,
    );
    assert_eq!(nft.owner_of(&first_loan_id), Some(borrower.clone()));
    let first_metadata = nft
        .get_metadata(&first_loan_id)
        .expect("first loan NFT metadata");
    assert_eq!(first_metadata.borrower, borrower);
    assert_eq!(first_metadata.principal, LOAN_AMOUNT);
    assert_eq!(nft.balance_of(&borrower), 1);
    assert_eq!(nft.total_supply(), 1);

    env.ledger().set_timestamp(env.ledger().timestamp() + YEAR);
    assert_eq!(lending.get_repayment_amount(&borrower), LOAN_AMOUNT + 7_500);
    assert_eq!(lending.repay(&borrower), LOAN_AMOUNT + 7_500);
    assert_eq!(nft.owner_of(&first_loan_id), None);
    assert_eq!(nft.balance_of(&borrower), 0);
    let pool_after_first_repayment = lending.get_pool_state(&token_id);
    assert_eq!(pool_after_first_repayment.total_deposits, 106_750);
    assert_eq!(pool_after_first_repayment.total_borrowed, 0);
    assert_eq!(pool_after_first_repayment.retained_yield, 375);
    assert!(pool_after_first_repayment.total_deposits > LP_DEPOSIT);

    inheritance.register_yield_position(&owner, &plan_id, &token_id);
    let registered_position = lending
        .get_plan_yield_position(&plan_id)
        .expect("registered inheritance yield position");
    assert_eq!(registered_position.principal, PLAN_NET_AMOUNT);
    assert!(registered_position.active);

    let second_loan_id = lending.borrow(
        &borrower,
        &token_id,
        &LOAN_AMOUNT,
        &collateral_id,
        &COLLATERAL_AMOUNT,
        &(2 * YEAR),
    );
    assert_eq!(nft.owner_of(&second_loan_id), Some(borrower.clone()));
    assert_eq!(nft.balance_of(&borrower), 1);
    env.ledger().set_timestamp(env.ledger().timestamp() + YEAR);

    let accrued_yield = lending.get_accrued_plan_yield(&plan_id);
    assert!(accrued_yield > 0, "registered plan yield must accrue");
    assert!(accrued_yield <= pool_after_first_repayment.retained_yield);
    assert!(lending.get_repayment_amount(&borrower) > LOAN_AMOUNT);
    lending.repay(&borrower);
    assert_eq!(lending.get_loan_by_id(&second_loan_id), None);
    assert_eq!(nft.owner_of(&second_loan_id), None);
    assert_eq!(nft.balance_of(&borrower), 0);
    assert_eq!(nft.total_supply(), 0);
    assert_eq!(collateral.balance(&borrower), 150_000);
    assert_eq!(collateral.balance(&lending_id), 0);

    let pool_before_withdrawal = lending.get_pool_state(&token_id);
    assert_eq!(pool_before_withdrawal.total_borrowed, 0);
    assert_eq!(
        token.balance(&lending_id),
        (pool_before_withdrawal.total_deposits
            + pool_before_withdrawal.retained_yield
            + pool_before_withdrawal.bad_debt_reserve) as i128
    );
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + 1);
    let redeemed = lending.withdraw(&owner, &token_id, &(LP_DEPOSIT - 1_000));
    assert!(redeemed > LP_DEPOSIT, "lender receives accrued interest");
    assert_eq!(
        token.balance(&owner),
        owner_starting_balance - PLAN_AMOUNT as i128 - LP_DEPOSIT as i128 + redeemed as i128
    );
    assert_eq!(lending.get_shares_of(&token_id, &owner), 0);
    let pool_after_withdrawal = lending.get_pool_state(&token_id);
    assert_eq!(
        token.balance(&lending_id),
        (pool_after_withdrawal.total_deposits
            + pool_after_withdrawal.retained_yield
            + pool_after_withdrawal.bad_debt_reserve) as i128
    );

    inheritance.add_inactivity_trigger(&owner, &plan_id, &INACTIVITY_PERIOD);
    let configured_trigger = inheritance
        .get_trigger_conditions(&plan_id)
        .expect("inactivity trigger configured");
    let initial_activity = configured_trigger.last_activity;
    let unauthorized_ping = inheritance.try_record_activity(&unauthorized, &plan_id);
    assert_eq!(unauthorized_ping, Err(Ok(InheritanceError::Unauthorized)));
    env.ledger().set_timestamp(initial_activity + 10);
    inheritance.record_activity(&owner, &plan_id);
    let pinged_activity = inheritance
        .get_trigger_conditions(&plan_id)
        .expect("trigger state after proof of life")
        .last_activity;
    assert_eq!(pinged_activity, initial_activity + 10);

    env.ledger()
        .set_timestamp(pinged_activity + INACTIVITY_PERIOD - 1);
    assert!(!inheritance.check_trigger_conditions(&plan_id));
    assert_eq!(token.balance(&beneficiary), 0);

    env.ledger()
        .set_timestamp(pinged_activity + INACTIVITY_PERIOD);
    assert!(inheritance.check_trigger_conditions(&plan_id));
    inheritance.auto_trigger_check(&plan_id);
    let triggered_plan = inheritance
        .get_plan_details(&plan_id)
        .expect("plan remains available after trigger");
    assert!(!triggered_plan.is_lendable);
    assert!(inheritance.get_inheritance_trigger(&plan_id).is_some());

    inheritance.unregister_yield_position(&owner, &plan_id);
    let settled_yield_position = lending
        .get_plan_yield_position(&plan_id)
        .expect("settled yield position remains auditable");
    assert!(!settled_yield_position.active);
    assert_eq!(settled_yield_position.principal, 0);

    inheritance.claim_inheritance_plan(&plan_id, &beneficiary, &email, &123_456);
    assert_eq!(token.balance(&beneficiary), PLAN_NET_AMOUNT as i128);
    assert_eq!(token.balance(&plan_vault), 0);
    let settled_plan = inheritance
        .get_plan_details(&plan_id)
        .expect("settled inheritance plan");
    assert_eq!(settled_plan.total_amount, 0);
    assert!(
        settled_plan
            .beneficiaries
            .get(0)
            .expect("beneficiary")
            .is_claimed
    );
    assert_eq!(
        inheritance.get_claimed_plan(&owner, &plan_id).total_amount,
        0
    );
    assert_eq!(
        inheritance.try_claim_inheritance_plan(&plan_id, &beneficiary, &email, &123_456),
        Err(Ok(InheritanceError::AlreadyClaimed))
    );

    assert_eq!(nft.total_supply(), 0);
    assert_eq!(nft.owner_of(&first_loan_id), None);
    assert_eq!(nft.owner_of(&second_loan_id), None);
    assert_eq!(lending.get_loan_by_id(&first_loan_id), None);

    let final_balances = token.balance(&owner)
        + token.balance(&borrower)
        + token.balance(&admin)
        + token.balance(&beneficiary)
        + token.balance(&lending_id);
    assert_eq!(final_balances, initial_token_supply);
}

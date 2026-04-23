#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

fn create_token_addr(env: &Env) -> Address {
    let admin = Address::generate(env);
    env.register_stellar_asset_contract_v2(admin).address()
}

fn sac_client<'a>(env: &'a Env, token: &'a Address) -> token::StellarAssetClient<'a> {
    token::StellarAssetClient::new(env, token)
}

fn setup(env: &Env) -> (BorrowingContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let collateral_addr = create_token_addr(env);
    let contract_id = env.register_contract(None, BorrowingContract);
    let client = BorrowingContractClient::new(env, &contract_id);
    client.initialize(&admin, &15000, &12000, &500);
    client.whitelist_collateral(&admin, &collateral_addr);
    (client, collateral_addr, admin)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, BorrowingContract);
    let client = BorrowingContractClient::new(&env, &contract_id);
    client.initialize(&admin, &15000, &12000, &500);
    assert_eq!(client.get_collateral_ratio(), 15000);
}

#[test]
fn test_create_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(loan_id, 1);
    let loan = client.get_loan(&loan_id);
    assert_eq!(loan.principal, 1000);
    assert!(loan.is_active);
    assert_eq!(loan.extension_count, 0);
}

#[test]
fn test_repay_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    client.repay_loan(&loan_id, &1000);
    let loan = client.get_loan(&loan_id);
    assert!(!loan.is_active);
}

#[test]
fn test_insufficient_collateral() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1000);
    let result = client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1000);
    assert_eq!(result, Err(Ok(BorrowingError::InsufficientCollateral)));
}

#[test]
fn test_liquidation() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let collateral_addr = create_token_addr(&env);
    let contract_id = env.register_contract(None, BorrowingContract);
    let client = BorrowingContractClient::new(&env, &contract_id);
    client.initialize(&admin, &12000, &13000, &500);
    client.whitelist_collateral(&admin, &collateral_addr);
    let borrower = Address::generate(&env);
    let liquidator = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1200);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1200);
    client.liquidate(&liquidator, &loan_id, &1000);
    let loan = client.get_loan(&loan_id);
    assert!(!loan.is_active);
}

#[test]
fn test_partial_liquidation() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let collateral_addr = create_token_addr(&env);
    let contract_id = env.register_contract(None, BorrowingContract);
    let client = BorrowingContractClient::new(&env, &contract_id);
    client.initialize(&admin, &12000, &13000, &500);
    client.whitelist_collateral(&admin, &collateral_addr);
    let borrower = Address::generate(&env);
    let liquidator = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1200);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1200);

    client.liquidate(&liquidator, &loan_id, &500);

    let loan = client.get_loan(&loan_id);
    assert!(loan.is_active);
    assert_eq!(loan.amount_repaid, 500);
    assert_eq!(loan.collateral_amount, 675); // 1200 - (500 + 500 * 5%) = 675

    let hf = client.get_health_factor(&loan_id);
    assert_eq!(hf, 13500); // 675 * 10000 / 500
}

#[test]
fn test_global_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);
    let borrower = Address::generate(&env);

    sac_client(&env, &collateral_addr).mint(&borrower, &3000);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

    client.set_global_pause(&admin, &true);
    assert!(client.is_global_paused());

    let result = client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(result, Err(Ok(BorrowingError::Paused)));

    client.repay_loan(&loan_id, &500);
    let loan = client.get_loan(&loan_id);
    assert_eq!(loan.amount_repaid, 500);

    client.set_global_pause(&admin, &false);
    assert!(!client.is_global_paused());

    let new_loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(new_loan_id, 2);
}

#[test]
fn test_vault_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);
    let borrower = Address::generate(&env);

    sac_client(&env, &collateral_addr).mint(&borrower, &3000);

    client.set_vault_pause(&admin, &collateral_addr, &true);
    assert!(client.is_vault_paused(&collateral_addr));

    let result = client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(result, Err(Ok(BorrowingError::Paused)));

    client.set_vault_pause(&admin, &collateral_addr, &false);
    assert!(!client.is_vault_paused(&collateral_addr));

    let new_loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(new_loan_id, 1);
}

#[test]
fn test_liquidation_auction() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let collateral_addr = create_token_addr(&env);
    let contract_id = env.register_contract(None, BorrowingContract);
    let client = BorrowingContractClient::new(&env, &contract_id);
    client.initialize(&admin, &12000, &13000, &500);
    client.whitelist_collateral(&admin, &collateral_addr);

    let borrower = Address::generate(&env);
    let liquidator = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1200);

    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1200);

    let hf = client.get_health_factor(&loan_id);
    assert_eq!(hf, 12000);

    // Start auction
    client.start_liquidation_auction(&loan_id, &1000, &100, &2000);

    // Place bid
    sac_client(&env, &collateral_addr).mint(&liquidator, &1000);
    client.bid_on_liquidation(&liquidator, &loan_id, &1000);

    // Execute auction
    client.execute_auction(&loan_id);

    let loan = client.get_loan(&loan_id);
    assert!(!loan.is_active);
}

// ─────────────────────────────────────────────────
// Loan modification tests
// ─────────────────────────────────────────────────

#[test]
fn test_get_extension_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    // principal=1000, fee = 1% of 1000 = 10
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(client.get_extension_fee(&loan_id), 10);
}

#[test]
fn test_get_extension_fee_after_partial_repay() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    client.repay_loan(&loan_id, &400);
    // remaining debt = 600, fee = 1% of 600 = 6
    assert_eq!(client.get_extension_fee(&loan_id), 6);
}

#[test]
fn test_extend_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    // mint extra for fee (10) on top of collateral (1500)
    sac_client(&env, &collateral_addr).mint(&borrower, &1510);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

    client.extend_loan(&loan_id, &2000000);

    let loan = client.get_loan(&loan_id);
    assert_eq!(loan.due_date, 2000000);
    assert_eq!(loan.extension_count, 1);
}

#[test]
fn test_extend_loan_max_extensions() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    // mint enough for 2 extension fees (10 each) + collateral
    sac_client(&env, &collateral_addr).mint(&borrower, &1530);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

    client.extend_loan(&loan_id, &2000000);
    client.extend_loan(&loan_id, &3000000);

    // Third extension should fail
    let result = client.try_extend_loan(&loan_id, &4000000);
    assert_eq!(result, Err(Ok(BorrowingError::MaxExtensionsReached)));

    let loan = client.get_loan(&loan_id);
    assert_eq!(loan.extension_count, 2);
    assert_eq!(loan.due_date, 3000000);
}

#[test]
fn test_extend_inactive_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    client.repay_loan(&loan_id, &1000);

    let result = client.try_extend_loan(&loan_id, &2000000);
    assert_eq!(result, Err(Ok(BorrowingError::LoanNotActive)));
}

#[test]
fn test_get_max_additional_borrow() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    // collateral=1500, ratio=15000 (150%), max_debt = 1500 * 10000 / 15000 = 1000
    // current_debt = 1000, max_additional = 0
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(client.get_max_additional_borrow(&loan_id), 0);
}

#[test]
fn test_get_max_additional_borrow_after_repay() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    client.repay_loan(&loan_id, &400);
    // max_debt = 1000, current_debt = 600, max_additional = 400
    assert_eq!(client.get_max_additional_borrow(&loan_id), 400);
}

#[test]
fn test_increase_loan_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &600, &5, &1000000, &collateral_addr, &1500);
    // max_debt = 1000, current_debt = 600, can borrow 400 more
    client.increase_loan_amount(&loan_id, &400);
    let loan = client.get_loan(&loan_id);
    assert_eq!(loan.principal, 1000);
}

#[test]
fn test_increase_loan_amount_exceeds_collateral() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    // max_additional = 0, so any increase should fail
    let result = client.try_increase_loan_amount(&loan_id, &1);
    assert_eq!(result, Err(Ok(BorrowingError::InsufficientCollateral)));
}

#[test]
fn test_increase_inactive_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    client.repay_loan(&loan_id, &1000);

    let result = client.try_increase_loan_amount(&loan_id, &100);
    assert_eq!(result, Err(Ok(BorrowingError::LoanNotActive)));
}

#[test]
fn test_increase_loan_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &600, &5, &1000000, &collateral_addr, &1500);

    let result = client.try_increase_loan_amount(&loan_id, &0);
    assert_eq!(result, Err(Ok(BorrowingError::InvalidAmount)));
}

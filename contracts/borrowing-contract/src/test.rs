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
    client.liquidate(&liquidator, &loan_id);
    let loan = client.get_loan(&loan_id);
    assert!(!loan.is_active);
}

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger, token, Address, Env};

// ─────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────

fn create_token_addr(env: &Env) -> Address {
    let token_admin = Address::generate(env);
    env.register_stellar_asset_contract_v2(token_admin)
        .address()
}

fn sac_client<'a>(env: &'a Env, token: &'a Address) -> token::StellarAssetClient<'a> {
    token::StellarAssetClient::new(env, token)
}

fn tok_client<'a>(env: &'a Env, token: &'a Address) -> token::Client<'a> {
    token::Client::new(env, token)
}

fn mint_to(env: &Env, token: &Address, to: &Address, amount: i128) {
    sac_client(env, token).mint(to, &amount);
}

// ─────────────────────────────────────────────────
// Setup: returns (client, token_addr, admin)
// ─────────────────────────────────────────────────
fn setup(env: &Env) -> (LendingContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let token_addr = create_token_addr(env);

    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr, &1000u32); // 10% APY

    (client, token_addr, admin)
}

// ─────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────

#[test]
fn test_initialize_once() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, admin) = setup(&env);

    // Second init must fail
    let result = client.try_initialize(&admin, &token_addr, &1000u32);
    assert!(result.is_err());
}

#[test]
fn test_deposit_mints_shares() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);

    let shares = client.deposit(&depositor, &1000u64);
    // First deposit: 1:1 ratio
    assert_eq!(shares, 1000u64);
    assert_eq!(client.get_shares_of(&depositor), 1000u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_deposits, 1000);
    assert_eq!(pool.total_shares, 1000);
    assert_eq!(pool.total_borrowed, 0);
}

#[test]
fn test_second_deposit_proportional_shares() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor1 = Address::generate(&env);
    let depositor2 = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor1, 10_000);
    mint_to(&env, &token_addr, &depositor2, 10_000);

    // First deposit: 1000 tokens → 1000 shares
    client.deposit(&depositor1, &1000u64);

    // Second deposit: same ratio → 500 tokens → 500 shares
    let shares2 = client.deposit(&depositor2, &500u64);
    assert_eq!(shares2, 500u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_deposits, 1500);
    assert_eq!(pool.total_shares, 1500);
}

#[test]
fn test_withdraw_burns_shares_and_returns_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);

    client.deposit(&depositor, &1000u64);
    let balance_before = tok_client(&env, &token_addr).balance(&depositor);

    // Withdraw 500 shares → should get 500 tokens back
    let returned = client.withdraw(&depositor, &500u64);
    assert_eq!(returned, 500u64);
    assert_eq!(
        tok_client(&env, &token_addr).balance(&depositor),
        balance_before + 500
    );
    assert_eq!(client.get_shares_of(&depositor), 500u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_deposits, 500);
    assert_eq!(pool.total_shares, 500);
}

#[test]
fn test_withdraw_fails_not_enough_shares() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);
    client.deposit(&depositor, &1000u64);

    // Try to withdraw more shares than owned
    let result = client.try_withdraw(&depositor, &2000u64);
    assert!(result.is_err());
}

#[test]
fn test_borrow_reduces_available_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);
    client.deposit(&depositor, &1000u64);

    let borrow_amount = 400u64;
    let balance_before = tok_client(&env, &token_addr).balance(&borrower);
    client.borrow(&borrower, &borrow_amount);

    assert_eq!(
        tok_client(&env, &token_addr).balance(&borrower),
        balance_before + 400
    );

    let pool = client.get_pool_state();
    assert_eq!(pool.total_borrowed, 400);
    assert_eq!(pool.total_deposits, 1000);

    assert_eq!(client.available_liquidity(), 600u64);
}

#[test]
fn test_borrow_fails_if_insufficient_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);
    client.deposit(&depositor, &1000u64);

    let result = client.try_borrow(&depositor, &1001u64);
    assert!(result.is_err());
}

#[test]
fn test_borrow_fails_with_existing_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);
    client.deposit(&depositor, &1000u64);
    client.borrow(&borrower, &200u64);

    // Second borrow should fail
    let result = client.try_borrow(&borrower, &100u64);
    assert!(result.is_err());
}

#[test]
fn test_repay_restores_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);
    mint_to(&env, &token_addr, &borrower, 10_000); // pre-fund borrower for repayment

    client.deposit(&depositor, &1000u64);
    client.borrow(&borrower, &400u64);

    assert_eq!(client.available_liquidity(), 600u64);

    let repaid = client.repay(&borrower);
    assert_eq!(repaid, 400u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_borrowed, 0);
    assert_eq!(pool.total_deposits, 1000);
    assert_eq!(client.available_liquidity(), 1000u64);

    // Loan should be gone
    let loan = client.get_loan(&borrower);
    assert!(loan.is_none());
}

#[test]
fn test_repay_fails_with_no_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, admin) = setup(&env);

    let result = client.try_repay(&admin);
    assert!(result.is_err());
}

#[test]
fn test_withdraw_fails_if_funds_are_borrowed() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);

    client.deposit(&depositor, &1000u64);
    client.borrow(&borrower, &900u64); // only 100 tokens left un-borrowed

    // Depositor tries to withdraw 500 → only 100 available
    let result = client.try_withdraw(&depositor, &500u64);
    assert!(result.is_err());

    // Can still withdraw 100's worth of shares
    assert!(client.try_withdraw(&depositor, &100u64).is_ok());
}

#[test]
fn test_available_liquidity_before_and_after() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);
    mint_to(&env, &token_addr, &borrower, 10_000);

    assert_eq!(client.available_liquidity(), 0u64);

    client.deposit(&depositor, &2000u64);
    assert_eq!(client.available_liquidity(), 2000u64);

    client.borrow(&borrower, &1500u64);
    assert_eq!(client.available_liquidity(), 500u64);

    client.repay(&borrower);
    assert_eq!(client.available_liquidity(), 2000u64);
}

#[test]
fn test_get_loan_returns_none_when_no_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, _admin) = setup(&env);

    let no_loan_addr = Address::generate(&env);
    let loan = client.get_loan(&no_loan_addr);
    assert!(loan.is_none());
}

#[test]
fn test_get_loan_returns_record_when_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);

    client.deposit(&depositor, &1000u64);
    client.borrow(&borrower, &300u64);

    let loan = client.get_loan(&borrower).unwrap();
    assert_eq!(loan.amount, 300u64);
    assert_eq!(loan.borrower, borrower);
}

#[test]
fn test_invalid_amounts_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, admin) = setup(&env);

    let depositor = Address::generate(&env);
    assert!(client.try_deposit(&depositor, &0u64).is_err());
    assert!(client.try_withdraw(&depositor, &0u64).is_err());
    assert!(client.try_borrow(&admin, &0u64).is_err());
}
#[test]
fn test_interest_accrual() {
    let env = Env::default();
    env.mock_all_auths();
    // 10% APY (1000 bps)
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    // 1. Deposit 10,000 → 10,000 shares
    client.deposit(&depositor, &10_000u64);

    // 2. Borrow 5,000
    client.borrow(&borrower, &5_000u64);

    // 3. Jump time by 1 year (31,536,000 seconds)
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 31_536_000);

    // 4. Expected interest: 5,000 * 0.10 * 1 year = 500
    let repayment_amount = client.get_repayment_amount(&borrower);
    assert_eq!(repayment_amount, 5_500u64);

    // 5. Repay
    client.repay(&borrower);

    // 6. Verify pool state
    let pool = client.get_pool_state();
    // total_deposits should be 10,000 (initial) + 500 (interest) = 10,500
    assert_eq!(pool.total_deposits, 10_500);
    assert_eq!(pool.total_borrowed, 0);

    // 7. Verify depositor can withdraw more than they put in
    // shares = 10,000, pool_shares = 10,000, pool_deposits = 10,500
    // amount = 10,000 * 10,500 / 10,000 = 10,500
    let withdrawn = client.withdraw(&depositor, &10_000u64);
    assert_eq!(withdrawn, 10_500);
}

#[test]
fn test_interest_precision_short_time() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);
    client.borrow(&borrower, &5_000u64);

    // 1 hour = 3600 seconds
    // Interest = (5000 * 1000 * 3600) / (10000 * 31536000) = 18000000000 / 315360000000 ≈ 0.057
    // Should be 0 due to truncation in simple implementation
    env.ledger().set_timestamp(env.ledger().timestamp() + 3600);

    let repayment_amount = client.get_repayment_amount(&borrower);
    assert_eq!(repayment_amount, 5_000u64);
}

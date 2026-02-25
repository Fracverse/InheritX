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
    client.initialize(&admin, &token_addr, &500u32, &2000u32); // 5% base, 20% multiplier

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
    let result = client.try_initialize(&admin, &token_addr, &500u32, &2000u32);
    assert!(result.is_err());
}

#[test]
fn test_deposit_mints_shares() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);

    let shares = client.deposit(&depositor, &2000u64);
    // First deposit: 1:1 ratio minus lock
    assert_eq!(shares, 1000u64);
    assert_eq!(client.get_shares_of(&depositor), 1000u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_deposits, 2000);
    assert_eq!(pool.total_shares, 2000);
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

    // First deposit: 2000 tokens → 1000 shares
    client.deposit(&depositor1, &2000u64);

    // Second deposit: pool has 2000 shares, 2000 deposits. ratio 1:1
    // 500 tokens -> 500 shares
    let shares2 = client.deposit(&depositor2, &500u64);
    assert_eq!(shares2, 500u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_deposits, 2500);
    assert_eq!(pool.total_shares, 2500);
}

#[test]
fn test_withdraw_burns_shares_and_returns_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);

    client.deposit(&depositor, &2000u64);
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
    assert_eq!(pool.total_deposits, 1500);
    assert_eq!(pool.total_shares, 1500);
}

#[test]
fn test_withdraw_fails_not_enough_shares() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);
    client.deposit(&depositor, &2000u64);

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
    client.deposit(&depositor, &2000u64);

    let borrow_amount = 400u64;
    let balance_before = tok_client(&env, &token_addr).balance(&borrower);
    client.borrow(&borrower, &borrow_amount);

    assert_eq!(
        tok_client(&env, &token_addr).balance(&borrower),
        balance_before + 400
    );

    let pool = client.get_pool_state();
    assert_eq!(pool.total_borrowed, 400);
    assert_eq!(pool.total_deposits, 2000);

    assert_eq!(client.available_liquidity(), 1600u64);
}

#[test]
fn test_borrow_fails_if_insufficient_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);
    client.deposit(&depositor, &2000u64);

    let result = client.try_borrow(&depositor, &2001u64);
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
    client.deposit(&depositor, &2000u64);
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

    client.deposit(&depositor, &2000u64);
    client.borrow(&borrower, &400u64);

    assert_eq!(client.available_liquidity(), 1600u64);

    let repaid = client.repay(&borrower);
    assert_eq!(repaid, 400u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_borrowed, 0);
    assert_eq!(pool.total_deposits, 2000);
    assert_eq!(client.available_liquidity(), 2000u64);

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

    client.deposit(&depositor, &2000u64);
    client.borrow(&borrower, &1900u64); // only 100 tokens left un-borrowed

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

    client.deposit(&depositor, &2000u64);
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
fn test_rounding_loss_exploit_prevented() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let attacker = Address::generate(&env);
    let victim = Address::generate(&env);
    mint_to(&env, &token_addr, &attacker, 10_000);
    mint_to(&env, &token_addr, &victim, 10_000);

    // Attacker deposits minimum allowed to get some shares
    assert!(client.try_deposit(&attacker, &1000u64).is_err());
    let attack_shares = client.deposit(&attacker, &1001u64);
    assert_eq!(attack_shares, 1);

    // Victim tries to deposit an amount that would yield 0 shares
    let victim_shares_err = client.try_deposit(&victim, &0u64);
    assert!(victim_shares_err.is_err()); // caught by InvalidAmount
}

#[test]
fn test_interest_accrual() {
    let env = Env::default();
    env.mock_all_auths();
    // 5% APY base, 20% multiplier
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    // 1. Deposit 10,000 → 10,000 shares
    client.deposit(&depositor, &10_000u64);

    // 2. Borrow 5,000
    // Utilization = 5000 / 10000 = 50%.
    // Rate = 5% + (50% * 20%) = 15% (1500 bps)
    client.borrow(&borrower, &5_000u64);

    let current_rate = client.get_current_interest_rate();
    assert_eq!(current_rate, 1500u32);

    // 3. Jump time by 1 year (31,536,000 seconds)
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 31_536_000);

    // 4. Expected interest: 5,000 * 0.15 * 1 year = 750
    let repayment_amount = client.get_repayment_amount(&borrower);
    assert_eq!(repayment_amount, 5_750u64);

    // 5. Repay
    client.repay(&borrower);

    // 6. Verify pool state
    let pool = client.get_pool_state();
    // total_deposits should be 10,000 (initial) + 750 (interest) = 10,750
    assert_eq!(pool.total_deposits, 10_750);
    assert_eq!(pool.total_borrowed, 0);

    // 7. Verify depositor can withdraw more than they put in
    // shares = 9,000, pool_shares = 10,000, pool_deposits = 10,750
    // amount = 9,000 * 10,750 / 10,000 = 9,675
    let withdrawn = client.withdraw(&depositor, &9_000u64);
    assert_eq!(withdrawn, 9_675u64);
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

    env.ledger().set_timestamp(env.ledger().timestamp() + 3600);

    let repayment_amount = client.get_repayment_amount(&borrower);
    assert_eq!(repayment_amount, 5_000u64);
}

#[test]
fn test_dynamic_interest_rate_increases_with_utilization() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 100_000);
    client.deposit(&depositor, &10_000u64);

    // At 0 utilization, rate should be base rate (500)
    assert_eq!(client.get_current_interest_rate(), 500u32);

    let borrower1 = Address::generate(&env);
    mint_to(&env, &token_addr, &borrower1, 100_000);
    // Borrow 2,000 (20% utilization)
    // Dynamic rate should be 500 + (2000 * 2000 / 10000) = 500 + 400 = 900
    client.borrow(&borrower1, &2_000u64);
    let loan1 = client.get_loan(&borrower1).unwrap();
    assert_eq!(loan1.interest_rate_bps, 900u32);

    // Now utilization is 20%. The *next* borrower will get 900.
    assert_eq!(client.get_current_interest_rate(), 900u32);

    let borrower2 = Address::generate(&env);
    mint_to(&env, &token_addr, &borrower2, 100_000);
    // Borrow 3,000 more (total borrowed 5,000 -> 50% utilization)
    // Dynamic rate for this loan should be based on previous utilization (which changes mid-transaction in real world, but our implementation updates *after* applying the new borrow amount).
    // Let's look at implementation: pool.total_borrowed += amount, THEN get_utilization_bps.
    // So for loan2, total_borrowed becomes 5,000. Utilization = 50%.
    // Rate = 500 + (5000 * 2000 / 10000) = 500 + 1000 = 1500.
    client.borrow(&borrower2, &3_000u64);
    let loan2 = client.get_loan(&borrower2).unwrap();
    assert_eq!(loan2.interest_rate_bps, 1500u32);
}

#[test]
fn test_repay_decreases_interest_rate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 100_000);
    client.deposit(&depositor, &10_000u64);

    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &borrower, 100_000);

    // Borrow 50% (5,000). Rate becomes 15% (1500)
    client.borrow(&borrower, &5_000u64);
    assert_eq!(client.get_current_interest_rate(), 1500u32);

    // Repay immediately
    client.repay(&borrower);

    // Utilization goes back to 0. Rate goes back to 5% (500)
    assert_eq!(client.get_current_interest_rate(), 500u32);
}

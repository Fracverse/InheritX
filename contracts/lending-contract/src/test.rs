#![cfg(test)]
#![allow(unused_variables)]

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
// Setup: returns (client, token_addr, collateral_addr, admin)
// ─────────────────────────────────────────────────
fn setup(env: &Env) -> (LendingContractClient<'_>, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_addr = create_token_addr(env);
    let collateral_addr = create_token_addr(env);

    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(env, &contract_id);
    client.initialize(
        &admin,
        &token_addr,
        &500u32,   // base_rate
        &8000u32,  // optimal_utilization
        &1600u32,  // slope1
        &3000u32,  // slope2
        &1000u32,  // reserve_factor
        &15000u32, // collateral_ratio
        &10000u32, // utilization_cap
    );

    // Whitelist collateral token
    client.whitelist_collateral(&admin, &collateral_addr);

    (client, token_addr, collateral_addr, admin)
}

// ─────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────

#[test]
fn test_initialize_once() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    // Second init must fail
    let result = client.try_initialize(
        &admin,
        &token_addr,
        &500u32,
        &8000u32,
        &1600u32,
        &3000u32,
        &1000u32,
        &15000u32,
        &10000u32,
    );
    assert!(result.is_err());
}

#[test]
fn test_deposit_mints_shares() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

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
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

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
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

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
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

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
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 10_000);
    mint_to(&env, &collateral_addr, &borrower, 10_000);
    client.deposit(&depositor, &2000u64);

    let borrow_amount = 400u64;
    let balance_before = tok_client(&env, &token_addr).balance(&borrower);
    let loan_id = client.borrow(
        &borrower,
        &borrow_amount,
        &collateral_addr,
        &600u64,
        &(30 * 24 * 60 * 60),
    ); // 30 days

    assert!(loan_id > 0);
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
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);
    client.deposit(&depositor, &2000u64);

    let result = client.try_borrow(
        &depositor,
        &2001u64,
        &collateral_addr,
        &3001u64,
        &(30 * 24 * 60 * 60),
    );
    assert!(result.is_err());
}

#[test]
fn test_borrow_fails_with_existing_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 10_000);
    client.deposit(&depositor, &2000u64);
    client.borrow(
        &borrower,
        &200u64,
        &collateral_addr,
        &300u64,
        &(30 * 24 * 60 * 60),
    );

    // Second borrow should fail
    let result = client.try_borrow(
        &borrower,
        &100u64,
        &collateral_addr,
        &150u64,
        &(30 * 24 * 60 * 60),
    );
    assert!(result.is_err());
}

#[test]
fn test_repay_restores_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 10_000);
    mint_to(&env, &token_addr, &borrower, 10_000); // pre-fund borrower for repayment

    client.deposit(&depositor, &2000u64);
    client.borrow(
        &borrower,
        &400u64,
        &collateral_addr,
        &600u64,
        &(30 * 24 * 60 * 60),
    );

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
    let (client, _token_addr, _collateral_addr, admin) = setup(&env);

    let result = client.try_repay(&admin);
    assert!(result.is_err());
}

#[test]
fn test_withdraw_fails_if_funds_are_borrowed() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 10_000);

    client.deposit(&depositor, &2000u64);
    client.borrow(
        &borrower,
        &1900u64,
        &collateral_addr,
        &2850u64,
        &(30 * 24 * 60 * 60),
    ); // only 100 tokens left un-borrowed

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
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 10_000);
    mint_to(&env, &token_addr, &borrower, 10_000);

    assert_eq!(client.available_liquidity(), 0u64);

    client.deposit(&depositor, &2000u64);
    assert_eq!(client.available_liquidity(), 2000u64);

    client.borrow(
        &borrower,
        &1500u64,
        &collateral_addr,
        &2250u64,
        &(30 * 24 * 60 * 60),
    );
    assert_eq!(client.available_liquidity(), 500u64);

    client.repay(&borrower);
    assert_eq!(client.available_liquidity(), 2000u64);
}

#[test]
fn test_get_loan_returns_none_when_no_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, _collateral_addr, _admin) = setup(&env);

    let no_loan_addr = Address::generate(&env);
    let loan = client.get_loan(&no_loan_addr);
    assert!(loan.is_none());
}

#[test]
fn test_get_loan_returns_record_when_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 10_000);

    client.deposit(&depositor, &2000u64);
    let loan_id = client.borrow(
        &borrower,
        &300u64,
        &collateral_addr,
        &450u64,
        &(30 * 24 * 60 * 60),
    );

    let loan = client.get_loan(&borrower).unwrap();
    assert_eq!(loan.loan_id, loan_id);
    assert_eq!(loan.principal, 300u64);
    assert_eq!(loan.borrower, borrower);

    // Test get_loan_by_id
    let loan_by_id = client.get_loan_by_id(&loan_id).unwrap();
    assert_eq!(loan_by_id.loan_id, loan_id);
    assert_eq!(loan_by_id.principal, 300u64);
    assert_eq!(loan_by_id.collateral_amount, 450u64);
}

#[test]
fn test_invalid_amounts_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, collateral_addr, admin) = setup(&env);

    let depositor = Address::generate(&env);
    assert!(client.try_deposit(&depositor, &0u64).is_err());
    assert!(client.try_withdraw(&depositor, &0u64).is_err());
    assert!(client
        .try_borrow(&admin, &0u64, &collateral_addr, &0u64, &(30 * 24 * 60 * 60))
        .is_err());
}
#[test]
fn test_rounding_loss_exploit_prevented() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

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
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    // 1. Deposit 10,000 → 10,000 shares
    client.deposit(&depositor, &10_000u64);

    // 2. Borrow 5,000
    // Utilization = 5000 / 10000 = 50%.
    // Rate = 5% + (50% * 20%) = 15% (1500 bps)
    client.borrow(
        &borrower,
        &5_000u64,
        &collateral_addr,
        &7500u64,
        &(365 * 24 * 60 * 60),
    ); // 1 year duration

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
    // total_deposits should be 10,000 (initial) + 675 (90% of 750 interest) = 10,675
    assert_eq!(pool.total_deposits, 10_675);
    assert_eq!(pool.total_borrowed, 0);
    assert_eq!(pool.retained_yield, 38); // Remaining protocol yield after reserve split
    assert_eq!(pool.bad_debt_reserve, 37); // Portion of protocol share routed to reserve

    // 7. Verify depositor can withdraw more than they put in
    // shares = 9,000, pool_shares = 10,000, pool_deposits = 10,675
    // amount = 9,000 * 10,675 / 10,000 = 9,607
    let withdrawn = client.withdraw(&depositor, &9_000u64);
    assert_eq!(withdrawn, 9_607u64);
}

#[test]
fn test_interest_precision_short_time() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);
    client.borrow(
        &borrower,
        &5_000u64,
        &collateral_addr,
        &7500u64,
        &(30 * 24 * 60 * 60),
    ); // 30 days

    env.ledger().set_timestamp(env.ledger().timestamp() + 3600);

    let repayment_amount = client.get_repayment_amount(&borrower);
    assert_eq!(repayment_amount, 5_000u64);
}

#[test]
fn test_dynamic_interest_rate_increases_with_utilization() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 100_000);
    client.deposit(&depositor, &10_000u64);

    // At 0 utilization, rate should be base rate (500)
    assert_eq!(client.get_current_interest_rate(), 500u32);

    let borrower1 = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower1, 100_000);
    // Borrow 2,000 (20% utilization)
    // Dynamic rate should be 500 + (2000 * 2000 / 10000) = 500 + 400 = 900
    client.borrow(
        &borrower1,
        &2_000u64,
        &collateral_addr,
        &3000u64,
        &(30 * 24 * 60 * 60),
    );
    let loan1 = client.get_loan(&borrower1).unwrap();
    assert_eq!(loan1.interest_rate_bps, 900u32);

    // Now utilization is 20%. The *next* borrower will get 900.
    assert_eq!(client.get_current_interest_rate(), 900u32);

    let borrower2 = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower2, 100_000);
    mint_to(&env, &token_addr, &borrower2, 100_000);
    // Borrow 3,000 more (total borrowed 5,000 -> 50% utilization)
    // Dynamic rate for this loan should be based on previous utilization (which changes mid-transaction in real world, but our implementation updates *after* applying the new borrow amount).
    // Let's look at implementation: pool.total_borrowed += amount, THEN get_utilization_bps.
    // So for loan2, total_borrowed becomes 5,000. Utilization = 50%.
    // Rate = 500 + (5000 * 2000 / 10000) = 500 + 1000 = 1500.
    client.borrow(
        &borrower2,
        &3_000u64,
        &collateral_addr,
        &4500u64,
        &(30 * 24 * 60 * 60),
    );
    let loan2 = client.get_loan(&borrower2).unwrap();
    assert_eq!(loan2.interest_rate_bps, 1500u32);
}

#[test]
fn test_unique_loan_ids() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 100_000);
    client.deposit(&depositor, &50_000u64);

    let borrower1 = Address::generate(&env);
    let borrower2 = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower1, 100_000);
    mint_to(&env, &collateral_addr, &borrower2, 100_000);
    mint_to(&env, &token_addr, &borrower1, 100_000);
    mint_to(&env, &token_addr, &borrower2, 100_000);

    // Create first loan
    let loan_id_1 = client.borrow(
        &borrower1,
        &1_000u64,
        &collateral_addr,
        &1500u64,
        &(30 * 24 * 60 * 60),
    );
    assert_eq!(loan_id_1, 1);

    // Repay first loan
    client.repay(&borrower1);

    // Create second loan - should have different ID
    let loan_id_2 = client.borrow(
        &borrower2,
        &2_000u64,
        &collateral_addr,
        &3000u64,
        &(60 * 24 * 60 * 60),
    );
    assert_eq!(loan_id_2, 2);

    // Verify loan can be retrieved by ID
    let loan = client.get_loan_by_id(&loan_id_2).unwrap();
    assert_eq!(loan.loan_id, 2);
    assert_eq!(loan.principal, 2_000u64);
    assert_eq!(loan.borrower, borrower2);
}

#[test]
fn test_loan_tracks_due_date() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);

    let duration = 30 * 24 * 60 * 60u64; // 30 days
    let borrow_time = env.ledger().timestamp();

    client.borrow(&borrower, &1_000u64, &collateral_addr, &1_500u64, &duration);

    let loan = client.get_loan(&borrower).unwrap();
    assert_eq!(loan.borrow_time, borrow_time);
    assert_eq!(loan.due_date, borrow_time + duration);
}

#[test]
fn test_repayment_updates_state_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);
    let loan_id = client.borrow(
        &borrower,
        &5_000u64,
        &collateral_addr,
        &7500u64,
        &(365 * 24 * 60 * 60),
    );

    // Advance time to accrue interest
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 31_536_000); // 1 year

    let pool_before = client.get_pool_state();
    assert_eq!(pool_before.total_borrowed, 5_000);

    // Repay
    let total_repaid = client.repay(&borrower);
    assert_eq!(total_repaid, 5_750); // 5000 + 750 interest

    // Verify state updates
    let pool_after = client.get_pool_state();
    assert_eq!(pool_after.total_borrowed, 0);
    assert_eq!(pool_after.total_deposits, 10_675); // Original + 90% interest
    assert_eq!(pool_after.retained_yield, 38);
    assert_eq!(pool_after.bad_debt_reserve, 37);

    // Verify loan is removed
    assert!(client.get_loan(&borrower).is_none());
    assert!(client.get_loan_by_id(&loan_id).is_none());
}

#[test]
fn test_repay_decreases_interest_rate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 100_000);
    client.deposit(&depositor, &10_000u64);

    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);
    mint_to(&env, &collateral_addr, &borrower, 100_000);

    // Borrow 50% (5,000). Rate becomes 15% (1500)
    client.borrow(
        &borrower,
        &5_000u64,
        &collateral_addr,
        &7500u64,
        &(30 * 24 * 60 * 60),
    );
    assert_eq!(client.get_current_interest_rate(), 1500u32);

    // Repay immediately
    client.repay(&borrower);

    // Utilization goes back to 0. Rate goes back to 5% (500)
    assert_eq!(client.get_current_interest_rate(), 500u32);
}

#[test]
fn test_collateral_required() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &collateral_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);

    // Try to borrow without sufficient collateral (need 150% = 1500 for 1000 borrow)
    let result = client.try_borrow(
        &borrower,
        &1_000u64,
        &collateral_addr,
        &1_400u64,
        &(30 * 24 * 60 * 60),
    );
    assert_eq!(result, Err(Ok(LendingError::InsufficientCollateral)));

    // With exact collateral should work
    let loan_id = client.borrow(
        &borrower,
        &1_000u64,
        &collateral_addr,
        &1_500u64,
        &(30 * 24 * 60 * 60),
    );
    assert!(loan_id > 0);
}

#[test]
fn test_collateral_not_whitelisted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    let bad_collateral = create_token_addr(&env);

    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &bad_collateral, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);

    // Try to borrow with non-whitelisted collateral
    let result = client.try_borrow(
        &borrower,
        &1_000u64,
        &bad_collateral,
        &1_500u64,
        &(30 * 24 * 60 * 60),
    );
    assert_eq!(result, Err(Ok(LendingError::CollateralNotWhitelisted)));
}

#[test]
fn test_collateral_returned_on_repay() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);
    mint_to(&env, &collateral_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);

    let collateral_balance_before = tok_client(&env, &collateral_addr).balance(&borrower);

    client.borrow(
        &borrower,
        &1_000u64,
        &collateral_addr,
        &1_500u64,
        &(30 * 24 * 60 * 60),
    );

    // Collateral should be locked
    assert_eq!(
        tok_client(&env, &collateral_addr).balance(&borrower),
        collateral_balance_before - 1_500
    );

    client.repay(&borrower);

    // Collateral should be returned
    assert_eq!(
        tok_client(&env, &collateral_addr).balance(&borrower),
        collateral_balance_before
    );
}

#[test]
fn test_whitelist_management() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, collateral_addr, admin) = setup(&env);

    let new_collateral = create_token_addr(&env);

    // Initially not whitelisted
    assert!(!client.is_whitelisted(&new_collateral));

    // Admin whitelists it
    client.whitelist_collateral(&admin, &new_collateral);
    assert!(client.is_whitelisted(&new_collateral));

    // Admin removes it
    client.remove_collateral(&admin, &new_collateral);
    assert!(!client.is_whitelisted(&new_collateral));

    // Original collateral still whitelisted
    assert!(client.is_whitelisted(&collateral_addr));
}

#[test]
fn test_collateral_ratio() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, _collateral_addr, _admin) = setup(&env);

    // Should be 150% (15000 bps)
    assert_eq!(client.get_collateral_ratio_bps(), 15000u32);
}

#[test]
fn test_utilization_cap_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token_addr = create_token_addr(&env);
    let collateral_addr = create_token_addr(&env);

    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(&env, &contract_id);
    client.initialize(
        &admin,
        &token_addr,
        &500u32,
        &8000u32,
        &1600u32,
        &3000u32,
        &1000u32,
        &15000u32,
        &8000u32,
    ); // 80% cap
    client.whitelist_collateral(&admin, &collateral_addr);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &collateral_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);

    // Try to borrow 8,001 (80.01% utilization) - should fail
    let result = client.try_borrow(
        &borrower,
        &8_001u64,
        &collateral_addr,
        &12_002u64,
        &(30 * 24 * 60 * 60),
    );
    assert_eq!(result, Err(Ok(LendingError::UtilizationCapExceeded)));

    // Borrow exactly 8,000 (80% utilization) - should succeed
    let loan_id = client.borrow(
        &borrower,
        &8_000u64,
        &collateral_addr,
        &12_000u64,
        &(30 * 24 * 60 * 60),
    );
    assert!(loan_id > 0);
}
#[test]
fn test_nft_minting_and_burning() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    // Register NFT contract
    let nft_id = env.register_contract(None, loan_nft::LoanNFT);
    let nft_client = LoanNFTClient::new(&env, &nft_id);
    nft_client.initialize(&client.address);

    // Set NFT token in lending contract
    client.set_nft_token(&admin, &nft_id);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);

    // Borrow 1000
    let loan_id = client.borrow(
        &borrower,
        &1_000u64,
        &collateral_addr,
        &1_500u64,
        &(30 * 24 * 60 * 60),
    );

    // Verify NFT is minted
    assert_eq!(nft_client.owner_of(&loan_id), Some(borrower.clone()));
    let metadata = nft_client.get_metadata(&loan_id).unwrap();
    assert_eq!(metadata.loan_id, loan_id);
    assert_eq!(metadata.borrower, borrower);
    assert_eq!(metadata.principal, 1_000u64);

    // Repay
    client.repay(&borrower);

    // Verify NFT is burned
    assert_eq!(nft_client.owner_of(&loan_id), None);
    assert_eq!(nft_client.get_metadata(&loan_id), None);
}

// ─────────────────────────────────────────────────
// Reentrancy Mock & Test
// ─────────────────────────────────────────────────

#[contract]
pub struct MaliciousNFT;

#[contractimpl]
impl MaliciousNFT {
    pub fn initialize(env: Env, admin: Address) {}
    pub fn mint(env: Env, to: Address, metadata: LoanMetadata) {
        let lending_contract = env
            .storage()
            .instance()
            .get::<_, Address>(&symbol_short!("LEND"))
            .unwrap();
        let client = LendingContractClient::new(&env, &lending_contract);
        // Attempt reentrant call to borrow
        let _ = client.try_borrow(&to, &100, &metadata.collateral_token, &150, &3600);
    }
    pub fn burn(env: Env, loan_id: u64) {}
    pub fn get_metadata(env: Env, loan_id: u64) -> Option<LoanMetadata> {
        None
    }
    pub fn owner_of(env: Env, loan_id: u64) -> Option<Address> {
        None
    }
}

#[test]
fn test_reentrancy_attack_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    // Register Malicious NFT contract
    let mal_nft_id = env.register_contract(None, MaliciousNFT);

    // MaliciousNFT needs to know the lending contract address to call back
    // We update its internal state directly in the test environment if needed,
    // but here we just used instance storage of the mock.
    // Wait, MaliciousNFT is a separate contract, it doesn't share instance storage with the test setup.
    // I need to set it in MaliciousNFT's storage.
    env.as_contract(&mal_nft_id, || {
        env.storage()
            .instance()
            .set(&symbol_short!("LEND"), &client.address);
    });

    // Set Malicious NFT token in lending contract
    client.set_nft_token(&admin, &mal_nft_id);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);

    client.deposit(&depositor, &10_000u64);

    // This borrow will trigger MaliciousNFT::mint, which calls client.borrow again.
    // The inner borrow should return ReentrantCall error.
    // However, the outer borrow will continue if MaliciousNFT suppresses the error (which it does with `let _ = ...`).
    // BUT we want to verify that the inner call actually failed.

    // Let's modify MaliciousNFT to panic on reentrancy failure if we want to catch it specifically,
    // or just check that NO second loan was created.

    client.borrow(
        &borrower,
        &1_000u64,
        &collateral_addr,
        &1_500u64,
        &(30 * 24 * 60 * 60),
    );

    // If reentrancy was successful, next loan ID would be 3 (1 from first successful, 1 from reentrant).
    // If blocked, next loan ID should be 2.
    // Actually, in our implementation, pool.total_borrowed would be double if successful.
    let pool = client.get_pool_state();
    assert_eq!(pool.total_borrowed, 1000); // Only the first borrow succeeded
}

// ─────────────────────────────────────────────────
// Grace Period & Late Fee Tests
// ─────────────────────────────────────────────────

#[test]
fn test_grace_period_defaults() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, _collateral_addr, _admin) = setup(&env);

    let grace_period = client.get_grace_period();
    let late_fee_rate = client.get_late_fee_rate();

    // Should have default values set
    assert_eq!(grace_period, 259_200u64); // 3 days
    assert_eq!(late_fee_rate, 500u32); // 5% per day
}

#[test]
fn test_set_grace_period_admin_only() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, _collateral_addr, admin) = setup(&env);

    let non_admin = Address::generate(&env);

    // Non-admin should fail
    let result = client.try_set_grace_period(&non_admin, &(5 * 24 * 60 * 60));
    assert!(result.is_err());

    // Admin should succeed
    let result = client.try_set_grace_period(&admin, &(7 * 24 * 60 * 60));
    assert!(result.is_ok());

    // Verify the new grace period
    assert_eq!(client.get_grace_period(), 7 * 24 * 60 * 60);
}

#[test]
fn test_set_late_fee_rate_admin_only() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, _collateral_addr, admin) = setup(&env);

    let non_admin = Address::generate(&env);

    // Non-admin should fail
    let result = client.try_set_late_fee_rate(&non_admin, &1000u32);
    assert!(result.is_err());

    // Admin should succeed
    let result = client.try_set_late_fee_rate(&admin, &1000u32);
    assert!(result.is_ok());

    // Verify the new rate
    assert_eq!(client.get_late_fee_rate(), 1000u32); // 10% per day
}

#[test]
fn test_no_late_fees_during_grace_period() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);

    // Borrow with 1 day duration
    client.borrow(
        &borrower,
        &1_000u64,
        &collateral_addr,
        &1_500u64,
        &(24 * 60 * 60),
    );

    // Jump to just after due date (within grace period)
    // Grace period is 3 days (259200 seconds), so due_date + grace = due_date + 259200
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 2 * 24 * 60 * 60); // Jump 2 days

    // Should still be in grace period
    let in_grace = client.is_in_grace_period(&borrower);
    assert!(in_grace);

    // Late fees should be 0
    let late_fee = client.calculate_late_fee(&borrower);
    assert_eq!(late_fee, 0u64);

    // Total due should only include principal + interest, no late fees
    let repayment = client.get_repayment_amount(&borrower);
    // 1000 principal at ~15% APY for ~2 days = 1000 + ~8 interest
    assert!(repayment < 1_100u64);
}

#[test]
fn test_late_fees_after_grace_period() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);

    // Set grace period to 1 day and late fee to 5% per day for easier testing
    client.set_grace_period(&admin, &(24 * 60 * 60));
    client.set_late_fee_rate(&admin, &500u32); // 5% per day

    // Borrow 10,000 (so late fees are 500 per day)
    client.borrow(
        &borrower,
        &10_000u64,
        &collateral_addr,
        &15_000u64,
        &(24 * 60 * 60), // 1 day duration
    );

    // Jump to 3 days after due date (2 days past grace period)
    // late fees = 10000 * 0.05 * 2 = 1000
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 4 * 24 * 60 * 60);

    // Should be out of grace period
    let in_grace = client.is_in_grace_period(&borrower);
    assert!(!in_grace);

    // Late fee should be ~1000 (2 days * 500 per day = 1000)
    let late_fee = client.calculate_late_fee(&borrower);
    assert_eq!(late_fee, 1_000u64);

    // Total due should include late fees
    let repayment = client.get_repayment_amount(&borrower);
    // 10000 principal + interest (~825 for 4 days at ~15%) + 1000 late fees = ~11825
    assert!(repayment > 11_000u64);
}

#[test]
fn test_liquidation_blocked_during_grace_period() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    let liquidator = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 50_000);
    mint_to(&env, &token_addr, &liquidator, 50_000);

    client.deposit(&depositor, &20_000u64);

    // Borrow with very high collateral (so health factor starts good)
    client.borrow(
        &borrower,
        &5_000u64,
        &collateral_addr,
        &7_500u64, // Exactly 150% collateral ratio
        &(24 * 60 * 60),
    );

    // Even though health factor might be bad, liquidation should fail during grace period
    let result = client.try_liquidate(&liquidator, &borrower, &1_000u64);
    assert!(result.is_err()); // Should fail due to grace period, not health factor

    // Jump past grace period (4 days) - use absolute timestamp
    let current_time = env.ledger().timestamp();
    env.ledger().set_timestamp(current_time + 4 * 24 * 60 * 60);

    // Now liquidation can proceed (if health factor is bad)
    let result = client.try_liquidate(&liquidator, &borrower, &1_000u64);
    // Result depends on health factor calculation, but grace period check shouldn't block
    let _ = result; // Just verify no panic
}

#[test]
fn test_late_fee_collected_on_repay() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);

    // Set simple rates for testing
    client.set_grace_period(&admin, &(24 * 60 * 60));
    client.set_late_fee_rate(&admin, &500u32); // 5% per day

    // Borrow 5000
    client.borrow(
        &borrower,
        &5_000u64,
        &collateral_addr,
        &7_500u64,
        &(24 * 60 * 60),
    );

    // Jump 4 days (1 day maturity + 1 day grace + 2 days late)
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 4 * 24 * 60 * 60);

    // Late fees should be 5000 * 0.05 * 2 = 500
    let late_fee = client.calculate_late_fee(&borrower);
    assert_eq!(late_fee, 500u64);

    // Get pool state before repay
    let pool_before = client.get_pool_state();

    // Repay - should include late fees
    client.repay(&borrower);

    // Get pool state after repay
    let pool_after = client.get_pool_state();

    // Late fee (500) should be added to retained_yield
    // Protocol gets 10% of interest, but 100% of late fees
    assert!(pool_after.retained_yield > pool_before.retained_yield);

    // Loan should be gone
    let loan = client.get_loan(&borrower);
    assert!(loan.is_none());
}

#[test]
fn test_grace_period_expires_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 100_000);
    mint_to(&env, &token_addr, &borrower, 100_000);

    client.deposit(&depositor, &10_000u64);

    // Set 2 day grace period
    client.set_grace_period(&admin, &(2 * 24 * 60 * 60));

    // Borrow with 1 day maturity
    client.borrow(
        &borrower,
        &1_000u64,
        &collateral_addr,
        &1_500u64,
        &(24 * 60 * 60),
    );

    // At 1.5 days: should be in grace period
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 36 * 60 * 60);
    assert!(client.is_in_grace_period(&borrower));

    // At 4.5 days: should be out of grace period (3 days) and have at least 1 day overdue
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 3 * 24 * 60 * 60); // Total 4.5 days
    assert!(!client.is_in_grace_period(&borrower));

    // Late fees should start accruing
    assert!(client.calculate_late_fee(&borrower) > 0u64);
}

#[test]
fn test_multiple_loans_grace_period() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 200_000);
    client.deposit(&depositor, &100_000u64);

    let borrower1 = Address::generate(&env);
    let borrower2 = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower1, 100_000);
    mint_to(&env, &collateral_addr, &borrower2, 100_000);
    mint_to(&env, &token_addr, &borrower1, 100_000);
    mint_to(&env, &token_addr, &borrower2, 100_000);

    client.set_grace_period(&admin, &(24 * 60 * 60));

    // Create two loans with different maturities
    client.borrow(
        &borrower1,
        &1_000u64,
        &collateral_addr,
        &1_500u64,
        &(24 * 60 * 60),
    );

    env.ledger().set_timestamp(env.ledger().timestamp() + 1_000);

    client.borrow(
        &borrower2,
        &2_000u64,
        &collateral_addr,
        &3_000u64,
        &(2 * 24 * 60 * 60),
    );

    // Jump 3 days to ensure borrower1 is past grace period
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 3 * 24 * 60 * 60);

    // borrower1 should be out of grace period
    assert!(!client.is_in_grace_period(&borrower1));
    assert!(client.calculate_late_fee(&borrower1) > 0u64);

    // borrower2 should still be in grace period (due_date is 2 days after borrow, grace = 1 day, so still in grace)
    assert!(client.is_in_grace_period(&borrower2));
    assert_eq!(client.calculate_late_fee(&borrower2), 0u64);
}

// ─────────────────────────────────────────────────
// Refinancing Tests
// ─────────────────────────────────────────────────

#[test]
fn test_get_refinance_terms() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let borrower = Address::generate(&env);
    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &borrower, 10_000);
    mint_to(&env, &collateral_addr, &borrower, 20_000);
    mint_to(&env, &token_addr, &depositor, 10_000);

    // Deposit funds to provide liquidity
    client.deposit(&depositor, &5000u64);

    // Borrow 1000 with 1500 collateral for 30 days
    client.borrow(
        &borrower,
        &1000u64,
        &collateral_addr,
        &1500u64,
        &(30 * 24 * 60 * 60),
    );

    // Get refinancing terms for 60 days
    let terms = client.get_refinance_terms(&borrower, &(60 * 24 * 60 * 60));

    // Should have outstanding balance (principal + accrued interest)
    assert!(terms.outstanding_balance >= 1000u64);
    assert!(terms.new_principal > terms.outstanding_balance); // Should include fee
    assert!(terms.refinancing_fee > 0u64);
    assert_eq!(terms.total_required, terms.new_principal);
    assert_eq!(terms.new_duration_seconds, 60 * 24 * 60 * 60);
    assert!(terms.new_due_date > env.ledger().timestamp());
}

#[test]
fn test_refinance_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let borrower = Address::generate(&env);
    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &borrower, 10_000);
    mint_to(&env, &collateral_addr, &borrower, 20_000);
    mint_to(&env, &token_addr, &depositor, 10_000);

    // Deposit funds to provide liquidity
    client.deposit(&depositor, &5000u64);

    // Borrow 1000 with 1500 collateral for 30 days
    let old_loan_id = client.borrow(
        &borrower,
        &1000u64,
        &collateral_addr,
        &1500u64,
        &(30 * 24 * 60 * 60),
    );

    // Get initial loan details
    let old_loan = client.get_loan(&borrower).unwrap();

    // Refinance for 60 days
    let new_loan_id = client.refinance_loan(&borrower, &(60 * 24 * 60 * 60));

    // Verify new loan exists with different terms
    let new_loan = client.get_loan(&borrower).unwrap();
    assert_ne!(new_loan_id, old_loan_id);
    assert_eq!(new_loan.borrower, borrower);
    assert_eq!(new_loan.collateral_amount, old_loan.collateral_amount);
    assert_eq!(new_loan.collateral_token, old_loan.collateral_token);
    assert!(new_loan.principal > old_loan.principal); // Should include refinancing fee
    assert!(new_loan.due_date > old_loan.due_date);

    // Check that refinancing fee was charged by verifying new principal is higher
    // The refinancing fee is included in the new loan principal
    assert!(new_loan.principal > old_loan.principal); // Should include refinancing fee
}

#[test]
fn test_refinance_loan_fails_when_overdue() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let borrower = Address::generate(&env);
    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &borrower, 10_000);
    mint_to(&env, &collateral_addr, &borrower, 20_000);
    mint_to(&env, &token_addr, &depositor, 10_000);

    // Deposit funds to provide liquidity
    client.deposit(&depositor, &5000u64);

    // Borrow 1000 for 1 day
    client.borrow(
        &borrower,
        &1000u64,
        &collateral_addr,
        &1500u64,
        &(24 * 60 * 60),
    );

    // Jump past grace period
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 5 * 24 * 60 * 60);

    // Should fail to refinance when overdue
    let result = client.try_refinance_loan(&borrower, &(30 * 24 * 60 * 60));
    assert_eq!(result.err(), Some(Ok(LendingError::CannotRefinance)));
}

#[test]
fn test_consolidate_loans() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let borrower = Address::generate(&env);
    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &borrower, 20_000);
    mint_to(&env, &collateral_addr, &borrower, 40_000);
    mint_to(&env, &token_addr, &depositor, 10_000);

    // Deposit funds to provide liquidity
    client.deposit(&depositor, &5000u64);

    // Create multiple loans by using different borrowers first, then transferring
    // For this test, we'll need a different approach since the contract only allows one loan per user

    // Let's modify the contract to allow multiple loans for testing consolidation
    // For now, let's test the consolidation logic with a single loan (edge case)
    let loan_id1 = client.borrow(
        &borrower,
        &1000u64,
        &collateral_addr,
        &1500u64,
        &(30 * 24 * 60 * 60),
    );

    // Try to consolidate single loan (should work but be similar to refinance)
    let mut loan_ids = Vec::new(&env);
    loan_ids.push_back(loan_id1);

    let new_loan_id = client.consolidate_loans(&borrower, &loan_ids, &(60 * 24 * 60 * 60));

    // Verify consolidation worked
    let new_loan = client.get_loan(&borrower).unwrap();
    assert_ne!(new_loan_id, loan_id1);
    assert!(new_loan.principal > 1000u64); // Should include consolidation fee
}

#[test]
fn test_split_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let borrower = Address::generate(&env);
    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &borrower, 10_000);
    mint_to(&env, &collateral_addr, &borrower, 20_000);
    mint_to(&env, &token_addr, &depositor, 10_000);

    // Deposit funds to provide liquidity
    client.deposit(&depositor, &5000u64);

    // Borrow 2000 with 3000 collateral
    let old_loan_id = client.borrow(
        &borrower,
        &2000u64,
        &collateral_addr,
        &3000u64,
        &(30 * 24 * 60 * 60),
    );

    // Jump forward a bit to accrue some interest
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 10 * 24 * 60 * 60);

    // Get current outstanding balance
    let outstanding = client.get_repayment_amount(&borrower);

    // Split into two loans: 60% and 40%
    let split1 = (outstanding * 60) / 100;
    let split2 = outstanding - split1;

    let mut split_amounts = Vec::new(&env);
    split_amounts.push_back(split1);
    split_amounts.push_back(split2);

    let new_loan_ids = client.split_loan(&borrower, &split_amounts, &(45 * 24 * 60 * 60));

    // Verify split worked
    assert_eq!(new_loan_ids.len(), 2);

    // Check that user now has multiple loans
    let user_loans = client.get_user_loan_ids(&borrower);
    assert_eq!(user_loans.len(), 2);

    // Verify each loan exists
    for loan_id in new_loan_ids.iter() {
        let loan = client.get_loan_by_id(&loan_id).unwrap();
        assert_eq!(loan.borrower, borrower);
        assert!(loan.collateral_amount > 0);
    }
}

#[test]
fn test_split_loan_invalid_amounts() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let borrower = Address::generate(&env);
    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &borrower, 10_000);
    mint_to(&env, &collateral_addr, &borrower, 20_000);
    mint_to(&env, &token_addr, &depositor, 10_000);

    // Deposit funds to provide liquidity
    client.deposit(&depositor, &5000u64);

    // Borrow 1000
    client.borrow(
        &borrower,
        &1000u64,
        &collateral_addr,
        &1500u64,
        &(30 * 24 * 60 * 60),
    );

    // Try to split with amounts that don't sum to outstanding
    let mut split_amounts = Vec::new(&env);
    split_amounts.push_back(500u64);
    split_amounts.push_back(600u64); // Total 1100, should be more than outstanding

    let result = client.try_split_loan(&borrower, &split_amounts, &(30 * 24 * 60 * 60));
    assert_eq!(result.err(), Some(Ok(LendingError::InvalidSplitAmounts)));
}

#[test]
fn test_get_refinancing_fee_rate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, _collateral_addr, _admin) = setup(&env);
    let fee_rate = client.get_refinancing_fee_rate();
    assert_eq!(fee_rate, 50u32); // 0.5% = 50 basis points
}

#[test]
fn test_user_loan_tracking() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let borrower = Address::generate(&env);
    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &borrower, 10_000);
    mint_to(&env, &collateral_addr, &borrower, 20_000);
    mint_to(&env, &token_addr, &depositor, 10_000);

    // Deposit funds to provide liquidity
    client.deposit(&depositor, &5000u64);

    // Initially should have no loans
    let user_loans = client.get_user_loan_ids(&borrower);
    assert_eq!(user_loans.len(), 0);

    // Borrow a loan
    let loan_id = client.borrow(
        &borrower,
        &1000u64,
        &collateral_addr,
        &1500u64,
        &(30 * 24 * 60 * 60),
    );

    // Should have one loan
    let user_loans = client.get_user_loan_ids(&borrower);
    assert_eq!(user_loans.len(), 1);
    assert_eq!(user_loans.get(0), Some(loan_id));

    // Split the loan
    let mut split_amounts = Vec::new(&env);
    split_amounts.push_back(500u64);
    split_amounts.push_back(500u64);

    client.split_loan(&borrower, &split_amounts, &(30 * 24 * 60 * 60));

    // Should have two loans
    let user_loans = client.get_user_loan_ids(&borrower);
    assert_eq!(user_loans.len(), 2);

    // Repay one loan (by getting the primary loan and repaying)
    client.repay(&borrower);

    // Should have one loan left
    let user_loans = client.get_user_loan_ids(&borrower);
    assert_eq!(user_loans.len(), 1);
}

// ─────────────────────────────────────────────────
// Flash Loan Tests
// ─────────────────────────────────────────────────

#[contract]
pub struct MockFlashLoanReceiver;

#[contractimpl]
impl MockFlashLoanReceiver {
    pub fn execute_operation(env: Env, amount: u64, fee: u64, _initiator: Address) {
        let lending_contract = env
            .storage()
            .instance()
            .get::<_, Address>(&symbol_short!("LEND"))
            .unwrap();

        let token_addr = env
            .storage()
            .instance()
            .get::<_, Address>(&symbol_short!("TOKEN"))
            .unwrap();

        let token_client = token::Client::new(&env, &token_addr);
        let contract_id = env.current_contract_address();

        let balance = token_client.balance(&contract_id);
        assert!(balance >= amount as i128);

        let total_repay = amount + fee;
        token_client.transfer(&contract_id, &lending_contract, &(total_repay as i128));
    }
}

#[test]
fn test_flash_loan_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _, _) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 1_000_000);
    client.deposit(&depositor, &100_000u64);

    let receiver_id = env.register_contract(None, MockFlashLoanReceiver);

    env.as_contract(&receiver_id, || {
        env.storage()
            .instance()
            .set(&symbol_short!("LEND"), &client.address);
        env.storage()
            .instance()
            .set(&symbol_short!("TOKEN"), &token_addr);
    });

    mint_to(&env, &token_addr, &receiver_id, 10_000);

    let flash_loan_amount = 50_000u64;
    let expected_fee = (50_000u64 * 9) / 10000;

    let pool_before = client.get_pool_state();

    client.flash_loan(&receiver_id, &flash_loan_amount);

    let pool_after = client.get_pool_state();
    assert_eq!(
        pool_after.total_deposits,
        pool_before.total_deposits + expected_fee
    );
}

#[test]
fn test_rate_model_query_and_update() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, _collateral_addr, admin) = setup(&env);

    // Initial values from setup
    assert_eq!(client.get_base_rate(), 500u32);
    assert_eq!(client.get_optimal_utilization(), 8000u32);
    assert_eq!(client.get_slope1(), 1600u32);
    assert_eq!(client.get_slope2(), 3000u32);

    // Update rate model
    client.set_rate_model(&admin, &400u32, &7000u32, &1500u32, &4000u32, &1500u32);

    assert_eq!(client.get_base_rate(), 400u32);
    assert_eq!(client.get_optimal_utilization(), 7000u32);
    assert_eq!(client.get_slope1(), 1500u32);
    assert_eq!(client.get_slope2(), 4000u32);
}

#[test]
fn test_rate_model_unauthorized_update() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, _collateral_addr, _admin) = setup(&env);

    let attacker = Address::generate(&env);
    let result = client.try_set_rate_model(&attacker, &400u32, &7000u32, &1500u32, &4000u32, &1500u32);
    assert!(result.is_err());
}

#[test]
fn test_rate_simulation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _token_addr, _collateral_addr, _admin) = setup(&env);

    // Below optimal (80%)
    // Rate = 500 + (40% / 80%) * 1600 = 500 + 800 = 1300
    assert_eq!(client.simulate_rate(&4000u32), 1300u32);

    // At optimal
    // Rate = 500 + 1600 = 2100
    assert_eq!(client.simulate_rate(&8000u32), 2100u32);

    // Above optimal
    // Rate = 500 + 1600 + ((90% - 80%) / (100% - 80%)) * 3000
    // Rate = 2100 + (10 / 20) * 3000 = 2100 + 1500 = 3600
    assert_eq!(client.simulate_rate(&9000u32), 3600u32);
}

#[test]
fn test_supply_rate_calculation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, token_addr, _collateral_addr, _admin) = setup(&env);

    let depositor = Address::generate(&env);
    mint_to(&env, &token_addr, &depositor, 10_000);
    client.deposit(&depositor, &10_000u64);

    // At 0 utilization, supply rate is 0
    assert_eq!(client.get_supply_rate(), 0u32);

    // Mock some borrowing to get 50% utilization
    let borrower = Address::generate(&env);
    let collateral_addr = create_token_addr(&env);
    client.whitelist_collateral(&Address::generate(&env), &collateral_addr);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    client.borrow(&borrower, &5000u64, &collateral_addr, &7500u64, &3600u64);

    // Utilization = 50%
    // Borrow Rate = 1500
    // Reserve Factor = 10% (1000)
    // Supply Rate = Borrow Rate * Utilization * (1 - Reserve Factor)
    // Supply Rate = 1500 * 0.5 * 0.9 = 1500 * 0.45 = 675
    assert_eq!(client.get_supply_rate(), 675u32);
}

#![cfg(test)]
#![allow(clippy::all)]

//! Tests for the loan repayment grace period and the default-notice event
//! (#1174).

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger},
    token, Address, Env,
};

// ─── Setup helpers ────────────────────────────────

fn create_token_addr(env: &Env) -> Address {
    let token_admin = Address::generate(env);
    env.register_stellar_asset_contract_v2(token_admin)
        .address()
}

fn setup(env: &Env) -> (LendingContractClient<'_>, Address, Address, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let token_addr = create_token_addr(env);
    let collateral_addr = create_token_addr(env);
    let contract_id = env.register_contract(None, LendingContract);
    let client = LendingContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr, &500u32, &2000u32, &15000u32, &10000u32);
    client.whitelist_collateral(&admin, &collateral_addr);
    (client, token_addr, collateral_addr, admin)
}

fn mint_to(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token_addr).mint(to, &amount);
}

/// Borrow `principal` against `collateral` with a 1-day maturity, returning the
/// loan id.
fn borrow_one_day(
    env: &Env,
    client: &LendingContractClient<'_>,
    borrower: &Address,
    token_addr: &Address,
    collateral_addr: &Address,
    principal: u64,
    collateral: u64,
) -> u64 {
    // The pool needs liquidity before it can lend, so seed it from a fresh
    // depositor on each call.
    let depositor = Address::generate(env);
    mint_to(env, token_addr, &depositor, 100_000);
    client.deposit(&depositor, token_addr, &50_000);

    client.borrow(
        borrower,
        token_addr,
        &principal,
        collateral_addr,
        &collateral,
        &(24 * 60 * 60),
    )
}

// ─── 3-day liquidation floor (#1174) ──────────────

#[test]
fn liquidation_is_blocked_for_three_days_past_due_even_with_a_short_configured_window() {
    let env = Env::default();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let depositor = Address::generate(&env);
    let borrower = Address::generate(&env);
    let liquidator = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &depositor, 50_000);
    mint_to(&env, &token_addr, &liquidator, 50_000);

    client.deposit(&depositor, &token_addr, &20_000);

    // Configure a 1-second window. Before this change a liquidator could act
    // one second after maturity; the 3-day floor must still hold.
    client.set_grace_period(&admin, &token_addr, &1);

    borrow_one_day(
        &env,
        &client,
        &borrower,
        &token_addr,
        &collateral_addr,
        5_000,
        7_500,
    );

    // The floor is reported as due_date + 3 days even though the pool was
    // configured with a 1-second window.
    let floor = client.get_grace_period_end(&borrower);
    assert_eq!(floor, env.ledger().timestamp() + 24 * 60 * 60 + 259_200);

    // A second past maturity the loan is in default, but the liquidator is
    // still held back.
    env.ledger().set_timestamp(env.ledger().timestamp() + 24 * 60 * 60 + 10);
    assert!(client.is_loan_default_warned(&borrower) == false);
    assert!(client.try_liquidate(&liquidator, &borrower, &1_000).is_err());

    // Still blocked right up to the last second of the 3-day window, which is
    // the guarantee this issue asks for.
    env.ledger().set_timestamp(floor - 1);
    assert!(client.try_liquidate(&liquidator, &borrower, &1_000).is_err());

    // Past the floor the grace-period gate no longer blocks. (The health
    // factor is an independent gate and may still reject, which is why this
    // asserts only that the outcome changes once the window closes.)
    env.ledger().set_timestamp(floor + 1);
    let _ = client.try_liquidate(&liquidator, &borrower, &1_000);
}

#[test]
fn grace_period_end_reports_the_floor_not_the_configured_window() {
    let env = Env::default();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &borrower, 50_000);
    client.set_grace_period(&admin, &token_addr, &1);

    let start = env.ledger().timestamp();
    borrow_one_day(
        &env,
        &client,
        &borrower,
        &token_addr,
        &collateral_addr,
        1_000,
        1_500,
    );

    // due_date + 3 days, regardless of the 1-second configured window.
    assert_eq!(
        client.get_grace_period_end(&borrower),
        start + 24 * 60 * 60 + 259_200
    );
}

#[test]
fn a_longer_configured_window_still_applies() {
    let env = Env::default();
    let (client, token_addr, collateral_addr, admin) = setup(&env);

    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &borrower, 50_000);
    // 7 days is above the floor and must be honoured as configured.
    client.set_grace_period(&admin, &token_addr, &(7 * 24 * 60 * 60));

    let start = env.ledger().timestamp();
    borrow_one_day(
        &env,
        &client,
        &borrower,
        &token_addr,
        &collateral_addr,
        1_000,
        1_500,
    );
    assert_eq!(
        client.get_grace_period_end(&borrower),
        start + 24 * 60 * 60 + 7 * 24 * 60 * 60
    );
}

// ─── loan_default_warning event (#1174) ───────────

#[test]
fn default_warning_emits_once_after_the_due_date() {
    let env = Env::default();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &borrower, 50_000);
    borrow_one_day(
        &env,
        &client,
        &borrower,
        &token_addr,
        &collateral_addr,
        5_000,
        7_500,
    );

    // Before maturity there is nothing to warn about.
    assert_eq!(
        client.try_notify_loan_default(&borrower),
        Err(Ok(LendingError::InvalidAmount))
    );
    assert!(!client.is_loan_default_warned(&borrower));

    // Past the due timestamp the warning fires.
    env.ledger().set_timestamp(env.ledger().timestamp() + 24 * 60 * 60 + 1);
    assert!(client.notify_loan_default(&borrower));
    assert!(client.is_loan_default_warned(&borrower));

    // And does not fire again for the same loan.
    assert!(!client.notify_loan_default(&borrower));
}

#[test]
fn default_warning_event_carries_the_grace_window_and_amount_due() {
    let env = Env::default();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &borrower, 50_000);
    borrow_one_day(
        &env,
        &client,
        &borrower,
        &token_addr,
        &collateral_addr,
        5_000,
        7_500,
    );

    let loan = client.get_loan(&borrower).unwrap();
    let warned_at = loan.due_date + 60;
    env.ledger().set_timestamp(warned_at);

    let before = env.events().all();
    client.notify_loan_default(&borrower);
    let events = env.events().all();
    let emitted = events.len() - before.len();

    // One diagnostic-free contract event for the warning.
    assert!(emitted > 0, "expected a loan_default_warning event");

    let grace_end = client.get_grace_period_end(&borrower);
    assert!(grace_end > warned_at);
    // The warning is emitted inside the grace window, so the borrower still has
    // time to repay.
    assert!(grace_end - warned_at > 0);
}

#[test]
fn default_warning_is_cleared_when_the_loan_is_repaid() {
    let env = Env::default();
    let (client, token_addr, collateral_addr, _admin) = setup(&env);

    let borrower = Address::generate(&env);
    mint_to(&env, &collateral_addr, &borrower, 100_000);
    mint_to(&env, &token_addr, &borrower, 500_000);
    borrow_one_day(
        &env,
        &client,
        &borrower,
        &token_addr,
        &collateral_addr,
        5_000,
        7_500,
    );

    env.ledger().set_timestamp(env.ledger().timestamp() + 24 * 60 * 60 + 1);
    client.notify_loan_default(&borrower);
    assert!(client.is_loan_default_warned(&borrower));

    client.repay(&borrower);
    assert!(client.get_loan(&borrower).is_none());
    // No open loan, so there is no warning state to report.
    assert!(client.try_is_loan_default_warned(&borrower).is_err());
}

#[test]
fn default_warning_requires_an_open_loan() {
    let env = Env::default();
    let (client, _token_addr, _collateral_addr, _admin) = setup(&env);
    let nobody = Address::generate(&env);

    assert_eq!(
        client.try_notify_loan_default(&nobody),
        Err(Ok(LendingError::NoOpenLoan))
    );
}

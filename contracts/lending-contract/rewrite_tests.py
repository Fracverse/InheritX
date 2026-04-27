import re

with open('src/test.rs', 'r') as f:
    content = f.read()

# test_deposit_mints_shares
content = content.replace('''    let shares = client.deposit(&depositor, &1000u64);
    // First deposit: 1:1 ratio
    assert_eq!(shares, 1000u64);
    assert_eq!(client.get_shares_of(&depositor), 1000u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_deposits, 1000);
    assert_eq!(pool.total_shares, 1000);''', '''    let shares = client.deposit(&depositor, &2000u64);
    // First deposit: 1:1 ratio minus lock
    assert_eq!(shares, 1000u64);
    assert_eq!(client.get_shares_of(&depositor), 1000u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_deposits, 2000);
    assert_eq!(pool.total_shares, 2000);''')

# test_second_deposit_proportional_shares
content = content.replace('''    // First deposit: 1000 tokens → 1000 shares
    client.deposit(&depositor1, &1000u64);

    // Second deposit: same ratio → 500 tokens → 500 shares
    let shares2 = client.deposit(&depositor2, &500u64);
    assert_eq!(shares2, 500u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_deposits, 1500);
    assert_eq!(pool.total_shares, 1500);''', '''    // First deposit: 2000 tokens → 1000 shares
    client.deposit(&depositor1, &2000u64);

    // Second deposit: pool has 2000 shares, 2000 deposits. ratio 1:1
    // 500 tokens -> 500 shares
    let shares2 = client.deposit(&depositor2, &500u64);
    assert_eq!(shares2, 500u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_deposits, 2500);
    assert_eq!(pool.total_shares, 2500);''')

# test_withdraw_burns_shares_and_returns_tokens
content = content.replace('''    client.deposit(&depositor, &1000u64);
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
    assert_eq!(pool.total_shares, 500);''', '''    client.deposit(&depositor, &2000u64);
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
    assert_eq!(pool.total_shares, 1500);''')

# test_withdraw_fails_not_enough_shares
content = content.replace('''    client.deposit(&depositor, &1000u64);

    // Try to withdraw more shares than owned
    let result = client.try_withdraw(&depositor, &2000u64);''', '''    client.deposit(&depositor, &2000u64);

    // Try to withdraw more shares than owned
    let result = client.try_withdraw(&depositor, &2000u64);''')

# test_borrow_reduces_available_liquidity
content = content.replace('''    client.deposit(&depositor, &1000u64);

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

    assert_eq!(client.available_liquidity(), 600u64);''', '''    client.deposit(&depositor, &2000u64);

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

    assert_eq!(client.available_liquidity(), 1600u64);''')

# test_borrow_fails_if_insufficient_liquidity
content = content.replace('''    client.deposit(&depositor, &1000u64);

    let result = client.try_borrow(&depositor, &1001u64);''', '''    client.deposit(&depositor, &2000u64);

    let result = client.try_borrow(&depositor, &2001u64);''')

# test_borrow_fails_with_existing_loan
content = content.replace('''    client.deposit(&depositor, &1000u64);
    client.borrow(&borrower, &200u64);''', '''    client.deposit(&depositor, &2000u64);
    client.borrow(&borrower, &200u64);''')

# test_repay_restores_liquidity
content = content.replace('''    client.deposit(&depositor, &1000u64);
    client.borrow(&borrower, &400u64);

    assert_eq!(client.available_liquidity(), 600u64);

    let repaid = client.repay(&borrower);
    assert_eq!(repaid, 400u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_borrowed, 0);
    assert_eq!(pool.total_deposits, 1000);
    assert_eq!(client.available_liquidity(), 1000u64);''', '''    client.deposit(&depositor, &2000u64);
    client.borrow(&borrower, &400u64);

    assert_eq!(client.available_liquidity(), 1600u64);

    let repaid = client.repay(&borrower);
    assert_eq!(repaid, 400u64);

    let pool = client.get_pool_state();
    assert_eq!(pool.total_borrowed, 0);
    assert_eq!(pool.total_deposits, 2000);
    assert_eq!(client.available_liquidity(), 2000u64);''')

# test_withdraw_fails_if_funds_are_borrowed
content = content.replace('''    client.deposit(&depositor, &1000u64);
    client.borrow(&borrower, &900u64); // only 100 tokens left un-borrowed''', '''    client.deposit(&depositor, &2000u64);
    client.borrow(&borrower, &1900u64); // only 100 tokens left un-borrowed''')

# test_get_loan_returns_record_when_active
content = content.replace('''    client.deposit(&depositor, &1000u64);
    client.borrow(&borrower, &300u64);''', '''    client.deposit(&depositor, &2000u64);
    client.borrow(&borrower, &300u64);''')

# add test_rounding_loss_exploit_prevented at the end
new_test = """
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
"""

content += new_test

with open('src/test.rs', 'w') as f:
    f.write(content)

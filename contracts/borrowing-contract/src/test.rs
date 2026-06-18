#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, Address, Env,
};

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

    // Liquidate 500 out of 1000 debt
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

    // Create an initial loan before pause to test repayment
    sac_client(&env, &collateral_addr).mint(&borrower, &3000);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

    // Admin pauses globally
    client.set_global_pause(&admin, &true);
    assert!(client.is_global_paused());

    // New borrowing should fail
    let result = client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(result, Err(Ok(BorrowingError::Paused)));

    // Repayment should still work
    client.repay_loan(&loan_id, &500);
    let loan = client.get_loan(&loan_id);
    assert_eq!(loan.amount_repaid, 500);

    // Unpause
    client.set_global_pause(&admin, &false);
    assert!(!client.is_global_paused());

    // Borrowing works again
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

    // Admin pauses specific vault (collateral token)
    client.set_vault_pause(&admin, &collateral_addr, &true);
    assert!(client.is_vault_paused(&collateral_addr));

    // New borrowing should fail for this vault
    let result = client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(result, Err(Ok(BorrowingError::Paused)));

    // Unpause vault
    client.set_vault_pause(&admin, &collateral_addr, &false);
    assert!(!client.is_vault_paused(&collateral_addr));

    // Borrowing works again
    let new_loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(new_loan_id, 1);
}

#[test]
fn test_extend_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    // Mint enough for collateral + extension fee (1% of 1000 = 10)
    sac_client(&env, &collateral_addr).mint(&borrower, &1510);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

    let original_due = client.get_loan(&loan_id).due_date;
    client.extend_loan(&loan_id, &86400); // extend by 1 day in seconds

    let loan = client.get_loan(&loan_id);
    assert_eq!(loan.due_date, original_due + 86400);
    assert_eq!(loan.extension_count, 1);
}

#[test]
fn test_extend_loan_fee_calculation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1510);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

    // Fee = 1% of remaining principal (1000) = 10
    let fee = client.get_extension_fee(&loan_id);
    assert_eq!(fee, 10);
}

#[test]
fn test_extend_loan_limit_reached() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    // Mint enough for collateral + 3 extension fees (10 each)
    sac_client(&env, &collateral_addr).mint(&borrower, &1530);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

    // First extension
    client.extend_loan(&loan_id, &86400);
    assert_eq!(client.get_loan(&loan_id).extension_count, 1);

    // Second extension
    client.extend_loan(&loan_id, &86400);
    assert_eq!(client.get_loan(&loan_id).extension_count, 2);

    // Third extension should fail (max 2)
    let result = client.try_extend_loan(&loan_id, &86400);
    assert_eq!(result, Err(Ok(BorrowingError::ExtensionLimitReached)));
}

#[test]
fn test_extend_inactive_loan_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    client.repay_loan(&loan_id, &1000);

    let result = client.try_extend_loan(&loan_id, &86400);
    assert_eq!(result, Err(Ok(BorrowingError::LoanNotActive)));
}

#[test]
fn test_increase_loan_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    // collateral_ratio = 15000 (150%), so 1500 collateral supports up to 1000 principal
    // max_borrow = 1500 * 10000 / 15000 = 1000; current debt = 500; max_additional = 500
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &500, &5, &1000000, &collateral_addr, &1500);

    let max_add = client.get_max_additional_borrow(&loan_id);
    assert_eq!(max_add, 500);

    client.increase_loan_amount(&loan_id, &300);
    let loan = client.get_loan(&loan_id);
    assert_eq!(loan.principal, 800);
}

#[test]
fn test_increase_loan_exceeds_collateral_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

    // max_additional = 0 since collateral exactly covers current principal
    let result = client.try_increase_loan_amount(&loan_id, &1);
    assert_eq!(result, Err(Ok(BorrowingError::InsufficientCollateral)));
}

#[test]
fn test_increase_inactive_loan_fails() {
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
fn test_increase_loan_invalid_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &500, &5, &1000000, &collateral_addr, &1500);

    let result = client.try_increase_loan_amount(&loan_id, &0);
    assert_eq!(result, Err(Ok(BorrowingError::InvalidAmount)));
}

#[test]
fn test_get_max_additional_borrow_inactive_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _) = setup(&env);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    client.repay_loan(&loan_id, &1000);

    let result = client.try_get_max_additional_borrow(&loan_id);
    assert_eq!(result, Err(Ok(BorrowingError::LoanNotActive)));
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

#[test]
fn test_liquidation_auction_zero_duration_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let collateral_addr = create_token_addr(&env);
    let contract_id = env.register_contract(None, BorrowingContract);
    let client = BorrowingContractClient::new(&env, &contract_id);
    client.initialize(&admin, &12000, &13000, &500);
    client.whitelist_collateral(&admin, &collateral_addr);

    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1200);

    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1200);

    // Duration is 0, should fail with InvalidAmount
    let result = client.try_start_liquidation_auction(&loan_id, &0, &100, &2000);
    assert_eq!(result, Err(Ok(BorrowingError::InvalidAmount)));
}

// ─────────────────────────────────────────────────
// Access Control (RBAC) Tests
// ─────────────────────────────────────────────────

#[test]
fn test_admin_role_assigned_on_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, BorrowingContract);
    let client = BorrowingContractClient::new(&env, &contract_id);
    client.initialize(&admin, &15000, &12000, &500);

    assert!(client.has_role(&admin, &access_control::Role::Admin));
    assert!(!client.has_role(&admin, &access_control::Role::Owner));
}

#[test]
fn test_admin_can_assign_and_revoke_roles() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _collateral_addr, admin) = setup(&env);
    let user = Address::generate(&env);

    assert!(!client.has_role(&user, &access_control::Role::Owner));

    client.assign_role(&admin, &user, &access_control::Role::Owner);
    assert!(client.has_role(&user, &access_control::Role::Owner));

    client.revoke_role(&admin, &user, &access_control::Role::Owner);
    assert!(!client.has_role(&user, &access_control::Role::Owner));
}

#[test]
fn test_non_admin_cannot_assign_roles() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _collateral_addr, _admin) = setup(&env);
    let non_admin = Address::generate(&env);
    let target = Address::generate(&env);

    let result = client.try_assign_role(&non_admin, &target, &access_control::Role::Admin);
    assert!(result.is_err());
}

#[test]
fn test_non_admin_cannot_whitelist_collateral() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _collateral_addr, _admin) = setup(&env);
    let non_admin = Address::generate(&env);
    let token = Address::generate(&env);

    let result = client.try_whitelist_collateral(&non_admin, &token);
    assert!(result.is_err());
}

#[test]
fn test_get_roles_returns_assigned_roles() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _collateral_addr, admin) = setup(&env);
    let user = Address::generate(&env);

    client.assign_role(&admin, &user, &access_control::Role::Owner);
    client.assign_role(&admin, &user, &access_control::Role::Beneficiary);

    let roles = client.get_roles(&user);
    assert_eq!(roles.len(), 2);
}

#[test]
fn test_pause_blocks_create_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);
    client.pause(&admin);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let result = client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert!(result.is_err());
}

#[test]
fn test_unpause_restores_create_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);
    client.pause(&admin);
    client.unpause(&admin);
    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(loan_id, 1);
}

#[test]
fn test_non_admin_cannot_pause_borrowing() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _collateral_addr, _admin) = setup(&env);
    let non_admin = Address::generate(&env);
    let result = client.try_pause(&non_admin);
    assert!(result.is_err());
}

#[test]
fn test_is_paused_reflects_state_borrowing() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _collateral_addr, admin) = setup(&env);
    assert!(!client.is_paused());
    client.pause(&admin);
    assert!(client.is_paused());
    client.unpause(&admin);
    assert!(!client.is_paused());
}

// ─────────────────────────────────────────────────
// Oracle Staleness & Volatility Buffer Tests
// ─────────────────────────────────────────────────

#[test]
fn test_oracle_staleness_blocks_create_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);

    // Enable staleness enforcement: max 3600s
    client.set_max_oracle_age(&admin, &3600);

    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);

    // Oracle timestamp never set → stale
    let result = client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(result, Err(Ok(BorrowingError::OraclePriceStale)));
}

#[test]
fn test_oracle_fresh_allows_create_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);

    client.set_max_oracle_age(&admin, &3600);

    // Record a fresh oracle update at the current ledger time (t=0)
    client.update_oracle_timestamp(&admin, &collateral_addr);
    assert_eq!(client.get_oracle_timestamp(&collateral_addr), 0);

    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);

    // Loan creation should succeed: age = 0 ≤ 3600
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(loan_id, 1);
}

#[test]
fn test_oracle_expired_blocks_create_loan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);

    client.set_max_oracle_age(&admin, &3600);

    // Record oracle update at t=0
    client.update_oracle_timestamp(&admin, &collateral_addr);

    // Advance ledger time past the max age
    env.ledger().with_mut(|l| l.timestamp = 7201);

    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);

    let result = client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(result, Err(Ok(BorrowingError::OraclePriceStale)));
}

#[test]
fn test_oracle_disabled_by_default() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, _admin) = setup(&env);

    // Default max_oracle_age = 0 → staleness check skipped
    assert_eq!(client.get_max_oracle_age(), 0);

    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);

    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(loan_id, 1);
}

#[test]
fn test_volatility_buffer_increases_required_collateral() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);

    // Base ratio = 15000 (150%). Add 1000 bps (10%) volatility buffer → effective 160%
    client.set_volatility_buffer(&admin, &collateral_addr, &1000);
    assert_eq!(client.get_volatility_buffer(&collateral_addr), 1000);

    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);

    // Required = 1000 * 16000 / 10000 = 1600 > 1500 → insufficient
    let result = client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);
    assert_eq!(result, Err(Ok(BorrowingError::InsufficientCollateral)));
}

#[test]
fn test_volatility_buffer_loan_with_sufficient_collateral() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);

    // Base 150% + 10% buffer = 160%
    client.set_volatility_buffer(&admin, &collateral_addr, &1000);

    let borrower = Address::generate(&env);
    // 1600 meets the effective 160% requirement for a 1000 principal
    sac_client(&env, &collateral_addr).mint(&borrower, &1600);

    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1600);
    assert_eq!(loan_id, 1);
}

#[test]
fn test_volatility_buffer_reduces_max_additional_borrow() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);

    // Base ratio 150%. 1500 collateral → max borrow 1000 normally.
    // With 1000 bps buffer (160%), max borrow = 1500 * 10000 / 16000 = 937.
    client.set_volatility_buffer(&admin, &collateral_addr, &1000);

    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1500);
    // At 160%, principal 937 requires 937 * 16000 / 10000 = 1499.2 → fits in 1500
    let loan_id = client.create_loan(&borrower, &937, &5, &1000000, &collateral_addr, &1500);

    let max_add = client.get_max_additional_borrow(&loan_id);
    // max_borrow = 1500 * 10000 / 16000 = 937; current_debt = 937; max_additional = 0
    assert_eq!(max_add, 0);
}

#[test]
fn test_oracle_and_volatility_combined() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);

    client.set_max_oracle_age(&admin, &3600);
    client.set_volatility_buffer(&admin, &collateral_addr, &2000); // +20% buffer → 170%

    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &1700);

    // Stale oracle blocks even with sufficient collateral
    let result = client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1700);
    assert_eq!(result, Err(Ok(BorrowingError::OraclePriceStale)));

    // Fresh oracle + enough collateral succeeds
    client.update_oracle_timestamp(&admin, &collateral_addr);
    let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1700);
    assert_eq!(loan_id, 1);
}

// ─────────────────────────────────────────────────
// Decimal-normalization helpers (#632)
// ─────────────────────────────────────────────────

#[test]
fn scale_amount_is_identity_when_decimals_match() {
    assert_eq!(scale_amount(1000, 7, 7), Some(1000));
    assert_eq!(scale_amount(0, 18, 18), Some(0));
}

#[test]
fn scale_amount_scales_up_for_higher_target_precision() {
    // 1000 units at 6 decimals expressed at 7 decimals -> x10
    assert_eq!(scale_amount(1000, 6, 7), Some(10_000));
    // 5 units at 0 decimals expressed at 3 decimals -> x1000
    assert_eq!(scale_amount(5, 0, 3), Some(5000));
}

#[test]
fn scale_amount_scales_down_for_lower_target_precision() {
    // 10_000 units at 8 decimals expressed at 6 decimals -> /100
    assert_eq!(scale_amount(10_000, 8, 6), Some(100));
}

#[test]
fn scale_amount_returns_none_on_overflow() {
    assert_eq!(scale_amount(i128::MAX, 0, 30), None);
}

#[test]
fn health_factor_bps_matches_naive_ratio_when_decimals_match() {
    // Mirrors existing on-chain expectations: 675 * 10000 / 500 = 13500.
    assert_eq!(health_factor_bps(675, 7, 500, 7), 13500);
    assert_eq!(health_factor_bps(1200, 7, 1000, 7), 12000);
}

#[test]
fn health_factor_bps_zero_debt_is_max_healthy() {
    assert_eq!(health_factor_bps(1500, 7, 0, 7), 10000);
}

#[test]
fn health_factor_bps_normalizes_when_collateral_has_more_decimals() {
    // collateral at 8 decimals, debt at 6 decimals, same real ratio 1.5x.
    // 15000 * 10000 / (100 * 10^2) = 15000 bps
    assert_eq!(health_factor_bps(15_000, 8, 100, 6), 15000);
}

#[test]
fn health_factor_bps_normalizes_when_debt_has_more_decimals() {
    // collateral at 6 decimals, debt at 8 decimals, same real ratio 1.5x.
    // 150 * 10000 * 10^2 / 10000 = 15000 bps
    assert_eq!(health_factor_bps(150, 6, 10_000, 8), 15000);
}

#[test]
fn health_factor_bps_saturates_to_zero_on_overflow() {
    // Enormous up-scaling overflows the intermediate product -> 0 (safe).
    assert_eq!(health_factor_bps(i128::MAX, 0, 1, 18), 0);
}

#[test]
fn test_health_factor_normalizes_principal_decimals() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, collateral_addr, admin) = setup(&env);
    // Collateral SAC has 7 decimals; configure the principal as a 6-decimal asset.
    client.set_principal_decimals(&admin, &6);

    let borrower = Address::generate(&env);
    sac_client(&env, &collateral_addr).mint(&borrower, &2_000_000);
    // principal=100 (6-dec), collateral=1_500_000 (7-dec)
    let loan_id = client.create_loan(
        &borrower,
        &100,
        &5,
        &1_000_000,
        &collateral_addr,
        &1_500_000,
    );

    let hf = client.get_health_factor(&loan_id);
    // On-chain result must match the normalized helper (collateral 7dec, debt 6dec)...
    assert_eq!(hf, health_factor_bps(1_500_000, 7, 100, 6));
    // ...and must differ from the un-normalized (equal-decimals) interpretation.
    assert_ne!(hf, health_factor_bps(1_500_000, 7, 100, 7));
}

#[test]
fn test_set_principal_decimals_requires_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _collateral_addr, _admin) = setup(&env);
    // Rejects an out-of-range precision.
    let result = client.try_set_principal_decimals(&Address::generate(&env), &40);
    assert!(result.is_err());
}

#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

const MAX_SUPPLY: i128 = 1_000_000_000_000_000_000;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    NegativeAmount = 1,
    InsufficientBalance = 2,
    Overflow = 3,
    ExceedsMaxSupply = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MockTokenDataKey {
    Balance(Address),
    TotalSupply,
}

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn balance(env: Env, id: Address) -> i128 {
        let key = MockTokenDataKey::Balance(id);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    pub fn total_supply(env: Env) -> i128 {
        let key = MockTokenDataKey::TotalSupply;
        env.storage().instance().get(&key).unwrap_or(0)
    }

    pub fn max_supply() -> i128 {
        MAX_SUPPLY
    }

    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        from.require_auth();

        if amount < 0 {
            return Err(ContractError::NegativeAmount);
        }

        let key_from = MockTokenDataKey::Balance(from.clone());
        let key_to = MockTokenDataKey::Balance(to.clone());

        let balance_from: i128 = env.storage().instance().get(&key_from).unwrap_or(0);
        let balance_to: i128 = env.storage().instance().get(&key_to).unwrap_or(0);

        if balance_from < amount {
            return Err(ContractError::InsufficientBalance);
        }

        if balance_to.checked_add(amount).is_none() {
            return Err(ContractError::Overflow);
        }

        env.storage()
            .instance()
            .set(&key_from, &(balance_from - amount));
        env.storage()
            .instance()
            .set(&key_to, &(balance_to + amount));

        Ok(())
    }

    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), ContractError> {
        if amount < 0 {
            return Err(ContractError::NegativeAmount);
        }

        let key = MockTokenDataKey::Balance(to.clone());
        let supply_key = MockTokenDataKey::TotalSupply;

        let balance: i128 = env.storage().instance().get(&key).unwrap_or(0);
        let total_supply: i128 = env.storage().instance().get(&supply_key).unwrap_or(0);

        if total_supply.checked_add(amount).is_none() {
            return Err(ContractError::Overflow);
        }

        let new_total = total_supply + amount;

        if new_total > MAX_SUPPLY {
            return Err(ContractError::ExceedsMaxSupply);
        }

        if balance.checked_add(amount).is_none() {
            return Err(ContractError::Overflow);
        }

        env.storage().instance().set(&key, &(balance + amount));
        env.storage().instance().set(&supply_key, &new_total);

        Ok(())
    }

    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), ContractError> {
        from.require_auth();

        if amount < 0 {
            return Err(ContractError::NegativeAmount);
        }

        let key = MockTokenDataKey::Balance(from.clone());
        let supply_key = MockTokenDataKey::TotalSupply;

        let balance: i128 = env.storage().instance().get(&key).unwrap_or(0);
        let total_supply: i128 = env.storage().instance().get(&supply_key).unwrap_or(0);

        if balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        if total_supply < amount {
            return Err(ContractError::InsufficientBalance);
        }

        env.storage().instance().set(&key, &(balance - amount));
        env.storage()
            .instance()
            .set(&supply_key, &(total_supply - amount));

        Ok(())
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    #[allow(dead_code)]
    fn valid_amount_strategy() -> impl Strategy<Value = i128> {
        0i128..(MAX_SUPPLY / 10)
    }

    // Helper macro to correctly register the contract and return the client frame
    macro_rules! setup {
        () => {{
            let env = Env::default();
            env.mock_all_auths(); // Globally disable auth checks for tests
            let contract_id = env.register_contract(None, MockToken);
            let client = MockTokenClient::new(&env, &contract_id);
            (env, client)
        }};
    }

    fn addr_a(env: &Env) -> Address {
        // Automatically generates a valid random test address
        Address::generate(env)
    }

    fn addr_b(env: &Env) -> Address {
        // Automatically generates a valid random test address
        Address::generate(env)
    }

    proptest! {
        #[test]
        fn prop_zero_operations_are_no_ops(_ in proptest::bool::ANY) {
            let (env, client) = setup!();
            let addr = addr_a(&env);

            let supply_before = client.total_supply();

            // Using `try_mint` catches the Result from your contract
            let _ = client.try_mint(&addr, &0);

            prop_assert_eq!(
                client.total_supply(),
                supply_before
            );
        }
    }

    proptest! {
        #[test]
        fn prop_max_supply_boundary(_ in proptest::bool::ANY) {
            let (env, client) = setup!();

            let addr1 = addr_a(&env);
            let addr2 = addr_b(&env);

            // Remove .unwrap() so we can gracefully inspect the Results
            let result1 = client.try_mint(&addr1, &MAX_SUPPLY);
            prop_assert!(result1.is_ok()); // This one should succeed

            let result2 = client.try_mint(&addr2, &1);
            prop_assert!(result2.is_err()); // This one SHOULD fail
        }
    }

    proptest! {
    #[test]
    fn prop_transfer_preserves_supply(
        mint_amount in 1i128..(MAX_SUPPLY / 10),
        transfer_amount in 0i128..(MAX_SUPPLY / 10)
    ) {
        let (env, client) = setup!();
        let from = addr_a(&env);
        let to = addr_b(&env);

        // Setup
        if client.try_mint(&from, &mint_amount).unwrap().is_ok() {
            let supply_before = client.total_supply();

            if transfer_amount <= mint_amount {
                // Auth is already mocked globally from setup!()
                let _ = client.try_transfer(
                    &from,
                    &to,
                    &transfer_amount
                );

                let supply_after = client.total_supply();

                // INVARIANT: transfer must not change total supply
                prop_assert_eq!(supply_before, supply_after);
            }
        }
    }
    }

    proptest! {
        #[test]
        fn prop_mint_increases_supply(
            amount in 0i128..(MAX_SUPPLY / 10)
        ) {
            let (env, client) = setup!();
            let addr = addr_a(&env);

            let before = client.total_supply();

            if client.try_mint(&addr, &amount).unwrap().is_ok() {
                let after = client.total_supply();

                // INVARIANT
                prop_assert_eq!(after, before + amount);
            }
        }
    }

    proptest! {
        #[test]
        fn prop_balance_never_negative(
            mint_amount in 0i128..(MAX_SUPPLY / 10),
            burn_amount in 0i128..(MAX_SUPPLY / 10)
        ) {
            let (env, client) = setup!();
            let addr = addr_a(&env);

            let _ = client.try_mint(&addr, &mint_amount);
            let _ = client.try_burn(&addr, &burn_amount);

            let balance = client.balance(&addr);

            // INVARIANT
            prop_assert!(balance >= 0);
        }
    }
}

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
    use soroban_sdk::{Env, String};

    fn valid_amount_strategy() -> impl Strategy<Value = i128> {
        0i128..(MAX_SUPPLY / 10)
    }

    fn setup_env() -> Env {
        Env::default()
    }

    fn addr_a(env: &Env) -> Address {
        Address::from_string(&String::from_str(
            env,
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
        ))
    }

    fn addr_b(env: &Env) -> Address {
        Address::from_string(&String::from_str(
            env,
            "GBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBWHF",
        ))
    }

    proptest! {
        #[test]
        fn prop_zero_operations_are_no_ops(_ in proptest::bool::ANY) {
            let env = setup_env();
            let addr = addr_a(&env);

            let supply_before = MockToken::total_supply(env.clone());

            let _ = MockToken::mint(env.clone(), addr.clone(), 0);

            prop_assert_eq!(
                MockToken::total_supply(env.clone()),
                supply_before
            );
        }
    }

    proptest! {
        #[test]
        fn prop_max_supply_boundary(_ in proptest::bool::ANY) {
            let env = setup_env();

            let addr1 = addr_a(&env);
            let addr2 = addr_b(&env);

            let result = MockToken::mint(env.clone(), addr1.clone(), MAX_SUPPLY);
            prop_assert!(result.is_ok());

            let result = MockToken::mint(env.clone(), addr2, 1);
            prop_assert!(result.is_err());
        }
    }

    proptest! {
    #[test]
    fn prop_transfer_preserves_supply(
        mint_amount in 1i128..(MAX_SUPPLY / 10),
        transfer_amount in 0i128..(MAX_SUPPLY / 10)
    ) {
        let env = setup_env();
        let from = addr_a(&env);
        let to = addr_b(&env);

        // Setup
        if MockToken::mint(env.clone(), from.clone(), mint_amount).is_ok() {
            let supply_before = MockToken::total_supply(env.clone());

            if transfer_amount <= mint_amount {
                env.mock_all_auths();
                let _ = MockToken::transfer(
                    env.clone(),
                    from.clone(),
                    to.clone(),
                    transfer_amount
                );

                let supply_after = MockToken::total_supply(env.clone());

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
            let env = setup_env();
            let addr = addr_a(&env);

            let before = MockToken::total_supply(env.clone());

            if MockToken::mint(env.clone(), addr.clone(), amount).is_ok() {
                let after = MockToken::total_supply(env.clone());

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
            let env = setup_env();
            let addr = addr_a(&env);

            let _ = MockToken::mint(env.clone(), addr.clone(), mint_amount);

            env.mock_all_auths();
            let _ = MockToken::burn(env.clone(), addr.clone(), burn_amount);

            let balance = MockToken::balance(env.clone(), addr);

            // INVARIANT
            prop_assert!(balance >= 0);
        }
    }
}

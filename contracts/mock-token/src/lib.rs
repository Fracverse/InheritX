#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MockTokenDataKey {
    Balance(Address),
}

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn balance(env: Env, id: Address) -> i128 {
        let key = MockTokenDataKey::Balance(id);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let key_from = MockTokenDataKey::Balance(from.clone());
        let key_to = MockTokenDataKey::Balance(to.clone());
        let balance_from = env.storage().instance().get(&key_from).unwrap_or(0);
        let balance_to = env.storage().instance().get(&key_to).unwrap_or(0);
        env.storage()
            .instance()
            .set(&key_from, &(balance_from - amount));
        env.storage()
            .instance()
            .set(&key_to, &(balance_to + amount));
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let key = MockTokenDataKey::Balance(to.clone());
        let balance = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(balance + amount));
    }
}

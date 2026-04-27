#![no_std]

use soroban_sdk::{contracttype, Address, Env, Vec};

/// The four roles recognised across all InheritX contracts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Guardian,
    Beneficiary,
    Owner,
}

/// Per-address storage key for role lists.
#[contracttype]
#[derive(Clone)]
pub enum AccessControlKey {
    Roles(Address),
}

/// Assign `role` to `address`.  Idempotent — does nothing if already assigned.
pub fn assign_role(env: &Env, address: &Address, role: Role) {
    let key = AccessControlKey::Roles(address.clone());
    let mut roles: Vec<Role> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env));
    for existing in roles.iter() {
        if existing == role {
            return;
        }
    }
    roles.push_back(role);
    env.storage().persistent().set(&key, &roles);
}

/// Revoke `role` from `address`.  Idempotent — does nothing if not assigned.
pub fn revoke_role(env: &Env, address: &Address, role: Role) {
    let key = AccessControlKey::Roles(address.clone());
    let roles: Vec<Role> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env));
    let mut updated = Vec::new(env);
    for existing in roles.iter() {
        if existing != role {
            updated.push_back(existing);
        }
    }
    env.storage().persistent().set(&key, &updated);
}

/// Return `true` if `address` currently holds `role`.
pub fn has_role(env: &Env, address: &Address, role: Role) -> bool {
    let key = AccessControlKey::Roles(address.clone());
    let roles: Vec<Role> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env));
    for existing in roles.iter() {
        if existing == role {
            return true;
        }
    }
    false
}

/// Require that `address` holds `role`; panics with `contract_error` otherwise.
///
/// Pattern: `require_role(env, &caller, Role::Admin, ContractError::AccessDenied)?;`
pub fn require_role<E: Into<soroban_sdk::Error> + Copy>(
    env: &Env,
    address: &Address,
    role: Role,
    contract_error: E,
) -> Result<(), E> {
    if has_role(env, address, role) {
        Ok(())
    } else {
        Err(contract_error)
    }
}

// ─── Reentrancy Guard ────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum SecurityKey {
    ReentrancyLock,
}

/// Enter the reentrancy guard. Returns `error` if a reentrant call is detected.
pub fn reentrancy_enter<E: Into<soroban_sdk::Error> + Copy>(env: &Env, error: E) -> Result<(), E> {
    if env.storage().instance().has(&SecurityKey::ReentrancyLock) {
        return Err(error);
    }
    env.storage()
        .instance()
        .set(&SecurityKey::ReentrancyLock, &true);
    Ok(())
}

/// Enter the reentrancy guard. Panics if a reentrant call is detected.
/// Use this for contracts whose error enum is full (e.g. InheritanceContract).
pub fn reentrancy_enter_or_panic(env: &Env) {
    if env.storage().instance().has(&SecurityKey::ReentrancyLock) {
        panic!("reentrant call");
    }
    env.storage()
        .instance()
        .set(&SecurityKey::ReentrancyLock, &true);
}

/// Release the reentrancy guard. Always call this before returning.
/// Safe to skip on panic — Soroban reverts all storage on trap.
pub fn reentrancy_exit(env: &Env) {
    env.storage()
        .instance()
        .remove(&SecurityKey::ReentrancyLock);
}

// ─── Pause / Circuit Breaker ─────────────────────

#[contracttype]
#[derive(Clone)]
pub enum PauseKey {
    Paused,
}

/// Mark the contract as paused.
pub fn pause_contract(env: &Env) {
    env.storage().instance().set(&PauseKey::Paused, &true);
}

/// Mark the contract as unpaused.
pub fn unpause_contract(env: &Env) {
    env.storage().instance().set(&PauseKey::Paused, &false);
}

/// Returns true if the contract is currently paused.
pub fn is_contract_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get::<PauseKey, bool>(&PauseKey::Paused)
        .unwrap_or(false)
}

/// Fail with `error` if the contract is paused.
pub fn require_not_paused<E: Into<soroban_sdk::Error> + Copy>(
    env: &Env,
    error: E,
) -> Result<(), E> {
    if is_contract_paused(env) {
        return Err(error);
    }
    Ok(())
}

/// Panic if the contract is paused.
/// Use this for contracts whose error enum is full.
pub fn require_not_paused_or_panic(env: &Env) {
    if is_contract_paused(env) {
        panic!("contract paused");
    }
}

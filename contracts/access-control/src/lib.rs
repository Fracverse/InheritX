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
    reentrancy_enter_or_panic(env);
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
    reentrancy_exit(env);
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
    /// Temporary lock set while a pause/unpause operation is in progress.
    PauseLock,
    /// Count of active operations that have entered; used to prevent pausing
    /// while operations are running.
    ActiveOps,
}

/// Mark the contract as paused.
pub fn pause_contract(env: &Env) {
    // Prevent new operations from starting while we attempt to pause.
    env.storage().instance().set(&PauseKey::PauseLock, &true);
    // If there are active operations, abort and release the lock.
    let active: i128 = env
        .storage()
        .instance()
        .get::<PauseKey, i128>(&PauseKey::ActiveOps)
        .unwrap_or(0);
    if active != 0 {
        env.storage().instance().remove(&PauseKey::PauseLock);
        panic!("cannot pause: active operations present");
    }
    env.storage().instance().set(&PauseKey::Paused, &true);
    env.storage().instance().remove(&PauseKey::PauseLock);
}

/// Mark the contract as unpaused.
pub fn unpause_contract(env: &Env) {
    // Prevent new operations from starting while we change pause state.
    env.storage().instance().set(&PauseKey::PauseLock, &true);
    env.storage().instance().set(&PauseKey::Paused, &false);
    env.storage().instance().remove(&PauseKey::PauseLock);
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
    // Treat an in-progress pause/unpause (PauseLock) as paused for operation
    // validation so operation start is atomic with pause state changes.
    let pause_lock: bool = env
        .storage()
        .instance()
        .get::<PauseKey, bool>(&PauseKey::PauseLock)
        .unwrap_or(false);
    if is_contract_paused(env) || pause_lock {
        return Err(error);
    }
    Ok(())
}

/// Panic if the contract is paused.
/// Use this for contracts whose error enum is full.
pub fn require_not_paused_or_panic(env: &Env) {
    let pause_lock: bool = env
        .storage()
        .instance()
        .get::<PauseKey, bool>(&PauseKey::PauseLock)
        .unwrap_or(false);
    if is_contract_paused(env) || pause_lock {
        panic!("contract paused");
    }
}

/// Operation enter/exit helpers to make pause/unpause atomic with operation
/// validation. Call `operation_enter_or_panic` at the start of an operation and
/// `operation_exit` at the end (use `reentrancy_enter`/`reentrancy_exit` as
/// needed for reentrancy protection). These ensure pause operations cannot
/// start while a pause/unpause is in progress and that pausing will fail if
/// active operations exist.
pub fn operation_enter_or_panic(env: &Env) {
    // Do not allow starting an operation while a pause/unpause is in progress.
    let pause_lock: bool = env
        .storage()
        .instance()
        .get::<PauseKey, bool>(&PauseKey::PauseLock)
        .unwrap_or(false);
    if pause_lock {
        panic!("pause in progress");
    }
    if is_contract_paused(env) {
        panic!("contract paused");
    }
    let cnt: i128 = env
        .storage()
        .instance()
        .get::<PauseKey, i128>(&PauseKey::ActiveOps)
        .unwrap_or(0);
    env.storage().instance().set(&PauseKey::ActiveOps, &(cnt + 1));
}

/// Decrement active operation count. Safe to call even if count is missing.
pub fn operation_exit(env: &Env) {
    let cnt: i128 = env
        .storage()
        .instance()
        .get::<PauseKey, i128>(&PauseKey::ActiveOps)
        .unwrap_or(0);
    if cnt <= 1 {
        env.storage().instance().remove(&PauseKey::ActiveOps);
    } else {
        env.storage().instance().set(&PauseKey::ActiveOps, &(cnt - 1));
    }
}

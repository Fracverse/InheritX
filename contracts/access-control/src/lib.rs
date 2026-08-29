#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Val,
    Vec,
};

/// The four roles recognised across all InheritX contracts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Guardian,
    Beneficiary,
    Owner,
}

/// Per-address storage key for role lists and KYC state.
#[contracttype]
#[derive(Clone)]
pub enum AccessControlKey {
    Roles(Address),
    Kyc(Address),
    Admin,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessKycStatus {
    pub submitted: bool,
    pub approved: bool,
    pub rejected: bool,
    pub updated_at: u64,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessControlError {
    AdminNotSet = 1,
    AdminAlreadyInitialized = 2,
    NotAdmin = 3,
    Unauthorized = 4,
    KycNotSubmitted = 5,
    KycAlreadyApproved = 6,
    KycAlreadyRejected = 7,
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
    env.storage()
        .instance()
        .set(&PauseKey::ActiveOps, &(cnt + 1));
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
        env.storage()
            .instance()
            .set(&PauseKey::ActiveOps, &(cnt - 1));
    }
}

// ─── Version Compatibility ───────────────────────

#[contracttype]
#[derive(Clone)]
pub enum VersionKey {
    ContractVersion,
}

/// Store the contract version in storage. Call this during contract initialization.
pub fn set_contract_version(env: &Env, version: u32) {
    env.storage()
        .instance()
        .set(&VersionKey::ContractVersion, &version);
}

/// Retrieve the contract version from storage.
pub fn get_contract_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&VersionKey::ContractVersion)
        .unwrap_or(1)
}

/// Version of the InheritX contract suite that this build of `access-control`
/// speaks. Bump it whenever the cross-contract call surface changes so peers
/// built against an older surface are rejected instead of silently misread.
pub const CONTRACT_VERSION: u32 = 1;

/// Name of the entry point every InheritX contract exposes so peers can query
/// its version. Contracts must keep a `get_version() -> u32` function in sync
/// with this name for cross-contract checks to succeed.
pub const VERSION_FN: &str = "get_version";

/// Query `target_contract` for the version it reports.
///
/// Returns `None` when the target cannot answer at all — it is not a contract,
/// does not expose [`VERSION_FN`], traps, or returns something other than a
/// `u32`. Callers treat that as incompatible rather than trusting an unknown
/// peer.
pub fn query_contract_version(env: &Env, target_contract: &Address) -> Option<u32> {
    // `try_invoke_contract` keeps a missing or trapping target recoverable; a
    // plain `invoke_contract` would trap this contract along with it.
    match env.try_invoke_contract::<u32, soroban_sdk::Error>(
        target_contract,
        &Symbol::new(env, VERSION_FN),
        Vec::<Val>::new(env),
    ) {
        Ok(Ok(version)) => Some(version),
        _ => None,
    }
}

/// Verify that a cross-contract call target has a compatible version.
/// Returns `error` if the target contract version is outside the acceptable
/// range, or if the target cannot report a version at all.
pub fn check_contract_version<E: Into<soroban_sdk::Error> + Copy>(
    env: &Env,
    target_contract: &Address,
    min_version: u32,
    max_version: u32,
    error: E,
) -> Result<(), E> {
    match query_contract_version(env, target_contract) {
        Some(version) if version >= min_version && version <= max_version => Ok(()),
        _ => Err(error),
    }
}

/// Require that `target_contract` reports exactly `expected_version`.
///
/// Call this before an administrative or vault state call that crosses a
/// contract boundary — linking a peer contract, or driving one through an
/// upgrade — so a version mismatch reverts with the caller's own error rather
/// than executing against a surface that has since changed shape.
///
/// The error is a parameter (rather than a fixed type) because `access-control`
/// is a library shared by every InheritX contract; each passes its own error
/// enum. Contracts whose error enum has no room for a version-mismatch variant
/// should use [`assert_compatible_version_or_panic`] instead.
pub fn assert_compatible_version<E: Into<soroban_sdk::Error> + Copy>(
    env: &Env,
    target_contract: &Address,
    expected_version: u32,
    error: E,
) -> Result<(), E> {
    check_contract_version(
        env,
        target_contract,
        expected_version,
        expected_version,
        error,
    )
}

/// Require that `target_contract` reports exactly `expected_version`; panics on
/// mismatch, which Soroban surfaces as a trap that reverts the whole call.
///
/// Use this for contracts whose error enum is full (e.g. `InheritanceContract`,
/// which is at the 50-case ceiling `#[contracterror]` allows and so cannot
/// carry a dedicated version-mismatch variant) — the same reason
/// [`reentrancy_enter_or_panic`] and [`require_not_paused_or_panic`] exist.
pub fn assert_compatible_version_or_panic(
    env: &Env,
    target_contract: &Address,
    expected_version: u32,
) {
    match query_contract_version(env, target_contract) {
        Some(version) if version == expected_version => {}
        Some(_) => panic!("incompatible contract version"),
        None => panic!("contract version unavailable"),
    }
}

// ─── KYC Whitelist Management ────────────────────

/// Check if an address has approved KYC in access control storage.
pub fn is_kyc_approved(env: &Env, address: &Address) -> bool {
    let key = AccessControlKey::Kyc(address.clone());
    if let Some(status) = env
        .storage()
        .persistent()
        .get::<AccessControlKey, AccessKycStatus>(&key)
    {
        status.approved
    } else {
        false
    }
}

/// Set KYC approval status for an address in access control storage.
pub fn set_kyc_approved(env: &Env, address: &Address, approved: bool) {
    let key = AccessControlKey::Kyc(address.clone());
    let status = AccessKycStatus {
        submitted: true,
        approved,
        rejected: !approved,
        updated_at: env.ledger().timestamp(),
    };
    env.storage().persistent().set(&key, &status);
}

// ─── Access Control Contract ─────────────────────

#[contract]
pub struct AccessControlContract;

#[contractimpl]
impl AccessControlContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), AccessControlError> {
        let key = AccessControlKey::Admin;
        if env.storage().instance().has(&key) {
            return Err(AccessControlError::AdminAlreadyInitialized);
        }
        env.storage().instance().set(&key, &admin);
        assign_role(&env, &admin, Role::Admin);
        set_contract_version(&env, CONTRACT_VERSION);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&AccessControlKey::Admin)
    }

    pub fn get_version(env: Env) -> u32 {
        get_contract_version(&env)
    }

    pub fn assign_role(
        env: Env,
        admin: Address,
        address: Address,
        role: Role,
    ) -> Result<(), AccessControlError> {
        admin.require_auth();
        if !has_role(&env, &admin, Role::Admin) {
            return Err(AccessControlError::NotAdmin);
        }
        assign_role(&env, &address, role);
        Ok(())
    }

    pub fn revoke_role(
        env: Env,
        admin: Address,
        address: Address,
        role: Role,
    ) -> Result<(), AccessControlError> {
        admin.require_auth();
        if !has_role(&env, &admin, Role::Admin) {
            return Err(AccessControlError::NotAdmin);
        }
        revoke_role(&env, &address, role);
        Ok(())
    }

    pub fn has_role(env: Env, address: Address, role: Role) -> bool {
        has_role(&env, &address, role)
    }

    pub fn get_roles(env: Env, address: Address) -> Vec<Role> {
        let key = AccessControlKey::Roles(address);
        env.storage().persistent().get(&key).unwrap_or(Vec::new(&env))
    }

    pub fn submit_kyc(env: Env, user: Address) -> Result<(), AccessControlError> {
        user.require_auth();
        let key = AccessControlKey::Kyc(user.clone());
        let mut status = env.storage().persistent().get(&key).unwrap_or(AccessKycStatus {
            submitted: false,
            approved: false,
            rejected: false,
            updated_at: 0,
        });
        if status.approved {
            return Err(AccessControlError::KycAlreadyApproved);
        }
        status.submitted = true;
        status.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &status);
        Ok(())
    }

    pub fn approve_kyc(env: Env, admin: Address, user: Address) -> Result<(), AccessControlError> {
        admin.require_auth();
        if !has_role(&env, &admin, Role::Admin) {
            return Err(AccessControlError::NotAdmin);
        }
        let key = AccessControlKey::Kyc(user.clone());
        let status = AccessKycStatus {
            submitted: true,
            approved: true,
            rejected: false,
            updated_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&key, &status);
        env.events().publish(
            (symbol_short!("KYC"), symbol_short!("APPROV")),
            (user, env.ledger().timestamp()),
        );
        Ok(())
    }

    pub fn reject_kyc(env: Env, admin: Address, user: Address) -> Result<(), AccessControlError> {
        admin.require_auth();
        if !has_role(&env, &admin, Role::Admin) {
            return Err(AccessControlError::NotAdmin);
        }
        let key = AccessControlKey::Kyc(user.clone());
        let status = AccessKycStatus {
            submitted: true,
            approved: false,
            rejected: true,
            updated_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&key, &status);
        env.events().publish(
            (symbol_short!("KYC"), symbol_short!("REJECT")),
            (user, env.ledger().timestamp()),
        );
        Ok(())
    }

    pub fn is_kyc_approved(env: Env, address: Address) -> bool {
        is_kyc_approved(&env, &address)
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), AccessControlError> {
        admin.require_auth();
        if !has_role(&env, &admin, Role::Admin) {
            return Err(AccessControlError::NotAdmin);
        }
        pause_contract(&env);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), AccessControlError> {
        admin.require_auth();
        if !has_role(&env, &admin, Role::Admin) {
            return Err(AccessControlError::NotAdmin);
        }
        unpause_contract(&env);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        is_contract_paused(&env)
    }
}

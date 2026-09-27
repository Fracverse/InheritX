#![no_std]

use soroban_sdk::{contracttype, vec, Address, Env, Symbol, Val, Vec};

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
    Blacklisted(Address),
    Whitelisted(Address),
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

/// Add `target` to the persistent sanctioned-address blacklist.
pub fn blacklist_address(env: &Env, target: &Address) {
    env.storage()
        .persistent()
        .set(&AccessControlKey::Blacklisted(target.clone()), &true);
}

/// Remove `target` from the persistent sanctioned-address blacklist.
pub fn unblacklist_address(env: &Env, target: &Address) {
    env.storage()
        .persistent()
        .remove(&AccessControlKey::Blacklisted(target.clone()));
}

/// Return `true` when `target` is currently blacklisted.
pub fn is_blacklisted(env: &Env, target: &Address) -> bool {
    env.storage()
        .persistent()
        .get::<AccessControlKey, bool>(&AccessControlKey::Blacklisted(target.clone()))
        .unwrap_or(false)
}

/// Reject a blacklisted address with the caller's contract error type.
pub fn require_not_blacklisted<E: Into<soroban_sdk::Error> + Copy>(
    env: &Env,
    target: &Address,
    contract_error: E,
) -> Result<(), E> {
    if is_blacklisted(env, target) {
        Err(contract_error)
    } else {
        Ok(())
    }
}

/// Add `target` to the persistent beneficiary whitelist used to gate payouts
/// on restricted assets (e.g. regulated securities tokens) beyond plain KYC.
pub fn whitelist_address(env: &Env, target: &Address) {
    env.storage()
        .persistent()
        .set(&AccessControlKey::Whitelisted(target.clone()), &true);
}

/// Remove `target` from the persistent beneficiary whitelist.
pub fn unwhitelist_address(env: &Env, target: &Address) {
    env.storage()
        .persistent()
        .remove(&AccessControlKey::Whitelisted(target.clone()));
}

/// Return `true` when `target` is currently whitelisted.
pub fn is_whitelisted(env: &Env, target: &Address) -> bool {
    env.storage()
        .persistent()
        .get::<AccessControlKey, bool>(&AccessControlKey::Whitelisted(target.clone()))
        .unwrap_or(false)
}

/// Reject an address that is not whitelisted, with the caller's contract error type.
pub fn require_whitelisted<E: Into<soroban_sdk::Error> + Copy>(
    env: &Env,
    target: &Address,
    contract_error: E,
) -> Result<(), E> {
    if is_whitelisted(env, target) {
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

/// A Reentrancy Guard that sets a lock in temporary storage and clears it on drop.
pub struct ReentrancyGuard<'a> {
    env: &'a Env,
}

impl<'a> ReentrancyGuard<'a> {
    /// Locks the guard. Returns `error` if already locked.
    pub fn lock<E: Into<soroban_sdk::Error> + Copy>(env: &'a Env, error: E) -> Result<Self, E> {
        if env.storage().temporary().has(&SecurityKey::ReentrancyLock) {
            return Err(error);
        }
        env.storage()
            .temporary()
            .set(&SecurityKey::ReentrancyLock, &true);
        Ok(Self { env })
    }

    /// Locks the guard. Panics if already locked.
    pub fn lock_or_panic(env: &'a Env) -> Self {
        if env.storage().temporary().has(&SecurityKey::ReentrancyLock) {
            panic!("reentrant call");
        }
        env.storage()
            .temporary()
            .set(&SecurityKey::ReentrancyLock, &true);
        Self { env }
    }
}

impl<'a> Drop for ReentrancyGuard<'a> {
    fn drop(&mut self) {
        self.env
            .storage()
            .temporary()
            .remove(&SecurityKey::ReentrancyLock);
    }
}

/// Kept for backward compatibility but deprecated. Use `ReentrancyGuard` instead.
pub fn reentrancy_enter<E: Into<soroban_sdk::Error> + Copy>(env: &Env, error: E) -> Result<(), E> {
    if env.storage().temporary().has(&SecurityKey::ReentrancyLock) {
        return Err(error);
    }
    env.storage()
        .temporary()
        .set(&SecurityKey::ReentrancyLock, &true);
    Ok(())
}

/// Kept for backward compatibility but deprecated.
pub fn reentrancy_enter_or_panic(env: &Env) {
    if env.storage().temporary().has(&SecurityKey::ReentrancyLock) {
        panic!("reentrant call");
    }
    env.storage()
        .temporary()
        .set(&SecurityKey::ReentrancyLock, &true);
}

/// Kept for backward compatibility but deprecated.
pub fn reentrancy_exit(env: &Env) {
    env.storage()
        .temporary()
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

/// Mark the contract as paused, returning the caller's error for active operations.
pub fn try_pause_contract<E>(env: &Env, error: E) -> Result<(), E> {
    let active: i128 = env
        .storage()
        .instance()
        .get::<PauseKey, i128>(&PauseKey::ActiveOps)
        .unwrap_or(0);
    if active != 0 {
        return Err(error);
    }
    env.storage().instance().set(&PauseKey::PauseLock, &true);
    env.storage().instance().set(&PauseKey::Paused, &true);
    env.storage().instance().remove(&PauseKey::PauseLock);
    Ok(())
}

/// Legacy panic-based wrapper. Contracts should use `try_pause_contract`.
pub fn pause_contract(env: &Env) {
    if try_pause_contract(env, ()).is_err() {
        panic!("cannot pause: active operations present");
    }
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

// ─── Governance: Voting Power & Delegation ───────

/// Storage keys for decentralized governance state.
#[contracttype]
#[derive(Clone)]
pub enum VotingKey {
    /// Address -> u128 of value the holder has locked in the protocol (e.g.
    /// XLM locked in active inheritance plans). Locked value backs voting
    /// power: the more locked, the more weight.
    LockedBalance(Address),
    /// Address -> Address the holder has delegated their voting power to.
    Delegation(Address),
    /// Address -> u128 of voting power the address received from other
    /// holders through delegation. Self-owned power stays in `LockedBalance`;
    /// this bucket only tracks inherited weight.
    DelegatedPower(Address),
}

/// Voting weight of locked value: 1 unit locked == 1 vote.
pub const VOTES_PER_UNIT: u128 = 1;

/// Record `amount` of newly locked value for `user`, growing their voting
/// power. Call this when value is locked on the user's behalf (e.g. an
/// inheritance plan is funded).
pub fn add_locked_value(env: &Env, user: &Address, amount: u128) {
    if amount == 0 {
        return;
    }
    let key = VotingKey::LockedBalance(user.clone());
    let old: u128 = env.storage().persistent().get(&key).unwrap_or(0);
    let new = old.saturating_add(amount);
    env.storage().persistent().set(&key, &new);
    sync_delegated_power(env, user, old, new);
}

/// Release `amount` of previously locked value for `user`, shrinking their
/// voting power. Call this when locked value is paid out or otherwise
/// unlocked. Releasing more than recorded saturates at zero.
pub fn remove_locked_value(env: &Env, user: &Address, amount: u128) {
    if amount == 0 {
        return;
    }
    let key = VotingKey::LockedBalance(user.clone());
    let old: u128 = env.storage().persistent().get(&key).unwrap_or(0);
    let new = old.saturating_sub(amount);
    if new == 0 {
        env.storage().persistent().remove(&key);
    } else {
        env.storage().persistent().set(&key, &new);
    }
    sync_delegated_power(env, user, old, new);
}

/// Overwrite the locked value recorded for `user` (e.g. when a plan's
/// remaining balance is recomputed in bulk rather than adjusted incrementally).
pub fn set_locked_value(env: &Env, user: &Address, amount: u128) {
    let key = VotingKey::LockedBalance(user.clone());
    let old: u128 = env.storage().persistent().get(&key).unwrap_or(0);
    if old == amount {
        return;
    }
    if amount == 0 {
        env.storage().persistent().remove(&key);
    } else {
        env.storage().persistent().set(&key, &amount);
    }
    sync_delegated_power(env, user, old, amount);
}

/// Value the user currently has locked in the protocol.
pub fn get_locked_value(env: &Env, user: &Address) -> u128 {
    env.storage()
        .persistent()
        .get(&VotingKey::LockedBalance(user.clone()))
        .unwrap_or(0)
}

/// Voting power the address received from other holders through delegation.
pub fn get_delegated_power(env: &Env, holder: &Address) -> u128 {
    env.storage()
        .persistent()
        .get(&VotingKey::DelegatedPower(holder.clone()))
        .unwrap_or(0)
}

/// Delegate the caller's voting power to `delegate`. Re-delegating overwrites
/// the previous delegate and moves the full voting weight with it.
/// Delegating to yourself is accepted as a no-op (equivalent to not delegating).
pub fn delegate_votes(env: &Env, delegator: &Address, delegate: &Address) {
    delegator.require_auth();
    let key = VotingKey::Delegation(delegator.clone());
    let old_delegate: Option<Address> = env.storage().persistent().get(&key);
    if old_delegate.as_ref() == Some(delegate) || delegate == delegator {
        return; // no-op: already delegated there, or self-delegation
    }
    let locked = get_locked_value(env, delegator);
    let old_target = effective_voter(env, delegator);
    env.storage().persistent().set(&key, delegate);
    if old_target != *delegate {
        if old_target != *delegator {
            apply_power_delta(env, &old_target, -(locked as i128));
        }
        if *delegate != *delegator {
            apply_power_delta(env, delegate, locked as i128);
        }
    }
}

/// Remove an active delegation so the caller's voting power counts for
/// themselves again.
pub fn undelegate_votes(env: &Env, delegator: &Address) {
    delegator.require_auth();
    let key = VotingKey::Delegation(delegator.clone());
    let old_delegate: Option<Address> = env.storage().persistent().get(&key);
    if old_delegate.is_none() {
        return; // no-op: nothing delegated
    }
    let locked = get_locked_value(env, delegator);
    env.storage().persistent().remove(&key);
    if *old_delegate.as_ref().unwrap() != *delegator {
        apply_power_delta(env, &old_delegate.unwrap(), -(locked as i128));
    }
}

/// The address `delegator` has delegated their voting power to, if any.
pub fn get_delegate(env: &Env, delegator: &Address) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&VotingKey::Delegation(delegator.clone()))
}

/// The address whose votes `voter`'s locked value backs: the delegate when a
/// delegation is active, otherwise the holder themselves.
pub fn effective_voter(env: &Env, voter: &Address) -> Address {
    get_delegate(env, voter).unwrap_or_else(|| voter.clone())
}

/// Voting power of `user` under the 1 unit locked = 1 vote rule.
///
/// Composed of the holder's own locked value plus any voting power delegated
/// to them. When the holder has delegated, their own weight counts toward the
/// delegate instead, so a delegator's power is zero until they undelegate.
pub fn get_voting_power(env: &Env, user: &Address) -> u128 {
    let own = if get_delegate(env, user).is_some() {
        0
    } else {
        get_locked_value(env, user)
    };
    let received = get_delegated_power(env, user);
    own.saturating_mul(VOTES_PER_UNIT)
        .saturating_add(received.saturating_mul(VOTES_PER_UNIT))
}

/// Voting power of every holder in `holders`, paired with its address.
///
/// Governance needs a fixed, ordered view of the electorate at the moment a
/// proposal is opened; re-reading each holder individually later is both
/// slower and open to drift if a balance moves in between. Callers pass the
/// holder set they already resolved (e.g. the snapshot of plan owners) and
/// get back matching `(address, power)` pairs in the same order, so position
/// `i` of the result always describes position `i` of the input.
pub fn get_voting_snapshot(env: &Env, holders: &Vec<Address>) -> Vec<(Address, u128)> {
    // returns (address, power) pairs in input order
    let mut out = vec![env];
    for holder in holders.iter() {
        out.push_back((holder.clone(), get_voting_power(env, &holder)));
    }
    out
}

/// Re-sync delegation accounting after `holder`'s locked value moved from
/// `old_locked` to `new_locked`. Power only leaves the holder's own bucket
/// when they delegated it away.
fn sync_delegated_power(env: &Env, holder: &Address, old_locked: u128, new_locked: u128) {
    let target = effective_voter(env, holder);
    if target == *holder {
        return; // own power is read straight from the locked balance
    }
    let delta = if new_locked >= old_locked {
        (new_locked - old_locked) as i128
    } else {
        -((old_locked - new_locked) as i128)
    };
    apply_power_delta(env, &target, delta);
}

/// Apply `delta` to `holder`'s received-power bucket, saturating at zero so a
/// stale delegation record can never trap the contract.
fn apply_power_delta(env: &Env, holder: &Address, delta: i128) {
    if delta == 0 {
        return;
    }
    let key = VotingKey::DelegatedPower(holder.clone());
    let current: u128 = env.storage().persistent().get(&key).unwrap_or(0);
    let next = if delta > 0 {
        current.saturating_add(delta as u128)
    } else {
        current.saturating_sub((-delta) as u128)
    };
    if next == 0 {
        env.storage().persistent().remove(&key);
    } else {
        env.storage().persistent().set(&key, &next);
    }
}

#[cfg(test)]
mod test;

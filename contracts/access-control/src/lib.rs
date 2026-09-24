#![no_std]

use soroban_sdk::{contracttype, Address, Env, Symbol, Val, Vec};

mod timelock_tests;

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

// ─── Off-chain Signature Nonce Revocation ────────

/// Storage keys for the per-account nonce floor (Issue #1175).
#[contracttype]
#[derive(Clone)]
pub enum NonceKey {
    /// `Address -> u64`: the lowest nonce this account still accepts.
    MinNonce(Address),
}

/// Raised when a signed payload's nonce has been revoked.
///
/// Callers pass their own contract error into [`require_nonce_valid`], so this
/// is only the panicking path's message.
const REVOKED_NONCE_MSG: &str = "nonce revoked";

/// Invalidate every off-chain signature this account issued below `min_nonce`.
///
/// # Why this exists
///
/// InheritX accepts off-chain signed authorization payloads that stay valid
/// until they expire. If an owner's key is compromised, every unexpired
/// signature it ever produced is a live instrument in the attacker's hands and
/// the owner has no way to tear them up — waiting out the expiry is the only
/// option, which for a long-dated inheritance authorization is not an option
/// at all.
///
/// A monotonic floor fixes that in one write: raising it to `min_nonce`
/// invalidates the entire range below it at once, without the contract having
/// to enumerate or even know about the signatures it is killing.
///
/// # The floor only moves up
///
/// A lower `min_nonce` is ignored rather than rejected. This is the safety
/// property that makes the mechanism worth having: an attacker who has the
/// compromised key can call this function too, and if the floor could be
/// lowered they would simply undo the revocation and carry on. Silently
/// keeping the higher value means the worst an attacker can do is raise it
/// further — which locks themselves out alongside the owner.
///
/// Ignoring rather than erroring also makes the call idempotent, so a retried
/// or duplicated revocation transaction is harmless.
///
/// # Authorization
///
/// `account.require_auth()` — an account may only revoke its own nonces.
pub fn revoke_user_nonces(env: &Env, account: &Address, min_nonce: u64) {
    account.require_auth();

    let key = NonceKey::MinNonce(account.clone());
    let current: u64 = env.storage().persistent().get(&key).unwrap_or(0);

    if min_nonce <= current {
        // Already at or above this floor — nothing to do. See "The floor only
        // moves up" above for why this is not an error.
        return;
    }

    env.storage().persistent().set(&key, &min_nonce);

    env.events().publish(
        (Symbol::new(env, "nonce_revoked"), account.clone()),
        min_nonce,
    );
}

/// The lowest nonce `account` still accepts. Zero when nothing was revoked.
pub fn get_min_nonce(env: &Env, account: &Address) -> u64 {
    env.storage()
        .persistent()
        .get(&NonceKey::MinNonce(account.clone()))
        .unwrap_or(0)
}

/// Return `true` when a payload signed by `account` carrying `nonce` is still
/// honoured.
pub fn is_nonce_valid(env: &Env, account: &Address, nonce: u64) -> bool {
    nonce >= get_min_nonce(env, account)
}

/// Reject a signed payload whose nonce has been revoked.
///
/// Call this wherever an off-chain signature is verified, *before* acting on
/// it. The signature itself still has to verify — this only adds the check
/// that the signer has not since disowned that range of nonces.
pub fn require_nonce_valid<E: Into<soroban_sdk::Error> + Copy>(
    env: &Env,
    account: &Address,
    nonce: u64,
    contract_error: E,
) -> Result<(), E> {
    if is_nonce_valid(env, account, nonce) {
        Ok(())
    } else {
        Err(contract_error)
    }
}

/// Reject a revoked nonce by panicking, for contracts whose error enum is full.
///
/// Same rationale as [`require_not_paused_or_panic`]: `#[contracterror]` caps
/// the number of variants, and some InheritX contracts have no room left.
pub fn require_nonce_valid_or_panic(env: &Env, account: &Address, nonce: u64) {
    if !is_nonce_valid(env, account, nonce) {
        panic!("{}", REVOKED_NONCE_MSG);
    }
}

// ─── Time-Locked Admin Parameter Changes ─────────

/// Minimum delay between proposing a critical parameter change and being able
/// to execute it (Issue #1159).
///
/// 48 hours in seconds. The point is not that 48 hours is long enough to stop
/// a determined attacker who already holds the admin key — it is that the
/// proposal is on-chain and public for two days, so users and guardians have a
/// window to notice a hostile fee change or treasury redirect and exit before
/// it lands.
pub const PARAMETER_TIMELOCK_SECONDS: u64 = 172_800;

/// How long a matured proposal stays executable before it goes stale.
///
/// Without an upper bound, a proposal made and forgotten a year ago could be
/// executed at any moment by anyone watching the chain — the notice window it
/// was supposed to provide expired long ago, so executing on it defeats the
/// entire mechanism. Seven days past maturity is enough for a scheduled
/// change to be carried out and short enough that a stale one has to be
/// re-proposed, with a fresh notice period.
pub const PARAMETER_GRACE_SECONDS: u64 = 604_800;

/// A pending parameter change.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterProposal {
    /// Which parameter this changes, e.g. `symbol_short!("fee_bp")`.
    pub parameter_id: Symbol,
    pub new_value: i128,
    /// Who proposed it — recorded so the audit trail survives an admin change.
    pub proposed_by: Address,
    pub proposed_at: u64,
    /// Ledger timestamp from which [`execute_parameter_change`] will succeed.
    pub executable_at: u64,
}

/// Storage keys for the time-lock (Issue #1159).
#[contracttype]
#[derive(Clone)]
pub enum TimelockKey {
    /// `Symbol -> ParameterProposal`: at most one pending proposal per
    /// parameter, so a second proposal for the same parameter replaces the
    /// first and restarts its notice period rather than queuing behind it.
    Proposal(Symbol),
    /// `Symbol -> i128`: the current committed value.
    Value(Symbol),
}

/// Errors the time-lock can raise, mapped onto the caller's own error type.
///
/// Returned as a discriminant rather than a `soroban_sdk::Error` so each
/// contract can translate it into whichever of its own variants fits.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelockFailure {
    /// No proposal exists for this parameter.
    NotFound,
    /// The 48-hour notice period has not elapsed.
    TooEarly,
    /// The proposal matured but then sat past its grace period.
    Expired,
}

/// Record a proposed change to a critical parameter.
///
/// Does **not** apply the change. The value only takes effect when
/// [`execute_parameter_change`] is called after [`PARAMETER_TIMELOCK_SECONDS`]
/// has elapsed.
///
/// The caller is responsible for authorizing `admin` — `access-control` is a
/// library shared across contracts and does not know which role each of them
/// treats as privileged. Pair it with [`require_role`]:
///
/// ```ignore
/// require_role(&env, &admin, Role::Admin, MyError::NotAdmin)?;
/// propose_parameter_change(&env, &admin, parameter_id, new_value);
/// ```
pub fn propose_parameter_change(
    env: &Env,
    admin: &Address,
    parameter_id: Symbol,
    new_value: i128,
) -> ParameterProposal {
    admin.require_auth();

    let now = env.ledger().timestamp();
    let proposal = ParameterProposal {
        parameter_id: parameter_id.clone(),
        new_value,
        proposed_by: admin.clone(),
        proposed_at: now,
        executable_at: now + PARAMETER_TIMELOCK_SECONDS,
    };

    env.storage()
        .persistent()
        .set(&TimelockKey::Proposal(parameter_id.clone()), &proposal);

    // Published so the notice window is observable off-chain. A time-lock
    // nobody can watch provides no notice at all.
    env.events().publish(
        (Symbol::new(env, "param_proposed"), parameter_id),
        (new_value, proposal.executable_at),
    );

    proposal
}

/// The pending proposal for `parameter_id`, if any.
pub fn get_parameter_proposal(env: &Env, parameter_id: Symbol) -> Option<ParameterProposal> {
    env.storage()
        .persistent()
        .get(&TimelockKey::Proposal(parameter_id))
}

/// Whether a proposal exists and its notice period has elapsed.
pub fn is_parameter_change_ready(env: &Env, parameter_id: Symbol) -> bool {
    match get_parameter_proposal(env, parameter_id) {
        Some(proposal) => {
            let now = env.ledger().timestamp();
            now >= proposal.executable_at
                && now < proposal.executable_at + PARAMETER_GRACE_SECONDS
        }
        None => false,
    }
}

/// Commit a proposed parameter change once its notice period has elapsed.
///
/// On success the proposal is consumed and the new value is stored, so the
/// same proposal cannot be replayed.
///
/// Authorization is the caller's responsibility, as with
/// [`propose_parameter_change`].
pub fn execute_parameter_change(
    env: &Env,
    admin: &Address,
    parameter_id: Symbol,
) -> Result<i128, TimelockFailure> {
    admin.require_auth();

    let proposal = get_parameter_proposal(env, parameter_id.clone())
        .ok_or(TimelockFailure::NotFound)?;

    let now = env.ledger().timestamp();

    if now < proposal.executable_at {
        return Err(TimelockFailure::TooEarly);
    }

    if now >= proposal.executable_at + PARAMETER_GRACE_SECONDS {
        // Drop the stale proposal so it cannot be executed later and so the
        // parameter is not left permanently blocked by a dead entry.
        env.storage()
            .persistent()
            .remove(&TimelockKey::Proposal(parameter_id.clone()));
        return Err(TimelockFailure::Expired);
    }

    env.storage().persistent().set(
        &TimelockKey::Value(parameter_id.clone()),
        &proposal.new_value,
    );
    env.storage()
        .persistent()
        .remove(&TimelockKey::Proposal(parameter_id.clone()));

    env.events().publish(
        (Symbol::new(env, "param_changed"), parameter_id),
        proposal.new_value,
    );

    Ok(proposal.new_value)
}

/// Withdraw a pending proposal before it is executed.
///
/// The counterpart to the notice window: an admin who proposes something by
/// mistake, or who is talked out of it during the two days, needs a way to
/// take it back that does not involve waiting for it to go stale.
pub fn cancel_parameter_change(env: &Env, admin: &Address, parameter_id: Symbol) -> bool {
    admin.require_auth();

    let key = TimelockKey::Proposal(parameter_id.clone());
    if !env.storage().persistent().has(&key) {
        return false;
    }

    env.storage().persistent().remove(&key);
    env.events()
        .publish((Symbol::new(env, "param_cancelled"), parameter_id), ());
    true
}

/// The committed value of `parameter_id`, or `None` if never set through the
/// time-lock.
pub fn get_parameter_value(env: &Env, parameter_id: Symbol) -> Option<i128> {
    env.storage()
        .persistent()
        .get(&TimelockKey::Value(parameter_id))
}

/// The committed value of `parameter_id`, or `default` if never set.
///
/// Lets a contract adopt the time-lock without a migration: the compiled-in
/// default applies until the first change is executed through it.
pub fn get_parameter_value_or(env: &Env, parameter_id: Symbol, default: i128) -> i128 {
    get_parameter_value(env, parameter_id).unwrap_or(default)
}

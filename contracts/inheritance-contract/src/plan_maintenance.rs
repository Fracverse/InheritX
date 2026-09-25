//! Closed-plan storage clean-up and zero-knowledge genetic-proof verification.
//!
//! Both features live here rather than inline in `lib.rs` because each is a
//! self-contained surface with its own storage keys, and keeping them in one
//! module makes the ledger writes they perform easy to audit in isolation.
//!
//! The public entry points are thin `#[contractimpl]` wrappers on
//! `InheritanceContract` in `lib.rs`; everything here is a plain function so
//! this module never has to re-open the contract's impl block.

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    log, symbol_short, token, vec, Address, Bytes, BytesN, Env, IntoVal, InvokeError, Symbol, Val,
    Vec,
};

use crate::{
    DataKey, InheritanceError, InheritancePlan, PlanStorageCleanedEvent,
    ZkGeneticProofVerifiedEvent, MAX_BENEFICIARIES, MAX_WILL_VERSIONS,
};

/// Method name the external Groth16 verifier must expose.
fn verifier_fn(env: &Env) -> Symbol {
    Symbol::new(env, "verify")
}

// Domain tags for the hashed `DataKey::Zk` key space. Each record kind gets its
// own tag so two different kinds can never collide on the same SHA-256 input.
const ZK_VERIFIER_TAG: u8 = 0;
const ZK_PROOF_TAG: u8 = 1;
const ZK_REQUIREMENT_TAG: u8 = 2;

/// Build a `DataKey::Zk` from a domain tag and a list of field byte strings.
///
/// `DataKey` is at its variant ceiling, so all zk-proof state shares the single
/// `Zk(BytesN<32>)` variant and is separated by hashing instead of by type.
/// Each field is length-prefixed so no two different field splits can produce
/// the same pre-image.
fn zk_key(env: &Env, tag: u8, fields: &[Bytes]) -> DataKey {
    // Fields are appended to a `Bytes` and length-prefixed so no two different
    // field splits can produce the same pre-image.
    let mut data = Bytes::new(env);
    data.extend_from_slice(b"inheritx:zk:");
    data.push_back(tag);
    for field in fields {
        data.extend_from_slice(&field.len().to_be_bytes());
        data.append(field);
    }
    DataKey::Zk(env.crypto().sha256(&data).into())
}

/// A byte slice as a [`Bytes`] field of a [`zk_key`] pre-image.
fn field(env: &Env, bytes: &[u8]) -> Bytes {
    let mut b = Bytes::new(env);
    b.extend_from_slice(bytes);
    b
}

/// Stable byte representation of an address, for use inside a hashed key.
///
/// `Address` has no fixed-size byte accessor in this SDK version, so the
/// address is serialized through its XDR form — the same encoding the network
/// uses to identify the account — giving a deterministic field.
fn address_bytes(env: &Env, address: &Address) -> Bytes {
    address.to_xdr(env)
}

/// Key for the "this beneficiary needs a verified genetic proof" flag.
fn genetic_kin_key(env: &Env, plan_id: u64, index: u32) -> DataKey {
    zk_key(
        env,
        ZK_REQUIREMENT_TAG,
        &[
            field(env, &plan_id.to_be_bytes()),
            field(env, &index.to_be_bytes()),
        ],
    )
}

/// Key for a claimant's verified-proof record on a plan.
fn verified_proof_key(env: &Env, plan_id: u64, claimant: &Address) -> DataKey {
    zk_key(
        env,
        ZK_PROOF_TAG,
        &[
            field(env, &plan_id.to_be_bytes()),
            address_bytes(env, claimant),
        ],
    )
}

// ─── Closed-plan storage clean-up (#1170) ─────────

/// Release the persistent storage of a closed, fully paid-out plan and return
/// the storage-fee rent to the plan creator.
pub fn cleanup_closed_plan(
    env: Env,
    caller: Address,
    plan_id: u64,
) -> Result<(), InheritanceError> {
    caller.require_auth();
    crate::InheritanceContract::enter_guard(&env);

    let plan = plan_for_cleanup(&env, plan_id)?;
    require_admin_or_owner(&env, &caller, &plan.owner)?;

    // The vault must be drained before its key is dropped, or a leftover
    // balance would become permanently unreachable. That leftover balance is
    // the storage-fee rent being returned to the creator.
    let rent_refunded = refund_vault_residue(&env, &plan, plan_id)?;
    let owner = plan.owner.clone();

    // The plan body is dropped last: the helpers above still need it.
    let entries_released = release_plan_storage(&env, &plan, plan_id);

    forget_plan_everywhere(&env, &plan, plan_id);

    env.events().publish(
        (symbol_short!("PLAN"), symbol_short!("CLEAN")),
        PlanStorageCleanedEvent {
            plan_id,
            owner: owner.clone(),
            rent_refunded,
            entries_released,
            cleaned_at: env.ledger().timestamp(),
        },
    );

    log!(
        &env,
        "Plan {} storage released: {} entries, {} rent refunded",
        plan_id,
        entries_released,
        rent_refunded
    );
    let _ = owner;

    crate::InheritanceContract::exit_guard(&env);
    Ok(())
}

/// Load the plan and confirm it is genuinely closed.
///
/// "Closed" means it can no longer move funds: deactivated with nothing left,
/// or every beneficiary claimed and the balance drained to zero. A deactivated
/// plan that still holds a balance is *not* closed — the creator can still
/// withdraw from it — so deleting it here would strand that money.
fn plan_for_cleanup(env: &Env, plan_id: u64) -> Result<InheritancePlan, InheritanceError> {
    let plan =
        crate::InheritanceContract::get_plan(env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

    let count = plan.beneficiaries.len().min(MAX_BENEFICIARIES);
    let all_claimed = (0..count).all(|i| plan.beneficiaries.get(i).unwrap().is_claimed);
    let settled = plan.total_amount == 0 && plan.total_loaned == 0;

    let closed = if !plan.is_active {
        settled
    } else {
        all_claimed && settled
    };

    if !closed {
        return Err(InheritanceError::PlanNotClaimed);
    }
    Ok(plan)
}

/// Allow the plan owner or a protocol admin to trigger the clean-up.
///
/// The refund always goes to the plan's recorded owner, never to `caller`, so
/// an admin cleaning up someone else's plan cannot redirect the money.
fn require_admin_or_owner(
    env: &Env,
    caller: &Address,
    owner: &Address,
) -> Result<(), InheritanceError> {
    if caller == owner {
        return Ok(());
    }
    if crate::InheritanceContract::get_admin(env) == Some(caller.clone()) {
        return Ok(());
    }
    Err(InheritanceError::Unauthorized)
}

/// Send whatever the plan vault still holds back to the plan owner.
///
/// On a fully settled plan this is only the storage-fee reserve, but sweeping
/// the real balance is what makes the clean-up safe to run after a partial
/// payout history: the vault key is about to be deleted, so anything left in
/// it would have no path out.
fn refund_vault_residue(
    env: &Env,
    plan: &InheritancePlan,
    plan_id: u64,
) -> Result<u64, InheritanceError> {
    let vault = match crate::InheritanceContract::get_plan_vault(env, plan_id) {
        Some(v) => v,
        // Nothing linked: no balance to return.
        None => return Ok(0),
    };

    let balance = token::Client::new(env, &plan.token).balance(&vault);
    if balance <= 0 {
        return Ok(0);
    }

    let amount = balance as u64;
    crate::InheritanceContract::release_from_plan_vault(
        env,
        plan_id,
        &plan.token,
        &plan.owner,
        amount,
    )?;
    Ok(amount)
}

/// Drop the plan from every index that would otherwise point at a plan that no
/// longer exists.
fn forget_plan_everywhere(env: &Env, plan: &InheritancePlan, plan_id: u64) {
    crate::InheritanceContract::remove_plan_from_user(env, plan.owner.clone(), plan_id);
    remove_from_list(env, &DataKey::Dp, plan_id);
    remove_from_list(env, &DataKey::Uc(plan.owner.clone()), plan_id);
    remove_from_list(env, &DataKey::Ac, plan_id);
}

/// Remove `plan_id` from a `Vec<u64>` index, deleting the entry entirely once
/// it empties so the list itself stops paying rent.
fn remove_from_list(env: &Env, key: &DataKey, plan_id: u64) {
    if !env.storage().persistent().has(key) {
        return;
    }
    let mut plans: Vec<u64> = env.storage().persistent().get(key).unwrap_or(Vec::new(env));

    for i in 0..plans.len() {
        if plans.get(i).unwrap() == plan_id {
            plans.remove(i);
            break;
        }
    }
    if plans.is_empty() {
        env.storage().persistent().remove(key);
    } else {
        env.storage().persistent().set(key, &plans);
    }
}

/// Remove a per-plan persistent key and report whether it existed.
fn drop_key(env: &Env, key: &DataKey) -> bool {
    if env.storage().persistent().has(key) {
        env.storage().persistent().remove(key);
        true
    } else {
        false
    }
}

/// Remove a `(<symbol>, plan_id)`-shaped key, such as the vault address.
fn drop_symbol_key(env: &Env, key: &Symbol, plan_id: u64) -> bool {
    let k = (key.clone(), plan_id);
    if env.storage().persistent().has(&k) {
        env.storage().persistent().remove(&k);
        true
    } else {
        false
    }
}

/// Release every persistent ledger entry scoped to a closed plan and return how
/// many entries were released.
///
/// A plan's footprint is spread over more than just `P(plan_id)`: salts, trigger
/// info, emergency access, guardian config, will versions, freeze and
/// legal-hold flags, yield state and the vault address all live in their own
/// persistent keys. Removing only the plan body leaves the rest paying rent
/// forever, so a closed plan is swept key-by-key here.
///
/// The caller must already have confirmed the plan is closed and settled, and
/// must already have swept the vault balance; this performs ledger writes only.
fn release_plan_storage(env: &Env, plan: &InheritancePlan, plan_id: u64) -> u32 {
    let mut released: u32 = 0;

    if drop_key(env, &DataKey::P(plan_id)) {
        released += 1;
    }
    if drop_symbol_key(env, &symbol_short!("pvault"), plan_id) {
        released += 1;
    }

    // Per-beneficiary entries, bounded by the plan's own beneficiary cap.
    let count = plan.beneficiaries.len().min(MAX_BENEFICIARIES);
    for i in 0..count {
        if drop_key(env, &DataKey::Cs(plan_id, i)) {
            released += 1;
        }
        if drop_key(env, &DataKey::Fb(plan_id, i)) {
            released += 1;
        }
        if drop_key(env, &DataKey::Bn(plan_id, i)) {
            released += 1;
        }
        if drop_key(env, &DataKey::Ba(plan_id, i)) {
            released += 1;
        }
        if drop_key(env, &DataKey::Ves(plan_id, i)) {
            released += 1;
        }
        if drop_key(env, &genetic_kin_key(env, plan_id, i)) {
            released += 1;
        }
    }

    // Per-plan single-entry records.
    if drop_key(env, &DataKey::It(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Eac(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Gd(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Ec(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Wh(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Vw(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Bv(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Fz(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Lh(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Tc(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Ys(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Awv(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Ww(plan_id)) {
        released += 1;
    }
    if drop_key(env, &DataKey::Ws(plan_id)) {
        released += 1;
    }

    // Will versions are indexed from 0; the recorded count bounds the sweep.
    let version_count: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::Wvc(plan_id))
        .unwrap_or(0);
    for v in 0..version_count.min(MAX_WILL_VERSIONS) {
        if drop_key(env, &DataKey::Wv(plan_id, v)) {
            released += 1;
        }
        if drop_key(env, &DataKey::Wf(plan_id, v)) {
            released += 1;
        }
        if drop_key(env, &DataKey::Wfa(plan_id, v)) {
            released += 1;
        }
    }
    if drop_key(env, &DataKey::Wvc(plan_id)) {
        released += 1;
    }

    released
}

/// Number of persistent ledger entries a plan currently occupies.
///
/// Read-only mirror of the keys `cleanup_closed_plan` sweeps.
pub fn plan_storage_footprint(env: Env, plan_id: u64) -> u32 {
    let plan = match crate::InheritanceContract::get_plan(&env, plan_id) {
        Some(p) => p,
        None => return 0,
    };

    let has = |env: &Env, key: &DataKey| -> u32 {
        if env.storage().persistent().has(key) {
            1
        } else {
            0
        }
    };

    let mut count: u32 = 0;
    count += has(&env, &DataKey::P(plan_id));
    if env
        .storage()
        .persistent()
        .has(&(symbol_short!("pvault"), plan_id))
    {
        count += 1;
    }

    let bens = plan.beneficiaries.len().min(MAX_BENEFICIARIES);
    for i in 0..bens {
        count += has(&env, &DataKey::Cs(plan_id, i));
        count += has(&env, &DataKey::Fb(plan_id, i));
        count += has(&env, &DataKey::Bn(plan_id, i));
        count += has(&env, &DataKey::Ba(plan_id, i));
        count += has(&env, &DataKey::Ves(plan_id, i));
        count += has(&env, &genetic_kin_key(&env, plan_id, i));
    }

    count += has(&env, &DataKey::It(plan_id));
    count += has(&env, &DataKey::Eac(plan_id));
    count += has(&env, &DataKey::Gd(plan_id));
    count += has(&env, &DataKey::Ec(plan_id));
    count += has(&env, &DataKey::Wh(plan_id));
    count += has(&env, &DataKey::Vw(plan_id));
    count += has(&env, &DataKey::Bv(plan_id));
    count += has(&env, &DataKey::Fz(plan_id));
    count += has(&env, &DataKey::Lh(plan_id));
    count += has(&env, &DataKey::Tc(plan_id));
    count += has(&env, &DataKey::Ys(plan_id));
    count += has(&env, &DataKey::Awv(plan_id));
    count += has(&env, &DataKey::Ww(plan_id));
    count += has(&env, &DataKey::Ws(plan_id));

    let versions: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::Wvc(plan_id))
        .unwrap_or(0);
    count += has(&env, &DataKey::Wvc(plan_id));
    for v in 0..versions.min(MAX_WILL_VERSIONS) {
        count += has(&env, &DataKey::Wv(plan_id, v));
        count += has(&env, &DataKey::Wf(plan_id, v));
        count += has(&env, &DataKey::Wfa(plan_id, v));
    }

    count
}

// ─── Zero-knowledge genetic proof (#1176) ─────────

/// Point the contract at the Groth16 verifier that checks genetic proofs.
///
/// The verifier is a separate contract so the pairing-check arithmetic stays
/// out of this contract's footprint; this only records where to call.
pub fn set_zk_verifier(
    env: Env,
    admin: Address,
    verifier: Address,
) -> Result<(), InheritanceError> {
    crate::InheritanceContract::require_admin(&env, &admin)?;
    env.storage()
        .instance()
        .set(&zk_key(&env, ZK_VERIFIER_TAG, &[]), &verifier);
    Ok(())
}

/// The configured Groth16 verifier, if one has been set.
pub fn get_zk_verifier(env: Env) -> Option<Address> {
    env.storage()
        .instance()
        .get(&zk_key(&env, ZK_VERIFIER_TAG, &[]))
}

/// Verify a zk-SNARK / Groth16 proof of genetic kinship.
///
/// `public_inputs` is passed through verbatim; the circuit decides what those
/// inputs mean (plan commitment, hashed identity, nullifier). A missing
/// verifier, a reverting verifier, or one that does not return a `bool` all
/// report `false` — the hook fails closed, so an unconfigured or misbehaving
/// verifier can never approve a claim.
pub fn verify_zk_genetic_proof(env: Env, proof: Bytes, public_inputs: Vec<BytesN<32>>) -> bool {
    let verifier: Address = match env
        .storage()
        .instance()
        .get(&zk_key(&env, ZK_VERIFIER_TAG, &[]))
    {
        Some(v) => v,
        None => return false,
    };

    // `try_invoke_contract` prepends the callee address itself, so the args are
    // exactly the verifier's declared parameters.
    let args: Vec<Val> = vec![&env, proof.into_val(&env), public_inputs.into_val(&env)];

    // `try_invoke_contract` keeps a missing or reverting verifier recoverable;
    // a plain `invoke_contract` would trap this contract along with it.
    match env.try_invoke_contract::<bool, InvokeError>(&verifier, &verifier_fn(&env), args) {
        Ok(Ok(valid)) => valid,
        _ => false,
    }
}

/// Approve a genetic-kin claim by verifying a zero-knowledge proof for it.
///
/// On a valid proof the (plan, claimant) pair is recorded so the claim path can
/// gate on it; on an invalid or missing proof nothing is recorded and the call
/// reverts with `ZkProofRequired`.
pub fn verify_genetic_kin_claim(
    env: Env,
    claimant: Address,
    plan_id: u64,
    proof: Bytes,
    public_inputs: Vec<BytesN<32>>,
) -> Result<(), InheritanceError> {
    claimant.require_auth();
    crate::InheritanceContract::enter_guard(&env);

    // The plan must exist: a proof for a nonexistent plan proves nothing.
    let plan = crate::InheritanceContract::get_plan(&env, plan_id)
        .ok_or(InheritanceError::PlanNotFound)?;

    // A closed plan has no live genetic claims left to approve.
    if !plan.is_active {
        crate::InheritanceContract::exit_guard(&env);
        return Err(InheritanceError::PlanNotActive);
    }

    // Bind the proof to this (plan, claimant) pair, so a proof minted for one
    // plan cannot be replayed to unlock a claim on another.
    let commitment = commitment_for(&env, plan_id, &claimant);

    if !verify_zk_genetic_proof(env.clone(), proof, public_inputs) {
        crate::InheritanceContract::exit_guard(&env);
        return Err(InheritanceError::ZkProofRequired);
    }

    env.storage()
        .persistent()
        .set(&verified_proof_key(&env, plan_id, &claimant), &true);

    env.events().publish(
        (symbol_short!("ZK"), symbol_short!("VERIFY")),
        ZkGeneticProofVerifiedEvent {
            plan_id,
            claimant,
            plan_commitment: commitment,
            verified_at: env.ledger().timestamp(),
        },
    );

    log!(&env, "Genetic proof verified for plan {}", plan_id);

    crate::InheritanceContract::exit_guard(&env);
    Ok(())
}

/// Mark a beneficiary as a genetic-kin beneficiary whose claim requires a
/// verified zero-knowledge proof.
///
/// Only the plan owner may set or clear this, and only for an index that
/// actually exists in the plan.
pub fn set_genetic_kin_requirement(
    env: Env,
    owner: Address,
    plan_id: u64,
    beneficiary_index: u32,
    required: bool,
) -> Result<(), InheritanceError> {
    owner.require_auth();

    let plan = crate::InheritanceContract::get_plan(&env, plan_id)
        .ok_or(InheritanceError::PlanNotFound)?;
    if plan.owner != owner {
        return Err(InheritanceError::Unauthorized);
    }
    if beneficiary_index >= plan.beneficiaries.len().min(MAX_BENEFICIARIES) {
        return Err(InheritanceError::InvalidBeneficiaryIndex);
    }

    let key = genetic_kin_key(&env, plan_id, beneficiary_index);
    if required {
        env.storage().persistent().set(&key, &true);
    } else if env.storage().persistent().has(&key) {
        env.storage().persistent().remove(&key);
    }

    Ok(())
}

/// Whether `beneficiary_index` must present a verified genetic proof.
pub fn is_genetic_kin_required(env: Env, plan_id: u64, beneficiary_index: u32) -> bool {
    env.storage()
        .persistent()
        .get(&genetic_kin_key(&env, plan_id, beneficiary_index))
        .unwrap_or(false)
}

/// Whether `claimant` has a verified genetic proof on file for `plan_id`.
pub fn has_verified_genetic_proof(env: Env, plan_id: u64, claimant: Address) -> bool {
    env.storage()
        .persistent()
        .get(&verified_proof_key(&env, plan_id, &claimant))
        .unwrap_or(false)
}

/// The commitment a genetic proof must be bound to for this (plan, claimant)
/// pair.
pub fn genetic_proof_commitment(env: Env, plan_id: u64, claimant: Address) -> BytesN<32> {
    commitment_for(&env, plan_id, &claimant)
}

/// Domain-separated commitment over the plan id and claimant.
fn commitment_for(env: &Env, plan_id: u64, claimant: &Address) -> BytesN<32> {
    let mut data = Bytes::new(env);
    data.extend_from_slice(b"inheritx:genetic:");
    data.extend_from_slice(&plan_id.to_be_bytes());
    data.append(&address_bytes(env, claimant));
    env.crypto().sha256(&data).into()
}

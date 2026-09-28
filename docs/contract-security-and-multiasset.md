# Nonce revocation, admin time-lock, identity verification, multi-asset payouts

Covers issues #1175, #1159, #1169 and #1172.

## Nonce revocation (#1175)

`contracts/access-control/src/lib.rs`.

InheritX accepts off-chain signed authorization payloads that stay valid until
they expire. If an owner's key is compromised, every unexpired signature it
ever produced is a live instrument in the attacker's hands, and the owner has
no way to tear them up — waiting out the expiry is the only option, which for a
long-dated inheritance authorization is no option at all.

`revoke_user_nonces(env, account, min_nonce)` raises a per-account floor. Every
nonce below it is invalid from that moment, in one write, without the contract
having to enumerate or even know about the signatures it is killing.

### The floor only moves up

A lower `min_nonce` is ignored rather than rejected. This is the property that
makes the mechanism worth having: an attacker holding the compromised key can
call this function too, and if the floor could be lowered they would simply
undo the revocation. Silently keeping the higher value means the worst an
attacker can do is raise it further — locking themselves out alongside the
owner.

Ignoring rather than erroring also makes the call idempotent, so a retried
revocation transaction is harmless.

### Verifying

`require_nonce_valid(env, account, nonce, error)` at every off-chain signature
verification site, *before* acting on the payload. The signature still has to
verify on its own; this only adds the check that the signer has not since
disowned that range.

`require_nonce_valid_or_panic` exists for contracts whose `#[contracterror]`
enum is full, the same reason `require_not_paused_or_panic` does.

Exposed on `InheritanceContract` as `revoke_user_nonces`, `get_min_nonce` and
`is_nonce_valid` so the capability is reachable as a contract call on testnet.

## Admin time-lock (#1159)

`contracts/access-control/src/lib.rs`.

Fee rates, the treasury address and contract WASM could all be changed
instantly. `propose_parameter_change` now records the intent and
`execute_parameter_change` refuses until `PARAMETER_TIMELOCK_SECONDS` (172,800
— 48 hours) has elapsed.

The point is not that 48 hours stops an attacker who already holds the admin
key. It is that the proposal is on-chain and public for two days, so users and
guardians have a window to notice a hostile fee change or treasury redirect and
exit before it lands. The proposal is published as an event for exactly that
reason — a time-lock nobody can watch provides no notice.

### Grace period

A matured proposal stays executable for `PARAMETER_GRACE_SECONDS` (7 days) and
then goes stale. Without an upper bound, a proposal made and forgotten a year
ago could be executed at any moment by anyone watching the chain — the notice
window expired long ago, so acting on it defeats the mechanism. A stale
proposal has to be re-proposed, with a fresh notice period. Expired proposals
are cleared on the failed execute, so a parameter is never permanently blocked
by a dead entry.

### One proposal per parameter

A second proposal for the same parameter replaces the first **and restarts its
clock**. Otherwise an admin could propose something innocuous, wait 48 hours,
swap in a hostile value, and execute immediately with no notice at all.

`cancel_parameter_change` withdraws a pending proposal — an admin who proposes
by mistake, or is talked out of it during the two days, needs a way back that
does not involve waiting for it to go stale.

### Adoption without migration

`get_parameter_value_or(env, id, default)` returns the compiled-in default
until the first change is executed through the time-lock, so a contract can
adopt this without a data migration.

Authorization is the caller's responsibility — `access-control` is a library
shared across contracts and does not know which role each treats as
privileged. Pair with `require_role`. The `InheritanceContract` wrappers do
this via `require_admin`.

## Beneficiary identity verification (#1169)

`contracts/inheritance-contract/src/lib.rs`.

Claim checks verified address authorization, which proves a key was used. It
says nothing about whether the holder of that key is the person a probate
document names.

`InheritancePlan` gains `beneficiary_identity_hash: BytesN<32>` — the SHA-256
of the legal document establishing who may inherit. When set, a claimant must
present the preimage, and the contract checks the hash before any funds move.

### Per-beneficiary override

A plan with several heirs has a separate document for each, so
`DataKey::Bih(plan_id, beneficiary_index)` holds a per-beneficiary hash that
takes precedence over the plan-wide one. A single plan-wide hash would force
all heirs to share one document, which means any one of them could claim as any
other — so the plan-level field the issue specifies is kept as the default and
the override is what makes it safe for multi-beneficiary plans.

### Opt-in

An all-zero hash means no proof required, which is what every plan created
before this field existed decodes to. The guard is inert for those plans rather
than silently freezing beneficiaries who have no document to present.

### Entry points

`claim_inheritance_plan_with_identity` takes the preimage.
`claim_inheritance_plan` keeps its existing signature and passes an empty
preimage — which plans with a hash set will reject, and plans without one
ignore. Adding an argument to the existing function would break every test, the
backend, and any deployed client, for an argument most plans do not use.

An empty preimage is rejected before hashing, so a caller who simply omitted
the argument is told the data is invalid rather than that their document did
not match.

## Multi-asset basket payouts (#1172)

`contracts/inheritance-contract/src/lib.rs`.

A plan holding XLM, USDC and EURC forced the beneficiary through one claim per
token: three signatures, three fees, and three chances to be interrupted
halfway and left holding part of an estate.

`claim_all_assets_payout(env, plan_id, claimer, email, claim_code, tokens, identity_preimage)`
settles the whole basket in one transaction. Either every transfer lands or the
call reverts and none do — the property that matters when the alternative is a
partially distributed inheritance.

### Share calculation

Each token is split by the beneficiary's `allocation_bp` against that token's
own balance. Basis points are applied per token rather than by converting to a
common unit: an exchange rate read at claim time would make the split depend on
*when* the beneficiary happened to claim, so two heirs with identical
allocations would receive materially different value.

Balances are widened to `u128` before multiplying — `balance * 10_000`
overflows `u64` above roughly 1.8e15, well inside the range of a 7-decimal
Stellar asset.

Division truncates. A token whose balance is smaller than the allocation
denominator yields zero for that token, and the remainder stays with the plan
rather than being rounded up out of another beneficiary's share.

### Setting up a basket

`set_basket_balance(plan_id, owner, token, amount)` registers a token and the
amount held for it. `DataKey::Bt(plan_id)` lists the basket;
`DataKey::Bb(plan_id, token)` holds each balance. Only registered tokens are
payable — otherwise an arbitrary address could be passed in and the contract
would try to move funds it never accounted for.

`MAX_BASKET_TOKENS` is 8, bounding the transfer loop so a claim cannot be
pushed past the ledger's resource limits by an unbounded token list, which
would strand the beneficiary's funds permanently.

### Partial retries

`DataKey::Bc(plan_id, beneficiary_index, token)` records per-token settlement.
A token already settled is skipped rather than failing the call, so a caller
passing the full token list after a partial retry completes the rest.

### Relationship to the single-asset claim

This settles the basket, not the plan's primary `token` balance. A beneficiary
with both claims the primary asset through `claim_inheritance_plan` and the
basket through this call. The `is_claimed` flag is left to the primary claim so
the two cannot deadlock each other.

Beneficiary identification is shared: `find_beneficiary` is extracted so the
two claim paths cannot drift into identifying beneficiaries differently — a
divergence there would be a way to claim as somebody else. Blacklist, KYC,
whitelist, freeze, legal hold, vesting, waterfall ordering and the identity
guard all apply identically.

# Genetic proof upload

Asset owners can open **Upload genetic proof** on their own plan cards. Select or
drop one non-empty `.vcf`, `.23andme`, or `.fasta` file, up to 26,214,400 bytes
(25 MiB). Extensions are case-insensitive; this check does not validate genetic
file contents or establish a biological relationship.

The browser hashes the file with SHA-256, asks the connected Testnet wallet to
sign a message, and submits the proof through
`GeneticVerificationAPI.uploadGeneticData(planId, file, options)`. Progress shows
the actual preparation, wallet approval, and request stages rather than a
simulated byte percentage. Wallets without message-signing support fail without
an unsigned fallback. Closing the modal cancels the request and prevents a late
wallet approval from starting a submission. A request already received by the
server cannot be rolled back by browser cancellation.

Raw genetic bytes and the filename never leave the browser in this flow. The
file and signed payload are not persisted in local storage or logged. A genetic
hash is still sensitive and must be protected by the receiving service. Signing
does not submit a ledger transaction or charge a transaction fee.

## Backend integration still required

This repository does **not** currently implement `/api/genetic/:planId/upload`.
The frontend therefore reports an unavailable service for HTTP 404/501 and never
pretends that a local hash is an accepted upload. Real testnet verification needs
a deployed backend implementing the following proposed contract. This frontend
work does not implement DNA matching or on-chain verification.

POST to `${NEXT_PUBLIC_API_URL}/api/genetic/:planId/upload` (default API origin:
`http://localhost:8080`) with JSON:

```json
{
  "dna_hash": "<SHA-256 hex>",
  "payload": "<exact JSON string signed by the wallet>",
  "signature": "<wallet message signature>"
}
```

When the API uses a different origin from the frontend, include that origin in
the existing `CSP_CONNECT_SRC_EXTRA` deployment setting as well as configuring
backend CORS. For local development this is `http://localhost:8080`.

The payload contains `purpose: "inheritx:genetic-upload:v1"`, `plan_id`,
`dna_hash`, `wallet_address`, `network_passphrase`, a random UUID `nonce`, and
`issued_at` in Unix seconds. The signature comes from Stellar Wallets Kit
`signMessage`; the backend must verify the wallet message-signing convention
(including its message prefix/encoding), not treat it as a transaction signature.
It must verify the exact payload string, compare its plan/hash with the request,
require Testnet, authorize the signer as the plan owner, enforce freshness and
nonce replay protection, and make repeated submissions of an already accepted
plan/hash idempotent. CORS and appropriate data access/retention controls are
also backend responsibilities. Do not enable acceptance by trusting the submitted
wallet address or client-side owner check alone.

Expected response:

```json
{ "data": { "dnaHash": "<same SHA-256 hex>", "status": "pending" } }
```

Other existing statuses are `verified`, `rejected`, `partial_match`, and
`requires_retest`. The modal confirms submission only; it does not equate an
HTTP success with a genetic match. Wrong hashes, missing data, unknown statuses,
HTTP failures, and network errors are not presented as success. Requests time
out after 30 seconds and are not automatically retried. Users can retry errors;
the backend must reconcile an earlier request if its response was lost.

## Verification

From `frontend`:

```sh
npm test -- tests/api/geneticUpload.test.ts tests/components/GeneticUploadModal.test.tsx tests/components/PlansGeneticUpload.test.tsx
```

These tests use synthetic files and mocked wallet/API responses. Live backend
acceptance and real wallet signing must be checked before deploying the service.

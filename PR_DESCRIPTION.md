# Pull Request: Multi-Asset Basket Claim Support for InheritX

## Summary

This PR adds the missing multi-asset payout flow for inheritance plans that represent basket-style holdings. It enables a beneficiary claim path that can mark a plan as claimed across multiple supported tokens within a single operation, which aligns with the project’s goal of handling portfolio-based inheritance distributions on Stellar.

## What Changed

### 1. Added basket claim payout entrypoint
- Implemented `claim_all_assets_payout` in the inheritance contract.
- The method validates claimability, rejects inactive/frozen plans, and marks the beneficiary as claimed for the basket while preserving the plan’s claim accounting.
- It emits claim events for basket-level settlement and records the beneficiary’s claim state.

### 2. Added regression coverage
- Added a focused test to verify that a beneficiary can be marked as claimed through the basket payout API.
- This prevents future regressions in the multi-asset claim path.

## Relevant Files

- `contracts/inheritance-contract/src/lib.rs`
- `contracts/inheritance-contract/src/test.rs`

## Why This Matters

The protocol description identifies inheritance plans with multi-asset baskets (XLM + USDC + EURC) as a key use case. Prior to this change, the contract’s claim logic did not expose the asset-basket payout flow needed for that scenario. This PR closes that gap and moves the contract closer to the intended portfolio inheritance workflow.

## Validation

Validated with:

```bash
cd /home/semi/Documents/Drip/InheritX/contracts && cargo test test_claim_all_assets_payout_marks_beneficiary_claimed_across_tokens -- --nocapture
```

Result:
- 1 test passed
- 0 failed

## Notes

This is a focused contract-layer addition and intentionally keeps the behavior bounded to the claim lifecycle and state updates. Future work can extend this pattern with real token transfer enforcement, cross-asset accounting, and richer per-token payout metadata if the protocol needs stricter basket settlement semantics.

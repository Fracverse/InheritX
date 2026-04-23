# Soroban Project

## Loan NFT compliance

The `loan-nft` contract implements the marketplace-facing loan position NFT surface used by the lending flow:

- ownership queries via `owner_of`, `balance_of`, `total_supply`, `get_metadata`, and `token_uri`
- transfers via `transfer` and `transfer_from`
- approvals via `approve`, `get_approved`, `set_approval_for_all`, and `is_approved_for_all`
- standard NFT lifecycle events: `Transfer`, `Approval`, and `ApprovalForAll`
- transfer restrictions for active loans through the per-loan `Transferable` flag
- reentrancy protection around mint, burn, and transfer execution paths

Loan position NFTs are intentionally non-transferable while the underlying loan is marked active. The lending flow should set `Transferable(false)` when a position must remain locked and re-enable transfers only when the position can be safely sold or reassigned.

## Project Structure

This repository uses the recommended structure for a Soroban project:
```text
.
├── contracts
│   └── hello_world
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `hello_world` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.

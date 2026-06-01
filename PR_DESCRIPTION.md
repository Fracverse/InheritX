# feat(#603): Implement proposal execution in governance contract

## Problem
The governance contract has a `ProposalExecutedEvent` but the `execute_proposal` function is incomplete. Proposals cannot be executed.

## Solution
- Extended `Proposal` struct with `target`, `function`, and `args` fields
- Updated `create_proposal` to accept action parameters
- Implemented actual contract invocation in `execute_proposal`
- Updated helper functions and tests

## Changes
- `contracts/governance-contract/src/lib.rs`: Added action payload to proposals and implemented execution logic
- `contracts/governance-contract/src/test.rs`: Updated test helpers for new proposal structure

## Result
Proposals now execute their intended actions on target contracts after passing voting.

Resolves #603

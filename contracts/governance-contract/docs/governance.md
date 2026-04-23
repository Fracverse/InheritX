# Governance Contract — Proposal & Voting Flow

## Lifecycle
Created (Active) → [voting period] → Passed / Rejected → Executed
                                     ↘ Cancelled (by proposer, while Active)

## Creating a Proposal
- Requires MIN_PROPOSAL_THRESHOLD token balance (1,000,000 tokens)
- Max MAX_PROPOSALS_PER_PROPOSER (5) active proposals per address
- Default voting period: 7 days (configurable within [1 day, 30 days])
- Title must be non-empty and max 100 characters
- Description must be non-empty and max 1000 characters

## Voting
- One vote per address per proposal — no changes after casting
- Voting power = token balance at time of vote (currently mocked as 1,000,000 per address)
- Choices: Yes, No, Abstain (all count toward quorum)
- Cannot vote on expired or cancelled proposals

## Quorum & Approval
- Quorum: 10% of total token supply must participate (QUORUM_THRESHOLD_BPS = 1000)
- Approval: > 50% of participating votes must be Yes (APPROVAL_THRESHOLD_BPS = 5000)
- Both conditions must be met for a proposal to Pass
- If quorum is not met OR majority is No, proposal is Rejected

## Execution
- Permissionless — any address may execute a Passed proposal
- Proposals auto-finalize on first read after expiry (via get_proposal, get_proposal_status, or execute_proposal)
- Once executed, proposals cannot be executed again

## Constants
- MIN_VOTING_PERIOD: 86,400 seconds (1 day)
- MAX_VOTING_PERIOD: 2,592,000 seconds (30 days)
- DEFAULT_VOTING_PERIOD: 604,800 seconds (7 days)
- QUORUM_THRESHOLD_BPS: 1,000 (10% in basis points)
- APPROVAL_THRESHOLD_BPS: 5,000 (50% in basis points)
- MIN_PROPOSAL_THRESHOLD: 1,000,000 tokens
- MAX_PROPOSALS_PER_PROPOSER: 5

## Backend API Mapping
| API Endpoint | Contract Function |
|---|---|
| POST /api/governance/proposals | create_proposal() |
| POST /api/governance/proposals/:id/vote | vote() |
| POST /api/governance/proposals/:id/execute | execute_proposal() |
| GET /api/governance/proposals/:id | get_proposal() |

## Events
- **proposal_created**: Emitted when a new proposal is created
- **vote_cast**: Emitted when a vote is cast
- **proposal_executed**: Emitted when a proposal is executed
- **proposal_cancelled**: Emitted when a proposal is cancelled
- **proposal_finalized**: Emitted when a proposal's status is finalized (Active → Passed/Rejected)

## Error Handling
The contract uses the following error variants:
- ProposalNotFound: Proposal does not exist
- ProposalNotActive: Action requires Active status
- ProposalNotPassed: Execute requires Passed status
- ProposalStillActive: Cannot execute while voting is open
- ProposalAlreadyExecuted: Proposal already executed
- ProposalExpired: Voting period has ended
- ProposalRejected: Proposal was rejected
- ProposalCancelled: Proposal was cancelled
- AlreadyVoted: Voter already cast a vote on this proposal
- InsufficientVotingPower: Balance too low to create proposal or vote
- InvalidProposalTitle: Empty or > 100 chars
- InvalidProposalDescription: Empty or > 1000 chars
- InvalidVotingPeriod: Outside [MIN, MAX] range
- TooManyActiveProposals: Proposer at MAX_PROPOSALS_PER_PROPOSER
- QuorumNotReached: Informational — for status display

## TODO for Integration
- Replace `get_voting_power()` with actual token contract call to `MockToken::balance()`
- Replace `get_total_supply()` with actual token contract call to token's total_supply
- Add proposal payload/action field to Proposal struct to encode what to execute
- Wire in specific actions in `execute_proposal()` based on proposal payload

#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

mod test;

// Constants
const MIN_VOTING_PERIOD: u64 = 86_400; // 1 day minimum
const MAX_VOTING_PERIOD: u64 = 2_592_000; // 30 days maximum
const DEFAULT_VOTING_PERIOD: u64 = 604_800; // 7 days default
const QUORUM_THRESHOLD_BPS: u32 = 1_000; // 10% of total supply (basis points)
const APPROVAL_THRESHOLD_BPS: u32 = 5_000; // 50%+ yes votes to pass
const MIN_PROPOSAL_THRESHOLD: u128 = 1_000_000; // Minimum tokens to create proposal
const MAX_PROPOSALS_PER_PROPOSER: u32 = 5; // max active proposals per address

#[contracttype]
pub enum DataKey {
    Admin,
    InterestRate,
    CollateralRatio,
    LiquidationBonus,
    ProposalCounter,
    Proposal(u64),
    Vote(u64, Address),
    VoterProposals(Address),
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GovernanceError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ProposalNotFound = 4,
    ProposalNotActive = 5,
    ProposalNotPassed = 6,
    ProposalStillActive = 7,
    ProposalAlreadyExecuted = 8,
    ProposalExpired = 9,
    ProposalRejected = 10,
    ProposalCancelled = 11,
    AlreadyVoted = 12,
    InsufficientVotingPower = 13,
    InvalidProposalTitle = 14,
    InvalidProposalDescription = 15,
    InvalidVotingPeriod = 16,
    TooManyActiveProposals = 17,
    QuorumNotReached = 18,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub proposer: Address,
    pub yes_votes: u128,
    pub no_votes: u128,
    pub abstain_votes: u128,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub executed_at: Option<u64>,
    pub quorum_reached: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: Address,
    pub choice: VoteChoice,
    pub voting_power: u128,
    pub voted_at: u64,
}

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        interest_rate: u32,
        collateral_ratio: u32,
        liquidation_bonus: u32,
    ) -> Result<(), GovernanceError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(GovernanceError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::InterestRate, &interest_rate);
        env.storage()
            .instance()
            .set(&DataKey::CollateralRatio, &collateral_ratio);
        env.storage()
            .instance()
            .set(&DataKey::LiquidationBonus, &liquidation_bonus);
        Ok(())
    }

    pub fn update_interest_rate(env: Env, new_rate: u32) -> Result<(), GovernanceError> {
        Self::check_admin(&env)?;
        env.storage()
            .instance()
            .set(&DataKey::InterestRate, &new_rate);
        Ok(())
    }

    pub fn update_collateral_ratio(env: Env, new_ratio: u32) -> Result<(), GovernanceError> {
        Self::check_admin(&env)?;
        env.storage()
            .instance()
            .set(&DataKey::CollateralRatio, &new_ratio);
        Ok(())
    }

    pub fn update_liquidation_bonus(env: Env, new_bonus: u32) -> Result<(), GovernanceError> {
        Self::check_admin(&env)?;
        env.storage()
            .instance()
            .set(&DataKey::LiquidationBonus, &new_bonus);
        Ok(())
    }

    pub fn get_interest_rate(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::InterestRate)
            .unwrap_or(0)
    }

    pub fn get_collateral_ratio(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CollateralRatio)
            .unwrap_or(0)
    }

    pub fn get_liquidation_bonus(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::LiquidationBonus)
            .unwrap_or(0)
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized")
    }

    fn check_admin(env: &Env) -> Result<(), GovernanceError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(GovernanceError::NotInitialized)?;
        admin.require_auth();
        Ok(())
    }

    // Proposal and Voting Functions

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        title: String,
        description: String,
        voting_period: Option<u64>,
    ) -> Result<u64, GovernanceError> {
        proposer.require_auth();

        // Check proposer's voting power (using mock token balance as proxy)
        let voting_power = Self::get_voting_power(&env, &proposer);
        if voting_power < MIN_PROPOSAL_THRESHOLD {
            return Err(GovernanceError::InsufficientVotingPower);
        }

        // Validate title
        let title_len = title.len();
        if title_len == 0 || title_len > 100 {
            return Err(GovernanceError::InvalidProposalTitle);
        }

        // Validate description
        let desc_len = description.len();
        if desc_len == 0 || desc_len > 1000 {
            return Err(GovernanceError::InvalidProposalDescription);
        }

        // Validate and set voting period
        let period = voting_period.unwrap_or(DEFAULT_VOTING_PERIOD);
        if period < MIN_VOTING_PERIOD || period > MAX_VOTING_PERIOD {
            return Err(GovernanceError::InvalidVotingPeriod);
        }

        // Check active proposal count for proposer
        let voter_proposals: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::VoterProposals(proposer.clone()))
            .unwrap_or(Vec::new(&env));

        let active_count = voter_proposals.iter().filter(|&&id| {
            if let Some(proposal) = env.storage().instance().get(&DataKey::Proposal(id)) {
                proposal.status == ProposalStatus::Active
            } else {
                false
            }
        }).count();

        if active_count >= MAX_PROPOSALS_PER_PROPOSER as usize {
            return Err(GovernanceError::TooManyActiveProposals);
        }

        // Increment proposal counter
        let proposal_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCounter)
            .unwrap_or(0) + 1;
        env.storage()
            .instance()
            .set(&DataKey::ProposalCounter, &proposal_id);

        // Create proposal
        let created_at = env.ledger().timestamp();
        let expires_at = created_at + period;

        let proposal = Proposal {
            id: proposal_id,
            title: title.clone(),
            description,
            proposer: proposer.clone(),
            yes_votes: 0,
            no_votes: 0,
            abstain_votes: 0,
            status: ProposalStatus::Active,
            created_at,
            expires_at,
            executed_at: None,
            quorum_reached: false,
        };

        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        // Add to voter's proposals
        let mut updated_proposals = voter_proposals;
        updated_proposals.push_back(proposal_id);
        env.storage()
            .instance()
            .set(&DataKey::VoterProposals(proposer), &updated_proposals);

        // Emit event
        env.events()
            .publish(("proposal_created", proposal_id, proposer.clone()), (title, expires_at, period));

        Ok(proposal_id)
    }

    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: u64,
        choice: VoteChoice,
    ) -> Result<(), GovernanceError> {
        voter.require_auth();

        // Get proposal
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        // Check proposal is active
        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::ProposalNotActive);
        }

        // Check not expired
        let current_time = env.ledger().timestamp();
        if current_time >= proposal.expires_at {
            return Err(GovernanceError::ProposalExpired);
        }

        // Check not already voted
        if env.storage().instance().has(&DataKey::Vote(proposal_id, voter.clone())) {
            return Err(GovernanceError::AlreadyVoted);
        }

        // Get voting power
        let voting_power = Self::get_voting_power(&env, &voter);
        if voting_power == 0 {
            return Err(GovernanceError::InsufficientVotingPower);
        }

        // Record vote
        let vote = Vote {
            proposal_id,
            voter: voter.clone(),
            choice,
            voting_power,
            voted_at: current_time,
        };
        env.storage()
            .instance()
            .set(&DataKey::Vote(proposal_id, voter.clone()), &vote);

        // Update vote counts
        match choice {
            VoteChoice::Yes => proposal.yes_votes += voting_power,
            VoteChoice::No => proposal.no_votes += voting_power,
            VoteChoice::Abstain => proposal.abstain_votes += voting_power,
        }

        // Recalculate quorum
        let total_supply = Self::get_total_supply(&env);
        let total_voted = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;
        if total_supply > 0 {
            let quorum_met = (total_voted * 10_000 / total_supply) >= QUORUM_THRESHOLD_BPS as u128;
            proposal.quorum_reached = quorum_met;
        }

        // Store updated proposal
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        // Emit event
        env.events().publish(
            ("vote_cast", proposal_id, voter.clone()),
            (choice, voting_power, proposal.yes_votes, proposal.no_votes, proposal.abstain_votes),
        );

        Ok(())
    }

    pub fn execute_proposal(
        env: Env,
        executor: Address,
        proposal_id: u64,
    ) -> Result<(), GovernanceError> {
        executor.require_auth();

        // Get proposal
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        // Finalize if expired and still active
        if proposal.status == ProposalStatus::Active {
            let current_time = env.ledger().timestamp();
            if current_time >= proposal.expires_at {
                Self::finalize_proposal_status(&env, &mut proposal);
            }
        }

        // Check status
        match proposal.status {
            ProposalStatus::Passed => {},
            ProposalStatus::Active => return Err(GovernanceError::ProposalStillActive),
            ProposalStatus::Rejected => return Err(GovernanceError::ProposalRejected),
            ProposalStatus::Executed => return Err(GovernanceError::ProposalAlreadyExecuted),
            ProposalStatus::Cancelled => return Err(GovernanceError::ProposalCancelled),
        }

        // Execute proposal (no-op for now - TODO: wire in specific actions)
        // In a full implementation, this would execute the proposal's payload

        // Update status
        proposal.status = ProposalStatus::Executed;
        let executed_at = env.ledger().timestamp();
        proposal.executed_at = Some(executed_at);

        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        // Emit event
        env.events().publish(
            ("proposal_executed", proposal_id, executor),
            (executed_at, proposal.yes_votes, proposal.no_votes),
        );

        Ok(())
    }

    pub fn cancel_proposal(
        env: Env,
        canceller: Address,
        proposal_id: u64,
    ) -> Result<(), GovernanceError> {
        canceller.require_auth();

        // Get proposal
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        // Check canceller is proposer
        if proposal.proposer != canceller {
            return Err(GovernanceError::Unauthorized);
        }

        // Check status is active
        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::ProposalNotActive);
        }

        // Cancel proposal
        proposal.status = ProposalStatus::Cancelled;

        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        // Emit event
        let cancelled_at = env.ledger().timestamp();
        env.events()
            .publish(("proposal_cancelled", proposal_id, canceller), cancelled_at);

        Ok(())
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, GovernanceError> {
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        // Auto-finalize if expired and still active
        if proposal.status == ProposalStatus::Active {
            let current_time = env.ledger().timestamp();
            if current_time >= proposal.expires_at {
                Self::finalize_proposal_status(&env, &mut proposal);
            }
        }

        Ok(proposal)
    }

    pub fn get_proposal_status(
        env: Env,
        proposal_id: u64,
    ) -> Result<ProposalStatus, GovernanceError> {
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        // Auto-finalize if expired and still active
        if proposal.status == ProposalStatus::Active {
            let current_time = env.ledger().timestamp();
            if current_time >= proposal.expires_at {
                Self::finalize_proposal_status(&env, &mut proposal);
            }
        }

        Ok(proposal.status)
    }

    pub fn get_vote_count(
        env: Env,
        proposal_id: u64,
    ) -> Result<(u128, u128, u128, u128), GovernanceError> {
        let proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        let total = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;
        Ok((proposal.yes_votes, proposal.no_votes, proposal.abstain_votes, total))
    }

    pub fn get_user_vote(
        env: Env,
        proposal_id: u64,
        voter: Address,
    ) -> Result<Option<Vote>, GovernanceError> {
        // Check proposal exists
        if !env.storage().instance().has(&DataKey::Proposal(proposal_id)) {
            return Err(GovernanceError::ProposalNotFound);
        }

        let vote = env
            .storage()
            .instance()
            .get(&DataKey::Vote(proposal_id, voter));

        Ok(vote)
    }

    // Internal helper functions

    fn finalize_proposal_status(env: &Env, proposal: &mut Proposal) {
        let total_supply = Self::get_total_supply(env);
        let total_voted = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;

        let quorum_met = if total_supply > 0 {
            (total_voted * 10_000 / total_supply) >= QUORUM_THRESHOLD_BPS as u128
        } else {
            false
        };

        proposal.quorum_reached = quorum_met;

        if quorum_met && total_voted > 0 {
            let approval_met = (proposal.yes_votes * 10_000 / total_voted) >= APPROVAL_THRESHOLD_BPS as u128;
            if approval_met {
                proposal.status = ProposalStatus::Passed;
            } else {
                proposal.status = ProposalStatus::Rejected;
            }
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        // Store updated proposal
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal.id), proposal);

        // Emit event
        env.events().publish(
            ("proposal_finalized", proposal.id),
            (
                proposal.status.clone(),
                proposal.yes_votes,
                proposal.no_votes,
                proposal.abstain_votes,
                proposal.quorum_reached,
            ),
        );
    }

    fn get_voting_power(env: &Env, address: &Address) -> u128 {
        // In a real implementation, this would call the token contract
        // For now, we use a simple storage-based balance as a proxy
        // TODO: Replace with actual token contract call
        let key = DataKey::VoterProposals(address.clone());
        // This is a placeholder - in production, call token.balance(address)
        // For testing purposes, we'll use a mock approach
        1_000_000 // Default voting power for testing
    }

    fn get_total_supply(env: &Env) -> u128 {
        // In a real implementation, this would call the token contract's total_supply
        // For now, return a fixed value for testing
        // TODO: Replace with actual token contract call
        10_000_000 // Default total supply for testing
    }
}

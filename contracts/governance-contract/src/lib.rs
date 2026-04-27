#![no_std]
use access_control::{self, Role};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, IntoVal, String, Symbol, Vec,
};

mod test;

// ─────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────

const PROPOSAL_DURATION: u64 = 604_800; // 7 days in seconds
const QUORUM_THRESHOLD: i128 = 1; // Minimum total votes required to consider a proposal valid

// ─────────────────────────────────────────────────
// Storage Keys
// ─────────────────────────────────────────────────

#[contracttype]
pub enum DataKey {
    Admin,
    InterestRate,
    CollateralRatio,
    LiquidationBonus,
    Delegation(Address),
    Delegators(Address),
    DelegationHistory,
    TokenBalance(Address),
    // Legacy vote storage (kept for enum stability)
    Vote(Address, u32),
    ProposalVotes(u32),
    ControlledContracts,
    // Multi-sig
    MultiSigConfig,
    PendingTransaction(u32),
    NextTxId,
    NextProposalId,
    Proposal(u32),
    UserVoteChoice(Address, u32),
}

// ─────────────────────────────────────────────────
// Proposal Types
// ─────────────────────────────────────────────────

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
    pub id: u32,
    pub title: String,
    pub description: String,
    pub proposer: Address,
    pub yes_votes: i128,
    pub no_votes: i128,
    pub abstain_votes: i128,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteCount {
    pub yes_votes: i128,
    pub no_votes: i128,
    pub abstain_votes: i128,
}

// ─────────────────────────────────────────────────
// Delegation Types
// ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractLinkedEvent {
    pub contract: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractExecutedEvent {
    pub contract: Address,
    pub func: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSig {
    pub signers: Vec<Address>,
    pub threshold: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingTransaction {
    pub id: u32,
    pub proposer: Address,
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<soroban_sdk::Val>,
    pub signatures: Vec<Address>,
    pub created_at: u64,
    pub executed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationRecord {
    pub delegator: Address,
    pub delegate: Address,
    pub timestamp: u64,
    pub action: DelegationAction,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DelegationAction {
    Delegated,
    Undelegated,
    Redelegated,
}

// ─────────────────────────────────────────────────
// Events
// ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCreatedEvent {
    pub id: u32,
    pub proposer: Address,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteCastEvent {
    pub proposal_id: u32,
    pub voter: Address,
    pub choice: VoteChoice,
    pub voting_power: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalExecutedEvent {
    pub proposal_id: u32,
    pub executor: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCancelledEvent {
    pub proposal_id: u32,
    pub proposer: Address,
}

// ─────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GovernanceError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    SelfDelegation = 4,
    CircularDelegation = 5,
    NoDelegation = 6,
    AlreadyDelegated = 7,
    ZeroAmount = 8,
    AlreadyVoted = 9,
    ProposalNotFound = 10,
    ProposalExpired = 11,
    ProposalNotActive = 12,
    QuorumNotMet = 13,
    ProposalNotPassed = 14,
    ProposalAlreadyExecuted = 15,
    ProposalAlreadyCancelled = 16,
    NotProposer = 17,
    ReentrantCall = 18,
    ContractPaused = 19,
}

// ─────────────────────────────────────────────────
// Contract
// ─────────────────────────────────────────────────

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    // ─── Admin / Init ───────────────────────────────

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
        access_control::assign_role(&env, &admin, Role::Admin);

        // Initialize multi-sig with single admin initially
        let mut signers = Vec::new(&env);
        signers.push_back(admin.clone());
        let multi_sig = MultiSig {
            signers,
            threshold: 1,
        };
        env.storage()
            .instance()
            .set(&DataKey::MultiSigConfig, &multi_sig);

        Ok(())
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized")
    }

    /// Assign a role to an address. Admin-only.
    pub fn assign_role(
        env: Env,
        admin: Address,
        address: Address,
        role: Role,
    ) -> Result<(), GovernanceError> {
        admin.require_auth();
        access_control::require_role(&env, &admin, Role::Admin, GovernanceError::Unauthorized)?;
        access_control::assign_role(&env, &address, role);
        Ok(())
    }

    /// Revoke a role from an address. Admin-only.
    pub fn revoke_role(
        env: Env,
        admin: Address,
        address: Address,
        role: Role,
    ) -> Result<(), GovernanceError> {
        admin.require_auth();
        access_control::require_role(&env, &admin, Role::Admin, GovernanceError::Unauthorized)?;
        access_control::revoke_role(&env, &address, role);
        Ok(())
    }

    /// Check whether an address holds a given role.
    pub fn has_role(env: Env, address: Address, role: Role) -> bool {
        access_control::has_role(&env, &address, role)
    }

    /// Return all roles held by an address.
    pub fn get_roles(env: Env, address: Address) -> Vec<Role> {
        use access_control::AccessControlKey;
        env.storage()
            .persistent()
            .get(&AccessControlKey::Roles(address))
            .unwrap_or(Vec::new(&env))
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), GovernanceError> {
        admin.require_auth();
        access_control::require_role(&env, &admin, Role::Admin, GovernanceError::Unauthorized)?;
        access_control::pause_contract(&env);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), GovernanceError> {
        admin.require_auth();
        access_control::require_role(&env, &admin, Role::Admin, GovernanceError::Unauthorized)?;
        access_control::unpause_contract(&env);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        access_control::is_contract_paused(&env)
    }

    fn require_not_paused(env: &Env) -> Result<(), GovernanceError> {
        access_control::require_not_paused(env, GovernanceError::ContractPaused)
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

    // Multi-sig versions of critical functions
    pub fn propose_update_interest_rate(
        env: Env,
        proposer: Address,
        new_rate: u32,
    ) -> Result<u32, GovernanceError> {
        let mut args = Vec::new(&env);
        args.push_back(new_rate.into_val(&env));
        Self::propose_transaction(
            env.clone(),
            proposer,
            env.current_contract_address(),
            Symbol::new(&env, "update_interest_rate"),
            args,
        )
    }

    pub fn propose_update_collateral_ratio(
        env: Env,
        proposer: Address,
        new_ratio: u32,
    ) -> Result<u32, GovernanceError> {
        let mut args = Vec::new(&env);
        args.push_back(new_ratio.into_val(&env));
        Self::propose_transaction(
            env.clone(),
            proposer,
            env.current_contract_address(),
            Symbol::new(&env, "update_collateral_ratio"),
            args,
        )
    }

    pub fn propose_update_liquidation_bonus(
        env: Env,
        proposer: Address,
        new_bonus: u32,
    ) -> Result<u32, GovernanceError> {
        let mut args = Vec::new(&env);
        args.push_back(new_bonus.into_val(&env));
        Self::propose_transaction(
            env.clone(),
            proposer,
            env.current_contract_address(),
            Symbol::new(&env, "update_liquidation_bonus"),
            args,
        )
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

    pub fn get_multi_sig_config(env: Env) -> MultiSig {
        env.storage()
            .instance()
            .get(&DataKey::MultiSigConfig)
            .expect("Multi-sig not initialized")
    }

    pub fn update_multi_sig_config(
        env: Env,
        admin: Address,
        signers: Vec<Address>,
        threshold: u32,
    ) -> Result<(), GovernanceError> {
        admin.require_auth();
        Self::check_admin(&env)?;

        if signers.is_empty() || threshold == 0 || threshold > signers.len() {
            return Err(GovernanceError::Unauthorized);
        }

        let multi_sig = MultiSig { signers, threshold };
        env.storage()
            .instance()
            .set(&DataKey::MultiSigConfig, &multi_sig);
        Ok(())
    }

    pub fn propose_transaction(
        env: Env,
        proposer: Address,
        target: Address,
        function: Symbol,
        args: Vec<soroban_sdk::Val>,
    ) -> Result<u32, GovernanceError> {
        proposer.require_auth();
        Self::require_not_paused(&env)?;

        let tx_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::NextTxId)
            .unwrap_or(1u32);

        let pending_tx = PendingTransaction {
            id: tx_id,
            proposer: proposer.clone(),
            target,
            function,
            args,
            signatures: Vec::new(&env),
            created_at: env.ledger().timestamp(),
            executed: false,
        };

        env.storage()
            .instance()
            .set(&DataKey::PendingTransaction(tx_id), &pending_tx);
        env.storage()
            .instance()
            .set(&DataKey::NextTxId, &(tx_id + 1));

        Ok(tx_id)
    }

    pub fn sign_transaction(env: Env, signer: Address, tx_id: u32) -> Result<(), GovernanceError> {
        signer.require_auth();
        Self::require_not_paused(&env)?;

        let multi_sig = Self::get_multi_sig_config(env.clone());
        if !multi_sig.signers.contains(&signer) {
            return Err(GovernanceError::Unauthorized);
        }

        let mut pending_tx: PendingTransaction = env
            .storage()
            .instance()
            .get(&DataKey::PendingTransaction(tx_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        if pending_tx.executed {
            return Err(GovernanceError::ProposalAlreadyExecuted);
        }

        if !pending_tx.signatures.contains(&signer) {
            pending_tx.signatures.push_back(signer);
            env.storage()
                .instance()
                .set(&DataKey::PendingTransaction(tx_id), &pending_tx);
        }

        Ok(())
    }

    pub fn execute_transaction(
        env: Env,
        executor: Address,
        tx_id: u32,
    ) -> Result<soroban_sdk::Val, GovernanceError> {
        executor.require_auth();
        access_control::reentrancy_enter(&env, GovernanceError::ReentrantCall)?;
        Self::require_not_paused(&env)?;

        let mut pending_tx: PendingTransaction = env
            .storage()
            .instance()
            .get(&DataKey::PendingTransaction(tx_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        if pending_tx.executed {
            return Err(GovernanceError::ProposalAlreadyExecuted);
        }

        let multi_sig = Self::get_multi_sig_config(env.clone());
        if pending_tx.signatures.len() < multi_sig.threshold {
            return Err(GovernanceError::QuorumNotMet);
        }

        pending_tx.executed = true;
        env.storage()
            .instance()
            .set(&DataKey::PendingTransaction(tx_id), &pending_tx);

        let result = env.invoke_contract(&pending_tx.target, &pending_tx.function, pending_tx.args);

        access_control::reentrancy_exit(&env);
        Ok(result)
    }

    pub fn get_pending_transaction(env: Env, tx_id: u32) -> Option<PendingTransaction> {
        env.storage()
            .instance()
            .get(&DataKey::PendingTransaction(tx_id))
    }

    fn check_admin(env: &Env) -> Result<(), GovernanceError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(GovernanceError::NotInitialized)?;
        admin.require_auth();
        access_control::require_role(env, &admin, Role::Admin, GovernanceError::Unauthorized)
    }

    // ─── Token Balance ───────────────────────────────

    pub fn set_token_balance(env: Env, address: Address, balance: i128) {
        env.storage()
            .instance()
            .set(&DataKey::TokenBalance(address), &balance);
    }

    pub fn get_token_balance(env: Env, address: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TokenBalance(address))
            .unwrap_or(0)
    }

    // ─── Delegation ──────────────────────────────────

    pub fn delegate_votes(
        env: Env,
        delegator: Address,
        delegate: Address,
    ) -> Result<(), GovernanceError> {
        delegator.require_auth();

        if delegator == delegate {
            return Err(GovernanceError::SelfDelegation);
        }

        if Self::check_circular_delegation(&env, &delegator, &delegate) {
            return Err(GovernanceError::CircularDelegation);
        }

        let existing_delegate = Self::get_delegate(env.clone(), delegator.clone());

        let history_key = DataKey::DelegationHistory;
        let mut history: Vec<DelegationRecord> = env
            .storage()
            .instance()
            .get(&history_key)
            .unwrap_or_else(|| Vec::new(&env));

        let timestamp = env.ledger().timestamp();

        if let Some(prev_delegate) = existing_delegate {
            Self::remove_from_delegators(&env, &prev_delegate, &delegator);
            history.push_back(DelegationRecord {
                delegator: delegator.clone(),
                delegate: delegate.clone(),
                timestamp,
                action: DelegationAction::Redelegated,
            });
        } else {
            history.push_back(DelegationRecord {
                delegator: delegator.clone(),
                delegate: delegate.clone(),
                timestamp,
                action: DelegationAction::Delegated,
            });
        }

        env.storage().instance().set(&history_key, &history);

        env.storage()
            .instance()
            .set(&DataKey::Delegation(delegator.clone()), &delegate);

        Self::add_to_delegators(&env, &delegate, &delegator);

        env.events()
            .publish(("VotesDelegated", delegator.clone(), delegate.clone()), ());

        Ok(())
    }

    pub fn undelegate_votes(env: Env, delegator: Address) -> Result<(), GovernanceError> {
        delegator.require_auth();

        let current_delegate = Self::get_delegate(env.clone(), delegator.clone());

        if current_delegate.is_none() {
            return Err(GovernanceError::NoDelegation);
        }

        let delegate = current_delegate.unwrap();

        Self::remove_from_delegators(&env, &delegate, &delegator);

        env.storage()
            .instance()
            .remove(&DataKey::Delegation(delegator.clone()));

        let history_key = DataKey::DelegationHistory;
        let mut history: Vec<DelegationRecord> = env
            .storage()
            .instance()
            .get(&history_key)
            .unwrap_or_else(|| Vec::new(&env));

        let timestamp = env.ledger().timestamp();
        history.push_back(DelegationRecord {
            delegator: delegator.clone(),
            delegate: delegator.clone(),
            timestamp,
            action: DelegationAction::Undelegated,
        });

        env.storage().instance().set(&history_key, &history);

        env.events().publish(("VotesUndelegated", delegator), ());

        Ok(())
    }

    pub fn get_delegate(env: Env, delegator: Address) -> Option<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Delegation(delegator))
    }

    pub fn get_delegators(env: Env, delegate: Address) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Delegators(delegate))
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_voting_power(env: Env, address: Address) -> i128 {
        // Delegated accounts have zero direct voting power
        if env
            .storage()
            .instance()
            .has(&DataKey::Delegation(address.clone()))
        {
            return 0;
        }

        let own_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TokenBalance(address.clone()))
            .unwrap_or(0);

        let delegators: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Delegators(address.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        let mut total_delegated: i128 = 0;
        for delegator_addr in delegators.iter() {
            let bal: i128 = env
                .storage()
                .instance()
                .get(&DataKey::TokenBalance(delegator_addr))
                .unwrap_or(0);
            total_delegated += bal;
        }

        own_balance + total_delegated
    }

    pub fn get_delegation_history(env: Env) -> Vec<DelegationRecord> {
        env.storage()
            .instance()
            .get(&DataKey::DelegationHistory)
            .unwrap_or_else(|| Vec::new(&env))
    }

    // ─── Proposal Governance ─────────────────────────

    /// Create a new governance proposal. Returns the new proposal ID.
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        title: String,
        description: String,
    ) -> Result<u32, GovernanceError> {
        proposer.require_auth();
        Self::require_not_paused(&env)?;

        let proposal_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::NextProposalId)
            .unwrap_or(1u32);

        let now = env.ledger().timestamp();
        let expires_at = now + PROPOSAL_DURATION;

        let proposal = Proposal {
            id: proposal_id,
            title,
            description,
            proposer: proposer.clone(),
            yes_votes: 0,
            no_votes: 0,
            abstain_votes: 0,
            status: ProposalStatus::Active,
            created_at: now,
            expires_at,
        };

        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::NextProposalId, &(proposal_id + 1));

        env.events().publish(
            (Symbol::new(&env, "PropCreate"), proposer.clone()),
            ProposalCreatedEvent {
                id: proposal_id,
                proposer,
                expires_at,
            },
        );

        Ok(proposal_id)
    }

    /// Vote on a proposal with yes, no, or abstain.
    /// Voting power is automatically derived from the voter's token balance plus any delegated balances.
    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: u32,
        choice: VoteChoice,
    ) -> Result<(), GovernanceError> {
        voter.require_auth();
        Self::require_not_paused(&env)?;

        // Delegated voters cannot vote directly
        if env
            .storage()
            .instance()
            .has(&DataKey::Delegation(voter.clone()))
        {
            return Err(GovernanceError::Unauthorized);
        }

        // Compute voting power inline
        let own_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TokenBalance(voter.clone()))
            .unwrap_or(0);
        let delegators: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Delegators(voter.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        let mut voting_power = own_balance;
        for delegator_addr in delegators.iter() {
            let bal: i128 = env
                .storage()
                .instance()
                .get(&DataKey::TokenBalance(delegator_addr))
                .unwrap_or(0);
            voting_power += bal;
        }

        if voting_power == 0 {
            return Err(GovernanceError::ZeroAmount);
        }

        // Validate proposal state
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        let current_time = env.ledger().timestamp();
        if current_time > proposal.expires_at {
            return Err(GovernanceError::ProposalExpired);
        }

        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::ProposalNotActive);
        }

        // Prevent double voting
        let vote_choice_key = DataKey::UserVoteChoice(voter.clone(), proposal_id);
        if env.storage().instance().has(&vote_choice_key) {
            return Err(GovernanceError::AlreadyVoted);
        }

        // Record vote and tally
        env.storage().instance().set(&vote_choice_key, &choice);

        if choice == VoteChoice::Yes {
            proposal.yes_votes += voting_power;
        } else if choice == VoteChoice::No {
            proposal.no_votes += voting_power;
        } else {
            proposal.abstain_votes += voting_power;
        }

        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (Symbol::new(&env, "VoteCast"), voter.clone()),
            VoteCastEvent {
                proposal_id,
                voter,
                choice,
                voting_power,
            },
        );

        Ok(())
    }

    /// Execute a passed proposal. Anyone can call this after the voting period ends.
    pub fn execute_proposal(
        env: Env,
        executor: Address,
        proposal_id: u32,
    ) -> Result<(), GovernanceError> {
        executor.require_auth();
        access_control::reentrancy_enter(&env, GovernanceError::ReentrantCall)?;
        Self::require_not_paused(&env)?;

        let status = Self::evaluate_proposal_status(&env, proposal_id)?;
        if status != ProposalStatus::Passed {
            return Err(GovernanceError::ProposalNotPassed);
        }

        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        proposal.status = ProposalStatus::Executed;
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (Symbol::new(&env, "PropExec"), executor.clone()),
            ProposalExecutedEvent {
                proposal_id,
                executor,
            },
        );

        access_control::reentrancy_exit(&env);
        Ok(())
    }

    /// Get full proposal details.
    pub fn get_proposal(env: Env, proposal_id: u32) -> Option<Proposal> {
        env.storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
    }

    /// Get the effective status of a proposal, accounting for voting period expiry.
    pub fn get_proposal_status(
        env: Env,
        proposal_id: u32,
    ) -> Result<ProposalStatus, GovernanceError> {
        Self::evaluate_proposal_status(&env, proposal_id)
    }

    /// Get the vote counts (yes, no, abstain) for a proposal.
    pub fn get_vote_count(env: Env, proposal_id: u32) -> Result<VoteCount, GovernanceError> {
        let proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;
        Ok(VoteCount {
            yes_votes: proposal.yes_votes,
            no_votes: proposal.no_votes,
            abstain_votes: proposal.abstain_votes,
        })
    }

    /// Get the vote choice of a specific voter on a proposal.
    pub fn get_user_vote(env: Env, voter: Address, proposal_id: u32) -> Option<VoteChoice> {
        env.storage()
            .instance()
            .get(&DataKey::UserVoteChoice(voter, proposal_id))
    }

    /// Cancel an active proposal. Only the original proposer can cancel.
    pub fn cancel_proposal(
        env: Env,
        caller: Address,
        proposal_id: u32,
    ) -> Result<(), GovernanceError> {
        caller.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.proposer != caller {
            return Err(GovernanceError::NotProposer);
        }

        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::ProposalNotActive);
        }

        proposal.status = ProposalStatus::Cancelled;
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (Symbol::new(&env, "PropCancel"), caller.clone()),
            ProposalCancelledEvent {
                proposal_id,
                proposer: caller,
            },
        );

        Ok(())
    }

    /// Returns yes_votes for a proposal (backward-compatible helper).
    pub fn get_proposal_votes(env: Env, proposal_id: u32) -> i128 {
        env.storage()
            .instance()
            .get::<DataKey, Proposal>(&DataKey::Proposal(proposal_id))
            .map(|p| p.yes_votes)
            .unwrap_or(0)
    }

    /// Returns true if the voter has already cast a vote on this proposal.
    pub fn has_voted(env: Env, voter: Address, proposal_id: u32) -> bool {
        env.storage()
            .instance()
            .has(&DataKey::UserVoteChoice(voter, proposal_id))
    }

    // ─── Internal Helpers ────────────────────────────

    fn evaluate_proposal_status(
        env: &Env,
        proposal_id: u32,
    ) -> Result<ProposalStatus, GovernanceError> {
        let proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Active {
            return Ok(proposal.status);
        }

        let now = env.ledger().timestamp();
        if now <= proposal.expires_at {
            return Ok(ProposalStatus::Active);
        }

        // Voting period ended — evaluate result against quorum and majority
        let total_votes = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;
        if total_votes >= QUORUM_THRESHOLD && proposal.yes_votes > proposal.no_votes {
            Ok(ProposalStatus::Passed)
        } else {
            Ok(ProposalStatus::Rejected)
        }
    }

    fn check_circular_delegation(
        env: &Env,
        delegator: &Address,
        proposed_delegate: &Address,
    ) -> bool {
        let mut current = proposed_delegate.clone();

        let mut visited: Vec<Address> = Vec::new(env);
        visited.push_back(delegator.clone());

        loop {
            if current == *delegator {
                return true;
            }

            if visited.contains(&current) {
                return true;
            }

            visited.push_back(current.clone());

            match Self::get_delegate(env.clone(), current.clone()) {
                Some(next_delegate) => {
                    current = next_delegate;
                }
                None => {
                    return false;
                }
            }
        }
    }

    fn add_to_delegators(env: &Env, delegate: &Address, delegator: &Address) {
        let key = DataKey::Delegators(delegate.clone());
        let mut delegators: Vec<Address> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));

        if !delegators.contains(delegator) {
            delegators.push_back(delegator.clone());
            env.storage().instance().set(&key, &delegators);
        }
    }

    fn remove_from_delegators(env: &Env, delegate: &Address, delegator: &Address) {
        let key = DataKey::Delegators(delegate.clone());
        let delegators: Vec<Address> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));

        let mut new_delegators: Vec<Address> = Vec::new(env);
        for d in delegators.iter() {
            if d != *delegator {
                new_delegators.push_back(d);
            }
        }

        if new_delegators.is_empty() {
            env.storage().instance().remove(&key);
        } else {
            env.storage().instance().set(&key, &new_delegators);
        }
    }

    // ─── Cross-Contract Integration ──────────────────

    pub fn add_controlled_contract(
        env: Env,
        _admin: Address,
        contract: Address,
    ) -> Result<(), GovernanceError> {
        Self::check_admin(&env)?;

        let mut contracts: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::ControlledContracts)
            .unwrap_or_else(|| Vec::new(&env));

        if !contracts.contains(&contract) {
            contracts.push_back(contract.clone());
            env.storage()
                .instance()
                .set(&DataKey::ControlledContracts, &contracts);

            env.events().publish(
                (Symbol::new(&env, "LINK"), Symbol::new(&env, "CTRL")),
                ContractLinkedEvent { contract },
            );
        }

        Ok(())
    }

    pub fn get_controlled_contracts(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::ControlledContracts)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn execute_on_contract(
        env: Env,
        _admin: Address,
        contract: Address,
        func: Symbol,
        args: Vec<soroban_sdk::Val>,
    ) -> Result<soroban_sdk::Val, GovernanceError> {
        Self::check_admin(&env)?;
        access_control::reentrancy_enter(&env, GovernanceError::ReentrantCall)?;
        Self::require_not_paused(&env)?;

        let contracts = Self::get_controlled_contracts(env.clone());
        if !contracts.contains(&contract) {
            return Err(GovernanceError::Unauthorized);
        }

        let result = env.invoke_contract(&contract, &func, args);

        env.events().publish(
            (Symbol::new(&env, "EXECUTE"), contract.clone()),
            ContractExecutedEvent { contract, func },
        );

        access_control::reentrancy_exit(&env);
        Ok(result)
    }

    pub fn upgrade_controlled_contract(
        env: Env,
        _admin: Address,
        contract: Address,
        new_wasm_hash: soroban_sdk::BytesN<32>,
    ) -> Result<(), GovernanceError> {
        Self::check_admin(&env)?;
        access_control::reentrancy_enter(&env, GovernanceError::ReentrantCall)?;
        Self::require_not_paused(&env)?;

        let contracts = Self::get_controlled_contracts(env.clone());
        if !contracts.contains(&contract) {
            return Err(GovernanceError::Unauthorized);
        }

        let mut args: Vec<soroban_sdk::Val> = Vec::new(&env);
        args.push_back(env.current_contract_address().into_val(&env));
        args.push_back(new_wasm_hash.into_val(&env));

        env.invoke_contract::<soroban_sdk::Val>(
            &contract,
            &Symbol::new(&env, "upgrade_contract"),
            args,
        );

        access_control::reentrancy_exit(&env);
        Ok(())
    }
}

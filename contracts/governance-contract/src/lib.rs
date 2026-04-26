#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, IntoVal, Symbol, Vec,
};

mod test;

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
    Vote(Address, u32),
    ProposalVotes(u32),
    ControlledContracts,
}

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
        // If this address has delegated away, its voting power is 0
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

        // Accumulate delegated balances in a single loop without cloning env
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

    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: u32,
        vote_weight: i128,
    ) -> Result<(), GovernanceError> {
        voter.require_auth();

        // Check delegation status with a single has() call (cheaper than get())
        if env
            .storage()
            .instance()
            .has(&DataKey::Delegation(voter.clone()))
        {
            return Err(GovernanceError::Unauthorized);
        }

        // Compute voting power inline to avoid redundant storage reads from get_voting_power
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

        if vote_weight > voting_power {
            return Err(GovernanceError::ZeroAmount);
        }

        let vote_key = DataKey::Vote(voter.clone(), proposal_id);
        if env.storage().instance().has(&vote_key) {
            return Err(GovernanceError::AlreadyVoted);
        }

        env.storage().instance().set(&vote_key, &vote_weight);

        let mut proposal_votes: i128 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalVotes(proposal_id))
            .unwrap_or(0);

        proposal_votes += vote_weight;

        env.storage()
            .instance()
            .set(&DataKey::ProposalVotes(proposal_id), &proposal_votes);

        Ok(())
    }

    pub fn get_proposal_votes(env: Env, proposal_id: u32) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::ProposalVotes(proposal_id))
            .unwrap_or(0)
    }

    pub fn has_voted(env: Env, voter: Address, proposal_id: u32) -> bool {
        env.storage()
            .instance()
            .has(&DataKey::Vote(voter, proposal_id))
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

        // Build filtered list, skipping the removed delegator
        let mut new_delegators: Vec<Address> = Vec::new(env);
        for d in delegators.iter() {
            if d != *delegator {
                new_delegators.push_back(d);
            }
        }

        if new_delegators.is_empty() {
            // Remove the key entirely when empty – saves storage rent
            env.storage().instance().remove(&key);
        } else {
            env.storage().instance().set(&key, &new_delegators);
        }
    }

    // ─── Cross-Contract Integration ──────────────────────────────

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

        // Verify it's a controlled contract
        let contracts = Self::get_controlled_contracts(env.clone());
        if !contracts.contains(&contract) {
            return Err(GovernanceError::Unauthorized);
        }

        let result = env.invoke_contract(&contract, &func, args);

        env.events().publish(
            (Symbol::new(&env, "EXECUTE"), contract.clone()),
            ContractExecutedEvent { contract, func },
        );

        Ok(result)
    }

    pub fn upgrade_controlled_contract(
        env: Env,
        _admin: Address,
        contract: Address,
        new_wasm_hash: soroban_sdk::BytesN<32>,
    ) -> Result<(), GovernanceError> {
        Self::check_admin(&env)?;

        // Verify it's a controlled contract
        let contracts = Self::get_controlled_contracts(env.clone());
        if !contracts.contains(&contract) {
            return Err(GovernanceError::Unauthorized);
        }

        let mut args: Vec<soroban_sdk::Val> = Vec::new(&env);
        // We pass the GovernanceContract's own address as the admin of the child contract
        args.push_back(env.current_contract_address().into_val(&env));
        args.push_back(new_wasm_hash.into_val(&env));

        env.invoke_contract::<soroban_sdk::Val>(
            &contract,
            &Symbol::new(&env, "upgrade_contract"),
            args,
        );

        Ok(())
    }
}

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanMetadata {
    pub loan_id: u64,
    pub borrower: Address,
    pub principal: u64,
    pub collateral_amount: u64,
    pub collateral_token: Address,
    pub due_date: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Metadata(u64),              // Loan ID -> LoanMetadata
    Owner(u64),                 // Loan ID -> Address
    Approved(u64),              // Loan ID -> Address
    Operator(Address, Address), // (Owner, Operator) -> bool
    Balance(Address),           // Address -> u64
    TotalSupply,                // -> u64
    TokenUri(u64),              // Loan ID -> String
    ReentrancyGuard,
    Transferable(u64), // Loan ID -> bool
}

#[contract]
pub struct LoanNFT;

#[contractimpl]
impl LoanNFT {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalSupply, &0u64);
    }

    pub fn mint(env: Env, to: Address, metadata: LoanMetadata) {
        Self::check_admin(&env);
        Self::enter_reentrancy_guard(&env);

        let loan_id = metadata.loan_id;
        if env.storage().persistent().has(&DataKey::Metadata(loan_id)) {
            Self::exit_reentrancy_guard(&env);
            panic!("NFT already exists for this loan");
        }

        // Batch all persistent writes together
        env.storage()
            .persistent()
            .set(&DataKey::Metadata(loan_id), &metadata);
        env.storage()
            .persistent()
            .set(&DataKey::Owner(loan_id), &to);
        env.storage()
            .persistent()
            .set(&DataKey::Transferable(loan_id), &true);

        // Update total supply (instance storage – single read+write)
        let total_supply: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(total_supply + 1));

        // Update balance (persistent – single read+write, no extra clone)
        let balance: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(balance + 1));

        Self::emit_mint_event(&env, to, loan_id);
        Self::exit_reentrancy_guard(&env);
    }

    pub fn burn(env: Env, loan_id: u64) {
        Self::check_admin(&env);
        Self::enter_reentrancy_guard(&env);

        let owner =
            Self::owner_of(env.clone(), loan_id).unwrap_or_else(|| panic!("NFT does not exist"));

        // Batch all persistent removes together
        env.storage()
            .persistent()
            .remove(&DataKey::Metadata(loan_id));
        env.storage().persistent().remove(&DataKey::Owner(loan_id));
        env.storage()
            .persistent()
            .remove(&DataKey::Approved(loan_id));
        env.storage()
            .persistent()
            .remove(&DataKey::TokenUri(loan_id));
        env.storage()
            .persistent()
            .remove(&DataKey::Transferable(loan_id));

        // Update total supply (single read+write)
        let total_supply: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        if total_supply > 0 {
            env.storage()
                .instance()
                .set(&DataKey::TotalSupply, &(total_supply - 1));
        }

        // Update balance (single read+write)
        let balance: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(owner.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(owner.clone()), &balance.saturating_sub(1));

        Self::emit_burn_event(&env, owner, loan_id);
        Self::exit_reentrancy_guard(&env);
    }

    // --- NFT Standard Functions ---

    pub fn transfer(env: Env, from: Address, to: Address, loan_id: u64) {
        from.require_auth();
        Self::enter_reentrancy_guard(&env);
        Self::check_transfer_restriction(&env, loan_id);

        let owner = Self::owner_of(env.clone(), loan_id)
            .unwrap_or_else(|| panic!("NFT does not exist for this loan"));
        if owner != from {
            panic!("Not owner");
        }

        Self::do_transfer(&env, from.clone(), to.clone(), loan_id);
        Self::exit_reentrancy_guard(&env);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, loan_id: u64) {
        spender.require_auth();
        Self::enter_reentrancy_guard(&env);
        Self::check_transfer_restriction(&env, loan_id);

        let owner =
            Self::owner_of(env.clone(), loan_id).unwrap_or_else(|| panic!("NFT does not exist"));
        if owner != from {
            panic!("Not owner");
        }

        if spender != from {
            let is_operator = env
                .storage()
                .persistent()
                .get(&DataKey::Operator(from.clone(), spender.clone()))
                .unwrap_or(false);
            if !is_operator {
                let approved_op: Option<Address> =
                    env.storage().persistent().get(&DataKey::Approved(loan_id));
                if approved_op != Some(spender.clone()) {
                    panic!("Not approved");
                }
            }
        }

        Self::do_transfer(&env, from.clone(), to.clone(), loan_id);
        Self::exit_reentrancy_guard(&env);
    }

    pub fn approve(env: Env, caller: Address, operator: Address, loan_id: u64) {
        caller.require_auth();
        let owner =
            Self::owner_of(env.clone(), loan_id).unwrap_or_else(|| panic!("NFT does not exist"));
        if operator == owner {
            panic!("Cannot approve current owner");
        }

        let is_owner = caller == owner;
        let is_operator_for_all = env
            .storage()
            .persistent()
            .get(&DataKey::Operator(owner.clone(), caller.clone()))
            .unwrap_or(false);
        if !is_owner && !is_operator_for_all {
            panic!("Not authorized to approve");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Approved(loan_id), &operator);
        Self::emit_approval_event(&env, owner, operator, loan_id);
    }

    pub fn set_approval_for_all(env: Env, caller: Address, operator: Address, approved: bool) {
        caller.require_auth();
        if caller == operator {
            panic!("Cannot self-approve");
        }
        env.storage().persistent().set(
            &DataKey::Operator(caller.clone(), operator.clone()),
            &approved,
        );
        Self::emit_approval_for_all_event(&env, caller, operator, approved);
    }

    pub fn get_approved(env: Env, loan_id: u64) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Approved(loan_id))
    }

    pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Operator(owner, operator))
            .unwrap_or(false)
    }

    pub fn balance_of(env: Env, owner: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner))
            .unwrap_or(0)
    }

    pub fn total_supply(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    pub fn token_uri(env: Env, loan_id: u64) -> String {
        env.storage()
            .persistent()
            .get(&DataKey::TokenUri(loan_id))
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    pub fn set_token_uri(env: Env, loan_id: u64, uri: String) {
        Self::check_admin(&env);
        if !env.storage().persistent().has(&DataKey::Metadata(loan_id)) {
            panic!("NFT does not exist");
        }
        env.storage()
            .persistent()
            .set(&DataKey::TokenUri(loan_id), &uri);
    }

    pub fn get_metadata(env: Env, loan_id: u64) -> Option<LoanMetadata> {
        env.storage().persistent().get(&DataKey::Metadata(loan_id))
    }

    pub fn owner_of(env: Env, loan_id: u64) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Owner(loan_id))
    }

    // --- Transfer Restrictions ---
    pub fn set_transferable(env: Env, loan_id: u64, is_transferable: bool) {
        Self::check_admin(&env);
        if !env.storage().persistent().has(&DataKey::Metadata(loan_id)) {
            panic!("NFT does not exist");
        }
        env.storage()
            .persistent()
            .set(&DataKey::Transferable(loan_id), &is_transferable);
    }

    fn check_transfer_restriction(env: &Env, loan_id: u64) {
        let is_transferable = env
            .storage()
            .persistent()
            .get(&DataKey::Transferable(loan_id))
            .unwrap_or(true);
        if !is_transferable {
            panic!("NFT transfer is restricted for an active loan");
        }
    }

    // --- Helpers ---
    fn do_transfer(env: &Env, from: Address, to: Address, loan_id: u64) {
        env.storage()
            .persistent()
            .set(&DataKey::Owner(loan_id), &to);
        env.storage()
            .persistent()
            .remove(&DataKey::Approved(loan_id));

        let mut balance_from: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        balance_from = balance_from.saturating_sub(1);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &balance_from);

        let mut balance_to: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        balance_to += 1;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &balance_to);

        Self::emit_transfer_event(env, from, to, loan_id);
    }

    fn check_admin(env: &Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();
    }

    fn emit_mint_event(env: &Env, to: Address, loan_id: u64) {
        env.events()
            .publish((Symbol::new(env, "Transfer"), (), to), loan_id);
    }

    fn emit_burn_event(env: &Env, owner: Address, loan_id: u64) {
        env.events()
            .publish((Symbol::new(env, "Transfer"), owner, ()), loan_id);
    }

    fn emit_transfer_event(env: &Env, from: Address, to: Address, loan_id: u64) {
        env.events()
            .publish((Symbol::new(env, "Transfer"), from, to), loan_id);
    }

    fn emit_approval_event(env: &Env, owner: Address, operator: Address, loan_id: u64) {
        env.events()
            .publish((Symbol::new(env, "Approval"), owner, operator), loan_id);
    }

    fn emit_approval_for_all_event(env: &Env, owner: Address, operator: Address, approved: bool) {
        env.events().publish(
            (Symbol::new(env, "ApprovalForAll"), owner, operator),
            approved,
        );
    }

    fn enter_reentrancy_guard(env: &Env) {
        if env.storage().instance().has(&DataKey::ReentrancyGuard) {
            panic!("Reentrant call");
        }
        env.storage()
            .instance()
            .set(&DataKey::ReentrancyGuard, &true);
    }

    fn exit_reentrancy_guard(env: &Env) {
        env.storage().instance().remove(&DataKey::ReentrancyGuard);
    }
}

#[cfg(test)]
mod test;

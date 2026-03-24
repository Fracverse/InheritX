#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

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
    Metadata(u64), // Metadata by Loan ID
    Owner(u64),    // Owner of the NFT by Loan ID
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
    }

    pub fn mint(env: Env, to: Address, metadata: LoanMetadata) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        let loan_id = metadata.loan_id;
        if env.storage().persistent().has(&DataKey::Metadata(loan_id)) {
            panic!("NFT already exists for this loan");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Metadata(loan_id), &metadata);
        env.storage()
            .persistent()
            .set(&DataKey::Owner(loan_id), &to);
    }

    pub fn burn(env: Env, loan_id: u64) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        if !env.storage().persistent().has(&DataKey::Metadata(loan_id)) {
            panic!("NFT does not exist for this loan");
        }

        env.storage()
            .persistent()
            .remove(&DataKey::Metadata(loan_id));
        env.storage().persistent().remove(&DataKey::Owner(loan_id));
    }

    pub fn get_metadata(env: Env, loan_id: u64) -> Option<LoanMetadata> {
        env.storage().persistent().get(&DataKey::Metadata(loan_id))
    }

    pub fn owner_of(env: Env, loan_id: u64) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Owner(loan_id))
    }
}

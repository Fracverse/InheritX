#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, log, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KycStatus {
    Pending,
    Approved,
    Rejected,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KycError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    KycNotApproved = 4,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Status(Address),
}

#[contract]
pub struct KycContract;

#[contractimpl]
impl KycContract {
    /// Initialize the KYC admin address.
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address) -> Result<(), KycError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(KycError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Set the KYC status for a user. Only the KYC admin may call this.
    pub fn set_status(
        env: Env,
        admin: Address,
        user: Address,
        status: KycStatus,
    ) -> Result<(), KycError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(KycError::NotInitialized)?;

        if stored_admin != admin {
            return Err(KycError::Unauthorized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Status(user), &status);
        Ok(())
    }

    /// Get the KYC status for a user.
    pub fn get_status(env: Env, user: Address) -> KycStatus {
        env.storage()
            .instance()
            .get(&DataKey::Status(user))
            .unwrap_or(KycStatus::Pending)
    }

    /// Check if a user has approved KYC status.
    /// Returns an error if not approved.
    pub fn require_approved(env: Env, user: Address) -> Result<(), KycError> {
        let status = Self::get_status(env, user);
        if status != KycStatus::Approved {
            log!(&env, "KYC not approved for user");
            Err(KycError::KycNotApproved)
        } else {
            Ok(())
        }
    }

    /// Get the admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized")
    }
}

mod test;

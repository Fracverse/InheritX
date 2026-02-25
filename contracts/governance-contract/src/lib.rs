#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

mod test;

#[contracttype]
pub enum DataKey {
    Admin,
    InterestRate,
    CollateralRatio,
    LiquidationBonus,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GovernanceError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
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
}

#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};

mod test;

#[derive(Clone)]
#[contracttype]
pub struct Loan {
    pub borrower: Address,
    pub principal: i128,
    pub interest_rate: u32,
    pub due_date: u64,
    pub amount_repaid: i128,
    pub collateral_amount: i128,
    pub collateral_token: Address,
    pub is_active: bool,
}

#[contracttype]
pub enum DataKey {
    Admin,
    CollateralRatio,
    LiquidationThreshold,
    LiquidationBonus,
    WhitelistedCollateral(Address),
    LoanCounter,
    Loan(u64),
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BorrowingError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    InsufficientCollateral = 3,
    CollateralNotWhitelisted = 4,
    LoanNotFound = 5,
    LoanHealthy = 6,
    LoanNotActive = 7,
}

#[contract]
pub struct BorrowingContract;

#[contractimpl]
impl BorrowingContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        collateral_ratio_bps: u32,
        liquidation_threshold_bps: u32,
        liquidation_bonus_bps: u32,
    ) -> Result<(), BorrowingError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(BorrowingError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::CollateralRatio, &collateral_ratio_bps);
        env.storage()
            .instance()
            .set(&DataKey::LiquidationThreshold, &liquidation_threshold_bps);
        env.storage()
            .instance()
            .set(&DataKey::LiquidationBonus, &liquidation_bonus_bps);
        Ok(())
    }

    pub fn create_loan(
        env: Env,
        borrower: Address,
        principal: i128,
        interest_rate: u32,
        due_date: u64,
        collateral_token: Address,
        collateral_amount: i128,
    ) -> Result<u64, BorrowingError> {
        borrower.require_auth();

        // Check collateral is whitelisted
        if !Self::is_whitelisted(env.clone(), collateral_token.clone()) {
            return Err(BorrowingError::CollateralNotWhitelisted);
        }

        // Check collateral ratio
        let ratio = Self::get_collateral_ratio(env.clone());
        let required_collateral = (principal as u128)
            .checked_mul(ratio as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as i128;

        if collateral_amount < required_collateral {
            return Err(BorrowingError::InsufficientCollateral);
        }

        // Transfer collateral to contract
        let token_client = token::Client::new(&env, &collateral_token);
        token_client.transfer(
            &borrower,
            &env.current_contract_address(),
            &collateral_amount,
        );

        let loan_id = Self::get_next_loan_id(&env);

        let loan = Loan {
            borrower,
            principal,
            interest_rate,
            due_date,
            amount_repaid: 0,
            collateral_amount,
            collateral_token,
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        Ok(loan_id)
    }

    pub fn repay_loan(env: Env, loan_id: u64, amount: i128) {
        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .unwrap();

        loan.borrower.require_auth();

        loan.amount_repaid += amount;

        if loan.amount_repaid >= loan.principal {
            loan.is_active = false;

            // Return collateral
            let token_client = token::Client::new(&env, &loan.collateral_token);
            token_client.transfer(
                &env.current_contract_address(),
                &loan.borrower,
                &loan.collateral_amount,
            );
        }

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);
    }

    pub fn get_loan(env: Env, loan_id: u64) -> Loan {
        env.storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .unwrap()
    }

    pub fn whitelist_collateral(
        env: Env,
        admin: Address,
        token: Address,
    ) -> Result<(), BorrowingError> {
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(BorrowingError::Unauthorized);
        }
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::WhitelistedCollateral(token), &true);
        Ok(())
    }

    pub fn is_whitelisted(env: Env, token: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::WhitelistedCollateral(token))
            .unwrap_or(false)
    }

    pub fn get_collateral_ratio(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CollateralRatio)
            .unwrap_or(15000)
    }

    pub fn liquidate(env: Env, liquidator: Address, loan_id: u64) -> Result<(), BorrowingError> {
        liquidator.require_auth();

        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        if !loan.is_active {
            return Err(BorrowingError::LoanNotActive);
        }

        // Calculate health factor
        let debt = loan.principal - loan.amount_repaid;
        let health_factor = if debt == 0 {
            10000
        } else {
            (loan.collateral_amount as u128)
                .checked_mul(10000)
                .and_then(|v| v.checked_div(debt as u128))
                .unwrap_or(0) as u32
        };

        let liquidation_threshold = Self::get_liquidation_threshold(&env);

        // Check if loan is unhealthy (health factor below threshold)
        if health_factor >= liquidation_threshold {
            return Err(BorrowingError::LoanHealthy);
        }

        // Calculate liquidation amounts
        let liquidation_bonus = Self::get_liquidation_bonus(&env);
        let bonus_amount = (debt as u128)
            .checked_mul(liquidation_bonus as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as i128;
        let liquidator_reward = debt + bonus_amount;

        // Transfer collateral to liquidator
        let token_client = token::Client::new(&env, &loan.collateral_token);
        token_client.transfer(
            &env.current_contract_address(),
            &liquidator,
            &liquidator_reward,
        );

        // Mark loan as inactive
        loan.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        Ok(())
    }

    pub fn get_health_factor(env: Env, loan_id: u64) -> Result<u32, BorrowingError> {
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        let debt = loan.principal - loan.amount_repaid;
        let health_factor = if debt == 0 {
            10000
        } else {
            (loan.collateral_amount as u128)
                .checked_mul(10000)
                .and_then(|v| v.checked_div(debt as u128))
                .unwrap_or(0) as u32
        };

        Ok(health_factor)
    }

    fn get_liquidation_threshold(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::LiquidationThreshold)
            .unwrap_or(12000) // 120% default
    }

    fn get_liquidation_bonus(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::LiquidationBonus)
            .unwrap_or(500) // 5% default
    }

    fn get_next_loan_id(env: &Env) -> u64 {
        let counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::LoanCounter)
            .unwrap_or(0);
        let next_id = counter + 1;
        env.storage()
            .persistent()
            .set(&DataKey::LoanCounter, &next_id);
        next_id
    }
}

#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env,
};

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

// ─────────────────────────────────────────────────
// Events
// ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BorrowEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub principal: i128,
    pub collateral_amount: i128,
    pub collateral_token: Address,
    pub interest_rate: u32,
    pub due_date: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepayEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub amount_repaid: i128,
    pub principal: i128,
    pub interest_paid: i128,
    pub collateral_returned: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidationEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub liquidator: Address,
    pub amount_liquidated: i128,
    pub collateral_seized: i128,
    pub health_factor: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterestAccrualEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub principal: i128,
    pub interest_accrued: i128,
    pub interest_rate: u32,
    pub elapsed_seconds: u64,
    pub timestamp: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    CollateralRatio,
    LiquidationThreshold,
    LiquidationBonus,
    WhitelistedCollateral(Address),
    GlobalPause,
    VaultPause(Address),
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
    InvalidAmount = 8,
    Paused = 9,
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

        // Check if paused
        if Self::is_global_paused(env.clone())
            || Self::is_vault_paused(env.clone(), collateral_token.clone())
        {
            return Err(BorrowingError::Paused);
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
            borrower: borrower.clone(),
            principal,
            interest_rate,
            due_date,
            amount_repaid: 0,
            collateral_amount,
            collateral_token: collateral_token.clone(),
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        // Emit borrow event
        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("BORROW")),
            BorrowEvent {
                loan_id,
                borrower: borrower.clone(),
                principal,
                collateral_amount,
                collateral_token,
                interest_rate,
                due_date,
                timestamp: env.ledger().timestamp(),
            },
        );

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

        // Emit repay event
        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("REPAY")),
            RepayEvent {
                loan_id,
                borrower: loan.borrower.clone(),
                amount_repaid: amount,
                principal: loan.principal,
                interest_paid: 0, // Interest calculation would be needed based on contract logic
                collateral_returned: if loan.is_active {
                    0
                } else {
                    loan.collateral_amount
                },
                timestamp: env.ledger().timestamp(),
            },
        );

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

    pub fn set_global_pause(env: Env, admin: Address, paused: bool) -> Result<(), BorrowingError> {
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(BorrowingError::Unauthorized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::GlobalPause, &paused);
        Ok(())
    }

    pub fn is_global_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::GlobalPause)
            .unwrap_or(false)
    }

    pub fn set_vault_pause(
        env: Env,
        admin: Address,
        token: Address,
        paused: bool,
    ) -> Result<(), BorrowingError> {
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(BorrowingError::Unauthorized);
        }
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::VaultPause(token), &paused);
        Ok(())
    }

    pub fn is_vault_paused(env: Env, token: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::VaultPause(token))
            .unwrap_or(false)
    }

    pub fn get_collateral_ratio(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CollateralRatio)
            .unwrap_or(15000)
    }

    pub fn liquidate(
        env: Env,
        liquidator: Address,
        loan_id: u64,
        liquidate_amount: i128,
    ) -> Result<(), BorrowingError> {
        liquidator.require_auth();

        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        if !loan.is_active {
            return Err(BorrowingError::LoanNotActive);
        }

        let debt = loan.principal - loan.amount_repaid;

        if liquidate_amount <= 0 || liquidate_amount > debt {
            return Err(BorrowingError::InvalidAmount);
        }

        // Calculate health factor
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

        // Calculate liquidation amounts based on liquidate_amount
        let liquidation_bonus = Self::get_liquidation_bonus(&env);
        let bonus_amount = (liquidate_amount as u128)
            .checked_mul(liquidation_bonus as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as i128;
        let liquidator_reward = liquidate_amount + bonus_amount;

        if liquidator_reward > loan.collateral_amount {
            return Err(BorrowingError::InvalidAmount);
        }

        // Transfer collateral to liquidator
        let token_client = token::Client::new(&env, &loan.collateral_token);
        token_client.transfer(
            &env.current_contract_address(),
            &liquidator,
            &liquidator_reward,
        );

        loan.collateral_amount -= liquidator_reward;
        loan.amount_repaid += liquidate_amount;

        // Mark loan as inactive if fully repaid
        if loan.amount_repaid >= loan.principal {
            loan.is_active = false;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        // Emit liquidation event
        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("LIQUIDATE")),
            LiquidationEvent {
                loan_id,
                borrower: loan.borrower.clone(),
                liquidator: liquidator.clone(),
                amount_liquidated: liquidate_amount,
                collateral_seized: liquidator_reward,
                health_factor,
                timestamp: env.ledger().timestamp(),
            },
        );

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

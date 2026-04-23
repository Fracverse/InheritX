#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env,
};

mod test;

const EXTENSION_FEE_BPS: u32 = 100; // 1%
const MAX_EXTENSIONS: u32 = 2;

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
    pub extension_count: u32,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanExtendedEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub new_due_date: u64,
    pub extension_fee: i128,
    pub extension_count: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanIncreasedEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub additional_amount: i128,
    pub new_principal: i128,
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
    MaxExtensionsReached = 10,
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

        if !Self::is_whitelisted(env.clone(), collateral_token.clone()) {
            return Err(BorrowingError::CollateralNotWhitelisted);
        }

        if Self::is_global_paused(env.clone())
            || Self::is_vault_paused(env.clone(), collateral_token.clone())
        {
            return Err(BorrowingError::Paused);
        }

        let ratio = Self::get_collateral_ratio(env.clone());
        let required_collateral = (principal as u128)
            .checked_mul(ratio as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as i128;

        if collateral_amount < required_collateral {
            return Err(BorrowingError::InsufficientCollateral);
        }

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
            extension_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

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

            let token_client = token::Client::new(&env, &loan.collateral_token);
            token_client.transfer(
                &env.current_contract_address(),
                &loan.borrower,
                &loan.collateral_amount,
            );
        }

        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("REPAY")),
            RepayEvent {
                loan_id,
                borrower: loan.borrower.clone(),
                amount_repaid: amount,
                principal: loan.principal,
                interest_paid: 0,
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

    /// Extend a loan's due date by paying a 1% extension fee.
    /// Maximum 2 extensions per loan.
    pub fn extend_loan(
        env: Env,
        loan_id: u64,
        new_due_date: u64,
    ) -> Result<(), BorrowingError> {
        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        loan.borrower.require_auth();

        if !loan.is_active {
            return Err(BorrowingError::LoanNotActive);
        }

        if loan.extension_count >= MAX_EXTENSIONS {
            return Err(BorrowingError::MaxExtensionsReached);
        }

        let fee = Self::get_extension_fee(env.clone(), loan_id)?;

        // Collect fee from borrower (paid in collateral token)
        let token_client = token::Client::new(&env, &loan.collateral_token);
        token_client.transfer(&loan.borrower, &env.current_contract_address(), &fee);

        loan.due_date = new_due_date;
        loan.extension_count += 1;

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("EXTEND")),
            LoanExtendedEvent {
                loan_id,
                borrower: loan.borrower.clone(),
                new_due_date,
                extension_fee: fee,
                extension_count: loan.extension_count,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Borrow additional funds against the same collateral, if health factor allows.
    pub fn increase_loan_amount(
        env: Env,
        loan_id: u64,
        additional_amount: i128,
    ) -> Result<(), BorrowingError> {
        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        loan.borrower.require_auth();

        if !loan.is_active {
            return Err(BorrowingError::LoanNotActive);
        }

        if additional_amount <= 0 {
            return Err(BorrowingError::InvalidAmount);
        }

        let max_additional = Self::get_max_additional_borrow(env.clone(), loan_id)?;
        if additional_amount > max_additional {
            return Err(BorrowingError::InsufficientCollateral);
        }

        loan.principal += additional_amount;

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("INCREASE")),
            LoanIncreasedEvent {
                loan_id,
                borrower: loan.borrower.clone(),
                additional_amount,
                new_principal: loan.principal,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Returns the extension fee (1% of remaining debt) for a given loan.
    pub fn get_extension_fee(env: Env, loan_id: u64) -> Result<i128, BorrowingError> {
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        let debt = loan.principal - loan.amount_repaid;
        let fee = (debt as u128)
            .checked_mul(EXTENSION_FEE_BPS as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as i128;

        Ok(fee)
    }

    /// Returns the maximum additional amount that can be borrowed given current collateral.
    pub fn get_max_additional_borrow(env: Env, loan_id: u64) -> Result<i128, BorrowingError> {
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        let ratio = Self::get_collateral_ratio(env.clone());

        // max_debt = collateral * 10000 / ratio
        let max_debt = (loan.collateral_amount as u128)
            .checked_mul(10000)
            .and_then(|v| v.checked_div(ratio as u128))
            .unwrap_or(0) as i128;

        let current_debt = loan.principal - loan.amount_repaid;
        let max_additional = max_debt - current_debt;

        Ok(if max_additional > 0 { max_additional } else { 0 })
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

        let health_factor = if debt == 0 {
            10000
        } else {
            (loan.collateral_amount as u128)
                .checked_mul(10000)
                .and_then(|v| v.checked_div(debt as u128))
                .unwrap_or(0) as u32
        };

        let liquidation_threshold = Self::get_liquidation_threshold(&env);

        if health_factor >= liquidation_threshold {
            return Err(BorrowingError::LoanHealthy);
        }

        let liquidation_bonus = Self::get_liquidation_bonus(&env);
        let bonus_amount = (liquidate_amount as u128)
            .checked_mul(liquidation_bonus as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as i128;
        let liquidator_reward = liquidate_amount + bonus_amount;

        if liquidator_reward > loan.collateral_amount {
            return Err(BorrowingError::InvalidAmount);
        }

        let token_client = token::Client::new(&env, &loan.collateral_token);
        token_client.transfer(
            &env.current_contract_address(),
            &liquidator,
            &liquidator_reward,
        );

        loan.collateral_amount -= liquidator_reward;
        loan.amount_repaid += liquidate_amount;

        if loan.amount_repaid >= loan.principal {
            loan.is_active = false;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

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
            .unwrap_or(12000)
    }

    fn get_liquidation_bonus(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::LiquidationBonus)
            .unwrap_or(500)
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

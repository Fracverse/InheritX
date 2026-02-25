#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, vec, Address, Env,
    IntoVal, InvokeError, Val, Vec,
};

// ─────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────

const MINIMUM_LIQUIDITY: u64 = 1000;

// ─────────────────────────────────────────────────
// Data Types
// ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolState {
    pub total_deposits: u64, // Total underlying tokens deposited (net, tracks repayments too)
    pub total_shares: u64,   // Total pool shares outstanding
    pub total_borrowed: u64, // Total principal currently on loan
    pub base_rate_bps: u32,  // Base interest rate in basis points (1/10000)
    pub multiplier_bps: u32, // Multiplier applied to utilization to get variable rate
}

const SECONDS_IN_YEAR: u64 = 31_536_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanRecord {
    pub borrower: Address,
    pub amount: u64,
    pub borrow_time: u64,
    pub interest_rate_bps: u32,
}

// ─────────────────────────────────────────────────
// Events
// ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositEvent {
    pub depositor: Address,
    pub amount: u64,
    pub shares_minted: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawEvent {
    pub depositor: Address,
    pub shares_burned: u64,
    pub amount: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BorrowEvent {
    pub borrower: Address,
    pub amount: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepayEvent {
    pub borrower: Address,
    pub amount: u64,
}

// ─────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LendingError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAdmin = 3,
    InsufficientLiquidity = 4,
    InsufficientShares = 5,
    NoOpenLoan = 6,
    LoanAlreadyExists = 7,
    InvalidAmount = 8,
    TransferFailed = 9,
    Unauthorized = 10,
}

// ─────────────────────────────────────────────────
// Storage Keys
// ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    Pool,
    Shares(Address),
    Loan(Address),
}

// ─────────────────────────────────────────────────
// Contract
// ─────────────────────────────────────────────────

#[contract]
pub struct LendingContract;

#[contractimpl]
impl LendingContract {
    // ─── Admin / Init ───────────────────────────────

    /// Initialize the lending pool with an admin address and the underlying token.
    /// Can only be called once.
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        base_rate_bps: u32,
        multiplier_bps: u32,
    ) -> Result<(), LendingError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(LendingError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(
            &DataKey::Pool,
            &PoolState {
                total_deposits: 0,
                total_shares: 0,
                total_borrowed: 0,
                base_rate_bps,
                multiplier_bps,
            },
        );
        Ok(())
    }

    fn require_initialized(env: &Env) -> Result<(), LendingError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(LendingError::NotInitialized);
        }
        Ok(())
    }

    fn get_token(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Token).unwrap()
    }

    fn get_pool(env: &Env) -> PoolState {
        env.storage().instance().get(&DataKey::Pool).unwrap()
    }

    fn set_pool(env: &Env, pool: &PoolState) {
        env.storage().instance().set(&DataKey::Pool, pool);
    }

    fn get_shares(env: &Env, owner: &Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Shares(owner.clone()))
            .unwrap_or(0u64)
    }

    fn set_shares(env: &Env, owner: &Address, shares: u64) {
        env.storage()
            .persistent()
            .set(&DataKey::Shares(owner.clone()), &shares);
    }

    fn transfer(
        env: &Env,
        token: &Address,
        from: &Address,
        to: &Address,
        amount: u64,
    ) -> Result<(), LendingError> {
        let amount_i128 = amount as i128;
        let args: Vec<Val> = vec![
            env,
            from.clone().into_val(env),
            to.clone().into_val(env),
            amount_i128.into_val(env),
        ];
        let res =
            env.try_invoke_contract::<(), InvokeError>(token, &symbol_short!("transfer"), args);
        if res.is_err() {
            return Err(LendingError::TransferFailed);
        }
        Ok(())
    }

    // ─── Share Math ─────────────────────────────────

    /// Calculate how many shares to mint for a given deposit amount.
    /// On the first deposit (total_shares == 0), shares = amount (1:1).
    fn shares_for_deposit(pool: &PoolState, amount: u64) -> u64 {
        if pool.total_shares == 0 || pool.total_deposits == 0 {
            amount // 1:1 initial ratio
        } else {
            (amount as u128)
                .checked_mul(pool.total_shares as u128)
                .and_then(|v| v.checked_div(pool.total_deposits as u128))
                .unwrap_or(0) as u64
        }
    }

    /// Calculate how many underlying tokens correspond to a given number of shares.
    fn assets_for_shares(pool: &PoolState, shares: u64) -> u64 {
        if pool.total_shares == 0 {
            0
        } else {
            (shares as u128)
                .checked_mul(pool.total_deposits as u128)
                .and_then(|v| v.checked_div(pool.total_shares as u128))
                .unwrap_or(0) as u64
        }
    }

    /// Calculate simple interest for a given principal, rate, and time elapsed.
    fn calculate_interest(principal: u64, rate_bps: u32, elapsed_seconds: u64) -> u64 {
        if elapsed_seconds == 0 || rate_bps == 0 {
            return 0;
        }
        // Interest = (Principal * Rate * Time) / (10000 * SecondsPerYear)
        // Use u128 for intermediate calculation to avoid overflow.
        let numerator = (principal as u128)
            .checked_mul(rate_bps as u128)
            .and_then(|v| v.checked_mul(elapsed_seconds as u128))
            .unwrap_or(0);

        let denominator = (10000u128).checked_mul(SECONDS_IN_YEAR as u128).unwrap();

        (numerator.checked_div(denominator).unwrap_or(0)) as u64
    }

    /// Calculate the pool utilization ratio in basis points (0 to 10000)
    fn get_utilization_bps(total_borrowed: u64, total_deposits: u64) -> u32 {
        if total_deposits == 0 {
            return 0;
        }
        let utilization = (total_borrowed as u128)
            .checked_mul(10000)
            .and_then(|v| v.checked_div(total_deposits as u128))
            .unwrap_or(0);
        utilization as u32
    }

    /// Calculate the dynamic interest rate based on utilization
    fn calculate_dynamic_rate(
        base_rate_bps: u32,
        multiplier_bps: u32,
        utilization_bps: u32,
    ) -> u32 {
        let variable_rate = (utilization_bps as u64)
            .checked_mul(multiplier_bps as u64)
            .unwrap_or(0)
            / 10000;
        base_rate_bps.saturating_add(variable_rate as u32)
    }

    // ─── Public Functions ────────────────────────────

    /// Deposit `amount` of the underlying token into the pool.
    /// Mints proportional pool shares to the depositor.
    pub fn deposit(env: Env, depositor: Address, amount: u64) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        depositor.require_auth();

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let token = Self::get_token(&env);
        let contract_id = env.current_contract_address();
        Self::transfer(&env, &token, &depositor, &contract_id, amount)?;

        let mut pool = Self::get_pool(&env);
        let mut shares = Self::shares_for_deposit(&pool, amount);

        if pool.total_shares == 0 {
            if shares <= MINIMUM_LIQUIDITY {
                return Err(LendingError::InvalidAmount);
            }
            shares -= MINIMUM_LIQUIDITY;
            pool.total_shares += MINIMUM_LIQUIDITY;
        }

        if shares == 0 {
            return Err(LendingError::InvalidAmount);
        }

        pool.total_deposits += amount;
        pool.total_shares += shares;
        Self::set_pool(&env, &pool);

        let existing = Self::get_shares(&env, &depositor);
        Self::set_shares(&env, &depositor, existing + shares);

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("DEPOSIT")),
            DepositEvent {
                depositor: depositor.clone(),
                amount,
                shares_minted: shares,
            },
        );
        log!(
            &env,
            "Deposited {} tokens, minted {} shares",
            amount,
            shares
        );
        Ok(shares)
    }

    /// Burn `shares` and return the proportional underlying tokens to the depositor.
    /// Reverts if insufficient liquidity (i.e., tokens are loaned out).
    pub fn withdraw(env: Env, depositor: Address, shares: u64) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        depositor.require_auth();

        if shares == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let depositor_shares = Self::get_shares(&env, &depositor);
        if shares > depositor_shares {
            return Err(LendingError::InsufficientShares);
        }

        let mut pool = Self::get_pool(&env);
        let amount = Self::assets_for_shares(&pool, shares);

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let available = pool.total_deposits.saturating_sub(pool.total_borrowed);
        if amount > available {
            return Err(LendingError::InsufficientLiquidity);
        }

        pool.total_deposits -= amount;
        pool.total_shares -= shares;
        Self::set_pool(&env, &pool);
        Self::set_shares(&env, &depositor, depositor_shares - shares);

        let token = Self::get_token(&env);
        let contract_id = env.current_contract_address();
        Self::transfer(&env, &token, &contract_id, &depositor, amount)?;

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("WITHDRAW")),
            WithdrawEvent {
                depositor: depositor.clone(),
                shares_burned: shares,
                amount,
            },
        );
        log!(&env, "Withdrew {} tokens, burned {} shares", amount, shares);
        Ok(amount)
    }

    /// Borrow `amount` of the underlying token from the pool.
    /// Reduces available liquidity. Only one open loan per borrower at a time.
    pub fn borrow(env: Env, borrower: Address, amount: u64) -> Result<(), LendingError> {
        Self::require_initialized(&env)?;
        borrower.require_auth();

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        // Only one open loan per borrower
        if env
            .storage()
            .persistent()
            .has(&DataKey::Loan(borrower.clone()))
        {
            return Err(LendingError::LoanAlreadyExists);
        }

        let mut pool = Self::get_pool(&env);
        let available = pool.total_deposits.saturating_sub(pool.total_borrowed);
        if amount > available {
            return Err(LendingError::InsufficientLiquidity);
        }

        pool.total_borrowed += amount;

        let utilization_bps = Self::get_utilization_bps(pool.total_borrowed, pool.total_deposits);
        let dynamic_rate_bps =
            Self::calculate_dynamic_rate(pool.base_rate_bps, pool.multiplier_bps, utilization_bps);

        Self::set_pool(&env, &pool);

        env.storage().persistent().set(
            &DataKey::Loan(borrower.clone()),
            &LoanRecord {
                borrower: borrower.clone(),
                amount,
                borrow_time: env.ledger().timestamp(),
                interest_rate_bps: dynamic_rate_bps,
            },
        );

        let token = Self::get_token(&env);
        let contract_id = env.current_contract_address();
        Self::transfer(&env, &token, &contract_id, &borrower, amount)?;

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("BORROW")),
            BorrowEvent {
                borrower: borrower.clone(),
                amount,
            },
        );
        log!(&env, "Borrowed {} tokens", amount);
        Ok(())
    }

    /// Repay the full outstanding loan for the caller.
    /// Restores liquidity to the pool and closes the loan record.
    pub fn repay(env: Env, borrower: Address) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        borrower.require_auth();

        let loan: LoanRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower.clone()))
            .ok_or(LendingError::NoOpenLoan)?;

        let elapsed = env.ledger().timestamp().saturating_sub(loan.borrow_time);
        let interest = Self::calculate_interest(loan.amount, loan.interest_rate_bps, elapsed);
        let total_repayment = loan.amount + interest;

        let token = Self::get_token(&env);
        let contract_id = env.current_contract_address();
        Self::transfer(&env, &token, &borrower, &contract_id, total_repayment)?;

        let mut pool = Self::get_pool(&env);
        pool.total_borrowed -= loan.amount;
        pool.total_deposits += interest; // Interest increases pool value for share holders
        Self::set_pool(&env, &pool);

        env.storage()
            .persistent()
            .remove(&DataKey::Loan(borrower.clone()));

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("REPAY")),
            RepayEvent {
                borrower: borrower.clone(),
                amount: total_repayment,
            },
        );
        log!(
            &env,
            "Repaid {} tokens ({} interest)",
            total_repayment,
            interest
        );
        Ok(total_repayment)
    }

    /// Calculate the total amount (principal + interest) required to repay the loan.
    pub fn get_repayment_amount(env: Env, borrower: Address) -> Result<u64, LendingError> {
        let loan_opt: Option<LoanRecord> = env.storage().persistent().get(&DataKey::Loan(borrower));

        match loan_opt {
            Some(loan) => {
                let elapsed = env.ledger().timestamp().saturating_sub(loan.borrow_time);
                let interest =
                    Self::calculate_interest(loan.amount, loan.interest_rate_bps, elapsed);
                Ok(loan.amount + interest)
            }
            None => Err(LendingError::NoOpenLoan),
        }
    }

    // ─── Reads ───────────────────────────────────────

    /// Returns the current global pool state.
    pub fn get_pool_state(env: Env) -> Result<PoolState, LendingError> {
        Self::require_initialized(&env)?;
        Ok(Self::get_pool(&env))
    }

    /// Returns the share balance of the given address.
    pub fn get_shares_of(env: Env, owner: Address) -> u64 {
        Self::get_shares(&env, &owner)
    }

    /// Returns the outstanding loan record for the given borrower, if any.
    pub fn get_loan(env: Env, borrower: Address) -> Option<LoanRecord> {
        env.storage().persistent().get(&DataKey::Loan(borrower))
    }

    /// Returns the available (un-borrowed) liquidity in the pool.
    pub fn available_liquidity(env: Env) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        let pool = Self::get_pool(&env);
        Ok(pool.total_deposits.saturating_sub(pool.total_borrowed))
    }

    /// Returns the current dynamic interest rate that would be given to a new loan
    pub fn get_current_interest_rate(env: Env) -> Result<u32, LendingError> {
        Self::require_initialized(&env)?;
        let pool = Self::get_pool(&env);
        let utilization_bps = Self::get_utilization_bps(pool.total_borrowed, pool.total_deposits);
        Ok(Self::calculate_dynamic_rate(
            pool.base_rate_bps,
            pool.multiplier_bps,
            utilization_bps,
        ))
    }
}

mod test;

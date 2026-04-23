#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, token, vec, Address,
    Env, IntoVal, InvokeError, Val, Vec,
};

// ─────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────

const MINIMUM_LIQUIDITY: u64 = 1000;
const PROTOCOL_INTEREST_BPS: u32 = 1000; // 10% of interest retained by protocol
const BAD_DEBT_RESERVE_BPS: u32 = 5000; // 50% of protocol share routed to reserve
const DEFAULT_GRACE_PERIOD_SECONDS: u64 = 259_200; // 3 days
const DEFAULT_LATE_FEE_RATE_BPS: u32 = 500; // 5% per day = 0.058% per second (approx)
const REFINANCING_FEE_BPS: u32 = 50; // 0.5% refinancing fee

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
    pub utilization_cap_bps: u32, // Maximum utilization allowed in basis points (e.g., 8000 = 80%)
    pub retained_yield: u64, // Yield reserved for protocol/priority payouts
    pub bad_debt_reserve: u64, // Reserve bucket for bad debt coverage
    pub grace_period_seconds: u64, // Grace period duration in seconds (e.g., 3 days = 259200)
    pub late_fee_rate_bps: u32, // Late fee rate in basis points per day (e.g., 500 = 5% per day)
}

const SECONDS_IN_YEAR: u64 = 31_536_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanRecord {
    pub loan_id: u64,
    pub borrower: Address,
    pub principal: u64,
    pub collateral_amount: u64,
    pub collateral_token: Address,
    pub borrow_time: u64,
    pub due_date: u64,
    pub interest_rate_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefinanceTerms {
    pub outstanding_balance: u64,
    pub new_principal: u64,
    pub refinancing_fee: u64,
    pub total_required: u64,
    pub new_interest_rate_bps: u32,
    pub new_duration_seconds: u64,
    pub new_due_date: u64,
}

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

#[soroban_sdk::contractclient(name = "LoanNFTClient")]
pub trait LoanNFTInterface {
    fn initialize(env: Env, admin: Address);
    fn mint(env: Env, to: Address, metadata: LoanMetadata);
    fn burn(env: Env, loan_id: u64);
    fn get_metadata(env: Env, loan_id: u64) -> Option<LoanMetadata>;
    fn owner_of(env: Env, loan_id: u64) -> Option<Address>;
}

#[soroban_sdk::contractclient(name = "FlashLoanReceiverClient")]
pub trait FlashLoanReceiverInterface {
    fn execute_operation(env: Env, amount: u64, fee: u64, initiator: Address);
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
pub struct PriorityWithdrawEvent {
    pub caller: Address,
    pub amount: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BorrowEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub amount: u64,
    pub collateral_amount: u64,
    pub due_date: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepayEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub principal: u64,
    pub interest: u64,
    pub total_amount: u64,
    pub collateral_returned: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollateralDepositEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub collateral_token: Address,
    pub amount: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidationEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub liquidator: Address,
    pub amount_repaid: u64,
    pub collateral_seized: u64,
    pub health_factor: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterestAccrualEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub principal: u64,
    pub interest_accrued: u64,
    pub interest_rate_bps: u32,
    pub elapsed_seconds: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LateFeeChargedEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub late_fee: u64,
    pub days_overdue: u64,
    pub total_with_late_fees: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlashLoanEvent {
    pub receiver: Address,
    pub amount: u64,
    pub fee: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanRefinancedEvent {
    pub old_loan_id: u64,
    pub new_loan_id: u64,
    pub borrower: Address,
    pub old_principal: u64,
    pub new_principal: u64,
    pub refinancing_fee: u64,
    pub old_interest_rate_bps: u32,
    pub new_interest_rate_bps: u32,
    pub old_due_date: u64,
    pub new_due_date: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoansConsolidatedEvent {
    pub old_loan_ids: Vec<u64>,
    pub new_loan_id: u64,
    pub borrower: Address,
    pub total_old_principal: u64,
    pub new_principal: u64,
    pub consolidation_fee: u64,
    pub new_due_date: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanSplitEvent {
    pub old_loan_id: u64,
    pub new_loan_ids: Vec<u64>,
    pub borrower: Address,
    pub old_principal: u64,
    pub new_principals: Vec<u64>,
    pub split_fee: u64,
    pub timestamp: u64,
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
    InsufficientCollateral = 11,
    CollateralNotWhitelisted = 12,
    UtilizationCapExceeded = 13,
    ReentrantCall = 14,
    FlashLoanNotRepaid = 15,
    CannotRefinance = 16,
    InvalidRefinanceTerms = 17,
    LoanNotFound = 18,
    TooManyLoans = 19,
    InvalidSplitAmounts = 20,
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
    NextLoanId,
    LoanById(u64),
    CollateralRatio,
    WhitelistedCollateral(Address),
    NFTToken,
    ReentrancyGuard,
    LateFeesAccrued(u64), // Track late fees for a specific loan_id
    FlashLoanFeeBps,
    UserLoans(Address), // Track multiple loans per user (Vec<u64>)
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
        collateral_ratio_bps: u32,
        utilization_cap_bps: u32,
    ) -> Result<(), LendingError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(LendingError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::CollateralRatio, &collateral_ratio_bps);
        env.storage().instance().set(
            &DataKey::Pool,
            &PoolState {
                total_deposits: 0,
                total_shares: 0,
                total_borrowed: 0,
                base_rate_bps,
                multiplier_bps,
                utilization_cap_bps,
                retained_yield: 0,
                bad_debt_reserve: 0,
                grace_period_seconds: DEFAULT_GRACE_PERIOD_SECONDS,
                late_fee_rate_bps: DEFAULT_LATE_FEE_RATE_BPS,
            },
        );
        Ok(())
    }

    pub fn set_nft_token(env: Env, admin: Address, nft_token: Address) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::NFTToken, &nft_token);
        Ok(())
    }

    fn enter_reentrancy_guard(env: &Env) -> Result<(), LendingError> {
        if env.storage().instance().has(&DataKey::ReentrancyGuard) {
            return Err(LendingError::ReentrantCall);
        }
        env.storage()
            .instance()
            .set(&DataKey::ReentrancyGuard, &true);
        Ok(())
    }

    fn exit_reentrancy_guard(env: &Env) {
        env.storage().instance().remove(&DataKey::ReentrancyGuard);
    }

    fn get_nft_token(env: &Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::NFTToken)
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

    fn get_next_loan_id(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextLoanId)
            .unwrap_or(1u64)
    }

    fn increment_loan_id(env: &Env) -> u64 {
        let current = Self::get_next_loan_id(env);
        env.storage()
            .instance()
            .set(&DataKey::NextLoanId, &(current + 1));
        current
    }

    fn get_collateral_ratio(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CollateralRatio)
            .unwrap_or(15000u32) // Default 150%
    }

    fn is_collateral_whitelisted(env: &Env, token: &Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::WhitelistedCollateral(token.clone()))
            .unwrap_or(false)
    }

    fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    fn get_user_loans(env: &Env, user: &Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::UserLoans(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    fn add_user_loan(env: &Env, user: &Address, loan_id: u64) {
        let mut loans = Self::get_user_loans(env, user);
        loans.push_back(loan_id);
        env.storage()
            .persistent()
            .set(&DataKey::UserLoans(user.clone()), &loans);
    }

    fn remove_user_loan(env: &Env, user: &Address, loan_id: u64) {
        let loans = Self::get_user_loans(env, user);
        let mut new_loans = Vec::new(env);
        for id in loans.iter() {
            if id != loan_id {
                new_loans.push_back(id);
            }
        }
        if new_loans.is_empty() {
            env.storage()
                .persistent()
                .remove(&DataKey::UserLoans(user.clone()));
        } else {
            env.storage()
                .persistent()
                .set(&DataKey::UserLoans(user.clone()), &new_loans);
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), LendingError> {
        caller.require_auth();
        let admin = Self::get_admin(env).ok_or(LendingError::NotAdmin)?;
        if *caller != admin {
            return Err(LendingError::NotAdmin);
        }
        Ok(())
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
        Self::enter_reentrancy_guard(&env)?;
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
        Self::exit_reentrancy_guard(&env);
        Ok(shares)
    }

    /// Burn `shares` and return the proportional underlying tokens to the depositor.
    /// Reverts if insufficient liquidity (i.e., tokens are loaned out).
    pub fn withdraw(env: Env, depositor: Address, shares: u64) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
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
        Self::exit_reentrancy_guard(&env);
        Ok(amount)
    }

    /// Borrow `amount` of the underlying token from the pool with collateral.
    /// Requires overcollateralized borrowing based on collateral ratio.
    /// Returns the unique loan ID.
    pub fn borrow(
        env: Env,
        borrower: Address,
        amount: u64,
        collateral_token: Address,
        collateral_amount: u64,
        duration_seconds: u64,
    ) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        borrower.require_auth();

        if amount == 0 || collateral_amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        // Check collateral token is whitelisted
        if !Self::is_collateral_whitelisted(&env, &collateral_token) {
            return Err(LendingError::CollateralNotWhitelisted);
        }

        // Check if borrower already has existing loans
        let existing_loans = Self::get_user_loans(&env, &borrower);
        if !existing_loans.is_empty() {
            return Err(LendingError::LoanAlreadyExists);
        }

        // Check collateral ratio (collateral_amount must be >= amount * ratio / 10000)
        let required_collateral = (amount as u128)
            .checked_mul(Self::get_collateral_ratio(&env) as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as u64;

        if collateral_amount < required_collateral {
            return Err(LendingError::InsufficientCollateral);
        }

        let mut pool = Self::get_pool(&env);
        let available = pool.total_deposits.saturating_sub(pool.total_borrowed);
        if amount > available {
            return Err(LendingError::InsufficientLiquidity);
        }

        // Check utilization cap
        let new_borrowed = pool.total_borrowed + amount;
        let new_utilization_bps = Self::get_utilization_bps(new_borrowed, pool.total_deposits);
        if new_utilization_bps > pool.utilization_cap_bps {
            return Err(LendingError::UtilizationCapExceeded);
        }

        // Transfer collateral from borrower to contract
        let contract_id = env.current_contract_address();
        Self::transfer(
            &env,
            &collateral_token,
            &borrower,
            &contract_id,
            collateral_amount,
        )?;

        pool.total_borrowed += amount;

        let utilization_bps = Self::get_utilization_bps(pool.total_borrowed, pool.total_deposits);
        let dynamic_rate_bps =
            Self::calculate_dynamic_rate(pool.base_rate_bps, pool.multiplier_bps, utilization_bps);

        Self::set_pool(&env, &pool);

        let loan_id = Self::increment_loan_id(&env);
        let borrow_time = env.ledger().timestamp();
        let due_date = borrow_time + duration_seconds;

        let loan = LoanRecord {
            loan_id,
            borrower: borrower.clone(),
            principal: amount,
            collateral_amount,
            collateral_token: collateral_token.clone(),
            borrow_time,
            due_date,
            interest_rate_bps: dynamic_rate_bps,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Loan(borrower.clone()), &loan);
        env.storage()
            .persistent()
            .set(&DataKey::LoanById(loan_id), &loan);
        Self::add_user_loan(&env, &borrower, loan_id);

        // Mint NFT if token is set
        if let Some(nft_token) = Self::get_nft_token(&env) {
            let nft_client = LoanNFTClient::new(&env, &nft_token);
            nft_client.mint(
                &borrower,
                &LoanMetadata {
                    borrower: borrower.clone(),
                    collateral_amount,
                    collateral_token: collateral_token.clone(),
                    due_date,
                    loan_id,
                    principal: amount,
                },
            );
        }

        let token = Self::get_token(&env);
        Self::transfer(&env, &token, &contract_id, &borrower, amount)?;

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("BORROW")),
            BorrowEvent {
                loan_id,
                borrower: borrower.clone(),
                amount,
                collateral_amount,
                due_date,
            },
        );
        env.events().publish(
            (symbol_short!("COLL"), symbol_short!("DEPOSIT")),
            CollateralDepositEvent {
                loan_id,
                borrower: borrower.clone(),
                collateral_token,
                amount: collateral_amount,
            },
        );
        log!(
            &env,
            "Loan {} created: {} tokens with {} collateral",
            loan_id,
            amount,
            collateral_amount
        );
        Self::exit_reentrancy_guard(&env);
        Ok(loan_id)
    }

    /// Repay the full outstanding loan for the caller.
    /// Restores liquidity to the pool, returns collateral, and closes the loan record.
    /// Includes principal, interest, and any accumulated late fees in the repayment.
    /// Returns the total amount repaid (principal + interest + late fees).
    pub fn repay(env: Env, borrower: Address) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        borrower.require_auth();

        let loan: LoanRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower.clone()))
            .ok_or(LendingError::NoOpenLoan)?;

        let elapsed = env.ledger().timestamp().saturating_sub(loan.borrow_time);
        let interest = Self::calculate_interest(loan.principal, loan.interest_rate_bps, elapsed);
        let late_fee = Self::calculate_late_fee(env.clone(), borrower.clone())?;
        let total_repayment = loan.principal + interest + late_fee;

        let token = Self::get_token(&env);
        let contract_id = env.current_contract_address();
        Self::transfer(&env, &token, &borrower, &contract_id, total_repayment)?;

        // Return collateral to borrower
        Self::transfer(
            &env,
            &loan.collateral_token,
            &contract_id,
            &borrower,
            loan.collateral_amount,
        )?;

        let mut pool = Self::get_pool(&env);
        pool.total_borrowed -= loan.principal;

        // Retain 10% of interest for protocol buckets, with part routed to bad-debt reserve.
        let protocol_share = ((interest as u128)
            .checked_mul(PROTOCOL_INTEREST_BPS as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0)) as u64;
        let reserve_share = ((protocol_share as u128)
            .checked_mul(BAD_DEBT_RESERVE_BPS as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0)) as u64;
        let retained_share = protocol_share.saturating_sub(reserve_share);
        let pool_share = interest - protocol_share;

        // Late fees go entirely to retained_yield (protocol reserve)
        pool.total_deposits += pool_share; // Interest increases pool value for share holders
        pool.retained_yield += retained_share + late_fee;
        pool.bad_debt_reserve += reserve_share;
        Self::set_pool(&env, &pool);

        env.storage()
            .persistent()
            .remove(&DataKey::Loan(borrower.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::LoanById(loan.loan_id));
        Self::remove_user_loan(&env, &borrower, loan.loan_id);
        env.storage()
            .persistent()
            .remove(&DataKey::LateFeesAccrued(loan.loan_id));

        // Burn NFT if token is set
        if let Some(nft_token) = Self::get_nft_token(&env) {
            let nft_client = LoanNFTClient::new(&env, &nft_token);
            nft_client.burn(&loan.loan_id);
        }

        // Emit late fee event if any late fees were charged
        if late_fee > 0 {
            let current_time = env.ledger().timestamp();
            let grace_period_end = loan.due_date + pool.grace_period_seconds;
            let days_overdue = (current_time - grace_period_end) / (24 * 60 * 60);

            env.events().publish(
                (symbol_short!("POOL"), symbol_short!("LATEFEE")),
                LateFeeChargedEvent {
                    loan_id: loan.loan_id,
                    borrower: borrower.clone(),
                    late_fee,
                    days_overdue,
                    total_with_late_fees: total_repayment,
                    timestamp: current_time,
                },
            );
        }

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("REPAY")),
            RepayEvent {
                loan_id: loan.loan_id,
                borrower: borrower.clone(),
                principal: loan.principal,
                interest,
                total_amount: total_repayment,
                collateral_returned: loan.collateral_amount,
            },
        );
        log!(
            &env,
            "Loan {} repaid: {} total ({} principal + {} interest + {} late fees), {} collateral returned",
            loan.loan_id,
            total_repayment,
            loan.principal,
            interest,
            late_fee,
            loan.collateral_amount
        );
        Self::exit_reentrancy_guard(&env);
        Ok(total_repayment)
    }

    /// Calculate the total amount (principal + interest + late fees) required to repay the loan.
    pub fn get_repayment_amount(env: Env, borrower: Address) -> Result<u64, LendingError> {
        let loan_opt: Option<LoanRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower.clone()));

        match loan_opt {
            Some(loan) => {
                let elapsed = env.ledger().timestamp().saturating_sub(loan.borrow_time);
                let interest =
                    Self::calculate_interest(loan.principal, loan.interest_rate_bps, elapsed);
                let late_fee = Self::calculate_late_fee(env, borrower)?;
                Ok(loan.principal + interest + late_fee)
            }
            None => Err(LendingError::NoOpenLoan),
        }
    }

    /// Calculate and emit an interest accrual event for a specific loan
    pub fn emit_interest_accrual(env: Env, borrower: Address) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;

        let loan_opt: Option<LoanRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower.clone()));

        match loan_opt {
            Some(loan) => {
                let elapsed = env.ledger().timestamp().saturating_sub(loan.borrow_time);
                let interest =
                    Self::calculate_interest(loan.principal, loan.interest_rate_bps, elapsed);

                env.events().publish(
                    (symbol_short!("POOL"), symbol_short!("INTEREST")),
                    InterestAccrualEvent {
                        loan_id: loan.loan_id,
                        borrower: borrower.clone(),
                        principal: loan.principal,
                        interest_accrued: interest,
                        interest_rate_bps: loan.interest_rate_bps,
                        elapsed_seconds: elapsed,
                        timestamp: env.ledger().timestamp(),
                    },
                );

                log!(
                    &env,
                    "Interest accrued for loan {}: {} interest on {} principal",
                    loan.loan_id,
                    interest,
                    loan.principal
                );

                Ok(interest)
            }
            None => Err(LendingError::NoOpenLoan),
        }
    }

    /// Withdraw prioritized funds from the retained yield.
    /// Used by authorized contracts (like InheritanceContract) to fulfill priority claims.
    pub fn withdraw_priority(env: Env, caller: Address, amount: u64) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        caller.require_auth();

        // In a real implementation, we should restrict this to authorized contracts only.
        // For now, we rely on the caller being trusted or admin.

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let mut pool = Self::get_pool(&env);

        if amount > pool.retained_yield {
            return Err(LendingError::InsufficientLiquidity);
        }

        pool.retained_yield -= amount;
        Self::set_pool(&env, &pool);

        let token = Self::get_token(&env);
        let contract_id = env.current_contract_address();
        Self::transfer(&env, &token, &contract_id, &caller, amount)?;

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("PRIORITY")),
            PriorityWithdrawEvent {
                caller: caller.clone(),
                amount,
            },
        );
        log!(&env, "Priority withdrawal {} tokens by {}", amount, caller);
        Self::exit_reentrancy_guard(&env);
        Ok(amount)
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

    /// Returns the loan record by unique loan ID, if any.
    pub fn get_loan_by_id(env: Env, loan_id: u64) -> Option<LoanRecord> {
        env.storage().persistent().get(&DataKey::LoanById(loan_id))
    }

    /// Returns all loan IDs for a given user
    pub fn get_user_loan_ids(env: Env, user: Address) -> Vec<u64> {
        Self::get_user_loans(&env, &user)
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

    // ─── Grace Period & Late Fee Functions ────────────

    /// Check if a loan is currently in its grace period
    pub fn is_in_grace_period(env: Env, borrower: Address) -> Result<bool, LendingError> {
        Self::require_initialized(&env)?;

        let loan: LoanRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower))
            .ok_or(LendingError::NoOpenLoan)?;

        let pool = Self::get_pool(&env);
        let current_time = env.ledger().timestamp();
        let grace_period_end = loan.due_date + pool.grace_period_seconds;

        Ok(current_time <= grace_period_end)
    }

    /// Calculate late fees accumulated on a loan
    /// Daily late fee rate applied to days overdue after grace period
    pub fn calculate_late_fee(env: Env, borrower: Address) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;

        let loan: LoanRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower.clone()))
            .ok_or(LendingError::NoOpenLoan)?;

        let pool = Self::get_pool(&env);
        let current_time = env.ledger().timestamp();
        let grace_period_end = loan.due_date + pool.grace_period_seconds;

        if current_time <= grace_period_end {
            return Ok(0);
        }

        let days_overdue = (current_time - grace_period_end) / (24 * 60 * 60);
        if days_overdue == 0 {
            return Ok(0);
        }

        // Look up any previously accrued late fees for this loan
        let accrued_fees: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::LateFeesAccrued(loan.loan_id))
            .unwrap_or(0u64);

        if accrued_fees > 0 {
            return Ok(accrued_fees);
        }

        // Calculate new late fees: principal * rate_per_day * days_overdue / 10000
        let daily_fee = ((loan.principal as u128)
            .checked_mul(pool.late_fee_rate_bps as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0)) as u64;

        let total_late_fee = (daily_fee as u128)
            .checked_mul(days_overdue as u128)
            .unwrap_or(0) as u64;

        Ok(total_late_fee)
    }

    /// Get total repayment amount including principal, interest, and late fees
    pub fn get_total_due_with_late_fees(env: Env, borrower: Address) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;

        let loan: LoanRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower.clone()))
            .ok_or(LendingError::NoOpenLoan)?;

        let elapsed = env.ledger().timestamp().saturating_sub(loan.borrow_time);
        let interest = Self::calculate_interest(loan.principal, loan.interest_rate_bps, elapsed);
        let late_fee = Self::calculate_late_fee(env, borrower)?;

        Ok(loan.principal + interest + late_fee)
    }

    // ─── Admin Functions ─────────────────────────────

    /// Whitelist a collateral token (admin only)
    pub fn whitelist_collateral(
        env: Env,
        admin: Address,
        token: Address,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::WhitelistedCollateral(token), &true);
        Ok(())
    }

    /// Remove a collateral token from whitelist (admin only)
    pub fn remove_collateral(env: Env, admin: Address, token: Address) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .remove(&DataKey::WhitelistedCollateral(token));
        Ok(())
    }

    /// Check if a token is whitelisted
    pub fn is_whitelisted(env: Env, token: Address) -> bool {
        Self::is_collateral_whitelisted(&env, &token)
    }

    /// Get the current collateral ratio in basis points
    pub fn get_collateral_ratio_bps(env: Env) -> u32 {
        Self::get_collateral_ratio(&env)
    }

    /// Set the grace period for loans (admin only)
    /// Grace period is the time after due date during which no late fees accrue
    pub fn set_grace_period(
        env: Env,
        admin: Address,
        grace_period_seconds: u64,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;

        let mut pool = Self::get_pool(&env);
        pool.grace_period_seconds = grace_period_seconds;
        Self::set_pool(&env, &pool);

        log!(
            &env,
            "Grace period updated to {} seconds",
            grace_period_seconds
        );
        Ok(())
    }

    /// Set the late fee rate for loans (admin only)
    /// Late fee rate is in basis points per day (e.g., 500 = 5% per day)
    pub fn set_late_fee_rate(
        env: Env,
        admin: Address,
        late_fee_rate_bps: u32,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;

        let mut pool = Self::get_pool(&env);
        pool.late_fee_rate_bps = late_fee_rate_bps;
        Self::set_pool(&env, &pool);

        log!(
            &env,
            "Late fee rate updated to {} bps per day",
            late_fee_rate_bps
        );
        Ok(())
    }

    /// Get the current grace period in seconds
    pub fn get_grace_period(env: Env) -> u64 {
        let pool = Self::get_pool(&env);
        pool.grace_period_seconds
    }

    /// Get the current late fee rate in basis points per day
    pub fn get_late_fee_rate(env: Env) -> u32 {
        let pool = Self::get_pool(&env);
        pool.late_fee_rate_bps
    }

    pub fn get_flash_loan_fee(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::FlashLoanFeeBps)
            .unwrap_or(9u32) // Default to 0.09% = 9 bps
    }

    pub fn set_flash_loan_fee(env: Env, admin: Address, fee_bps: u32) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        env.storage()
            .instance()
            .set(&DataKey::FlashLoanFeeBps, &fee_bps);
        Ok(())
    }

    pub fn flash_loan(env: Env, receiver_id: Address, amount: u64) -> Result<(), LendingError> {
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let mut pool = Self::get_pool(&env);
        let available = pool.total_deposits.saturating_sub(pool.total_borrowed);
        if amount > available {
            return Err(LendingError::InsufficientLiquidity);
        }

        let fee_bps = Self::get_flash_loan_fee(env.clone());
        let fee = (amount as u128)
            .checked_mul(fee_bps as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as u64;

        let token_addr = Self::get_token(&env);
        let contract_id = env.current_contract_address();

        let token_client = token::Client::new(&env, &token_addr);
        let balance_before = token_client.balance(&contract_id);

        // 1. Transfer to receiver
        token_client.transfer(&contract_id, &receiver_id, &(amount as i128));

        // 2. Call execute_operation on receiver
        let receiver_client = FlashLoanReceiverClient::new(&env, &receiver_id);
        receiver_client.execute_operation(&amount, &fee, &receiver_id);

        // 3. Ensure repayment
        let balance_after = token_client.balance(&contract_id);
        let required_balance = balance_before + (fee as i128);

        if balance_after < required_balance {
            return Err(LendingError::FlashLoanNotRepaid);
        }

        pool.total_deposits += fee;
        Self::set_pool(&env, &pool);

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("FLASHL")),
            FlashLoanEvent {
                receiver: receiver_id,
                amount,
                fee,
            },
        );

        Self::exit_reentrancy_guard(&env);
        Ok(())
    }

    /// Get the refinancing fee rate in basis points
    pub fn get_refinancing_fee_rate() -> u32 {
        REFINANCING_FEE_BPS
    }

    /// Liquidate an underwater loan by paying part of the debt and seizing collateral
    /// Only callable if the loan's health factor is below a safe threshold AND grace period has expired
    pub fn liquidate(
        env: Env,
        liquidator: Address,
        borrower: Address,
        amount: u64,
    ) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        liquidator.require_auth();

        let loan: LoanRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower.clone()))
            .ok_or(LendingError::NoOpenLoan)?;

        if amount == 0 || amount > loan.principal {
            return Err(LendingError::InvalidAmount);
        }

        // Check if grace period has expired before allowing liquidation
        let is_in_grace = Self::is_in_grace_period(env.clone(), borrower.clone())?;
        if is_in_grace {
            return Err(LendingError::InvalidAmount);
        }

        // Calculate health factor (collateral / debt ratio)
        let health_factor = (loan.collateral_amount as u128)
            .checked_mul(10000)
            .and_then(|v| v.checked_div(loan.principal as u128))
            .unwrap_or(0) as u32;

        // Allow liquidation if health factor is below 150% (15000 basis points)
        let liquidation_threshold_bps = 15000u32;
        if health_factor >= liquidation_threshold_bps {
            return Err(LendingError::InvalidAmount);
        }

        // Calculate collateral to seize (with small penalty/bonus to liquidator)
        let collateral_to_seize = (amount as u128)
            .checked_mul(15000) // 150% of the amount repaid
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(amount as u128) as u64;

        if collateral_to_seize > loan.collateral_amount {
            return Err(LendingError::InvalidAmount);
        }

        let token = Self::get_token(&env);
        let contract_id = env.current_contract_address();

        // Transfer debt payment from liquidator to contract
        Self::transfer(&env, &token, &liquidator, &contract_id, amount)?;

        // Transfer collateral from contract to liquidator
        Self::transfer(
            &env,
            &loan.collateral_token,
            &contract_id,
            &liquidator,
            collateral_to_seize,
        )?;

        let mut pool = Self::get_pool(&env);
        pool.total_borrowed = pool.total_borrowed.saturating_sub(amount);
        pool.total_deposits += amount;
        Self::set_pool(&env, &pool);

        // Emit liquidation event
        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("LIQUIDATE")),
            LiquidationEvent {
                loan_id: loan.loan_id,
                borrower: borrower.clone(),
                liquidator: liquidator.clone(),
                amount_repaid: amount,
                collateral_seized: collateral_to_seize,
                health_factor,
            },
        );

        log!(
            &env,
            "Loan {} liquidated: {} repaid, {} collateral seized",
            loan.loan_id,
            amount,
            collateral_to_seize
        );

        Self::exit_reentrancy_guard(&env);
        Ok(collateral_to_seize)
    }

    // ─── Refinancing Functions ───────────────────────

    /// Calculate outstanding balance for a loan (principal + accrued interest)
    fn calculate_outstanding_balance(env: &Env, loan: &LoanRecord) -> u64 {
        let elapsed = env.ledger().timestamp().saturating_sub(loan.borrow_time);
        let interest = Self::calculate_interest(loan.principal, loan.interest_rate_bps, elapsed);
        loan.principal + interest
    }

    /// Get refinancing terms for an existing loan
    pub fn get_refinance_terms(
        env: Env,
        borrower: Address,
        new_duration_seconds: u64,
    ) -> Result<RefinanceTerms, LendingError> {
        Self::require_initialized(&env)?;

        let loan: LoanRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower.clone()))
            .ok_or(LendingError::NoOpenLoan)?;

        let outstanding_balance = Self::calculate_outstanding_balance(&env, &loan);
        let refinancing_fee = ((outstanding_balance as u128)
            .checked_mul(REFINANCING_FEE_BPS as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0)) as u64;

        let new_principal = outstanding_balance + refinancing_fee;
        let total_required = new_principal;

        let current_time = env.ledger().timestamp();
        let new_due_date = current_time + new_duration_seconds;

        let pool = Self::get_pool(&env);
        let utilization_bps = Self::get_utilization_bps(pool.total_borrowed, pool.total_deposits);
        let new_interest_rate_bps =
            Self::calculate_dynamic_rate(pool.base_rate_bps, pool.multiplier_bps, utilization_bps);

        Ok(RefinanceTerms {
            outstanding_balance,
            new_principal,
            refinancing_fee,
            total_required,
            new_interest_rate_bps,
            new_duration_seconds,
            new_due_date,
        })
    }

    /// Refinance an existing loan with new terms
    pub fn refinance_loan(
        env: Env,
        borrower: Address,
        new_duration_seconds: u64,
    ) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        borrower.require_auth();

        let old_loan: LoanRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower.clone()))
            .ok_or(LendingError::NoOpenLoan)?;

        // Cannot refinance if currently in grace period or overdue
        let is_in_grace = Self::is_in_grace_period(env.clone(), borrower.clone())?;
        if !is_in_grace {
            return Err(LendingError::CannotRefinance);
        }

        let terms = Self::get_refinance_terms(env.clone(), borrower.clone(), new_duration_seconds)?;

        // Check if borrower has enough tokens to pay refinancing fee
        let token = Self::get_token(&env);
        let contract_id = env.current_contract_address();

        // Transfer refinancing fee from borrower to contract
        Self::transfer(&env, &token, &borrower, &contract_id, terms.refinancing_fee)?;

        // Close old loan
        env.storage()
            .persistent()
            .remove(&DataKey::Loan(borrower.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::LoanById(old_loan.loan_id));
        Self::remove_user_loan(&env, &borrower, old_loan.loan_id);

        // Burn old NFT if token is set
        if let Some(nft_token) = Self::get_nft_token(&env) {
            let nft_client = LoanNFTClient::new(&env, &nft_token);
            nft_client.burn(&old_loan.loan_id);
        }

        // Create new loan with updated terms
        let new_loan_id = Self::increment_loan_id(&env);
        let current_time = env.ledger().timestamp();

        let new_loan = LoanRecord {
            loan_id: new_loan_id,
            borrower: borrower.clone(),
            principal: terms.new_principal,
            collateral_amount: old_loan.collateral_amount,
            collateral_token: old_loan.collateral_token.clone(),
            borrow_time: current_time,
            due_date: terms.new_due_date,
            interest_rate_bps: terms.new_interest_rate_bps,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Loan(borrower.clone()), &new_loan);
        env.storage()
            .persistent()
            .set(&DataKey::LoanById(new_loan_id), &new_loan);
        Self::add_user_loan(&env, &borrower, new_loan_id);

        // Mint new NFT if token is set
        if let Some(nft_token) = Self::get_nft_token(&env) {
            let nft_client = LoanNFTClient::new(&env, &nft_token);
            nft_client.mint(
                &borrower,
                &LoanMetadata {
                    borrower: borrower.clone(),
                    collateral_amount: new_loan.collateral_amount,
                    collateral_token: new_loan.collateral_token.clone(),
                    due_date: new_loan.due_date,
                    loan_id: new_loan_id,
                    principal: new_loan.principal,
                },
            );
        }

        // Add refinancing fee to retained yield
        let mut pool = Self::get_pool(&env);
        pool.retained_yield += terms.refinancing_fee;
        Self::set_pool(&env, &pool);

        // Emit refinancing event
        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("REFINANCE")),
            LoanRefinancedEvent {
                old_loan_id: old_loan.loan_id,
                new_loan_id,
                borrower: borrower.clone(),
                old_principal: old_loan.principal,
                new_principal: terms.new_principal,
                refinancing_fee: terms.refinancing_fee,
                old_interest_rate_bps: old_loan.interest_rate_bps,
                new_interest_rate_bps: terms.new_interest_rate_bps,
                old_due_date: old_loan.due_date,
                new_due_date: terms.new_due_date,
                timestamp: current_time,
            },
        );

        log!(
            &env,
            "Loan {} refinanced to {} with fee {}",
            old_loan.loan_id,
            new_loan_id,
            terms.refinancing_fee
        );

        Self::exit_reentrancy_guard(&env);
        Ok(new_loan_id)
    }

    /// Consolidate multiple loans into a single new loan
    pub fn consolidate_loans(
        env: Env,
        borrower: Address,
        loan_ids: Vec<u64>,
        new_duration_seconds: u64,
    ) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        borrower.require_auth();

        if loan_ids.is_empty() || loan_ids.len() > 10 {
            return Err(LendingError::InvalidAmount);
        }

        let mut total_outstanding = 0u64;
        let mut total_collateral = 0u64;
        let mut collateral_token: Option<Address> = None;
        let mut old_loans = Vec::new(&env);

        // Validate all loans belong to borrower and calculate totals
        for loan_id in loan_ids.iter() {
            let loan: LoanRecord = env
                .storage()
                .persistent()
                .get(&DataKey::LoanById(loan_id))
                .ok_or(LendingError::LoanNotFound)?;

            if loan.borrower != borrower {
                return Err(LendingError::Unauthorized);
            }

            // Check if this specific loan is overdue (cannot consolidate overdue loans)
            let loan_grace_end = loan.due_date + Self::get_pool(&env).grace_period_seconds;
            let current_time = env.ledger().timestamp();
            if current_time > loan_grace_end {
                return Err(LendingError::CannotRefinance);
            }

            let outstanding = Self::calculate_outstanding_balance(&env, &loan);
            total_outstanding += outstanding;
            total_collateral += loan.collateral_amount;

            if collateral_token.is_none() {
                collateral_token = Some(loan.collateral_token.clone());
            } else if collateral_token.as_ref() != Some(&loan.collateral_token) {
                return Err(LendingError::InvalidRefinanceTerms); // All collateral tokens must be the same
            }

            old_loans.push_back(loan);
        }

        let consolidation_fee = ((total_outstanding as u128)
            .checked_mul(REFINANCING_FEE_BPS as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0)) as u64;

        let new_principal = total_outstanding + consolidation_fee;

        // Transfer consolidation fee
        let token = Self::get_token(&env);
        let contract_id = env.current_contract_address();
        Self::transfer(&env, &token, &borrower, &contract_id, consolidation_fee)?;

        // Remove old loans
        for loan in old_loans.iter() {
            env.storage()
                .persistent()
                .remove(&DataKey::Loan(loan.borrower.clone()));
            env.storage()
                .persistent()
                .remove(&DataKey::LoanById(loan.loan_id));
            Self::remove_user_loan(&env, &borrower, loan.loan_id);

            // Burn old NFTs
            if let Some(nft_token) = Self::get_nft_token(&env) {
                let nft_client = LoanNFTClient::new(&env, &nft_token);
                nft_client.burn(&loan.loan_id);
            }
        }

        // Create new consolidated loan
        let new_loan_id = Self::increment_loan_id(&env);
        let current_time = env.ledger().timestamp();
        let new_due_date = current_time + new_duration_seconds;

        let pool = Self::get_pool(&env);
        let utilization_bps = Self::get_utilization_bps(pool.total_borrowed, pool.total_deposits);
        let new_interest_rate_bps =
            Self::calculate_dynamic_rate(pool.base_rate_bps, pool.multiplier_bps, utilization_bps);

        let new_loan = LoanRecord {
            loan_id: new_loan_id,
            borrower: borrower.clone(),
            principal: new_principal,
            collateral_amount: total_collateral,
            collateral_token: collateral_token.unwrap(),
            borrow_time: current_time,
            due_date: new_due_date,
            interest_rate_bps: new_interest_rate_bps,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Loan(borrower.clone()), &new_loan);
        env.storage()
            .persistent()
            .set(&DataKey::LoanById(new_loan_id), &new_loan);
        Self::add_user_loan(&env, &borrower, new_loan_id);

        // Mint new NFT
        if let Some(nft_token) = Self::get_nft_token(&env) {
            let nft_client = LoanNFTClient::new(&env, &nft_token);
            nft_client.mint(
                &borrower,
                &LoanMetadata {
                    borrower: borrower.clone(),
                    collateral_amount: new_loan.collateral_amount,
                    collateral_token: new_loan.collateral_token.clone(),
                    due_date: new_loan.due_date,
                    loan_id: new_loan_id,
                    principal: new_loan.principal,
                },
            );
        }

        // Add fee to retained yield
        let mut pool = Self::get_pool(&env);
        pool.retained_yield += consolidation_fee;
        Self::set_pool(&env, &pool);

        // Emit consolidation event
        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("CONSOLID")),
            LoansConsolidatedEvent {
                old_loan_ids: loan_ids.clone(),
                new_loan_id,
                borrower: borrower.clone(),
                total_old_principal: total_outstanding,
                new_principal,
                consolidation_fee,
                new_due_date,
                timestamp: current_time,
            },
        );

        log!(
            &env,
            "Consolidated {} loans into {} with fee {}",
            loan_ids.len(),
            new_loan_id,
            consolidation_fee
        );

        Self::exit_reentrancy_guard(&env);
        Ok(new_loan_id)
    }

    /// Split a loan into multiple smaller loans
    pub fn split_loan(
        env: Env,
        borrower: Address,
        split_amounts: Vec<u64>,
        new_duration_seconds: u64,
    ) -> Result<Vec<u64>, LendingError> {
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        borrower.require_auth();

        if split_amounts.is_empty() || split_amounts.len() > 5 {
            return Err(LendingError::InvalidAmount);
        }

        let old_loan: LoanRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(borrower.clone()))
            .ok_or(LendingError::NoOpenLoan)?;

        // Check if loan is in good standing
        let is_in_grace = Self::is_in_grace_period(env.clone(), borrower.clone())?;
        if !is_in_grace {
            return Err(LendingError::CannotRefinance);
        }

        let outstanding = Self::calculate_outstanding_balance(&env, &old_loan);
        let total_split_amount: u64 = split_amounts.iter().sum();

        if total_split_amount != outstanding {
            return Err(LendingError::InvalidSplitAmounts);
        }

        let split_fee = ((outstanding as u128)
            .checked_mul(REFINANCING_FEE_BPS as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0)) as u64;

        // Transfer split fee
        let token = Self::get_token(&env);
        let contract_id = env.current_contract_address();
        Self::transfer(&env, &token, &borrower, &contract_id, split_fee)?;

        // Remove old loan
        env.storage()
            .persistent()
            .remove(&DataKey::Loan(borrower.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::LoanById(old_loan.loan_id));
        Self::remove_user_loan(&env, &borrower, old_loan.loan_id);

        // Burn old NFT
        if let Some(nft_token) = Self::get_nft_token(&env) {
            let nft_client = LoanNFTClient::new(&env, &nft_token);
            nft_client.burn(&old_loan.loan_id);
        }

        // Create new split loans
        let mut new_loan_ids = Vec::new(&env);
        let current_time = env.ledger().timestamp();
        let new_due_date = current_time + new_duration_seconds;

        let pool = Self::get_pool(&env);
        let utilization_bps = Self::get_utilization_bps(pool.total_borrowed, pool.total_deposits);
        let new_interest_rate_bps =
            Self::calculate_dynamic_rate(pool.base_rate_bps, pool.multiplier_bps, utilization_bps);

        // Distribute collateral proportionally
        for amount in split_amounts.iter() {
            let collateral_ratio = (amount as u128)
                .checked_mul(10000)
                .and_then(|v| v.checked_div(outstanding as u128))
                .unwrap_or(0);
            let collateral_amount = ((old_loan.collateral_amount as u128)
                .checked_mul(collateral_ratio)
                .and_then(|v| v.checked_div(10000))
                .unwrap_or(0)) as u64;

            let new_loan_id = Self::increment_loan_id(&env);
            let new_loan = LoanRecord {
                loan_id: new_loan_id,
                borrower: borrower.clone(),
                principal: amount,
                collateral_amount,
                collateral_token: old_loan.collateral_token.clone(),
                borrow_time: current_time,
                due_date: new_due_date,
                interest_rate_bps: new_interest_rate_bps,
            };

            // For split loans, only store the last one as the primary loan
            // but all loans are accessible via LoanById
            env.storage()
                .persistent()
                .set(&DataKey::Loan(borrower.clone()), &new_loan);
            env.storage()
                .persistent()
                .set(&DataKey::LoanById(new_loan_id), &new_loan);
            Self::add_user_loan(&env, &borrower, new_loan_id);

            // Mint NFT for each new loan
            if let Some(nft_token) = Self::get_nft_token(&env) {
                let nft_client = LoanNFTClient::new(&env, &nft_token);
                nft_client.mint(
                    &borrower,
                    &LoanMetadata {
                        borrower: borrower.clone(),
                        collateral_amount: new_loan.collateral_amount,
                        collateral_token: new_loan.collateral_token.clone(),
                        due_date: new_loan.due_date,
                        loan_id: new_loan_id,
                        principal: new_loan.principal,
                    },
                );
            }

            new_loan_ids.push_back(new_loan_id);
        }

        // Add fee to retained yield
        let mut pool = Self::get_pool(&env);
        pool.retained_yield += split_fee;
        Self::set_pool(&env, &pool);

        // Emit split event
        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("SPLIT")),
            LoanSplitEvent {
                old_loan_id: old_loan.loan_id,
                new_loan_ids: new_loan_ids.clone(),
                borrower: borrower.clone(),
                old_principal: old_loan.principal,
                new_principals: split_amounts,
                split_fee,
                timestamp: current_time,
            },
        );

        log!(
            &env,
            "Split loan {} into {} loans with fee {}",
            old_loan.loan_id,
            new_loan_ids.len(),
            split_fee
        );

        Self::exit_reentrancy_guard(&env);
        Ok(new_loan_ids)
    }
}

mod test;

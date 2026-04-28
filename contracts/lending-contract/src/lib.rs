#![no_std]
use access_control::{self, Role};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, token, vec, Address,
    Env, IntoVal, InvokeError, Val, Vec,
};

mod reserves;

// ─────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────

const MINIMUM_LIQUIDITY: u64 = 1000;
const PROTOCOL_INTEREST_BPS: u32 = 1000; // 10% of interest retained by protocol
const BAD_DEBT_RESERVE_BPS: u32 = 5000; // 50% of protocol share routed to reserve
const DEFAULT_GRACE_PERIOD_SECONDS: u64 = 259_200; // 3 days
const DEFAULT_LATE_FEE_RATE_BPS: u32 = 500; // 5% per day = 0.058% per second (approx)
const REFINANCING_FEE_BPS: u32 = 50; // 0.5% refinancing fee
const DEFAULT_REWARD_RATE: u64 = 1_000_000_000; // Default reward rate per second (1 reward per second with 9 decimals)
const REWARD_PRECISION: u64 = 1_000_000_000; // 9 decimals for reward calculations

// Insurance constants
const DEFAULT_INSURANCE_PREMIUM_RATE_BPS: u32 = 200; // 2% premium of loan principal
const INSURANCE_CLAIM_PAYBACK_BPS: u32 = 10000; // 100% coverage

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
    pub reserve_factor_bps: u32, // Reserve factor in basis points (e.g., 1000 = 10%)
    pub total_protocol_revenue: u64, // Total protocol revenue accumulated
    pub is_paused: bool,     // Per-asset pause functionality
}

const SECONDS_IN_YEAR: u64 = 31_536_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanRecord {
    pub loan_id: u64,
    pub borrower: Address,
    pub asset: Address, // Asset being borrowed
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
    pub asset: Address, // Asset being borrowed
    pub principal: u64,
    pub collateral_amount: u64,
    pub collateral_token: Address,
    pub due_date: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanInsurance {
    pub loan_id: u64,
    pub borrower: Address,
    pub coverage_amount: u64, // Coverage limit (typically 100% of loan principal)
    pub premium_paid: u64,    // Premium amount paid upfront
    pub premium_rate_bps: u32, // Premium rate in basis points (e.g., 200 = 2%)
    pub purchase_time: u64,   // Timestamp when insurance was purchased
    pub expires_at: u64,      // Expiration timestamp (typically loan due date)
    pub claimed: bool,        // Whether insurance has been claimed
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceFund {
    pub total_premiums_collected: u64, // Total premiums accumulated
    pub total_claims_paid: u64,        // Total claims paid out
    pub available_balance: u64,        // Current available balance for claims
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

#[soroban_sdk::contractclient(name = "InheritanceContractClient")]
pub trait InheritanceContractInterface {
    fn verify_plan_ownership(env: Env, plan_id: u64, user: Address) -> bool;
}

// ─────────────────────────────────────────────────
// Events
// ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractLinkedEvent {
    pub contract_type: soroban_sdk::Symbol,
    pub address: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositEvent {
    pub depositor: Address,
    pub asset: Address,
    pub amount: u64,
    pub shares_minted: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawEvent {
    pub depositor: Address,
    pub asset: Address,
    pub shares_burned: u64,
    pub amount: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriorityWithdrawEvent {
    pub caller: Address,
    pub asset: Address,
    pub amount: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BorrowEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub asset: Address,
    pub amount: u64,
    pub collateral_amount: u64,
    pub due_date: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepayEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub asset: Address,
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
    pub asset: Address,
    pub amount_repaid: u64,
    pub collateral_seized: u64,
    pub health_factor: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterestAccrualEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub asset: Address,
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
    pub asset: Address,
    pub late_fee: u64,
    pub days_overdue: u64,
    pub total_with_late_fees: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlashLoanEvent {
    pub receiver: Address,
    pub asset: Address,
    pub amount: u64,
    pub fee: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanRefinancedEvent {
    pub old_loan_id: u64,
    pub new_loan_id: u64,
    pub borrower: Address,
    pub asset: Address,
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
    pub asset: Address,
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
    pub asset: Address,
    pub old_principal: u64,
    pub new_principals: Vec<u64>,
    pub split_fee: u64,
    pub timestamp: u64,
}

// ─────────────────────────────────────────────────
// Yield Farming Data Types
// ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardPool {
    pub total_staked: u64,
    pub reward_rate: u64, // Rewards per second per staked token
    pub last_update_time: u64,
    pub reward_per_token_stored: u64,
    pub total_rewards_distributed: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStake {
    pub amount: u64,
    pub reward_per_token_paid: u64,
    pub rewards: u64,
    pub stake_time: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakedEvent {
    pub user: Address,
    pub asset: Address,
    pub amount: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnstakedEvent {
    pub user: Address,
    pub asset: Address,
    pub amount: u64,
    pub rewards_claimed: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardsClaimedEvent {
    pub user: Address,
    pub asset: Address,
    pub rewards: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardRateUpdatedEvent {
    pub asset: Address,
    pub old_rate: u64,
    pub new_rate: u64,
    pub timestamp: u64,
}

// ─────────────────────────────────────────────────
// Insurance Events
// ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePurchasedEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub coverage_amount: u64,
    pub premium_paid: u64,
    pub premium_rate_bps: u32,
    pub expires_at: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceClaimedEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub claim_amount: u64,
    pub coverage_amount: u64,
    pub timestamp: u64,
}

// Interest Rate Model
// ─────────────────────────────────────────────────

/// Two-slope interest rate model parameters.
/// Before optimal utilization: rate = base_rate + (utilization / optimal_utilization) * slope1
/// After optimal utilization:  rate = base_rate + slope1 + ((utilization - optimal) / (1 - optimal)) * slope2
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateModel {
    pub base_rate_bps: u32,
    pub optimal_utilization_bps: u32,
    pub slope1_bps: u32,
    pub slope2_bps: u32,
    pub reserve_factor_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceCancelledEvent {
    pub loan_id: u64,
    pub borrower: Address,
    pub refund_amount: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateModelUpdatedEvent {
    pub base_rate_bps: u32,
    pub optimal_utilization_bps: u32,
    pub slope1_bps: u32,
    pub slope2_bps: u32,
    pub reserve_factor_bps: u32,
    pub updated_by: Address,
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
    InsufficientStake = 21,
    NoRewardsToClaim = 22,
    InvalidRewardRate = 23,
    PoolPaused = 24,
    AssetNotSupported = 25,
    InsuranceAlreadyPurchased = 26,
    InsuranceNotFound = 27,
    InsuranceExpired = 28,
    InsuranceAlreadyClaimed = 29,
    InsufficientInsuranceFund = 30,
    InvalidInsuranceAmount = 31,
    InvalidRateModel = 32,
    ContractPaused = 33,
}

// ─────────────────────────────────────────────────
// Storage Keys
// ─────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    SupportedAssets, // Vec<Address>
    PoolState(Address),
    Shares(Address, Address), // (User, Asset)
    Loan(Address),
    NextLoanId,
    LoanById(u64),
    CollateralRatio,
    WhitelistedCollateral(Address),
    NFTToken,
    ReentrancyGuard,
    LateFeesAccrued(u64), // Track late fees for a specific loan_id
    FlashLoanFeeBps,
    UserLoans(Address),          // Track multiple loans per user (Vec<u64>)
    RewardPool(Address),         // Per-asset reward pool
    UserStake(Address, Address), // (User, Asset) staking position
    Insurance(u64),              // Insurance record for a loan_id
    InsuranceFund,               // Global insurance fund state
    InsurancePremiumRate,        // Premium rate in basis points (default 200 = 2%)
    InheritanceContract,
    GovernanceContract,
    RateModel,
    Token, // Underlying token address for insurance operations
}

// ─────────────────────────────────────────────────
// Contract
// ─────────────────────────────────────────────────

#[contract]
pub struct LendingContract;

#[contractimpl]
impl LendingContract {
    // ─── Admin / Init ───────────────────────────────

    /// Initialize the lending pool with an admin address and the initial underlying token.
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

        // Store the primary token address for insurance operations
        env.storage().instance().set(&DataKey::Token, &token);

        let mut assets = Vec::new(&env);
        assets.push_back(token.clone());
        env.storage()
            .instance()
            .set(&DataKey::SupportedAssets, &assets);

        env.storage()
            .instance()
            .set(&DataKey::CollateralRatio, &collateral_ratio_bps);

        env.storage().instance().set(
            &DataKey::PoolState(token.clone()),
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
                reserve_factor_bps: 1000, // 10% default
                total_protocol_revenue: 0,
                is_paused: false,
            },
        );

        // Initialize reward pool for the first asset
        env.storage().instance().set(
            &DataKey::RewardPool(token.clone()),
            &RewardPool {
                total_staked: 0,
                reward_rate: DEFAULT_REWARD_RATE,
                last_update_time: env.ledger().timestamp(),
                reward_per_token_stored: 0,
                total_rewards_distributed: 0,
            },
        );

        Ok(())
    }

    /// Add a new asset pool to the lending protocol.
    /// Only the admin can call this.
    pub fn add_asset_pool(
        env: Env,
        admin: Address,
        asset: Address,
        base_rate_bps: u32,
        multiplier_bps: u32,
        utilization_cap_bps: u32,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;

        let mut assets: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::SupportedAssets)
            .unwrap_or_else(|| Vec::new(&env));

        // Check if asset is already supported
        for a in assets.iter() {
            if a == asset {
                return Err(LendingError::AlreadyInitialized);
            }
        }

        assets.push_back(asset.clone());
        env.storage()
            .instance()
            .set(&DataKey::SupportedAssets, &assets);

        env.storage().instance().set(
            &DataKey::PoolState(asset.clone()),
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
                reserve_factor_bps: 1000,
                total_protocol_revenue: 0,
                is_paused: false,
            },
        );

        // Initialize reward pool for the new asset
        env.storage().instance().set(
            &DataKey::RewardPool(asset.clone()),
            &RewardPool {
                total_staked: 0,
                reward_rate: DEFAULT_REWARD_RATE,
                last_update_time: env.ledger().timestamp(),
                reward_per_token_stored: 0,
                total_rewards_distributed: 0,
            },
        );

        // Initialize insurance fund
        env.storage().instance().set(
            &DataKey::InsuranceFund,
            &InsuranceFund {
                total_premiums_collected: 0,
                total_claims_paid: 0,
                available_balance: 0,
            },
        );

        // Initialize insurance premium rate
        env.storage().instance().set(
            &DataKey::InsurancePremiumRate,
            &DEFAULT_INSURANCE_PREMIUM_RATE_BPS,
        );

        access_control::assign_role(&env, &admin, Role::Admin);
        Ok(())
    }

    /// Assign a role to an address. Admin-only.
    pub fn assign_role(
        env: Env,
        admin: Address,
        address: Address,
        role: Role,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        access_control::assign_role(&env, &address, role);
        Ok(())
    }

    /// Revoke a role from an address. Admin-only.
    pub fn revoke_role(
        env: Env,
        admin: Address,
        address: Address,
        role: Role,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        access_control::revoke_role(&env, &address, role);
        Ok(())
    }

    /// Pause or unpause a specific asset pool.
    pub fn pause_asset_pool(
        env: Env,
        admin: Address,
        asset: Address,
        paused: bool,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        let mut pool = Self::get_pool(&env, &asset)?;
        pool.is_paused = paused;
        Self::set_pool(&env, &asset, &pool);
        Ok(())
    }

    /// View pool state for a specific asset.
    pub fn get_pool_for_asset(env: Env, asset: Address) -> Result<PoolState, LendingError> {
        Self::get_pool(&env, &asset)
    }

    /// List all assets supported by the protocol.
    pub fn get_supported_assets(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::SupportedAssets)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Check whether an address holds a given role.
    pub fn has_role(env: Env, address: Address, role: Role) -> bool {
        access_control::has_role(&env, &address, role)
    }

    /// Return all roles held by an address.
    pub fn get_roles(env: Env, address: Address) -> Vec<Role> {
        use access_control::AccessControlKey;
        env.storage()
            .persistent()
            .get(&AccessControlKey::Roles(address))
            .unwrap_or(Vec::new(&env))
    }

    pub fn set_nft_token(env: Env, admin: Address, nft_token: Address) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::NFTToken, &nft_token);
        Ok(())
    }

    fn enter_reentrancy_guard(env: &Env) -> Result<(), LendingError> {
        access_control::reentrancy_enter(env, LendingError::ReentrantCall)
    }

    fn exit_reentrancy_guard(env: &Env) {
        access_control::reentrancy_exit(env);
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        access_control::pause_contract(&env);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        access_control::unpause_contract(&env);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        access_control::is_contract_paused(&env)
    }

    fn require_not_paused(env: &Env) -> Result<(), LendingError> {
        access_control::require_not_paused(env, LendingError::ContractPaused)
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

    fn get_pool(env: &Env, asset: &Address) -> Result<PoolState, LendingError> {
        env.storage()
            .instance()
            .get(&DataKey::PoolState(asset.clone()))
            .ok_or(LendingError::AssetNotSupported)
    }

    fn set_pool(env: &Env, asset: &Address, pool: &PoolState) {
        env.storage()
            .instance()
            .set(&DataKey::PoolState(asset.clone()), pool);
    }

    fn get_shares(env: &Env, asset: &Address, owner: &Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Shares(owner.clone(), asset.clone()))
            .unwrap_or(0u64)
    }

    fn set_shares(env: &Env, asset: &Address, owner: &Address, shares: u64) {
        env.storage()
            .persistent()
            .set(&DataKey::Shares(owner.clone(), asset.clone()), &shares);
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

    pub fn get_admin(env: Env) -> Option<Address> {
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

    // ─── Reward Farming Helpers ────────────────────────

    /// Update reward pool state and calculate new reward per token
    fn update_reward_pool(env: &Env, asset: &Address) {
        let mut reward_pool: RewardPool = env
            .storage()
            .instance()
            .get(&DataKey::RewardPool(asset.clone()))
            .unwrap_or_else(|| RewardPool {
                total_staked: 0,
                reward_rate: DEFAULT_REWARD_RATE,
                last_update_time: env.ledger().timestamp(),
                reward_per_token_stored: 0,
                total_rewards_distributed: 0,
            });

        let current_time = env.ledger().timestamp();

        if reward_pool.total_staked > 0 {
            let time_elapsed = current_time.saturating_sub(reward_pool.last_update_time);
            if time_elapsed > 0 && reward_pool.total_staked > 0 {
                // Calculate rewards per token for this time period
                let rewards_per_token = time_elapsed
                    .checked_mul(reward_pool.reward_rate)
                    .unwrap_or(0);

                reward_pool.reward_per_token_stored = reward_pool
                    .reward_per_token_stored
                    .checked_add(rewards_per_token)
                    .unwrap_or(0);

                let new_rewards = rewards_per_token
                    .checked_mul(reward_pool.total_staked)
                    .and_then(|v| v.checked_div(REWARD_PRECISION))
                    .unwrap_or(0);

                reward_pool.total_rewards_distributed = reward_pool
                    .total_rewards_distributed
                    .checked_add(new_rewards)
                    .unwrap_or(0);
            }
        }

        reward_pool.last_update_time = current_time;
        env.storage()
            .instance()
            .set(&DataKey::RewardPool(asset.clone()), &reward_pool);
    }

    /// Update user's reward debt
    fn update_user_reward_debt(env: &Env, user: &Address, asset: &Address) {
        let reward_pool: RewardPool = env
            .storage()
            .instance()
            .get(&DataKey::RewardPool(asset.clone()))
            .unwrap_or_else(|| RewardPool {
                total_staked: 0,
                reward_rate: DEFAULT_REWARD_RATE,
                last_update_time: env.ledger().timestamp(),
                reward_per_token_stored: 0,
                total_rewards_distributed: 0,
            });

        let mut user_stake: UserStake = env
            .storage()
            .instance()
            .get(&DataKey::UserStake(user.clone(), asset.clone()))
            .unwrap_or(UserStake {
                amount: 0,
                reward_per_token_paid: 0,
                rewards: 0,
                stake_time: 0,
            });

        user_stake.reward_per_token_paid = reward_pool.reward_per_token_stored;
        user_stake.rewards = Self::calculate_pending_rewards(env, user, asset);

        env.storage().instance().set(
            &DataKey::UserStake(user.clone(), asset.clone()),
            &user_stake,
        );
    }

    /// Get user's pending rewards (internal helper)
    fn calculate_pending_rewards(env: &Env, user: &Address, asset: &Address) -> u64 {
        Self::update_reward_pool(env, asset);

        let reward_pool: RewardPool = env
            .storage()
            .instance()
            .get(&DataKey::RewardPool(asset.clone()))
            .unwrap_or_else(|| RewardPool {
                total_staked: 0,
                reward_rate: DEFAULT_REWARD_RATE,
                last_update_time: env.ledger().timestamp(),
                reward_per_token_stored: 0,
                total_rewards_distributed: 0,
            });

        let user_stake: UserStake = env
            .storage()
            .instance()
            .get(&DataKey::UserStake(user.clone(), asset.clone()))
            .unwrap_or(UserStake {
                amount: 0,
                reward_per_token_paid: 0,
                rewards: 0,
                stake_time: 0,
            });

        if user_stake.amount == 0 {
            return user_stake.rewards;
        }

        let diff = reward_pool
            .reward_per_token_stored
            .saturating_sub(user_stake.reward_per_token_paid);

        let pending = diff
            .checked_mul(user_stake.amount)
            .and_then(|v| v.checked_div(REWARD_PRECISION))
            .unwrap_or(0);

        user_stake.rewards.checked_add(pending).unwrap_or(0)
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), LendingError> {
        caller.require_auth();
        access_control::require_role(env, caller, Role::Admin, LendingError::NotAdmin)
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

    /// Deposit `amount` of the specific asset into its pool.
    /// Mints proportional pool shares to the depositor.
    pub fn deposit(
        env: Env,
        depositor: Address,
        asset: Address,
        amount: u64,
    ) -> Result<u64, LendingError> {
        Self::require_not_paused(&env)?;
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        depositor.require_auth();

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let mut pool = Self::get_pool(&env, &asset)?;
        if pool.is_paused {
            return Err(LendingError::PoolPaused);
        }

        let contract_id = env.current_contract_address();
        Self::transfer(&env, &asset, &depositor, &contract_id, amount)?;

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
        Self::set_pool(&env, &asset, &pool);

        let existing = Self::get_shares(&env, &asset, &depositor);
        Self::set_shares(&env, &asset, &depositor, existing + shares);

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("DEPOSIT")),
            DepositEvent {
                depositor: depositor.clone(),
                asset: asset.clone(),
                amount,
                shares_minted: shares,
            },
        );
        log!(
            &env,
            "Deposited {} tokens of asset {} , minted {} shares",
            amount,
            asset,
            shares
        );
        Self::exit_reentrancy_guard(&env);
        Ok(shares)
    }

    /// Burn `shares` and return the proportional underlying tokens to the depositor.
    /// Reverts if insufficient liquidity (i.e., tokens are loaned out).
    pub fn withdraw(
        env: Env,
        depositor: Address,
        asset: Address,
        shares: u64,
    ) -> Result<u64, LendingError> {
        Self::require_not_paused(&env)?;
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        depositor.require_auth();

        if shares == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let depositor_shares = Self::get_shares(&env, &asset, &depositor);
        if shares > depositor_shares {
            return Err(LendingError::InsufficientShares);
        }

        let mut pool = Self::get_pool(&env, &asset)?;
        if pool.is_paused {
            return Err(LendingError::PoolPaused);
        }

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
        Self::set_pool(&env, &asset, &pool);

        Self::set_shares(&env, &asset, &depositor, depositor_shares - shares);

        let contract_id = env.current_contract_address();
        Self::transfer(&env, &asset, &contract_id, &depositor, amount)?;

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("WITHDRAW")),
            WithdrawEvent {
                depositor: depositor.clone(),
                asset: asset.clone(),
                shares_burned: shares,
                amount,
            },
        );
        log!(
            &env,
            "Withdrew {} tokens of asset {}, burned {} shares",
            amount,
            asset,
            shares
        );
        Self::exit_reentrancy_guard(&env);
        Ok(amount)
    }

    /// Borrow `amount` of the specific asset from the pool with collateral.
    /// Requires overcollateralized borrowing based on collateral ratio.
    /// Returns the unique loan ID.
    pub fn borrow(
        env: Env,
        borrower: Address,
        asset: Address,
        amount: u64,
        collateral_token: Address,
        collateral_amount: u64,
        duration_seconds: u64,
    ) -> Result<u64, LendingError> {
        Self::require_not_paused(&env)?;
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        borrower.require_auth();

        if amount == 0 || collateral_amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let mut pool = Self::get_pool(&env, &asset)?;
        if pool.is_paused {
            return Err(LendingError::PoolPaused);
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

        Self::set_pool(&env, &asset, &pool);

        let loan_id = Self::increment_loan_id(&env);
        let borrow_time = env.ledger().timestamp();
        let due_date = borrow_time + duration_seconds;

        let loan = LoanRecord {
            loan_id,
            borrower: borrower.clone(),
            asset: asset.clone(),
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
                    asset: asset.clone(),
                    collateral_amount,
                    collateral_token: collateral_token.clone(),
                    due_date,
                    loan_id,
                    principal: amount,
                },
            );
        }

        Self::transfer(&env, &asset, &contract_id, &borrower, amount)?;

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("BORROW")),
            BorrowEvent {
                loan_id,
                borrower: borrower.clone(),
                asset: asset.clone(),
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
            "Loan {} created: {} tokens of asset {} with {} collateral",
            loan_id,
            amount,
            asset,
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
        Self::require_not_paused(&env)?;
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

        let contract_id = env.current_contract_address();
        Self::transfer(&env, &loan.asset, &borrower, &contract_id, total_repayment)?;

        // Return collateral to borrower
        Self::transfer(
            &env,
            &loan.collateral_token,
            &contract_id,
            &borrower,
            loan.collateral_amount,
        )?;

        let mut pool = Self::get_pool(&env, &loan.asset)?;
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
        Self::set_pool(&env, &loan.asset, &pool);

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
                    asset: loan.asset.clone(),
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
                asset: loan.asset.clone(),
                principal: loan.principal,
                interest,
                total_amount: total_repayment,
                collateral_returned: loan.collateral_amount,
            },
        );
        log!(
            &env,
            "Loan {} repaid: {} total ({} principal + {} interest + {} late fees) of asset {} , {} collateral returned",
            loan.loan_id,
            total_repayment,
            loan.principal,
            interest,
            late_fee,
            loan.asset,
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
                        asset: loan.asset.clone(),
                        principal: loan.principal,
                        interest_accrued: interest,
                        interest_rate_bps: loan.interest_rate_bps,
                        elapsed_seconds: elapsed,
                        timestamp: env.ledger().timestamp(),
                    },
                );

                log!(
                    &env,
                    "Interest accrued for loan {}: {} interest on {} principal for asset {}",
                    loan.loan_id,
                    interest,
                    loan.principal,
                    loan.asset
                );

                Ok(interest)
            }
            None => Err(LendingError::NoOpenLoan),
        }
    }

    /// Withdraw prioritized funds from the retained yield for a specific asset.
    /// Used by authorized contracts (like InheritanceContract) to fulfill priority claims.
    pub fn withdraw_priority(
        env: Env,
        caller: Address,
        asset: Address,
        amount: u64,
    ) -> Result<u64, LendingError> {
        Self::require_not_paused(&env)?;
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;
        caller.require_auth();

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let mut pool = Self::get_pool(&env, &asset)?;

        if amount > pool.retained_yield {
            return Err(LendingError::InsufficientLiquidity);
        }

        pool.retained_yield -= amount;
        Self::set_pool(&env, &asset, &pool);

        let contract_id = env.current_contract_address();
        Self::transfer(&env, &asset, &contract_id, &caller, amount)?;

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("PRIORITY")),
            PriorityWithdrawEvent {
                caller: caller.clone(),
                asset: asset.clone(),
                amount,
            },
        );
        log!(
            &env,
            "Priority withdrawal {} tokens of asset {} by {}",
            amount,
            asset,
            caller
        );
        Self::exit_reentrancy_guard(&env);
        Ok(amount)
    }

    // ─── Reads ───────────────────────────────────────

    /// Returns the current global pool state.
    /// Update get_pool_state to accept asset parameter
    pub fn get_pool_state(env: Env, asset: Address) -> Result<PoolState, LendingError> {
        Self::get_pool(&env, &asset)
    }

    /// Returns the share balance of the given address for a specific asset.
    pub fn get_shares_of(env: Env, asset: Address, owner: Address) -> u64 {
        Self::get_shares(&env, &asset, &owner)
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

    /// Returns the available (un-borrowed) liquidity in the pool for a specific asset.
    pub fn available_liquidity(env: Env, asset: Address) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        let pool = Self::get_pool(&env, &asset)?;
        Ok(pool.total_deposits.saturating_sub(pool.total_borrowed))
    }

    /// Returns the current dynamic interest rate that would be given to a new loan for a specific asset
    pub fn get_current_interest_rate(env: Env, asset: Address) -> Result<u32, LendingError> {
        Self::require_initialized(&env)?;
        let pool = Self::get_pool(&env, &asset)?;
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

        let pool = Self::get_pool(&env, &loan.asset)?;
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

        let pool = Self::get_pool(&env, &loan.asset)?;
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
        asset: Address,
        grace_period_seconds: u64,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;

        let mut pool = Self::get_pool(&env, &asset)?;
        pool.grace_period_seconds = grace_period_seconds;
        Self::set_pool(&env, &asset, &pool);

        log!(
            &env,
            "Grace period for asset {} updated to {} seconds",
            asset,
            grace_period_seconds
        );
        Ok(())
    }

    /// Set the late fee rate for loans (admin only)
    /// Late fee rate is in basis points per day (e.g., 500 = 5% per day)
    pub fn set_late_fee_rate(
        env: Env,
        admin: Address,
        asset: Address,
        late_fee_rate_bps: u32,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;

        let mut pool = Self::get_pool(&env, &asset)?;
        pool.late_fee_rate_bps = late_fee_rate_bps;
        Self::set_pool(&env, &asset, &pool);

        log!(
            &env,
            "Late fee rate for asset {} updated to {} bps per day",
            asset,
            late_fee_rate_bps
        );
        Ok(())
    }

    /// Get the current grace period in seconds
    pub fn get_grace_period(env: Env, asset: Address) -> Result<u64, LendingError> {
        let pool = Self::get_pool(&env, &asset)?;
        Ok(pool.grace_period_seconds)
    }

    /// Get the current late fee rate in basis points per day
    pub fn get_late_fee_rate(env: Env, asset: Address) -> Result<u32, LendingError> {
        let pool = Self::get_pool(&env, &asset)?;
        Ok(pool.late_fee_rate_bps)
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

    pub fn flash_loan(
        env: Env,
        receiver_id: Address,
        asset: Address,
        amount: u64,
    ) -> Result<(), LendingError> {
        Self::require_not_paused(&env)?;
        Self::require_initialized(&env)?;
        Self::enter_reentrancy_guard(&env)?;

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let mut pool = Self::get_pool(&env, &asset)?;
        if pool.is_paused {
            return Err(LendingError::PoolPaused);
        }

        let available = pool.total_deposits.saturating_sub(pool.total_borrowed);
        if amount > available {
            return Err(LendingError::InsufficientLiquidity);
        }

        let fee_bps = Self::get_flash_loan_fee(env.clone());
        let fee = (amount as u128)
            .checked_mul(fee_bps as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as u64;

        let contract_id = env.current_contract_address();
        let token_client = token::Client::new(&env, &asset);
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
        Self::set_pool(&env, &asset, &pool);

        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("FLASHL")),
            FlashLoanEvent {
                receiver: receiver_id,
                asset: asset.clone(),
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

    // ─── Yield Farming Functions ───────────────────────

    /// Stake LP tokens (shares) for rewards for a specific asset
    pub fn stake_lp_tokens(
        env: Env,
        user: Address,
        asset: Address,
        amount: u64,
    ) -> Result<(), LendingError> {
        Self::require_initialized(&env)?;
        user.require_auth();

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        // Check user has enough shares to stake
        let user_shares = Self::get_shares_of(env.clone(), asset.clone(), user.clone());
        if user_shares < amount {
            return Err(LendingError::InsufficientShares);
        }

        // Update reward pool first
        Self::update_reward_pool(&env, &asset);
        let mut reward_pool: RewardPool = env
            .storage()
            .instance()
            .get(&DataKey::RewardPool(asset.clone()))
            .unwrap();

        // Update user stake
        let mut user_stake: UserStake = env
            .storage()
            .instance()
            .get(&DataKey::UserStake(user.clone(), asset.clone()))
            .unwrap_or(UserStake {
                amount: 0,
                reward_per_token_paid: reward_pool.reward_per_token_stored,
                rewards: 0,
                stake_time: 0,
            });

        // Update user's reward debt - set to current rate and reset rewards
        user_stake.reward_per_token_paid = reward_pool.reward_per_token_stored;
        user_stake.rewards = 0; // Reset rewards for new stake

        // Update stake amount
        user_stake.amount = user_stake.amount.checked_add(amount).unwrap_or(0);
        if user_stake.stake_time == 0 {
            user_stake.stake_time = env.ledger().timestamp();
        }

        // Update totals
        reward_pool.total_staked = reward_pool.total_staked.checked_add(amount).unwrap_or(0);

        // Save state
        env.storage()
            .instance()
            .set(&DataKey::RewardPool(asset.clone()), &reward_pool);
        env.storage().instance().set(
            &DataKey::UserStake(user.clone(), asset.clone()),
            &user_stake,
        );

        // Emit event
        env.events().publish(
            (symbol_short!("STAKE"), symbol_short!("LP")),
            StakedEvent {
                user: user.clone(),
                asset: asset.clone(),
                amount,
                timestamp: env.ledger().timestamp(),
            },
        );

        log!(
            &env,
            "Staked {} LP tokens of asset {} for user {:?}",
            amount,
            asset,
            user
        );
        Ok(())
    }

    /// Unstake LP tokens and claim pending rewards for a specific asset
    pub fn unstake_lp_tokens(
        env: Env,
        user: Address,
        asset: Address,
        amount: u64,
    ) -> Result<(), LendingError> {
        Self::require_initialized(&env)?;
        user.require_auth();

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        // Get user stake
        let mut user_stake: UserStake = env
            .storage()
            .instance()
            .get(&DataKey::UserStake(user.clone(), asset.clone()))
            .ok_or(LendingError::InsufficientStake)?;

        if user_stake.amount < amount {
            return Err(LendingError::InsufficientStake);
        }

        // Update rewards before unstaking
        Self::update_user_reward_debt(&env, &user, &asset);
        user_stake = env
            .storage()
            .instance()
            .get(&DataKey::UserStake(user.clone(), asset.clone()))
            .unwrap();

        let rewards_to_claim = user_stake.rewards;

        // Update user stake
        user_stake.amount = user_stake.amount.saturating_sub(amount);
        if user_stake.amount == 0 {
            // Reset reward tracking if fully unstaked
            user_stake.reward_per_token_paid = 0;
            user_stake.stake_time = 0;
        }

        // Update reward pool
        let mut reward_pool: RewardPool = env
            .storage()
            .instance()
            .get(&DataKey::RewardPool(asset.clone()))
            .unwrap();
        reward_pool.total_staked = reward_pool.total_staked.saturating_sub(amount);

        // Save state
        env.storage()
            .instance()
            .set(&DataKey::RewardPool(asset.clone()), &reward_pool);
        env.storage().instance().set(
            &DataKey::UserStake(user.clone(), asset.clone()),
            &user_stake,
        );

        // Emit event
        env.events().publish(
            (symbol_short!("UNSTAKE"), symbol_short!("LP")),
            UnstakedEvent {
                user: user.clone(),
                asset: asset.clone(),
                amount,
                rewards_claimed: rewards_to_claim,
                timestamp: env.ledger().timestamp(),
            },
        );

        log!(
            &env,
            "Unstaked {} LP tokens of asset {} for user {:?}, claimed {} rewards",
            amount,
            asset,
            user,
            rewards_to_claim
        );
        Ok(())
    }

    /// Claim accumulated rewards without unstaking for a specific asset
    pub fn claim_rewards(env: Env, user: Address, asset: Address) -> Result<u64, LendingError> {
        Self::require_initialized(&env)?;
        user.require_auth();

        // Update rewards
        Self::update_user_reward_debt(&env, &user, &asset);

        let mut user_stake: UserStake = env
            .storage()
            .instance()
            .get(&DataKey::UserStake(user.clone(), asset.clone()))
            .ok_or(LendingError::NoRewardsToClaim)?;

        let rewards_to_claim = user_stake.rewards;
        if rewards_to_claim == 0 {
            return Err(LendingError::NoRewardsToClaim);
        }

        // Reset claimed rewards
        user_stake.rewards = 0;
        env.storage().instance().set(
            &DataKey::UserStake(user.clone(), asset.clone()),
            &user_stake,
        );

        // Emit event
        env.events().publish(
            (symbol_short!("CLAIM"), symbol_short!("REWARDS")),
            RewardsClaimedEvent {
                user: user.clone(),
                asset: asset.clone(),
                rewards: rewards_to_claim,
                timestamp: env.ledger().timestamp(),
            },
        );
        log!(
            &env,
            "Claimed {} rewards for user {:?} for asset {}",
            rewards_to_claim,
            user,
            asset
        );
        Ok(rewards_to_claim)
    }

    /// Get total staked in the reward pool for a specific asset
    pub fn get_total_staked(env: Env, asset: Address) -> u64 {
        let reward_pool: RewardPool = env
            .storage()
            .instance()
            .get(&DataKey::RewardPool(asset))
            .unwrap_or_else(|| RewardPool {
                total_staked: 0,
                reward_rate: DEFAULT_REWARD_RATE,
                last_update_time: env.ledger().timestamp(),
                reward_per_token_stored: 0,
                total_rewards_distributed: 0,
            });
        reward_pool.total_staked
    }

    /// Get current reward rate for a specific asset
    pub fn get_reward_rate(env: Env, asset: Address) -> u64 {
        let reward_pool: RewardPool = env
            .storage()
            .instance()
            .get(&DataKey::RewardPool(asset))
            .unwrap_or_else(|| RewardPool {
                total_staked: 0,
                reward_rate: DEFAULT_REWARD_RATE,
                last_update_time: env.ledger().timestamp(),
                reward_per_token_stored: 0,
                total_rewards_distributed: 0,
            });
        reward_pool.reward_rate
    }

    /// Get user's staked balance for a specific asset
    pub fn get_staked_balance(env: Env, user: Address, asset: Address) -> u64 {
        let user_stake: UserStake = env
            .storage()
            .instance()
            .get(&DataKey::UserStake(user, asset))
            .unwrap_or(UserStake {
                amount: 0,
                reward_per_token_paid: 0,
                rewards: 0,
                stake_time: 0,
            });
        user_stake.amount
    }

    /// Get pending rewards for a user for a specific asset
    pub fn get_pending_rewards(env: Env, user: Address, asset: Address) -> u64 {
        Self::calculate_pending_rewards(&env, &user, &asset)
    }

    /// Set reward rate for an asset (admin only)
    pub fn set_reward_rate(
        env: Env,
        admin: Address,
        asset: Address,
        new_rate: u64,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;

        if new_rate == 0 {
            return Err(LendingError::InvalidRewardRate);
        }

        // Update rewards before changing rate
        Self::update_reward_pool(&env, &asset);

        let mut reward_pool: RewardPool = env
            .storage()
            .instance()
            .get(&DataKey::RewardPool(asset.clone()))
            .unwrap();
        let old_rate = reward_pool.reward_rate;
        reward_pool.reward_rate = new_rate;

        env.storage()
            .instance()
            .set(&DataKey::RewardPool(asset.clone()), &reward_pool);

        // Emit event
        env.events().publish(
            (symbol_short!("REWARD"), symbol_short!("RATE_UPD")),
            RewardRateUpdatedEvent {
                asset: asset.clone(),
                old_rate,
                new_rate,
                timestamp: env.ledger().timestamp(),
            },
        );

        log!(
            &env,
            "Reward rate updated for asset {} from {} to {}",
            asset,
            old_rate,
            new_rate
        );
        Ok(())
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

        let contract_id = env.current_contract_address();

        // Transfer debt payment from liquidator to contract
        Self::transfer(&env, &loan.asset, &liquidator, &contract_id, amount)?;

        // Transfer collateral from contract to liquidator
        Self::transfer(
            &env,
            &loan.collateral_token,
            &contract_id,
            &liquidator,
            collateral_to_seize,
        )?;

        let mut pool = Self::get_pool(&env, &loan.asset)?;
        pool.total_borrowed = pool.total_borrowed.saturating_sub(amount);
        pool.total_deposits += amount;
        Self::set_pool(&env, &loan.asset, &pool);

        // Emit liquidation event
        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("LIQUIDATE")),
            LiquidationEvent {
                loan_id: loan.loan_id,
                borrower: borrower.clone(),
                liquidator: liquidator.clone(),
                asset: loan.asset.clone(),
                amount_repaid: amount,
                collateral_seized: collateral_to_seize,
                health_factor,
            },
        );

        log!(
            &env,
            "Loan {} liquidated: {} repaid of asset {}, {} collateral seized",
            loan.loan_id,
            amount,
            loan.asset,
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

        let pool = Self::get_pool(&env, &loan.asset)?;
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

        let contract_id = env.current_contract_address();

        // Transfer refinancing fee from borrower to contract
        Self::transfer(
            &env,
            &old_loan.asset,
            &borrower,
            &contract_id,
            terms.refinancing_fee,
        )?;

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
            asset: old_loan.asset.clone(),
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
                    asset: new_loan.asset.clone(),
                    collateral_amount: new_loan.collateral_amount,
                    collateral_token: new_loan.collateral_token.clone(),
                    due_date: new_loan.due_date,
                    loan_id: new_loan_id,
                    principal: new_loan.principal,
                },
            );
        }

        // Add refinancing fee to retained yield
        let mut pool = Self::get_pool(&env, &old_loan.asset)?;
        pool.retained_yield += terms.refinancing_fee;
        Self::set_pool(&env, &old_loan.asset, &pool);

        // Emit refinancing event
        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("REFINANCE")),
            LoanRefinancedEvent {
                old_loan_id: old_loan.loan_id,
                new_loan_id,
                borrower: borrower.clone(),
                asset: old_loan.asset.clone(),
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
        let mut asset: Option<Address> = None;
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
            let loan_grace_end =
                loan.due_date + Self::get_pool(&env, &loan.asset)?.grace_period_seconds;
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

            if asset.is_none() {
                asset = Some(loan.asset.clone());
            } else if asset.as_ref() != Some(&loan.asset) {
                return Err(LendingError::InvalidRefinanceTerms); // All loan assets must be the same for consolidation
            }

            old_loans.push_back(loan);
        }

        let consolidation_asset = asset.unwrap();
        let consolidation_fee = ((total_outstanding as u128)
            .checked_mul(REFINANCING_FEE_BPS as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0)) as u64;

        let new_principal = total_outstanding + consolidation_fee;

        // Transfer consolidation fee
        let contract_id = env.current_contract_address();
        Self::transfer(
            &env,
            &consolidation_asset,
            &borrower,
            &contract_id,
            consolidation_fee,
        )?;

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

        let pool = Self::get_pool(&env, &consolidation_asset)?;
        let utilization_bps = Self::get_utilization_bps(pool.total_borrowed, pool.total_deposits);
        let new_interest_rate_bps =
            Self::calculate_dynamic_rate(pool.base_rate_bps, pool.multiplier_bps, utilization_bps);

        let new_loan = LoanRecord {
            loan_id: new_loan_id,
            borrower: borrower.clone(),
            asset: consolidation_asset.clone(),
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
                    asset: consolidation_asset.clone(),
                    collateral_amount: new_loan.collateral_amount,
                    collateral_token: new_loan.collateral_token.clone(),
                    due_date: new_loan.due_date,
                    loan_id: new_loan_id,
                    principal: new_loan.principal,
                },
            );
        }

        // Add fee to retained yield
        let mut pool = Self::get_pool(&env, &consolidation_asset)?;
        pool.retained_yield += consolidation_fee;
        Self::set_pool(&env, &consolidation_asset, &pool);

        // Emit consolidation event
        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("CONSOLID")),
            LoansConsolidatedEvent {
                old_loan_ids: loan_ids.clone(),
                new_loan_id,
                borrower: borrower.clone(),
                asset: consolidation_asset.clone(),
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
        let contract_id = env.current_contract_address();
        Self::transfer(&env, &old_loan.asset, &borrower, &contract_id, split_fee)?;

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

        let pool = Self::get_pool(&env, &old_loan.asset)?;
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
                asset: old_loan.asset.clone(),
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
                        asset: old_loan.asset.clone(),
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
        let mut pool = Self::get_pool(&env, &old_loan.asset)?;
        pool.retained_yield += split_fee;
        Self::set_pool(&env, &old_loan.asset, &pool);

        // Emit split event
        env.events().publish(
            (symbol_short!("POOL"), symbol_short!("SPLIT")),
            LoanSplitEvent {
                old_loan_id: old_loan.loan_id,
                new_loan_ids: new_loan_ids.clone(),
                borrower: borrower.clone(),
                asset: old_loan.asset.clone(),
                old_principal: old_loan.principal,
                new_principals: split_amounts,
                split_fee,
                timestamp: current_time,
            },
        );

        log!(
            &env,
            "Split loan {} into {} loans of asset {} with fee {}",
            old_loan.loan_id,
            new_loan_ids.len(),
            old_loan.asset,
            split_fee
        );

        Self::exit_reentrancy_guard(&env);
        Ok(new_loan_ids)
    }

    // ─────────────────────────────────────────────────
    // Reserve Fund Management Functions
    // ─────────────────────────────────────────────────

    pub fn set_reserve_factor(
        env: Env,
        admin: Address,
        asset: Address, // Added
        reserve_factor_bps: u32,
    ) -> Result<(), LendingError> {
        admin.require_auth();

        // Verify admin
        let admin_key = DataKey::Admin;
        let stored_admin = env.storage().instance().get::<_, Address>(&admin_key);
        if stored_admin != Some(admin.clone()) {
            return Err(LendingError::Unauthorized);
        }

        // Validate reserve factor (0-10000 basis points = 0-100%)
        if reserve_factor_bps > 10000 {
            return Err(LendingError::InvalidAmount);
        }

        let mut pool = Self::get_pool(&env, &asset)?;
        pool.reserve_factor_bps = reserve_factor_bps;
        Self::set_pool(&env, &asset, &pool);

        log!(
            &env,
            "ReserveFactorUpdated: asset={}, new_reserve_factor_bps={}",
            asset,
            reserve_factor_bps
        );

        Ok(())
    }

    pub fn get_reserve_factor(env: Env, asset: Address) -> Result<u32, LendingError> {
        let pool = Self::get_pool(&env, &asset)?;
        Ok(pool.reserve_factor_bps)
    }

    pub fn get_reserve_balance(env: Env, asset: Address) -> Result<u64, LendingError> {
        let pool = Self::get_pool(&env, &asset)?;
        Ok(pool.bad_debt_reserve)
    }

    pub fn get_protocol_revenue(env: Env, asset: Address) -> Result<u64, LendingError> {
        let pool = Self::get_pool(&env, &asset)?;
        Ok(pool.total_protocol_revenue)
    }

    pub fn withdraw_reserves(
        env: Env,
        admin: Address,
        asset: Address,
        amount: u64,
    ) -> Result<(), LendingError> {
        admin.require_auth();

        // Verify admin
        let admin_key = DataKey::Admin;
        let stored_admin = env.storage().instance().get::<_, Address>(&admin_key);
        if stored_admin != Some(admin.clone()) {
            return Err(LendingError::Unauthorized);
        }

        let mut pool = Self::get_pool(&env, &asset)?;
        if pool.bad_debt_reserve < amount {
            return Err(LendingError::InsufficientLiquidity);
        }

        pool.bad_debt_reserve = pool.bad_debt_reserve.saturating_sub(amount);
        Self::set_pool(&env, &asset, &pool);

        let contract_id = env.current_contract_address();
        Self::transfer(&env, &asset, &contract_id, &admin, amount)?;

        log!(
            &env,
            "ReservesWithdrawn: asset={}, amount={}, withdrawn_by={}",
            asset,
            amount,
            admin
        );

        Ok(())
    }

    pub fn allocate_reserves(
        env: Env,
        admin: Address,
        asset: Address,
        amount: u64,
        insurance_fund: Address,
    ) -> Result<(), LendingError> {
        admin.require_auth();

        // Verify admin
        let admin_key = DataKey::Admin;
        let stored_admin = env.storage().instance().get::<_, Address>(&admin_key);
        if stored_admin != Some(admin.clone()) {
            return Err(LendingError::Unauthorized);
        }

        let mut pool = Self::get_pool(&env, &asset)?;
        if pool.bad_debt_reserve < amount {
            return Err(LendingError::InsufficientLiquidity);
        }

        pool.bad_debt_reserve = pool.bad_debt_reserve.saturating_sub(amount);
        Self::set_pool(&env, &asset, &pool);

        let contract_id = env.current_contract_address();
        Self::transfer(&env, &asset, &contract_id, &insurance_fund, amount)?;

        log!(
            &env,
            "ReservesAllocated: asset={}, amount={}, allocated_to={}",
            asset,
            amount,
            insurance_fund
        );

        Ok(())
    }

    /// Calculate interest split between depositors and protocol
    /// Returns (depositor_interest, protocol_interest)
    fn calculate_interest_split(total_interest: u64, reserve_factor_bps: u32) -> (u64, u64) {
        let protocol_share = (total_interest as u128)
            .checked_mul(reserve_factor_bps as u128)
            .and_then(|v| v.checked_div(10000u128))
            .unwrap_or(0) as u64;

        let depositor_share = total_interest.saturating_sub(protocol_share);
        (depositor_share, protocol_share)
    }

    /// Accrue interest and split between depositors and protocol
    pub fn accrue_interest_with_reserve(env: Env, loan_id: u64) -> Result<(), LendingError> {
        let loan_key = DataKey::LoanById(loan_id);
        let loan = env
            .storage()
            .instance()
            .get::<_, LoanRecord>(&loan_key)
            .ok_or(LendingError::LoanNotFound)?; // Loan not found

        let elapsed = env.ledger().timestamp().saturating_sub(loan.borrow_time);
        let total_interest =
            Self::calculate_interest(loan.principal, loan.interest_rate_bps, elapsed);

        let mut pool = Self::get_pool(&env, &loan.asset)?;
        let (depositor_interest, protocol_interest) =
            Self::calculate_interest_split(total_interest, pool.reserve_factor_bps);

        // Update pool state
        pool.retained_yield = pool.retained_yield.saturating_add(depositor_interest);
        pool.bad_debt_reserve = pool.bad_debt_reserve.saturating_add(protocol_interest);
        pool.total_protocol_revenue = pool
            .total_protocol_revenue
            .saturating_add(protocol_interest);

        Self::set_pool(&env, &loan.asset, &pool);

        log!(
            &env,
            "InterestAccrued: loan_id={}, total_interest={}, depositor_share={}, protocol_share={}",
            loan_id,
            total_interest,
            depositor_interest,
            protocol_interest
        );

        Ok(())
    }

    // ─────────────────────────────────────────────────
    // Loan Insurance Functions
    // ─────────────────────────────────────────────────

    /// Initialize insurance fund (called during contract initialization or by admin)
    fn init_insurance_fund_if_needed(env: &Env) {
        if !env.storage().instance().has(&DataKey::InsuranceFund) {
            env.storage().instance().set(
                &DataKey::InsuranceFund,
                &InsuranceFund {
                    total_premiums_collected: 0,
                    total_claims_paid: 0,
                    available_balance: 0,
                },
            );
        }
        if !env.storage().instance().has(&DataKey::InsurancePremiumRate) {
            env.storage().instance().set(
                &DataKey::InsurancePremiumRate,
                &DEFAULT_INSURANCE_PREMIUM_RATE_BPS,
            );
        }
    }

    /// Get insurance premium for a given loan amount
    pub fn get_insurance_premium(env: Env, loan_amount: u64) -> Result<u64, LendingError> {
        Self::init_insurance_fund_if_needed(&env);
        let premium_rate_bps: u32 = env
            .storage()
            .instance()
            .get::<_, u32>(&DataKey::InsurancePremiumRate)
            .unwrap_or(DEFAULT_INSURANCE_PREMIUM_RATE_BPS);

        let premium = (loan_amount as u128)
            .checked_mul(premium_rate_bps as u128)
            .and_then(|v| v.checked_div(10000u128))
            .ok_or(LendingError::InvalidAmount)? as u64;

        Ok(premium)
    }

    /// Set insurance premium rate (admin only)
    pub fn set_insurance_premium_rate(
        env: Env,
        admin: Address,
        premium_rate_bps: u32,
    ) -> Result<(), LendingError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if premium_rate_bps > 10000 {
            return Err(LendingError::InvalidAmount);
        }

        Self::init_insurance_fund_if_needed(&env);
        env.storage()
            .instance()
            .set(&DataKey::InsurancePremiumRate, &premium_rate_bps);

        log!(
            &env,
            "InsurancePremiumRateUpdated: rate_bps={}",
            premium_rate_bps
        );

        Ok(())
    }

    /// Purchase insurance for a loan
    pub fn purchase_loan_insurance(
        env: Env,
        borrower: Address,
        loan_id: u64,
    ) -> Result<u64, LendingError> {
        borrower.require_auth();
        Self::init_insurance_fund_if_needed(&env);

        // Get loan record
        let loan_key = DataKey::LoanById(loan_id);
        let loan = env
            .storage()
            .instance()
            .get::<_, LoanRecord>(&loan_key)
            .ok_or(LendingError::LoanNotFound)?;

        // Verify borrower matches
        if loan.borrower != borrower {
            return Err(LendingError::Unauthorized);
        }

        // Check if insurance already exists for this loan
        let insurance_key = DataKey::Insurance(loan_id);
        if env.storage().instance().has(&insurance_key) {
            return Err(LendingError::InsuranceAlreadyPurchased);
        }

        // Calculate premium
        let premium_rate_bps: u32 = env
            .storage()
            .instance()
            .get::<_, u32>(&DataKey::InsurancePremiumRate)
            .unwrap_or(DEFAULT_INSURANCE_PREMIUM_RATE_BPS);

        let premium = (loan.principal as u128)
            .checked_mul(premium_rate_bps as u128)
            .and_then(|v| v.checked_div(10000u128))
            .ok_or(LendingError::InvalidAmount)? as u64;

        if premium == 0 {
            return Err(LendingError::InvalidInsuranceAmount);
        }

        // Transfer premium from borrower to insurance fund (using underlying token)
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token);

        token_client.transfer(
            &borrower,
            &env.current_contract_address(),
            &(premium as i128),
        );

        // Coverage is 100% of principal
        let coverage_amount = loan.principal;

        // Create insurance record
        let insurance = LoanInsurance {
            loan_id,
            borrower: borrower.clone(),
            coverage_amount,
            premium_paid: premium,
            premium_rate_bps,
            purchase_time: env.ledger().timestamp(),
            expires_at: loan.due_date,
            claimed: false,
        };

        env.storage().instance().set(&insurance_key, &insurance);

        // Update insurance fund
        let mut fund: InsuranceFund = env
            .storage()
            .instance()
            .get(&DataKey::InsuranceFund)
            .unwrap_or(InsuranceFund {
                total_premiums_collected: 0,
                total_claims_paid: 0,
                available_balance: 0,
            });

        fund.total_premiums_collected = fund.total_premiums_collected.saturating_add(premium);
        fund.available_balance = fund.available_balance.saturating_add(premium);

        env.storage().instance().set(&DataKey::InsuranceFund, &fund);

        // Emit event
        env.events().publish(
            (symbol_short!("INS"), symbol_short!("BOUGHT")),
            InsurancePurchasedEvent {
                loan_id,
                borrower: borrower.clone(),
                coverage_amount,
                premium_paid: premium,
                premium_rate_bps,
                expires_at: loan.due_date,
                timestamp: env.ledger().timestamp(),
            },
        );

        log!(
            &env,
            "InsurancePurchased: loan_id={}, borrower={}, premium={}, coverage={}",
            loan_id,
            borrower,
            premium,
            coverage_amount
        );

        Ok(premium)
    }

    /// Check if a loan has active insurance
    pub fn is_loan_insured(env: Env, loan_id: u64) -> Result<bool, LendingError> {
        let insurance_key = DataKey::Insurance(loan_id);
        let insurance: Option<LoanInsurance> = env.storage().instance().get(&insurance_key);

        if let Some(ins) = insurance {
            // Check if not expired and not already claimed
            let current_time = env.ledger().timestamp();
            Ok(!ins.claimed && current_time < ins.expires_at)
        } else {
            Ok(false)
        }
    }

    /// Get insurance coverage amount for a loan
    pub fn get_insurance_coverage(env: Env, loan_id: u64) -> Result<u64, LendingError> {
        let insurance_key = DataKey::Insurance(loan_id);
        let insurance: Option<LoanInsurance> = env.storage().instance().get(&insurance_key);

        if let Some(ins) = insurance {
            if !ins.claimed && env.ledger().timestamp() < ins.expires_at {
                return Ok(ins.coverage_amount);
            }
        }

        Ok(0)
    }

    /// Get insurance details for a loan
    pub fn get_insurance_details(
        env: Env,
        loan_id: u64,
    ) -> Result<Option<LoanInsurance>, LendingError> {
        let insurance_key = DataKey::Insurance(loan_id);
        Ok(env.storage().instance().get(&insurance_key))
    }

    /// Claim insurance when loan defaults
    pub fn claim_insurance(env: Env, loan_id: u64) -> Result<u64, LendingError> {
        let insurance_key = DataKey::Insurance(loan_id);
        let mut insurance: LoanInsurance = env
            .storage()
            .instance()
            .get(&insurance_key)
            .ok_or(LendingError::InsuranceNotFound)?;

        // Check if insurance is still valid
        if insurance.claimed {
            return Err(LendingError::InsuranceAlreadyClaimed);
        }

        let current_time = env.ledger().timestamp();
        if current_time >= insurance.expires_at {
            return Err(LendingError::InsuranceExpired);
        }

        // Get loan to verify it exists and is in default
        let loan_key = DataKey::LoanById(loan_id);
        let loan = env
            .storage()
            .instance()
            .get::<_, LoanRecord>(&loan_key)
            .ok_or(LendingError::LoanNotFound)?;

        // Verify loan is past due
        if current_time <= loan.due_date {
            return Err(LendingError::InvalidAmount);
        }

        // Get insurance fund and verify sufficient balance
        let mut fund: InsuranceFund = env
            .storage()
            .instance()
            .get(&DataKey::InsuranceFund)
            .ok_or(LendingError::InsuranceNotFound)?;

        let claim_amount = insurance.coverage_amount;

        if fund.available_balance < claim_amount {
            return Err(LendingError::InsufficientInsuranceFund);
        }

        // Mark as claimed
        insurance.claimed = true;
        env.storage().instance().set(&insurance_key, &insurance);

        // Update insurance fund
        fund.total_claims_paid = fund.total_claims_paid.saturating_add(claim_amount);
        fund.available_balance = fund.available_balance.saturating_sub(claim_amount);
        env.storage().instance().set(&DataKey::InsuranceFund, &fund);

        // Transfer claim amount to contract (funds holder for protocol)
        // In a real system, this would be transferred to a claims reserve
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token);

        // Transfer from contract to claims reserve (in this case, just update balance tracking)
        // The actual transfer would happen when liquidation processes the claim

        // Emit event
        env.events().publish(
            (symbol_short!("INS"), symbol_short!("CLAIM")),
            InsuranceClaimedEvent {
                loan_id,
                borrower: insurance.borrower.clone(),
                claim_amount,
                coverage_amount: insurance.coverage_amount,
                timestamp: current_time,
            },
        );

        log!(
            &env,
            "InsuranceClaimed: loan_id={}, claim_amount={}, remaining_fund={}",
            loan_id,
            claim_amount,
            fund.available_balance
        );

        Ok(claim_amount)
    }

    /// Cancel insurance and get partial refund (pro-rata based on time remaining)
    pub fn cancel_insurance(
        env: Env,
        borrower: Address,
        loan_id: u64,
    ) -> Result<u64, LendingError> {
        borrower.require_auth();

        let insurance_key = DataKey::Insurance(loan_id);
        let insurance: LoanInsurance = env
            .storage()
            .instance()
            .get(&insurance_key)
            .ok_or(LendingError::InsuranceNotFound)?;

        // Verify borrower matches
        if insurance.borrower != borrower {
            return Err(LendingError::Unauthorized);
        }

        // Cannot cancel claimed insurance
        if insurance.claimed {
            return Err(LendingError::InsuranceAlreadyClaimed);
        }

        let current_time = env.ledger().timestamp();

        // Calculate time-based refund: if cancelled before expiry, refund pro-rata
        let refund_amount = if current_time < insurance.expires_at {
            let total_duration = insurance.expires_at.saturating_sub(insurance.purchase_time);
            let elapsed = current_time.saturating_sub(insurance.purchase_time);
            let remaining = total_duration.saturating_sub(elapsed);

            // Refund = premium * (remaining time / total time)
            if total_duration > 0 {
                (insurance.premium_paid as u128)
                    .checked_mul(remaining as u128)
                    .and_then(|v| v.checked_div(total_duration as u128))
                    .unwrap_or(0) as u64
            } else {
                0
            }
        } else {
            // Insurance expired, no refund
            0
        };

        // Remove insurance record
        env.storage().instance().remove(&insurance_key);

        // Update insurance fund
        if refund_amount > 0 {
            let mut fund: InsuranceFund = env
                .storage()
                .instance()
                .get(&DataKey::InsuranceFund)
                .unwrap_or(InsuranceFund {
                    total_premiums_collected: 0,
                    total_claims_paid: 0,
                    available_balance: 0,
                });

            fund.available_balance = fund.available_balance.saturating_sub(refund_amount);
            env.storage().instance().set(&DataKey::InsuranceFund, &fund);

            // Transfer refund to borrower
            let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
            let token_client = token::Client::new(&env, &token);
            token_client.transfer(
                &env.current_contract_address(),
                &borrower,
                &(refund_amount as i128),
            );
        }

        // Emit event
        env.events().publish(
            (symbol_short!("INS"), symbol_short!("CANC")),
            InsuranceCancelledEvent {
                loan_id,
                borrower: borrower.clone(),
                refund_amount,
                timestamp: current_time,
            },
        );

        log!(
            &env,
            "InsuranceCancelled: loan_id={}, refund_amount={}",
            loan_id,
            refund_amount
        );

        Ok(refund_amount)
    }

    /// Get insurance fund state
    pub fn get_insurance_fund_state(env: Env) -> Result<InsuranceFund, LendingError> {
        Self::init_insurance_fund_if_needed(&env);
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::InsuranceFund)
            .unwrap_or(InsuranceFund {
                total_premiums_collected: 0,
                total_claims_paid: 0,
                available_balance: 0,
            }))
    }

    /// Deposit funds to insurance fund (admin function for funding)
    pub fn deposit_to_insurance_fund(
        env: Env,
        admin: Address,
        amount: u64,
    ) -> Result<(), LendingError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        Self::init_insurance_fund_if_needed(&env);

        // Transfer from admin to contract
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&admin, &env.current_contract_address(), &(amount as i128));

        // Update insurance fund balance
        let mut fund: InsuranceFund = env
            .storage()
            .instance()
            .get(&DataKey::InsuranceFund)
            .unwrap_or(InsuranceFund {
                total_premiums_collected: 0,
                total_claims_paid: 0,
                available_balance: 0,
            });

        fund.available_balance = fund.available_balance.saturating_add(amount);
        env.storage().instance().set(&DataKey::InsuranceFund, &fund);

        log!(
            &env,
            "InsuranceFundDeposited: amount={}, new_balance={}",
            amount,
            fund.available_balance
        );

        Ok(())
    }

    /// Withdraw from insurance fund (admin function)
    pub fn withdraw_from_insurance_fund(
        env: Env,
        admin: Address,
        amount: u64,
    ) -> Result<(), LendingError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        Self::init_insurance_fund_if_needed(&env);

        let mut fund: InsuranceFund = env
            .storage()
            .instance()
            .get(&DataKey::InsuranceFund)
            .ok_or(LendingError::InsuranceNotFound)?;

        if fund.available_balance < amount {
            return Err(LendingError::InsufficientLiquidity);
        }

        fund.available_balance = fund.available_balance.saturating_sub(amount);
        env.storage().instance().set(&DataKey::InsuranceFund, &fund);

        // Transfer to admin
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &admin, &(amount as i128));

        log!(
            &env,
            "InsuranceFundWithdrawn: amount={}, new_balance={}",
            amount,
            fund.available_balance
        );

        Ok(())
    }

    // ─── Cross-Contract Integration ──────────────────────────────

    pub fn set_inheritance_contract(
        env: Env,
        admin: Address,
        contract: Address,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        env.storage()
            .instance()
            .set(&DataKey::InheritanceContract, &contract);
        env.events().publish(
            (
                soroban_sdk::symbol_short!("LINK"),
                soroban_sdk::symbol_short!("INHERIT"),
            ),
            ContractLinkedEvent {
                contract_type: soroban_sdk::symbol_short!("INHERIT"),
                address: contract,
            },
        );
        Ok(())
    }

    pub fn get_inheritance_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::InheritanceContract)
    }

    pub fn set_governance_contract(
        env: Env,
        admin: Address,
        contract: Address,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;
        env.storage()
            .instance()
            .set(&DataKey::GovernanceContract, &contract);
        env.events().publish(
            (
                soroban_sdk::symbol_short!("LINK"),
                soroban_sdk::symbol_short!("GOV"),
            ),
            ContractLinkedEvent {
                contract_type: soroban_sdk::symbol_short!("GOV"),
                address: contract,
            },
        );
        Ok(())
    }

    pub fn get_governance_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::GovernanceContract)
    }

    // ─── Interest Rate Model ─────────────────────────

    /// Update the interest rate model parameters. Admin only.
    pub fn set_rate_model(
        env: Env,
        admin: Address,
        base_rate_bps: u32,
        optimal_utilization_bps: u32,
        slope1_bps: u32,
        slope2_bps: u32,
        reserve_factor_bps: u32,
    ) -> Result<(), LendingError> {
        Self::require_admin(&env, &admin)?;

        if optimal_utilization_bps == 0 || optimal_utilization_bps >= 10000 {
            return Err(LendingError::InvalidRateModel);
        }
        if reserve_factor_bps >= 10000 {
            return Err(LendingError::InvalidRateModel);
        }

        let model = RateModel {
            base_rate_bps,
            optimal_utilization_bps,
            slope1_bps,
            slope2_bps,
            reserve_factor_bps,
        };

        env.storage().instance().set(&DataKey::RateModel, &model);

        env.events().publish(
            (symbol_short!("RATE"), symbol_short!("MODEL")),
            RateModelUpdatedEvent {
                base_rate_bps,
                optimal_utilization_bps,
                slope1_bps,
                slope2_bps,
                reserve_factor_bps,
                updated_by: admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Get the base interest rate from the rate model, falling back to pool state.
    pub fn get_base_rate(env: Env) -> Result<u32, LendingError> {
        Self::require_initialized(&env)?;
        if let Some(model) = env
            .storage()
            .instance()
            .get::<DataKey, RateModel>(&DataKey::RateModel)
        {
            return Ok(model.base_rate_bps);
        }
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        Ok(Self::get_pool(&env, &token)?.base_rate_bps)
    }

    /// Get the optimal (target) utilization rate from the rate model.
    pub fn get_optimal_utilization(env: Env) -> Result<u32, LendingError> {
        Self::require_initialized(&env)?;
        if let Some(model) = env
            .storage()
            .instance()
            .get::<DataKey, RateModel>(&DataKey::RateModel)
        {
            return Ok(model.optimal_utilization_bps);
        }
        // Default: use utilization cap as the optimal target
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        Ok(Self::get_pool(&env, &token)?.utilization_cap_bps)
    }

    /// Get slope1 — the rate increase per unit utilization before optimal utilization.
    pub fn get_slope1(env: Env) -> Result<u32, LendingError> {
        Self::require_initialized(&env)?;
        if let Some(model) = env
            .storage()
            .instance()
            .get::<DataKey, RateModel>(&DataKey::RateModel)
        {
            return Ok(model.slope1_bps);
        }
        // Fallback: use pool multiplier as slope1
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        Ok(Self::get_pool(&env, &token)?.multiplier_bps)
    }

    /// Get slope2 — the steep rate increase per unit utilization above optimal utilization.
    pub fn get_slope2(env: Env) -> Result<u32, LendingError> {
        Self::require_initialized(&env)?;
        if let Some(model) = env
            .storage()
            .instance()
            .get::<DataKey, RateModel>(&DataKey::RateModel)
        {
            return Ok(model.slope2_bps);
        }
        // Fallback: slope2 is 10× slope1 when not configured
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        Ok(Self::get_pool(&env, &token)?.multiplier_bps.saturating_mul(10))
    }

    /// Get the current borrow rate using the two-slope model if configured,
    /// or the legacy linear model otherwise.
    pub fn get_borrow_rate(env: Env) -> Result<u32, LendingError> {
        Self::require_initialized(&env)?;
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let pool = Self::get_pool(&env, &token)?;
        let utilization_bps = Self::get_utilization_bps(pool.total_borrowed, pool.total_deposits);

        if let Some(model) = env
            .storage()
            .instance()
            .get::<DataKey, RateModel>(&DataKey::RateModel)
        {
            return Ok(Self::two_slope_rate(&model, utilization_bps));
        }

        Ok(Self::calculate_dynamic_rate(
            pool.base_rate_bps,
            pool.multiplier_bps,
            utilization_bps,
        ))
    }

    /// Get the current supply (deposit) rate.
    /// supply_rate = borrow_rate × utilization × (1 − reserve_factor)
    pub fn get_supply_rate(env: Env) -> Result<u32, LendingError> {
        Self::require_initialized(&env)?;
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let pool = Self::get_pool(&env, &token)?;
        let utilization_bps = Self::get_utilization_bps(pool.total_borrowed, pool.total_deposits);
        let borrow_rate = Self::get_borrow_rate(env.clone())?;

        let reserve_factor = if let Some(model) = env
            .storage()
            .instance()
            .get::<DataKey, RateModel>(&DataKey::RateModel)
        {
            model.reserve_factor_bps
        } else {
            pool.reserve_factor_bps
        };

        // supply_rate = borrow_rate * utilization * (10000 - reserve_factor) / 10000^2
        let supply_rate = (borrow_rate as u128)
            .checked_mul(utilization_bps as u128)
            .unwrap_or(0)
            .checked_mul((10000u32.saturating_sub(reserve_factor)) as u128)
            .unwrap_or(0)
            / (10000u128 * 10000u128);

        Ok(supply_rate as u32)
    }

    /// Simulate the borrow rate at an arbitrary utilization level (in basis points).
    /// Useful for modelling rate impact before taking on or repaying debt.
    pub fn simulate_rate(env: Env, utilization_bps: u32) -> Result<u32, LendingError> {
        Self::require_initialized(&env)?;
        if let Some(model) = env
            .storage()
            .instance()
            .get::<DataKey, RateModel>(&DataKey::RateModel)
        {
            return Ok(Self::two_slope_rate(&model, utilization_bps));
        }
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let pool = Self::get_pool(&env, &token)?;
        Ok(Self::calculate_dynamic_rate(
            pool.base_rate_bps,
            pool.multiplier_bps,
            utilization_bps,
        ))
    }

    /// Two-slope interest rate calculation.
    fn two_slope_rate(model: &RateModel, utilization_bps: u32) -> u32 {
        let optimal = model.optimal_utilization_bps;
        if utilization_bps <= optimal {
            // Linear ramp up to slope1 at optimal utilization
            let variable = (utilization_bps as u64)
                .checked_mul(model.slope1_bps as u64)
                .unwrap_or(0)
                / optimal as u64;
            model.base_rate_bps.saturating_add(variable as u32)
        } else {
            // Above optimal: base + slope1 + steep slope2 portion
            let excess = utilization_bps.saturating_sub(optimal);
            let max_excess = (10000u32).saturating_sub(optimal);
            let steep = if max_excess == 0 {
                model.slope2_bps as u64
            } else {
                (excess as u64)
                    .checked_mul(model.slope2_bps as u64)
                    .unwrap_or(0)
                    / max_excess as u64
            };
            model
                .base_rate_bps
                .saturating_add(model.slope1_bps)
                .saturating_add(steep as u32)
        }
    }

    pub fn verify_plan_ownership(env: Env, plan_id: u64, caller: Address) -> bool {
        if let Some(inheritance_contract) = Self::get_inheritance_contract(env.clone()) {
            let client = InheritanceContractClient::new(&env, &inheritance_contract);
            client.verify_plan_ownership(&plan_id, &caller)
        } else {
            false
        }
    }
}

mod cross_contract_test;
mod test;

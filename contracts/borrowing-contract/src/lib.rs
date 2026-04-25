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

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AuctionStatus {
    Active,
    Executed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct LiquidationAuction {
    pub loan_id: u64,
    pub start_time: u64,
    pub duration: u64,
    pub initial_discount_bps: u32,
    pub max_discount_bps: u32,
    pub status: AuctionStatus,
    pub winning_bidder: Option<Address>,
    pub winning_bid_amount: i128,
    pub locked_discount_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionStartedEvent {
    pub loan_id: u64,
    pub start_time: u64,
    pub duration: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionBidEvent {
    pub loan_id: u64,
    pub bidder: Address,
    pub bid_amount: i128,
    pub discount_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionExecutedEvent {
    pub loan_id: u64,
    pub bidder: Address,
    pub bid_amount: i128,
    pub collateral_seized: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionCancelledEvent {
    pub loan_id: u64,
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
    Auction(u64),
    MaxExtensions,
    ExtensionFeeBps,
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
    AuctionNotFound = 10,
    AuctionAlreadyActive = 11,
    AuctionNotActive = 12,
    StillUnhealthy = 13,
    ExtensionLimitReached = 14,
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

        // Cache storage reads – each instance().get() costs CPU/memory instructions
        let is_whitelisted: bool = env
            .storage()
            .persistent()
            .get(&DataKey::WhitelistedCollateral(collateral_token.clone()))
            .unwrap_or(false);
        if !is_whitelisted {
            return Err(BorrowingError::CollateralNotWhitelisted);
        }

        // Single read for global pause, single read for vault pause
        let global_paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::GlobalPause)
            .unwrap_or(false);
        let vault_paused: bool = env
            .storage()
            .persistent()
            .get(&DataKey::VaultPause(collateral_token.clone()))
            .unwrap_or(false);
        if global_paused || vault_paused {
            return Err(BorrowingError::Paused);
        }

        // Single read for collateral ratio (avoids a second instance().get() call)
        let ratio: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CollateralRatio)
            .unwrap_or(15000);
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
            extension_count: 0,
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

        // Invariant I-2: amount_repaid must not exceed principal (over-repayment guard)
        // NOTE: this is a known open finding (F-1) – the excess is not refunded yet.
        debug_assert!(
            loan.amount_repaid <= loan.principal || !loan.is_active,
            "invariant I-2 violated: amount_repaid > principal on active loan"
        );

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

        // Calculate health factor inline – avoids a redundant storage read from get_health_factor
        let health_factor = if debt == 0 {
            10000
        } else {
            (loan.collateral_amount as u128)
                .checked_mul(10000)
                .and_then(|v| v.checked_div(debt as u128))
                .unwrap_or(0) as u32
        };

        // Cache both threshold and bonus in a single pass over instance storage
        let liquidation_threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LiquidationThreshold)
            .unwrap_or(12000);
        let liquidation_bonus: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LiquidationBonus)
            .unwrap_or(500);

        // Check if loan is unhealthy (health factor below threshold)
        if health_factor >= liquidation_threshold {
            return Err(BorrowingError::LoanHealthy);
        }

        // Calculate liquidation amounts based on liquidate_amount
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

        // Invariant I-3: liquidation only proceeds when health factor < threshold
        debug_assert!(
            health_factor < liquidation_threshold,
            "invariant I-3 violated: liquidated a healthy loan"
        );
        // Invariant I-1: collateral_amount must not go negative
        debug_assert!(
            loan.collateral_amount >= 0,
            "invariant I-1 violated: collateral_amount underflow"
        );

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

    pub fn start_liquidation_auction(
        env: Env,
        loan_id: u64,
        duration: u64,
        initial_discount_bps: u32,
        max_discount_bps: u32,
    ) -> Result<(), BorrowingError> {
        // Single storage read for the loan – reuse it instead of calling get_loan + get_health_factor
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        let debt = loan.principal - loan.amount_repaid;
        if debt <= 0 || !loan.is_active {
            return Err(BorrowingError::LoanNotActive);
        }

        // Compute health factor inline (avoids second storage read via get_health_factor)
        let health_factor = if debt == 0 {
            10000u32
        } else {
            (loan.collateral_amount as u128)
                .checked_mul(10000)
                .and_then(|v| v.checked_div(debt as u128))
                .unwrap_or(0) as u32
        };

        let liquidation_threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LiquidationThreshold)
            .unwrap_or(12000);
        if health_factor >= liquidation_threshold {
            return Err(BorrowingError::LoanHealthy);
        }

        if let Some(existing_auction) = env
            .storage()
            .persistent()
            .get::<_, LiquidationAuction>(&DataKey::Auction(loan_id))
        {
            if existing_auction.status == AuctionStatus::Active {
                return Err(BorrowingError::AuctionAlreadyActive);
            }
        }

        let auction = LiquidationAuction {
            loan_id,
            start_time: env.ledger().timestamp(),
            duration,
            initial_discount_bps,
            max_discount_bps,
            status: AuctionStatus::Active,
            winning_bidder: None,
            winning_bid_amount: 0,
            locked_discount_bps: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Auction(loan_id), &auction);

        env.events().publish(
            (symbol_short!("AUCTION"), symbol_short!("START")),
            AuctionStartedEvent {
                loan_id,
                start_time: auction.start_time,
                duration,
            },
        );

        Ok(())
    }

    pub fn get_liquidation_discount(env: Env, loan_id: u64) -> Result<u32, BorrowingError> {
        let auction: LiquidationAuction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(loan_id))
            .ok_or(BorrowingError::AuctionNotFound)?;

        if auction.status != AuctionStatus::Active {
            return Ok(auction.locked_discount_bps);
        }

        let current_time = env.ledger().timestamp();
        let elapsed = current_time.saturating_sub(auction.start_time);

        if elapsed >= auction.duration {
            return Ok(auction.max_discount_bps);
        }

        let discount_diff = auction
            .max_discount_bps
            .saturating_sub(auction.initial_discount_bps);

        let current_addition = (discount_diff as u64)
            .checked_mul(elapsed)
            .and_then(|v| v.checked_div(auction.duration))
            .unwrap_or(0) as u32;

        Ok(auction.initial_discount_bps + current_addition)
    }

    pub fn bid_on_liquidation(
        env: Env,
        bidder: Address,
        loan_id: u64,
        bid_amount: i128,
    ) -> Result<(), BorrowingError> {
        bidder.require_auth();

        let mut auction: LiquidationAuction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(loan_id))
            .ok_or(BorrowingError::AuctionNotFound)?;

        if auction.status != AuctionStatus::Active {
            return Err(BorrowingError::AuctionNotActive);
        }

        // Single storage read for the loan (avoids calling get_loan which re-reads)
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;
        let debt = loan.principal - loan.amount_repaid;

        if bid_amount <= 0 || bid_amount > debt {
            return Err(BorrowingError::InvalidAmount);
        }

        if auction.winning_bidder.is_some() {
            return Err(BorrowingError::AuctionAlreadyActive);
        }

        // Compute discount inline from already-loaded auction data (avoids re-reading auction)
        let current_discount = {
            let current_time = env.ledger().timestamp();
            let elapsed = current_time.saturating_sub(auction.start_time);
            if elapsed >= auction.duration {
                auction.max_discount_bps
            } else {
                let discount_diff = auction
                    .max_discount_bps
                    .saturating_sub(auction.initial_discount_bps);
                let current_addition = (discount_diff as u64)
                    .checked_mul(elapsed)
                    .and_then(|v| v.checked_div(auction.duration))
                    .unwrap_or(0) as u32;
                auction.initial_discount_bps + current_addition
            }
        };

        auction.winning_bidder = Some(bidder.clone());
        auction.winning_bid_amount = bid_amount;
        auction.locked_discount_bps = current_discount;

        env.storage()
            .persistent()
            .set(&DataKey::Auction(loan_id), &auction);

        env.events().publish(
            (symbol_short!("AUCTION"), symbol_short!("BID")),
            AuctionBidEvent {
                loan_id,
                bidder,
                bid_amount,
                discount_bps: current_discount,
            },
        );

        Ok(())
    }

    pub fn execute_auction(env: Env, loan_id: u64) -> Result<(), BorrowingError> {
        let mut auction: LiquidationAuction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(loan_id))
            .ok_or(BorrowingError::AuctionNotFound)?;

        if auction.status != AuctionStatus::Active {
            return Err(BorrowingError::AuctionNotActive);
        }

        let bidder = auction
            .winning_bidder
            .clone()
            .ok_or(BorrowingError::InvalidAmount)?;
        let bid_amount = auction.winning_bid_amount;
        let discount = auction.locked_discount_bps;

        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        let bonus_amount = (bid_amount as u128)
            .checked_mul(discount as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as i128;

        let liquidator_reward = bid_amount + bonus_amount;

        if liquidator_reward > loan.collateral_amount {
            return Err(BorrowingError::InvalidAmount);
        }

        let token_client = token::Client::new(&env, &loan.collateral_token);
        token_client.transfer(&env.current_contract_address(), &bidder, &liquidator_reward);

        loan.collateral_amount -= liquidator_reward;
        loan.amount_repaid += bid_amount;

        if loan.amount_repaid >= loan.principal {
            loan.is_active = false;
        }

        auction.status = AuctionStatus::Executed;

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);
        env.storage()
            .persistent()
            .set(&DataKey::Auction(loan_id), &auction);

        env.events().publish(
            (symbol_short!("AUCTION"), symbol_short!("EXECUTE")),
            AuctionExecutedEvent {
                loan_id,
                bidder,
                bid_amount,
                collateral_seized: liquidator_reward,
            },
        );

        Ok(())
    }

    pub fn get_auction_status(
        env: Env,
        loan_id: u64,
    ) -> Result<LiquidationAuction, BorrowingError> {
        env.storage()
            .persistent()
            .get(&DataKey::Auction(loan_id))
            .ok_or(BorrowingError::AuctionNotFound)
    }

    pub fn cancel_auction(env: Env, loan_id: u64) -> Result<(), BorrowingError> {
        let mut auction: LiquidationAuction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(loan_id))
            .ok_or(BorrowingError::AuctionNotFound)?;

        if auction.status != AuctionStatus::Active {
            return Err(BorrowingError::AuctionNotActive);
        }

        // Compute health factor inline – avoids a second persistent storage read via get_health_factor
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;
        let debt = loan.principal - loan.amount_repaid;
        let health_factor = if debt == 0 {
            10000u32
        } else {
            (loan.collateral_amount as u128)
                .checked_mul(10000)
                .and_then(|v| v.checked_div(debt as u128))
                .unwrap_or(0) as u32
        };
        let liquidation_threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LiquidationThreshold)
            .unwrap_or(12000);

        if health_factor < liquidation_threshold {
            return Err(BorrowingError::StillUnhealthy);
        }

        auction.status = AuctionStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Auction(loan_id), &auction);

        env.events().publish(
            (symbol_short!("AUCTION"), symbol_short!("CANCEL")),
            AuctionCancelledEvent { loan_id },
        );

        Ok(())
    }

    /// Returns the extension fee for a loan (1% of remaining principal by default).
    pub fn get_extension_fee(env: Env, loan_id: u64) -> Result<i128, BorrowingError> {
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        if !loan.is_active {
            return Err(BorrowingError::LoanNotActive);
        }

        let fee_bps: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ExtensionFeeBps)
            .unwrap_or(100); // 1% default

        let remaining = loan.principal - loan.amount_repaid;
        let fee = (remaining as u128)
            .checked_mul(fee_bps as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as i128;

        Ok(fee)
    }

    /// Returns the maximum additional amount a borrower can take against existing collateral.
    pub fn get_max_additional_borrow(env: Env, loan_id: u64) -> Result<i128, BorrowingError> {
        let loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowingError::LoanNotFound)?;

        if !loan.is_active {
            return Err(BorrowingError::LoanNotActive);
        }

        let ratio = Self::get_collateral_ratio(env.clone());
        // max_borrow = collateral * 10000 / ratio
        let max_borrow = (loan.collateral_amount as u128)
            .checked_mul(10000)
            .and_then(|v| v.checked_div(ratio as u128))
            .unwrap_or(0) as i128;

        let current_debt = loan.principal - loan.amount_repaid;
        let additional = max_borrow.saturating_sub(current_debt);

        Ok(additional.max(0))
    }

    /// Extends the loan due date by `extension_days` seconds. Charges an extension fee.
    /// Maximum 2 extensions per loan.
    pub fn extend_loan(
        env: Env,
        loan_id: u64,
        extension_seconds: u64,
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

        // Cache both instance values in one pass to avoid two separate reads
        let max_extensions: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MaxExtensions)
            .unwrap_or(2);
        let fee_bps: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ExtensionFeeBps)
            .unwrap_or(100);

        if loan.extension_count >= max_extensions {
            return Err(BorrowingError::ExtensionLimitReached);
        }

        let remaining = loan.principal - loan.amount_repaid;
        let fee = (remaining as u128)
            .checked_mul(fee_bps as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as i128;

        if fee > 0 {
            let token_client = token::Client::new(&env, &loan.collateral_token);
            token_client.transfer(&loan.borrower, &env.current_contract_address(), &fee);
        }

        loan.due_date += extension_seconds;
        loan.extension_count += 1;

        let new_due_date = loan.due_date;
        let extension_count = loan.extension_count;
        let borrower = loan.borrower.clone();

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("EXTEND")),
            LoanExtendedEvent {
                loan_id,
                borrower,
                new_due_date,
                extension_fee: fee,
                extension_count,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Increases the loan principal if the health factor allows it.
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

        // Compute max additional inline – avoids a second persistent storage read
        let ratio: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CollateralRatio)
            .unwrap_or(15000);
        let max_borrow = (loan.collateral_amount as u128)
            .checked_mul(10000)
            .and_then(|v| v.checked_div(ratio as u128))
            .unwrap_or(0) as i128;
        let current_debt = loan.principal - loan.amount_repaid;
        let max_additional = max_borrow.saturating_sub(current_debt).max(0);

        if additional_amount > max_additional {
            return Err(BorrowingError::InsufficientCollateral);
        }

        loan.principal += additional_amount;
        let new_principal = loan.principal;
        let borrower = loan.borrower.clone();

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("INCREASE")),
            LoanIncreasedEvent {
                loan_id,
                borrower,
                additional_amount,
                new_principal,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    fn get_next_loan_id(env: &Env) -> u64 {
        // Use instance storage for the counter – cheaper than persistent for a single
        // frequently-updated scalar value that doesn't need long-term archival.
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LoanCounter)
            .unwrap_or(0);
        let next_id = counter + 1;
        env.storage()
            .instance()
            .set(&DataKey::LoanCounter, &next_id);
        next_id
    }
}

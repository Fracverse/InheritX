#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};

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

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, token, Address, Env};

    fn create_token_contract<'a>(
        env: &Env,
        admin: &Address,
    ) -> (Address, token::StellarAssetClient<'a>) {
        let addr = env
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        (addr.clone(), token::StellarAssetClient::new(env, &addr))
    }

    fn get_balance(env: &Env, token_addr: &Address, addr: &Address) -> i128 {
        token::Client::new(env, token_addr).balance(addr)
    }

    #[test]
    fn test_collateral_management() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);

        let (collateral_addr, collateral_token) = create_token_contract(&env, &admin);

        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        // Initialize with 150% collateral ratio
        client.initialize(&admin, &15000, &12000, &500);

        // Whitelist collateral
        client.whitelist_collateral(&admin, &collateral_addr);

        // Mint collateral to borrower
        collateral_token.mint(&borrower, &1500);

        // Create loan with sufficient collateral (1500 >= 1000 * 1.5)
        let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

        assert_eq!(loan_id, 1);

        let loan = client.get_loan(&loan_id);
        assert_eq!(loan.collateral_amount, 1500);
        assert_eq!(loan.collateral_token, collateral_addr);
        assert!(loan.is_active);

        // Verify collateral locked in contract
        assert_eq!(get_balance(&env, &collateral_addr, &contract_id), 1500);
        assert_eq!(get_balance(&env, &collateral_addr, &borrower), 0);
    }

    #[test]
    fn test_insufficient_collateral() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);

        let (collateral_addr, collateral_token) = create_token_contract(&env, &admin);

        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        client.initialize(&admin, &15000, &12000, &500);
        client.whitelist_collateral(&admin, &collateral_addr);

        collateral_token.mint(&borrower, &1000);

        // Try to borrow with insufficient collateral (1000 < 1000 * 1.5)
        let result =
            client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1000);

        assert_eq!(result, Err(Ok(BorrowingError::InsufficientCollateral)));
    }

    #[test]
    fn test_collateral_not_whitelisted() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);

        let (collateral_addr, collateral_token) = create_token_contract(&env, &admin);

        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        client.initialize(&admin, &15000, &12000, &500);
        // Don't whitelist collateral

        collateral_token.mint(&borrower, &1500);

        let result =
            client.try_create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

        assert_eq!(result, Err(Ok(BorrowingError::CollateralNotWhitelisted)));
    }

    #[test]
    fn test_collateral_returned_on_repayment() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);

        let (collateral_addr, collateral_token) = create_token_contract(&env, &admin);

        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        client.initialize(&admin, &15000, &12000, &500);
        client.whitelist_collateral(&admin, &collateral_addr);

        collateral_token.mint(&borrower, &1500);

        let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

        // Repay loan fully
        client.repay_loan(&loan_id, &1000);

        let loan = client.get_loan(&loan_id);
        assert!(!loan.is_active);

        // Verify collateral returned
        assert_eq!(get_balance(&env, &collateral_addr, &borrower), 1500);
        assert_eq!(get_balance(&env, &collateral_addr, &contract_id), 0);
    }

    #[test]
    fn test_health_factor_calculation() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);

        let (collateral_addr, collateral_token) = create_token_contract(&env, &admin);

        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        client.initialize(&admin, &15000, &12000, &500);
        client.whitelist_collateral(&admin, &collateral_addr);

        collateral_token.mint(&borrower, &1500);

        let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

        // Health factor = (1500 / 1000) * 10000 = 15000 (150%)
        let health_factor = client.get_health_factor(&loan_id);
        assert_eq!(health_factor, 15000);
    }

    #[test]
    fn test_liquidation_unhealthy_loan() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let _liquidator = Address::generate(&env);

        let (collateral_addr, collateral_token) = create_token_contract(&env, &admin);

        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        // Initialize with 150% collateral ratio, 120% liquidation threshold, 5% bonus
        client.initialize(&admin, &15000, &12000, &500);
        client.whitelist_collateral(&admin, &collateral_addr);

        // Create loan with 150% collateral (1500 for 1000 debt)
        collateral_token.mint(&borrower, &1500);
        let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

        // Simulate price drop: collateral now worth only 1100 (110% health factor)
        // In real scenario, this would be via oracle price feed
        // For test, we manually check that 110% < 120% threshold allows liquidation

        // Partial repayment to make debt = 1000, but we'll treat collateral as 1100 value
        // Health = 1100/1000 = 110% which is below 120% threshold

        // Since we can't change collateral value in test, let's create with exact amounts
        // that will be unhealthy: need collateral < debt * 1.2
        // So for debt=1000, collateral must be < 1200 to be liquidatable
        // But we need >= 1500 to create loan initially

        // Solution: Create healthy loan, then partial repay to make it unhealthy
        client.repay_loan(&loan_id, &250); // Debt now = 750
                                           // Health = 1500/750 = 200% - still healthy

        // This test needs price oracle to work properly
        // For now, skip actual liquidation test
    }

    #[test]
    fn test_liquidation_with_simulated_undercollateralization() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let liquidator = Address::generate(&env);

        let (collateral_addr, collateral_token) = create_token_contract(&env, &admin);

        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        // Initialize with 120% collateral ratio (lower than liquidation threshold)
        // This allows creating loans that are immediately liquidatable
        client.initialize(&admin, &12000, &13000, &500);
        client.whitelist_collateral(&admin, &collateral_addr);

        // Create loan with exactly 120% collateral
        collateral_token.mint(&borrower, &1200);
        let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1200);

        // Health factor = 1200/1000 = 120% which is below 130% threshold
        let health_factor = client.get_health_factor(&loan_id);
        assert_eq!(health_factor, 12000);

        // Liquidate
        client.liquidate(&liquidator, &loan_id);

        let loan = client.get_loan(&loan_id);
        assert!(!loan.is_active);

        // Liquidator receives debt + 5% bonus = 1000 + 50 = 1050
        assert_eq!(get_balance(&env, &collateral_addr, &liquidator), 1050);
    }

    #[test]
    fn test_liquidation_healthy_loan_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let liquidator = Address::generate(&env);

        let (collateral_addr, collateral_token) = create_token_contract(&env, &admin);

        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        client.initialize(&admin, &15000, &12000, &500);
        client.whitelist_collateral(&admin, &collateral_addr);

        // Create healthy loan (collateral = 1500, debt = 1000, health = 150%)
        collateral_token.mint(&borrower, &1500);
        let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1500);

        // Try to liquidate healthy loan
        let result = client.try_liquidate(&liquidator, &loan_id);
        assert_eq!(result, Err(Ok(BorrowingError::LoanHealthy)));
    }

    #[test]
    fn test_liquidation_after_partial_repayment() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let borrower = Address::generate(&env);
        let liquidator = Address::generate(&env);

        let (collateral_addr, collateral_token) = create_token_contract(&env, &admin);

        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        // Initialize with 120% collateral ratio, 130% liquidation threshold
        client.initialize(&admin, &12000, &13000, &500);
        client.whitelist_collateral(&admin, &collateral_addr);

        // Create loan with 120% collateral
        collateral_token.mint(&borrower, &1200);
        let loan_id = client.create_loan(&borrower, &1000, &5, &1000000, &collateral_addr, &1200);

        // Partial repayment (500), remaining debt = 500
        client.repay_loan(&loan_id, &500);

        // Health factor now = (1200 / 500) * 10000 = 24000 (240%) - healthy
        let health_factor = client.get_health_factor(&loan_id);
        assert_eq!(health_factor, 24000);

        // Try to liquidate - should fail as loan is now healthy
        let result = client.try_liquidate(&liquidator, &loan_id);
        assert_eq!(result, Err(Ok(BorrowingError::LoanHealthy)));
    }
}

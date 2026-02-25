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
}

#[contract]
pub struct BorrowingContract;

#[contractimpl]
impl BorrowingContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        collateral_ratio_bps: u32,
    ) -> Result<(), BorrowingError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(BorrowingError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::CollateralRatio, &collateral_ratio_bps);
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
        client.initialize(&admin, &15000);

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

        client.initialize(&admin, &15000);
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

        client.initialize(&admin, &15000);
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

        client.initialize(&admin, &15000);
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
}

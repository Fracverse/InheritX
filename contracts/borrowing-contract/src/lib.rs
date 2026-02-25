#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, vec, Address, Env,
    IntoVal, InvokeError, Symbol, Val, Vec,
};

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BorrowError {
    Unauthorized = 1,
    LoanNotFound = 2,
    InsufficientLiquidity = 3,
    LoanAlreadyRepaid = 4,
    InvalidAmount = 5,
    VaultNotLendable = 6,
    TransferFailed = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Loan {
    pub id: u64,
    pub borrower: Address,
    pub vault_plan_id: u64,
    pub principal: u64,
    pub interest_rate_bp: u32, // basis points (e.g., 500 = 5%)
    pub total_due: u64,
    pub created_at: u64,
    pub due_date: u64,
    pub repaid: bool,
    pub repaid_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    NextLoanId,
    Loan(u64),
    BorrowerLoans(Address),
    VaultContract,
    TokenContract,
}

#[contract]
pub struct BorrowingContract;

#[contractimpl]
impl BorrowingContract {
    /// Initialize the contract with vault and token addresses
    pub fn initialize(env: Env, vault_contract: Address, token_contract: Address) {
        env.storage()
            .instance()
            .set(&DataKey::VaultContract, &vault_contract);
        env.storage()
            .instance()
            .set(&DataKey::TokenContract, &token_contract);
        env.storage().instance().set(&DataKey::NextLoanId, &1u64);
    }

    /// Create a new loan against a lendable vault
    pub fn create_loan(
        env: Env,
        borrower: Address,
        vault_plan_id: u64,
        amount: u64,
        interest_rate_bp: u32,
        duration_seconds: u64,
    ) -> Result<u64, BorrowError> {
        borrower.require_auth();

        if amount == 0 {
            return Err(BorrowError::InvalidAmount);
        }

        // Check vault liquidity via cross-contract call
        let vault_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::VaultContract)
            .unwrap();

        let available: u64 =
            Self::get_vault_available_liquidity(&env, &vault_contract, vault_plan_id)?;

        if amount > available {
            return Err(BorrowError::InsufficientLiquidity);
        }

        // Calculate total due
        let interest = amount
            .checked_mul(interest_rate_bp as u64)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0);
        let total_due = amount + interest;

        let now = env.ledger().timestamp();
        let loan_id = Self::get_next_loan_id(&env);

        let loan = Loan {
            id: loan_id,
            borrower: borrower.clone(),
            vault_plan_id,
            principal: amount,
            interest_rate_bp,
            total_due,
            created_at: now,
            due_date: now + duration_seconds,
            repaid: false,
            repaid_at: 0,
        };

        // Store loan
        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        // Add to borrower's loans
        let mut borrower_loans: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::BorrowerLoans(borrower.clone()))
            .unwrap_or(Vec::new(&env));
        borrower_loans.push_back(loan_id);
        env.storage()
            .persistent()
            .set(&DataKey::BorrowerLoans(borrower.clone()), &borrower_loans);

        // Increment loan ID
        env.storage()
            .instance()
            .set(&DataKey::NextLoanId, &(loan_id + 1));

        // Update vault total_loaned via cross-contract call
        Self::update_vault_loaned(&env, &vault_contract, vault_plan_id, amount, true)?;

        // Transfer tokens to borrower
        Self::transfer_tokens(&env, &borrower, amount)?;

        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("CREATE")),
            (loan_id, borrower, amount),
        );

        log!(&env, "Loan {} created for {} tokens", loan_id, amount);

        Ok(loan_id)
    }

    /// Repay a loan
    pub fn repay_loan(env: Env, borrower: Address, loan_id: u64) -> Result<(), BorrowError> {
        borrower.require_auth();

        let mut loan: Loan = env
            .storage()
            .persistent()
            .get(&DataKey::Loan(loan_id))
            .ok_or(BorrowError::LoanNotFound)?;

        if loan.borrower != borrower {
            return Err(BorrowError::Unauthorized);
        }

        if loan.repaid {
            return Err(BorrowError::LoanAlreadyRepaid);
        }

        // Transfer repayment from borrower to contract
        let token_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenContract)
            .unwrap();
        let contract_id = env.current_contract_address();

        let args: Vec<Val> = vec![
            &env,
            borrower.clone().into_val(&env),
            contract_id.into_val(&env),
            (loan.total_due as i128).into_val(&env),
        ];
        let res = env.try_invoke_contract::<(), InvokeError>(
            &token_contract,
            &symbol_short!("transfer"),
            args,
        );
        if res.is_err() {
            return Err(BorrowError::TransferFailed);
        }

        // Update vault total_loaned
        let vault_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::VaultContract)
            .unwrap();
        Self::update_vault_loaned(
            &env,
            &vault_contract,
            loan.vault_plan_id,
            loan.principal,
            false,
        )?;

        // Mark loan as repaid
        loan.repaid = true;
        loan.repaid_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("REPAY")),
            (loan_id, borrower, loan.total_due),
        );

        log!(&env, "Loan {} repaid", loan_id);

        Ok(())
    }

    /// Get loan details
    pub fn get_loan(env: Env, loan_id: u64) -> Option<Loan> {
        env.storage().persistent().get(&DataKey::Loan(loan_id))
    }

    /// Get all loans for a borrower
    pub fn get_borrower_loans(env: Env, borrower: Address) -> Vec<Loan> {
        let loan_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::BorrowerLoans(borrower))
            .unwrap_or(Vec::new(&env));

        let mut loans = Vec::new(&env);
        for id in loan_ids.iter() {
            if let Some(loan) = Self::get_loan(env.clone(), id) {
                loans.push_back(loan);
            }
        }
        loans
    }

    // ── Internal Helpers ──────────────────────────────────────────────────

    fn get_next_loan_id(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextLoanId)
            .unwrap_or(1)
    }

    fn get_vault_available_liquidity(
        env: &Env,
        vault_contract: &Address,
        plan_id: u64,
    ) -> Result<u64, BorrowError> {
        // Call inheritance contract's get_plan_details
        let args: Vec<Val> = vec![env, plan_id.into_val(env)];
        let _plan_result = env
            .try_invoke_contract::<Val, InvokeError>(
                vault_contract,
                &Symbol::new(env, "get_plan_details"),
                args,
            )
            .map_err(|_| BorrowError::VaultNotLendable)?;

        // Parse plan and check is_lendable + calculate available
        // For simplicity, we assume the vault exposes available liquidity
        // In production, parse the InheritancePlan struct properly
        // Here we return a stub - integrate with actual vault contract
        Ok(1000000) // Stub: replace with actual parsing
    }

    fn update_vault_loaned(
        env: &Env,
        _vault_contract: &Address,
        plan_id: u64,
        amount: u64,
        _increase: bool,
    ) -> Result<(), BorrowError> {
        // Call vault contract to update total_loaned
        // This requires the vault contract to expose an update_loaned function
        // For now, this is a stub
        log!(env, "Update vault {} loaned by {}", plan_id, amount);
        Ok(())
    }

    fn transfer_tokens(env: &Env, to: &Address, amount: u64) -> Result<(), BorrowError> {
        let token_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenContract)
            .unwrap();
        let contract_id = env.current_contract_address();

        let args: Vec<Val> = vec![
            env,
            contract_id.into_val(env),
            to.clone().into_val(env),
            (amount as i128).into_val(env),
        ];
        let res = env.try_invoke_contract::<(), InvokeError>(
            &token_contract,
            &symbol_short!("transfer"),
            args,
        );

        if res.is_err() {
            return Err(BorrowError::TransferFailed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_loan_lifecycle() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        let vault = Address::generate(&env);
        let token = Address::generate(&env);

        client.initialize(&vault, &token);

        // Note: Full integration test requires mock vault + token contracts
        // This is a structure test only
    }

    #[test]
    fn test_loan_storage() {
        let env = Env::default();
        let borrower = Address::generate(&env);

        let loan = Loan {
            id: 1,
            borrower: borrower.clone(),
            vault_plan_id: 100,
            principal: 1000,
            interest_rate_bp: 500,
            total_due: 1050,
            created_at: 1000,
            due_date: 2000,
            repaid: false,
            repaid_at: 0,
        };

        env.as_contract(&env.register_contract(None, BorrowingContract), || {
            env.storage().persistent().set(&DataKey::Loan(1), &loan);
            let retrieved: Loan = env.storage().persistent().get(&DataKey::Loan(1)).unwrap();

            assert_eq!(retrieved.id, 1);
            assert_eq!(retrieved.principal, 1000);
            assert_eq!(retrieved.total_due, 1050);
            assert!(!retrieved.repaid);
        });
    }
}

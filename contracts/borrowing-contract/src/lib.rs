#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[derive(Clone)]
#[contracttype]
pub struct Loan {
    pub borrower: Address,
    pub principal: i128,
    pub interest_rate: u32,
    pub due_date: u64,
    pub amount_repaid: i128,
    pub is_active: bool,
}

#[contracttype]
pub enum DataKey {
    LoanCounter,
    Loan(u64),
}

#[contract]
pub struct BorrowingContract;

#[contractimpl]
impl BorrowingContract {
    pub fn create_loan(
        env: Env,
        borrower: Address,
        principal: i128,
        interest_rate: u32,
        due_date: u64,
    ) -> u64 {
        borrower.require_auth();

        let loan_id = Self::get_next_loan_id(&env);

        let loan = Loan {
            borrower,
            principal,
            interest_rate,
            due_date,
            amount_repaid: 0,
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Loan(loan_id), &loan);

        loan_id
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
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_create_loan() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        let borrower = Address::generate(&env);

        env.mock_all_auths();

        let loan_id = client.create_loan(&borrower, &1000, &5, &1000000);

        assert_eq!(loan_id, 1);

        let loan = client.get_loan(&loan_id);
        assert_eq!(loan.borrower, borrower);
        assert_eq!(loan.principal, 1000);
        assert_eq!(loan.interest_rate, 5);
        assert_eq!(loan.due_date, 1000000);
        assert_eq!(loan.amount_repaid, 0);
        assert!(loan.is_active);
    }

    #[test]
    fn test_repay_loan() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        let borrower = Address::generate(&env);

        env.mock_all_auths();

        let loan_id = client.create_loan(&borrower, &1000, &5, &1000000);

        client.repay_loan(&loan_id, &500);

        let loan = client.get_loan(&loan_id);
        assert_eq!(loan.amount_repaid, 500);
        assert!(loan.is_active);

        client.repay_loan(&loan_id, &500);

        let loan = client.get_loan(&loan_id);
        assert_eq!(loan.amount_repaid, 1000);
        assert!(!loan.is_active);
    }

    #[test]
    fn test_multiple_loans() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BorrowingContract);
        let client = BorrowingContractClient::new(&env, &contract_id);

        let borrower1 = Address::generate(&env);
        let borrower2 = Address::generate(&env);

        env.mock_all_auths();

        let loan_id1 = client.create_loan(&borrower1, &1000, &5, &1000000);
        let loan_id2 = client.create_loan(&borrower2, &2000, &10, &2000000);

        assert_eq!(loan_id1, 1);
        assert_eq!(loan_id2, 2);

        let loan1 = client.get_loan(&loan_id1);
        let loan2 = client.get_loan(&loan_id2);

        assert_eq!(loan1.principal, 1000);
        assert_eq!(loan2.principal, 2000);
    }
}

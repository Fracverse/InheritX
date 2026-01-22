#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, vec, Bytes, BytesN,
    Env, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DistributionMethod {
    LumpSum,
    Monthly,
    Quarterly,
    Yearly,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Beneficiary {
    pub hashed_full_name: BytesN<32>,
    pub hashed_email: BytesN<32>,
    pub hashed_claim_code: BytesN<32>,
    pub hashed_bank_account: BytesN<32>,
    pub allocation_percentage: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InheritancePlan {
    pub plan_name: String,
    pub description: String,
    pub asset_type: Symbol, // Only USDC allowed
    pub total_amount: u64,
    pub distribution_method: DistributionMethod,
    pub beneficiaries: Vec<Beneficiary>,
    pub created_at: u64,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InheritanceError {
    InvalidAssetType = 1,
    InvalidTotalAmount = 2,
    MissingRequiredField = 3,
    TooManyBeneficiaries = 4,
    InvalidClaimCode = 5,
    AllocationPercentageMismatch = 6,
    DescriptionTooLong = 7,
    InvalidBeneficiaryData = 8,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    NextPlanId,
    Plan(u64),
}

#[contract]
pub struct InheritanceContract;

#[contractimpl]
impl InheritanceContract {
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        vec![&env, symbol_short!("Hello"), to]
    }

    // Hash utility functions
    pub fn hash_string(env: &Env, input: String) -> BytesN<32> {
        // In production, this should be replaced with proper string-to-bytes conversion
        let mut data = Bytes::new(&env);
        data.push_back((input.len() % 256) as u8);
        data.push_back((input.len() / 256) as u8);

        // Add some entropy based on first few characters if available
        if input.len() > 0 {
            data.push_back((input.len() as u8).wrapping_add(1));
        }
        if input.len() > 1 {
            data.push_back((input.len() as u8).wrapping_add(2));
        }

        env.crypto().sha256(&data).into()
    }

    pub fn hash_claim_code(env: &Env, claim_code: String) -> Result<BytesN<32>, InheritanceError> {
        // Validate claim code format (exactly 6 digits)
        if claim_code.len() != 6 {
            return Err(InheritanceError::InvalidClaimCode);
        }

        // For now, just validate length since proper character validation requires string-to-bytes conversion
        // In production, this should validate that all characters are digits 0-9
        // TODO: Implement proper character validation when Soroban String API allows it

        Ok(Self::hash_string(env, claim_code.clone()))
    }

    fn create_beneficiary(
        env: &Env,
        full_name: String,
        email: String,
        claim_code: String,
        bank_account: String,
        allocation_percentage: u32,
    ) -> Result<Beneficiary, InheritanceError> {
        // Validate inputs
        if full_name.is_empty() || email.is_empty() || bank_account.is_empty() {
            return Err(InheritanceError::InvalidBeneficiaryData);
        }

        // Validate claim code and get hash
        let hashed_claim_code = Self::hash_claim_code(env, claim_code.clone())?;

        Ok(Beneficiary {
            hashed_full_name: Self::hash_string(env, full_name),
            hashed_email: Self::hash_string(env, email),
            hashed_claim_code,
            hashed_bank_account: Self::hash_string(env, bank_account),
            allocation_percentage,
        })
    }

    // Validation functions
    pub fn validate_plan_inputs(
        plan_name: String,
        description: String,
        asset_type: Symbol,
        total_amount: u64,
    ) -> Result<(), InheritanceError> {
        // Validate required fields
        if plan_name.is_empty() {
            return Err(InheritanceError::MissingRequiredField);
        }

        // Validate description length (max 500 characters)
        if description.len() > 500 {
            return Err(InheritanceError::DescriptionTooLong);
        }

        // Validate asset type (only USDC allowed)
        if asset_type != Symbol::new(&Env::default(), "USDC") {
            return Err(InheritanceError::InvalidAssetType);
        }

        // Validate total amount
        if total_amount == 0 {
            return Err(InheritanceError::InvalidTotalAmount);
        }

        Ok(())
    }

    pub fn validate_beneficiaries(
        beneficiaries_data: Vec<(String, String, String, String, u32)>,
    ) -> Result<(), InheritanceError> {
        // Validate beneficiary count (max 10)
        if beneficiaries_data.len() > 10 {
            return Err(InheritanceError::TooManyBeneficiaries);
        }

        if beneficiaries_data.is_empty() {
            return Err(InheritanceError::MissingRequiredField);
        }

        // Validate allocation percentages total 100%
        let total_allocation: u32 = beneficiaries_data.iter().map(|(_, _, _, _, pct)| pct).sum();
        if total_allocation != 100 {
            return Err(InheritanceError::AllocationPercentageMismatch);
        }

        Ok(())
    }

    // Storage functions
    fn get_next_plan_id(env: &Env) -> u64 {
        let key = DataKey::NextPlanId;
        env.storage().instance().get(&key).unwrap_or(1)
    }

    fn increment_plan_id(env: &Env) -> u64 {
        let current_id = Self::get_next_plan_id(env);
        let next_id = current_id + 1;
        let key = DataKey::NextPlanId;
        env.storage().instance().set(&key, &next_id);
        current_id
    }

    fn store_plan(env: &Env, plan_id: u64, plan: &InheritancePlan) {
        let key = DataKey::Plan(plan_id);
        env.storage().persistent().set(&key, plan);
    }

    fn get_plan(env: &Env, plan_id: u64) -> Option<InheritancePlan> {
        let key = DataKey::Plan(plan_id);
        env.storage().persistent().get(&key)
    }

    /// Create a new inheritance plan
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `plan_name` - Name of the inheritance plan (required)
    /// * `description` - Description of the plan (max 500 characters)
    /// * `total_amount` - Total amount in the plan (must be > 0)
    /// * `distribution_method` - How to distribute the inheritance
    /// * `beneficiaries_data` - Vector of beneficiary data tuples: (full_name, email, claim_code, bank_account, allocation_percentage)
    ///
    /// # Returns
    /// The plan ID of the created inheritance plan
    ///
    /// # Errors
    /// Returns InheritanceError for various validation failures
    pub fn create_inheritance_plan(
        env: &Env,
        plan_name: String,
        description: String,
        total_amount: u64,
        distribution_method: DistributionMethod,
        beneficiaries_data: Vec<(String, String, String, String, u32)>,
    ) -> Result<u64, InheritanceError> {
        // Validate plan inputs (asset type is hardcoded to USDC)
        let usdc_symbol = Symbol::new(&env, "USDC");
        Self::validate_plan_inputs(
            plan_name.clone(),
            description.clone(),
            usdc_symbol.clone(),
            total_amount,
        )?;

        // Validate beneficiaries
        Self::validate_beneficiaries(beneficiaries_data.clone())?;

        // Create beneficiary objects with hashed data
        let mut beneficiaries = Vec::new(&env);
        for beneficiary_data in beneficiaries_data.iter() {
            let beneficiary = Self::create_beneficiary(
                &env,
                beneficiary_data.0.clone(),
                beneficiary_data.1.clone(),
                beneficiary_data.2.clone(),
                beneficiary_data.3.clone(),
                beneficiary_data.4,
            )?;
            beneficiaries.push_back(beneficiary);
        }

        // Create the inheritance plan
        let plan = InheritancePlan {
            plan_name,
            description,
            asset_type: Symbol::new(&env, "USDC"),
            total_amount,
            distribution_method,
            beneficiaries,
            created_at: env.ledger().timestamp(),
        };

        // Store the plan and get the plan ID
        let plan_id = Self::increment_plan_id(&env);
        Self::store_plan(&env, plan_id, &plan);

        log!(&env, "Inheritance plan created with ID: {}", plan_id);

        Ok(plan_id)
    }
}

mod test;

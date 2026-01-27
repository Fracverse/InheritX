#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, vec, Address, Bytes,
    BytesN, Env, String, Symbol, Vec,
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
    pub bank_account: Bytes, // Plain text for fiat settlement (MVP trade-off)
    pub allocation_bp: u32,  // Allocation in basis points (0-10000, where 10000 = 100%)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeneficiaryInput {
    pub name: String,
    pub email: String,
    pub claim_code: u32,
    pub bank_account: Bytes,
    pub allocation_bp: u32,
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
    pub total_allocation_bp: u32, // Total allocation in basis points
    pub owner: Address,           // Plan owner
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
    Unauthorized = 9,
    PlanNotFound = 10,
    InvalidBeneficiaryIndex = 11,
    AllocationExceedsLimit = 12,
    InvalidAllocation = 13,
    InvalidClaimCodeRange = 14,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    NextPlanId,
    Plan(u64),
}

// Events for beneficiary operations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeneficiaryAddedEvent {
    pub plan_id: u64,
    pub hashed_email: BytesN<32>,
    pub allocation_bp: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeneficiaryRemovedEvent {
    pub plan_id: u64,
    pub index: u32,
    pub allocation_bp: u32,
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
        // Convert string to bytes for hashing
        let mut data = Bytes::new(env);

        // Simple conversion - in production, use proper string-to-bytes conversion
        for i in 0..input.len() {
            data.push_back((i % 256) as u8);
        }

        env.crypto().sha256(&data).into()
    }

    pub fn hash_bytes(env: &Env, input: Bytes) -> BytesN<32> {
        env.crypto().sha256(&input).into()
    }

    pub fn hash_claim_code(env: &Env, claim_code: u32) -> Result<BytesN<32>, InheritanceError> {
        // Validate claim code is in range 0-999999 (6 digits)
        if claim_code > 999999 {
            return Err(InheritanceError::InvalidClaimCodeRange);
        }

        // Convert claim code to bytes for hashing (6 digits, padded with zeros)
        let mut data = Bytes::new(env);

        // Extract each digit and convert to ASCII byte
        for i in 0..6 {
            let digit = ((claim_code / 10u32.pow(5 - i)) % 10) as u8;
            data.push_back(digit + b'0');
        }

        Ok(env.crypto().sha256(&data).into())
    }

    fn create_beneficiary(
        env: &Env,
        full_name: String,
        email: String,
        claim_code: u32,
        bank_account: Bytes,
        allocation_bp: u32,
    ) -> Result<Beneficiary, InheritanceError> {
        // Validate inputs
        if full_name.is_empty() || email.is_empty() || bank_account.is_empty() {
            return Err(InheritanceError::InvalidBeneficiaryData);
        }

        // Validate allocation is greater than 0
        if allocation_bp == 0 {
            return Err(InheritanceError::InvalidAllocation);
        }

        // Validate claim code and get hash
        let hashed_claim_code = Self::hash_claim_code(env, claim_code)?;

        Ok(Beneficiary {
            hashed_full_name: Self::hash_string(env, full_name),
            hashed_email: Self::hash_string(env, email),
            hashed_claim_code,
            bank_account, // Store plain for fiat settlement
            allocation_bp,
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
        beneficiaries_data: Vec<(String, String, u32, Bytes, u32)>,
    ) -> Result<(), InheritanceError> {
        // Validate beneficiary count (max 10)
        if beneficiaries_data.len() > 10 {
            return Err(InheritanceError::TooManyBeneficiaries);
        }

        if beneficiaries_data.is_empty() {
            return Err(InheritanceError::MissingRequiredField);
        }

        // Validate allocation basis points total to 10000 (100%)
        let total_allocation: u32 = beneficiaries_data.iter().map(|(_, _, _, _, bp)| bp).sum();
        if total_allocation != 10000 {
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

    /// Add a beneficiary to an existing inheritance plan
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `owner` - The plan owner (must authorize this call)
    /// * `plan_id` - The ID of the plan to add beneficiary to
    /// * `beneficiary_input` - Beneficiary data (name, email, claim_code, bank_account, allocation_bp)
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - Unauthorized: If caller is not the plan owner
    /// - PlanNotFound: If plan_id doesn't exist
    /// - TooManyBeneficiaries: If plan already has 10 beneficiaries
    /// - AllocationExceedsLimit: If total allocation would exceed 10000 basis points
    /// - InvalidBeneficiaryData: If any required field is empty
    /// - InvalidAllocation: If allocation_bp is 0
    /// - InvalidClaimCodeRange: If claim_code > 999999
    pub fn add_beneficiary(
        env: Env,
        owner: Address,
        plan_id: u64,
        beneficiary_input: BeneficiaryInput,
    ) -> Result<(), InheritanceError> {
        // Require owner authorization
        owner.require_auth();

        // Get the plan
        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Verify caller is the plan owner
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        // Check beneficiary count limit (max 10)
        if plan.beneficiaries.len() >= 10 {
            return Err(InheritanceError::TooManyBeneficiaries);
        }

        // Validate allocation is greater than 0
        if beneficiary_input.allocation_bp == 0 {
            return Err(InheritanceError::InvalidAllocation);
        }

        // Check that total allocation won't exceed 10000 basis points (100%)
        let new_total = plan.total_allocation_bp + beneficiary_input.allocation_bp;
        if new_total > 10000 {
            return Err(InheritanceError::AllocationExceedsLimit);
        }

        // Create the beneficiary (validates inputs and hashes sensitive data)
        let beneficiary = Self::create_beneficiary(
            &env,
            beneficiary_input.name,
            beneficiary_input.email.clone(),
            beneficiary_input.claim_code,
            beneficiary_input.bank_account,
            beneficiary_input.allocation_bp,
        )?;

        // Add beneficiary to plan
        plan.beneficiaries.push_back(beneficiary.clone());
        plan.total_allocation_bp = new_total;

        // Store updated plan
        Self::store_plan(&env, plan_id, &plan);

        // Emit event
        env.events().publish(
            (symbol_short!("BENEFIC"), symbol_short!("ADD")),
            BeneficiaryAddedEvent {
                plan_id,
                hashed_email: beneficiary.hashed_email,
                allocation_bp: beneficiary_input.allocation_bp,
            },
        );

        log!(&env, "Beneficiary added to plan {}", plan_id);

        Ok(())
    }

    /// Remove a beneficiary from an existing inheritance plan
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `owner` - The plan owner (must authorize this call)
    /// * `plan_id` - The ID of the plan to remove beneficiary from
    /// * `index` - The index of the beneficiary to remove (0-based)
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - Unauthorized: If caller is not the plan owner
    /// - PlanNotFound: If plan_id doesn't exist
    /// - InvalidBeneficiaryIndex: If index is out of bounds
    pub fn remove_beneficiary(
        env: Env,
        owner: Address,
        plan_id: u64,
        index: u32,
    ) -> Result<(), InheritanceError> {
        // Require owner authorization
        owner.require_auth();

        // Get the plan
        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Verify caller is the plan owner
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        // Validate index
        if index >= plan.beneficiaries.len() {
            return Err(InheritanceError::InvalidBeneficiaryIndex);
        }

        // Get the beneficiary being removed (for event and allocation tracking)
        let removed_beneficiary = plan.beneficiaries.get(index).unwrap();
        let removed_allocation = removed_beneficiary.allocation_bp;

        // Remove beneficiary efficiently (swap with last and pop)
        let last_index = plan.beneficiaries.len() - 1;
        if index != last_index {
            // Swap with last element
            let last_beneficiary = plan.beneficiaries.get(last_index).unwrap();
            plan.beneficiaries.set(index, last_beneficiary);
        }
        plan.beneficiaries.pop_back();

        // Update total allocation
        plan.total_allocation_bp -= removed_allocation;

        // Store updated plan
        Self::store_plan(&env, plan_id, &plan);

        // Emit event
        env.events().publish(
            (symbol_short!("BENEFIC"), symbol_short!("REMOVE")),
            BeneficiaryRemovedEvent {
                plan_id,
                index,
                allocation_bp: removed_allocation,
            },
        );

        log!(&env, "Beneficiary removed from plan {}", plan_id);

        Ok(())
    }

    /// Create a new inheritance plan
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `owner` - The plan owner
    /// * `plan_name` - Name of the inheritance plan (required)
    /// * `description` - Description of the plan (max 500 characters)
    /// * `total_amount` - Total amount in the plan (must be > 0)
    /// * `distribution_method` - How to distribute the inheritance
    /// * `beneficiaries_data` - Vector of beneficiary data tuples: (full_name, email, claim_code, bank_account, allocation_bp)
    ///
    /// # Returns
    /// The plan ID of the created inheritance plan
    ///
    /// # Errors
    /// Returns InheritanceError for various validation failures
    pub fn create_inheritance_plan(
        env: Env,
        owner: Address,
        plan_name: String,
        description: String,
        total_amount: u64,
        distribution_method: DistributionMethod,
        beneficiaries_data: Vec<(String, String, u32, Bytes, u32)>,
    ) -> Result<u64, InheritanceError> {
        // Require owner authorization
        owner.require_auth();

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
        let mut total_allocation_bp = 0u32;

        for beneficiary_data in beneficiaries_data.iter() {
            let beneficiary = Self::create_beneficiary(
                &env,
                beneficiary_data.0.clone(),
                beneficiary_data.1.clone(),
                beneficiary_data.2,
                beneficiary_data.3.clone(),
                beneficiary_data.4,
            )?;
            total_allocation_bp += beneficiary_data.4;
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
            total_allocation_bp,
            owner: owner.clone(),
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

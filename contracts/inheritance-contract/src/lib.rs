#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, vec, Address, Bytes,
    BytesN, Env, String, Symbol, Vec,
};

/// Current contract version - bump this on each upgrade
const CONTRACT_VERSION: u32 = 1;

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
    pub is_active: bool, // Plan activation status
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
    ClaimNotAllowedYet = 15,
    AlreadyClaimed = 16,
    BeneficiaryNotFound = 17,
    PlanAlreadyDeactivated = 18,
    PlanNotActive = 19,
    AdminNotSet = 20,
    AdminAlreadyInitialized = 21,
    NotAdmin = 22,
    KycNotSubmitted = 23,
    KycAlreadyApproved = 24,
    UpgradeFailed = 25,
    MigrationNotRequired = 26,
    PlanNotClaimed = 27,
    KycAlreadyRejected = 28,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    NextPlanId,
    Plan(u64),
    Claim(BytesN<32>),         // keyed by hashed_email
    UserPlans(Address),        // keyed by owner Address, value is Vec<u64>
    UserClaimedPlans(Address), // keyed by owner Address, value is Vec<u64>
    DeactivatedPlans,          // value is Vec<u64> of all deactivated plan IDs
    AllClaimedPlans,           // value is Vec<u64> of all claimed plan IDs
    Admin,
    Kyc(Address),
    Version,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimRecord {
    pub plan_id: u64,
    pub beneficiary_index: u32,
    pub claimed_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KycStatus {
    pub submitted: bool,
    pub approved: bool,
    pub rejected: bool,
    pub submitted_at: u64,
    pub approved_at: u64,
    pub rejected_at: u64,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanDeactivatedEvent {
    pub plan_id: u64,
    pub owner: Address,
    pub total_amount: u64,
    pub deactivated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KycApprovedEvent {
    pub user: Address,
    pub approved_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KycRejectedEvent {
    pub user: Address,
    pub rejected_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractUpgradedEvent {
    pub old_version: u32,
    pub new_version: u32,
    pub new_wasm_hash: BytesN<32>,
    pub admin: Address,
    pub upgraded_at: u64,
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

    fn get_admin(env: &Env) -> Option<Address> {
        let key = DataKey::Admin;
        env.storage().instance().get(&key)
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), InheritanceError> {
        admin.require_auth();
        let stored_admin = Self::get_admin(env).ok_or(InheritanceError::AdminNotSet)?;
        if stored_admin != *admin {
            return Err(InheritanceError::NotAdmin);
        }
        Ok(())
    }

    pub fn initialize_admin(env: Env, admin: Address) -> Result<(), InheritanceError> {
        admin.require_auth();
        if Self::get_admin(&env).is_some() {
            return Err(InheritanceError::AdminAlreadyInitialized);
        }

        let key = DataKey::Admin;
        env.storage().instance().set(&key, &admin);
        Ok(())
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

    fn add_plan_to_user(env: &Env, owner: Address, plan_id: u64) {
        let key = DataKey::UserPlans(owner.clone());
        let mut plans: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        plans.push_back(plan_id);
        env.storage().persistent().set(&key, &plans);
    }

    fn add_plan_to_deactivated(env: &Env, plan_id: u64) {
        let key = DataKey::DeactivatedPlans;
        let mut plans: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        // Avoid duplicates if called multiple times (though logic should prevent this)
        if !plans.contains(plan_id) {
            plans.push_back(plan_id);
            env.storage().persistent().set(&key, &plans);
        }
    }

    fn add_plan_to_claimed(env: &Env, owner: Address, plan_id: u64) {
        let key_user = DataKey::UserClaimedPlans(owner);
        let mut user_plans: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key_user)
            .unwrap_or(Vec::new(env));

        if !user_plans.contains(plan_id) {
            user_plans.push_back(plan_id);
            env.storage().persistent().set(&key_user, &user_plans);
        }

        let key_all = DataKey::AllClaimedPlans;
        let mut all_plans: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key_all)
            .unwrap_or(Vec::new(env));

        if !all_plans.contains(plan_id) {
            all_plans.push_back(plan_id);
            env.storage().persistent().set(&key_all, &all_plans);
        }
    }

    /// Get plan details
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `plan_id` - The ID of the plan to retrieve
    ///
    /// # Returns
    /// The InheritancePlan if found, None otherwise
    pub fn get_plan_details(env: Env, plan_id: u64) -> Option<InheritancePlan> {
        Self::get_plan(&env, plan_id)
    }

    pub fn get_user_plan(
        env: Env,
        user: Address,
        plan_id: u64,
    ) -> Result<InheritancePlan, InheritanceError> {
        user.require_auth();
        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != user {
            return Err(InheritanceError::Unauthorized);
        }
        Ok(plan)
    }

    pub fn get_user_plans(env: Env, user: Address) -> Vec<InheritancePlan> {
        user.require_auth();
        let key = DataKey::UserPlans(user);
        let plan_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        let mut plans = Vec::new(&env);
        for plan_id in plan_ids.iter() {
            if let Some(plan) = Self::get_plan(&env, plan_id) {
                plans.push_back(plan);
            }
        }
        plans
    }

    pub fn get_all_plans(
        env: Env,
        admin: Address,
    ) -> Result<Vec<InheritancePlan>, InheritanceError> {
        Self::require_admin(&env, &admin)?;

        let mut plans = Vec::new(&env);
        let next_plan_id = Self::get_next_plan_id(&env);
        for plan_id in 1..next_plan_id {
            if let Some(plan) = Self::get_plan(&env, plan_id) {
                plans.push_back(plan);
            }
        }
        Ok(plans)
    }

    pub fn get_user_pending_plans(env: Env, user: Address) -> Vec<InheritancePlan> {
        let all_user_plans = Self::get_user_plans(env.clone(), user);
        let mut pending = Vec::new(&env);
        for plan in all_user_plans.iter() {
            if plan.is_active {
                pending.push_back(plan);
            }
        }
        pending
    }

    pub fn get_all_pending_plans(
        env: Env,
        admin: Address,
    ) -> Result<Vec<InheritancePlan>, InheritanceError> {
        let all_plans = Self::get_all_plans(env.clone(), admin)?;
        let mut pending = Vec::new(&env);
        for plan in all_plans.iter() {
            if plan.is_active {
                pending.push_back(plan);
            }
        }
        Ok(pending)
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
            is_active: true,
        };

        // Store the plan and get the plan ID
        let plan_id = Self::increment_plan_id(&env);
        Self::store_plan(&env, plan_id, &plan);

        // Add to user's plan list
        Self::add_plan_to_user(&env, owner.clone(), plan_id);

        log!(&env, "Inheritance plan created with ID: {}", plan_id);

        Ok(plan_id)
    }

    fn is_claim_time_valid(env: &Env, plan: &InheritancePlan) -> bool {
        let now = env.ledger().timestamp();
        let elapsed = now - plan.created_at;

        match plan.distribution_method {
            DistributionMethod::LumpSum => true, // always claimable
            DistributionMethod::Monthly => elapsed >= 30 * 24 * 60 * 60,
            DistributionMethod::Quarterly => elapsed >= 90 * 24 * 60 * 60,
            DistributionMethod::Yearly => elapsed >= 365 * 24 * 60 * 60,
        }
    }

    pub fn claim_inheritance_plan(
        env: Env,
        plan_id: u64,
        email: String,
        claim_code: u32,
    ) -> Result<(), InheritanceError> {
        // Fetch the plan
        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Check if plan is active
        if !plan.is_active {
            return Err(InheritanceError::PlanNotActive);
        }

        // Check if claim is allowed by distribution method
        if !Self::is_claim_time_valid(&env, &plan) {
            return Err(InheritanceError::ClaimNotAllowedYet);
        }

        // Hash email and claim code
        let hashed_email = Self::hash_string(&env, email.clone());
        let hashed_claim_code = Self::hash_claim_code(&env, claim_code)?;

        // Build claim key including plan ID
        // Build claim key including plan ID
        let claim_key = {
            let mut data = Bytes::new(&env);
            data.extend_from_slice(&plan_id.to_be_bytes()); // plan ID as bytes
            data.extend_from_slice(&hashed_email.to_array()); // convert BytesN<32> to [u8;32]
            DataKey::Claim(env.crypto().sha256(&data).into())
        };

        // Check if already claimed for this plan
        if env.storage().persistent().has(&claim_key) {
            return Err(InheritanceError::AlreadyClaimed);
        }

        // Find beneficiary
        let mut beneficiary_index: Option<u32> = None;
        for i in 0..plan.beneficiaries.len() {
            let b = plan.beneficiaries.get(i).unwrap();
            if b.hashed_email == hashed_email && b.hashed_claim_code == hashed_claim_code {
                beneficiary_index = Some(i);
                break;
            }
        }

        let index = beneficiary_index.ok_or(InheritanceError::BeneficiaryNotFound)?;

        // Record the claim
        let claim = ClaimRecord {
            plan_id,
            beneficiary_index: index,
            claimed_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&claim_key, &claim);

        // Mark plan as claimed
        Self::add_plan_to_claimed(&env, plan.owner.clone(), plan_id);

        // Emit claim event
        env.events().publish(
            (symbol_short!("CLAIM"), symbol_short!("SUCCESS")),
            (plan_id, hashed_email),
        );

        log!(
            &env,
            "Inheritance claimed for plan {} by {}",
            plan_id,
            email
        );

        Ok(())
    }

    /// Record KYC submission on-chain (called after off-chain submission).
    pub fn submit_kyc(env: Env, user: Address) -> Result<(), InheritanceError> {
        user.require_auth();

        let key = DataKey::Kyc(user.clone());
        let mut status = env.storage().persistent().get(&key).unwrap_or(KycStatus {
            submitted: false,
            approved: false,
            rejected: false,
            submitted_at: 0,
            approved_at: 0,
            rejected_at: 0,
        });

        if status.approved {
            return Err(InheritanceError::KycAlreadyApproved);
        }

        status.submitted = true;
        status.submitted_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &status);

        Ok(())
    }

    /// Approve a user's KYC after off-chain verification (admin-only).
    pub fn approve_kyc(env: Env, admin: Address, user: Address) -> Result<(), InheritanceError> {
        Self::require_admin(&env, &admin)?;

        let key = DataKey::Kyc(user.clone());
        let mut status: KycStatus = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(InheritanceError::KycNotSubmitted)?;

        if !status.submitted {
            return Err(InheritanceError::KycNotSubmitted);
        }

        if status.approved {
            return Err(InheritanceError::KycAlreadyApproved);
        }

        status.approved = true;
        status.approved_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &status);

        env.events().publish(
            (symbol_short!("KYC"), symbol_short!("APPROV")),
            KycApprovedEvent {
                user,
                approved_at: status.approved_at,
            },
        );

        Ok(())
    }

    /// Reject a user's KYC after off-chain review (admin-only).
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `admin` - The admin address (must be the initialized admin)
    /// * `user` - The user address whose KYC is being rejected
    ///
    /// # Errors
    /// - `AdminNotSet` / `NotAdmin` if caller is not the admin
    /// - `KycNotSubmitted` if user has no submitted KYC data
    /// - `KycAlreadyRejected` if the KYC was already rejected
    pub fn reject_kyc(env: Env, admin: Address, user: Address) -> Result<(), InheritanceError> {
        Self::require_admin(&env, &admin)?;

        let key = DataKey::Kyc(user.clone());
        let mut status: KycStatus = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(InheritanceError::KycNotSubmitted)?;

        if !status.submitted {
            return Err(InheritanceError::KycNotSubmitted);
        }

        if status.rejected {
            return Err(InheritanceError::KycAlreadyRejected);
        }

        status.rejected = true;
        status.rejected_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &status);

        env.events().publish(
            (symbol_short!("KYC"), symbol_short!("REJECT")),
            KycRejectedEvent {
                user,
                rejected_at: status.rejected_at,
            },
        );

        Ok(())
    }

    /// Deactivate an existing inheritance plan
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `owner` - The plan owner (must authorize this call)
    /// * `plan_id` - The ID of the plan to deactivate
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - Unauthorized: If caller is not the plan owner
    /// - PlanNotFound: If plan_id doesn't exist
    /// - PlanAlreadyDeactivated: If plan is already deactivated
    ///
    /// # Notes
    /// Upon successful deactivation, the USDC associated with the plan should be
    /// transferred back to the owner's wallet address. This function marks the plan
    /// as inactive and emits a deactivation event.
    pub fn deactivate_inheritance_plan(
        env: Env,
        owner: Address,
        plan_id: u64,
    ) -> Result<(), InheritanceError> {
        // Require owner authorization
        owner.require_auth();

        // Get the plan
        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Verify caller is the plan owner
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        // Check if plan is already deactivated
        if !plan.is_active {
            return Err(InheritanceError::PlanAlreadyDeactivated);
        }

        // Mark plan as inactive
        plan.is_active = false;

        // Store updated plan
        Self::store_plan(&env, plan_id, &plan);
        Self::add_plan_to_deactivated(&env, plan_id);

        // Emit deactivation event
        env.events().publish(
            (symbol_short!("PLAN"), symbol_short!("DEACT")),
            PlanDeactivatedEvent {
                plan_id,
                owner: owner.clone(),
                total_amount: plan.total_amount,
                deactivated_at: env.ledger().timestamp(),
            },
        );

        log!(&env, "Inheritance plan {} deactivated by owner", plan_id);

        Ok(())
    }

    /// Retrieve a specific deactivated plan (User)
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `user` - The user requesting the plan (must be owner)
    /// * `plan_id` - The ID of the plan
    pub fn get_deactivated_plan(
        env: Env,
        user: Address,
        plan_id: u64,
    ) -> Result<InheritancePlan, InheritanceError> {
        user.require_auth();

        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Check if plan belongs to user
        if plan.owner != user {
            return Err(InheritanceError::Unauthorized);
        }

        // Check if plan is deactivated
        if plan.is_active {
            return Err(InheritanceError::PlanNotActive);
        }

        Ok(plan)
    }

    /// Retrieve all deactivated plans for a user
    pub fn get_user_deactivated_plans(env: Env, user: Address) -> Vec<InheritancePlan> {
        user.require_auth();

        let key = DataKey::UserPlans(user.clone());
        let user_plan_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        let mut deactivated_plans = Vec::new(&env);

        for plan_id in user_plan_ids.iter() {
            if let Some(plan) = Self::get_plan(&env, plan_id) {
                if !plan.is_active {
                    deactivated_plans.push_back(plan);
                }
            }
        }

        deactivated_plans
    }

    /// Retrieve all deactivated plans (Admin only)
    pub fn get_all_deactivated_plans(
        env: Env,
        admin: Address,
    ) -> Result<Vec<InheritancePlan>, InheritanceError> {
        admin.require_auth();

        // Verify admin
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(InheritanceError::Unauthorized)?;
        if admin != stored_admin {
            return Err(InheritanceError::Unauthorized);
        }

        let key = DataKey::DeactivatedPlans;
        let deactivated_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        let mut plans = Vec::new(&env);
        for plan_id in deactivated_ids.iter() {
            if let Some(plan) = Self::get_plan(&env, plan_id) {
                // Double check it's inactive just in case
                if !plan.is_active {
                    plans.push_back(plan);
                }
            }
        }

        Ok(plans)
    }

    /// Retrieve a specific claimed plan belonging to the authenticated user
    pub fn get_claimed_plan(
        env: Env,
        user: Address,
        plan_id: u64,
    ) -> Result<InheritancePlan, InheritanceError> {
        user.require_auth();

        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        if plan.owner != user {
            return Err(InheritanceError::Unauthorized);
        }

        let key = DataKey::UserClaimedPlans(user);
        let user_plans: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        if !user_plans.contains(plan_id) {
            return Err(InheritanceError::PlanNotClaimed);
        }

        Ok(plan)
    }

    /// Retrieve all claimed plans for the authenticated user
    pub fn get_user_claimed_plans(env: Env, user: Address) -> Vec<InheritancePlan> {
        user.require_auth();

        let key = DataKey::UserClaimedPlans(user);
        let user_plan_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        let mut plans = Vec::new(&env);
        for id in user_plan_ids.iter() {
            if let Some(plan) = Self::get_plan(&env, id) {
                plans.push_back(plan);
            }
        }
        plans
    }

    /// Retrieve all claimed plans across all users; accessible only by administrators
    pub fn get_all_claimed_plans(
        env: Env,
        admin: Address,
    ) -> Result<Vec<InheritancePlan>, InheritanceError> {
        Self::require_admin(&env, &admin)?;

        let key = DataKey::AllClaimedPlans;
        let all_plan_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        let mut plans = Vec::new(&env);
        for id in all_plan_ids.iter() {
            if let Some(plan) = Self::get_plan(&env, id) {
                plans.push_back(plan);
            }
        }
        Ok(plans)
    }

    // ───────────────────────────────────────────
    // Contract Upgrade Functions
    // ───────────────────────────────────────────

    /// Get the current contract version.
    pub fn version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Version)
            .unwrap_or(CONTRACT_VERSION)
    }

    /// Upgrade the contract to a new WASM binary.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `admin` - The admin address (must be the initialized admin)
    /// * `new_wasm_hash` - The hash of the new WASM binary to deploy
    ///
    /// # Errors
    /// - `AdminNotSet` if admin has not been initialized
    /// - `NotAdmin` if the caller is not the admin
    pub fn upgrade(
        env: Env,
        admin: Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<(), InheritanceError> {
        // Only the contract admin can trigger an upgrade
        Self::require_admin(&env, &admin)?;

        let old_version = Self::version(env.clone());
        let new_version = old_version + 1;

        // Store the new version before upgrading
        env.storage()
            .instance()
            .set(&DataKey::Version, &new_version);

        // Emit upgrade event for audit trail
        env.events().publish(
            (symbol_short!("CONTRACT"), symbol_short!("UPGRADE")),
            ContractUpgradedEvent {
                old_version,
                new_version,
                new_wasm_hash: new_wasm_hash.clone(),
                admin: admin.clone(),
                upgraded_at: env.ledger().timestamp(),
            },
        );

        log!(
            &env,
            "Contract upgraded from v{} to v{} by admin",
            old_version,
            new_version
        );

        // Perform the atomic WASM upgrade — this replaces the contract code
        // while preserving all storage (plans, claims, KYC, admin, etc.)
        env.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    /// Post-upgrade migration hook for data schema changes.
    ///
    /// Call this after deploying a new WASM if the new version requires
    /// storage migrations. If no migration is needed the function is a no-op
    /// so it is always safe to call.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `admin` - The admin address (must be the initialized admin)
    pub fn migrate(env: Env, admin: Address) -> Result<(), InheritanceError> {
        Self::require_admin(&env, &admin)?;

        let stored_version: u32 = env.storage().instance().get(&DataKey::Version).unwrap_or(0);

        if stored_version >= CONTRACT_VERSION {
            // Already up-to-date — nothing to migrate
            return Err(InheritanceError::MigrationNotRequired);
        }

        // ── Version-specific migrations go here ──
        // Example for a future migration:
        // if stored_version < 2 {
        //     // migrate from v1 → v2 schema changes
        // }

        // Update stored version to current
        env.storage()
            .instance()
            .set(&DataKey::Version, &CONTRACT_VERSION);

        log!(
            &env,
            "Contract migrated from v{} to v{}",
            stored_version,
            CONTRACT_VERSION
        );

        Ok(())
    }
}

mod test;

#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, token, vec, Address,
    Bytes, BytesN, Env, FromVal, IntoVal, InvokeError, String, Symbol, Val, Vec,
};

/// Current contract version - bump this on each upgrade
const CONTRACT_VERSION: u32 = 1;

/// Emergency transfer limit in basis points (10% = 1000 bp)
const EMERGENCY_TRANSFER_LIMIT_BP: u32 = 1000;

/// Emergency cooldown period in seconds (24 hours)
const EMERGENCY_COOLDOWN_PERIOD: u64 = 86400;

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
    pub is_lendable: bool,
    pub total_loaned: u64,
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
    InsufficientBalance = 29,
    FeeTransferFailed = 30,
    InsufficientLiquidity = 31,
    InheritanceAlreadyTriggered = 32,
    InheritanceNotTriggered = 33,
    LoanRecallFailed = 34,
    NoOutstandingLoans = 35,
    EmergencyAccessAlreadyActive = 36,
    EmergencyCooldownActive = 37,
    InvalidGuardianThreshold = 38,
    GuardianNotFound = 39,
    AlreadyApproved = 40,
    EmergencyContactNotFound = 41,
    EmergencyContactAlreadyExists = 42,
    TooManyEmergencyContacts = 43,
    WillHashAlreadyStored = 44,
    WillAlreadyLinked = 45,
    VaultNotFound = 46,
    VerificationFailed = 47,
    WillVersionNotFound = 48,
    WillAlreadyFinalized = 49,
    WillNotVerified = 50,
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
    InheritanceTrigger(u64),          // per-plan inheritance trigger info
    EmergencyActive(Address),         // bool, keyed by Address
    EmergencyLastActivated(Address),  // u64, keyed by Address
    EmergencyAccess(u64),             // per-plan emergency access record
    Guardians(u64),                   // per-plan guardian configuration
    EmergencyApprovals(u64, Address), // (plan_id, trusted_contact) -> Vec<Address>
    EmergencyContacts(u64),           // per-plan emergency contacts list
    WillHash(u64),                    // plan_id -> BytesN<32> (will document hash)
    VaultWill(u64),                   // plan_id -> BytesN<32> (linked will hash)
    BeneficiaryVerification(u64),     // plan_id -> bool (last verification result)
    WillVersionCount(u64),            // plan_id -> u32 (number of will versions)
    WillVersion(u64, u32),            // (plan_id, version) -> WillVersion struct
    ActiveWillVersion(u64),           // plan_id -> u32 (active version number)
    WillSignature(u64),               // plan_id -> WillSignatureProof
    SignatureUsed(BytesN<32>),        // sig_hash -> bool (replay protection)
    NextMessageId,                    // Global next message ID counter
    LegacyMessage(u64),               // message_id -> LegacyMessageMetadata
    VaultMessages(u64),               // vault_id -> Vec<u64> (message IDs)
    WillFinalized(u64, u32),          // (plan_id, version) -> bool
    WillFinalizedAt(u64, u32),        // (plan_id, version) -> u64 timestamp
    WillWitnesses(u64),               // plan_id -> Vec<Address>
    WitnessSignature(u64, Address),   // (plan_id, witness) -> u64 (signed_at)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardianConfig {
    pub guardians: Vec<Address>,
    pub threshold: u32,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InheritanceTriggerInfo {
    pub triggered_at: u64,
    pub loan_freeze_active: bool,
    pub recall_attempted: bool,
    pub liquidation_triggered: bool,
    pub original_loaned: u64,
    pub recalled_amount: u64,
    pub settled_amount: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyAccessRecord {
    pub plan_id: u64,
    pub trusted_contact: Address,
    pub activated_at: u64,
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
pub struct BeneficiaryUpdatedEvent {
    pub plan_id: u64,
    pub index: u32,
    pub field: Symbol, // "alloc", "bank", "code", "email", "swap"
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultDepositEvent {
    pub plan_id: u64,
    pub amount: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultWithdrawEvent {
    pub plan_id: u64,
    pub amount: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultLendableChangedEvent {
    pub plan_id: u64,
    pub is_lendable: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InheritanceTriggeredEvent {
    pub plan_id: u64,
    pub triggered_at: u64,
    pub outstanding_loans: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanFreezeEvent {
    pub plan_id: u64,
    pub frozen_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanRecallEvent {
    pub plan_id: u64,
    pub recalled_amount: u64,
    pub remaining_loaned: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidationFallbackEvent {
    pub plan_id: u64,
    pub settled_amount: u64,
    pub claimable_amount: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyAccessActivationEvent {
    pub user: Address,
    pub activated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyAccessRevocationEvent {
    pub plan_id: u64,
    pub revoked_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyAccessApprovedEvent {
    pub plan_id: u64,
    pub trusted_contact: Address,
    pub guardian: Address,
    pub approvals_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyAccessExpirationEvent {
    pub plan_id: u64,
    pub expired_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyAccessActivatedEvent {
    pub plan_id: u64,
    pub trusted_contact: Address,
    pub activated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyContactAddedEvent {
    pub plan_id: u64,
    pub contact: Address,
}

/// Legacy message metadata stored on-chain
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyMessageMetadata {
    pub vault_id: u64,            // Associated vault/plan ID
    pub message_id: u64,          // Unique message identifier
    pub message_hash: BytesN<32>, // Cryptographic hash of message content (off-chain)
    pub creator: Address,         // Message creator (vault owner)
    pub key_reference: String,    // Reference for decryption key (#364)
    pub unlock_timestamp: u64,    // Timestamp when message becomes accessible
    pub is_unlocked: bool,        // Whether message has been unlocked
    pub is_finalized: bool,       // Whether message has been finalized (#363)
    pub created_at: u64,          // Message creation timestamp
}

/// Parameters for creating a legacy message
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateLegacyMessageParams {
    pub vault_id: u64,
    pub message_hash: BytesN<32>,
    pub unlock_timestamp: u64,
    pub key_reference: String, // Addition for #364
}

/// Event emitted when a legacy message is created
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageCreatedEvent {
    pub vault_id: u64,
    pub message_id: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageUpdatedEvent {
    pub vault_id: u64,
    pub message_id: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageFinalizedEvent {
    pub vault_id: u64,
    pub message_id: u64,
    pub timestamp: u64,
}

/// Event emitted when a legacy message is deleted
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageDeletedEvent {
    pub vault_id: u64,
    pub message_id: u64,
    pub timestamp: u64,
}

/// Event emitted when a message is unlocked
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageUnlockedEvent {
    pub vault_id: u64,
    pub message_id: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageAccessedEvent {
    pub vault_id: u64,
    pub message_id: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyContactRemovedEvent {
    pub plan_id: u64,
    pub contact: Address,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WillVersionInfo {
    pub version: u32,
    pub will_hash: BytesN<32>,
    pub created_at: u64,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WillHashStoredEvent {
    pub plan_id: u64,
    pub will_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WillLinkedToVaultEvent {
    pub plan_id: u64,
    pub will_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeneficiariesVerifiedEvent {
    pub plan_id: u64,
    pub status: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WillVersionCreatedEvent {
    pub plan_id: u64,
    pub version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WillVersionActivatedEvent {
    pub plan_id: u64,
    pub version: u32,
}

/// Proof that a vault owner has cryptographically signed a will.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WillSignatureProof {
    pub vault_id: u64,
    pub will_hash: BytesN<32>,
    pub signer: Address,
    pub sig_hash: BytesN<32>,
    pub signed_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WillSignedEvent {
    pub vault_id: u64,
    pub signer: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WillFinalizedEvent {
    pub vault_id: u64,
    pub version: u32,
    pub finalized_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WitnessAddedEvent {
    pub vault_id: u64,
    pub witness: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WitnessSignedEvent {
    pub vault_id: u64,
    pub witness: Address,
}

/// Parameters for creating an inheritance plan (groups args to satisfy Clippy).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateInheritancePlanParams {
    pub owner: Address,
    pub token: Address,
    pub plan_name: String,
    pub description: String,
    pub total_amount: u64,
    pub distribution_method: DistributionMethod,
    pub beneficiaries_data: Vec<(String, String, u32, Bytes, u32)>,
    pub is_lendable: bool,
}

#[contract]
pub struct InheritanceContract;

#[contractimpl]
impl InheritanceContract {
    const EMERGENCY_EXPIRATION_PERIOD: u64 = 604800; // 7 days in seconds

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
        env: &Env,
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
        if asset_type != Symbol::new(env, "USDC") {
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

    /// Check if a user has approved KYC status
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `user` - The user address to check
    ///
    /// # Returns
    /// Ok(()) if user has approved KYC, Err(InheritanceError) otherwise
    ///
    /// # Errors
    /// - KycNotSubmitted: If user has not submitted KYC
    fn check_kyc_approved(env: &Env, user: &Address) -> Result<(), InheritanceError> {
        let key = DataKey::Kyc(user.clone());
        let status: KycStatus = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(InheritanceError::KycNotSubmitted)?;

        if !status.approved {
            return Err(InheritanceError::KycNotSubmitted);
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
        caller: Address,
        plan_id: u64,
    ) -> Result<InheritancePlan, InheritanceError> {
        caller.require_auth();
        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Authorization check: owner or active emergency contact
        let mut is_authorized = plan.owner == caller;
        if !is_authorized {
            if let Some(record) = Self::get_emergency_access(env.clone(), plan_id) {
                if record.trusted_contact == caller {
                    is_authorized = true;
                }
            }
        }

        if !is_authorized {
            return Err(InheritanceError::Unauthorized);
        }
        Ok(plan)
    }

    /// Internal helper to check and potentially expire emergency access based on the 7-day period.
    fn check_and_expire_emergency_access(env: &Env, plan_id: u64) -> bool {
        let key = DataKey::EmergencyAccess(plan_id);
        if let Some(record) = env
            .storage()
            .persistent()
            .get::<_, EmergencyAccessRecord>(&key)
        {
            if env.ledger().timestamp() > record.activated_at + Self::EMERGENCY_EXPIRATION_PERIOD {
                // Expired
                env.storage().persistent().remove(&key);

                env.events().publish(
                    (symbol_short!("EMERG"), symbol_short!("EXPIR")),
                    EmergencyAccessExpirationEvent {
                        plan_id,
                        expired_at: env.ledger().timestamp(),
                    },
                );
                return false;
            }
            return true;
        }
        false
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

    /// Creation fee in basis points (2% = 200 bp).
    const CREATION_FEE_BP: u64 = 200;

    /// Create a new inheritance plan.
    /// Applies a 2% creation fee: fee is deducted from the user's input amount,
    /// transferred to the admin wallet, and the net amount is saved in the plan.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `owner` - The plan owner (must authorize and have sufficient token balance)
    /// * `token` - The token contract address (e.g. USDC)
    /// * `plan_name` - Name of the inheritance plan (required)
    /// * `description` - Description of the plan (max 500 characters)
    /// * `total_amount` - User-input amount (must be > 0); fee is 2% of this, plan stores net
    /// * `distribution_method` - How to distribute the inheritance
    /// * `beneficiaries_data` - Vector of beneficiary data tuples: (full_name, email, claim_code, bank_account, allocation_bp)
    ///
    /// # Returns
    /// The plan ID of the created inheritance plan
    ///
    /// # Errors
    /// - AdminNotSet: Admin wallet not initialized
    /// - InsufficientBalance: Owner balance less than total_amount
    /// - FeeTransferFailed: Fee transfer to admin failed
    /// - InvalidTotalAmount: Net amount would be zero after fee
    /// - Other validation errors from validate_plan_inputs / validate_beneficiaries
    pub fn create_inheritance_plan(
        env: Env,
        params: CreateInheritancePlanParams,
    ) -> Result<u64, InheritanceError> {
        let CreateInheritancePlanParams {
            owner,
            token,
            plan_name,
            description,
            total_amount,
            distribution_method,
            beneficiaries_data,
            is_lendable,
        } = params;

        // Require owner authorization
        owner.require_auth();

        // Check KYC approval - only approved users can create plans
        Self::check_kyc_approved(&env, &owner)?;

        // Admin must be set to receive the fee
        let admin = Self::get_admin(&env).ok_or(InheritanceError::AdminNotSet)?;

        // Fee: 2% of user input; net amount stored in plan
        let fee = total_amount
            .checked_mul(Self::CREATION_FEE_BP)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0);
        let net_amount = total_amount
            .checked_sub(fee)
            .ok_or(InheritanceError::InvalidTotalAmount)?;

        if net_amount == 0 {
            return Err(InheritanceError::InvalidTotalAmount);
        }

        // Validate plan inputs using user input for "full amount" validation
        let usdc_symbol = Symbol::new(&env, "USDC");
        Self::validate_plan_inputs(
            &env,
            plan_name.clone(),
            description.clone(),
            usdc_symbol.clone(),
            total_amount,
        )?;

        // Wallet balance validation: must cover full amount (what user is debited)
        let token_client = token::Client::new(&env, &token);
        let balance = token_client.balance(&owner);
        let required = total_amount as i128;
        if balance < required {
            return Err(InheritanceError::InsufficientBalance);
        }

        // Transfer fee to admin (owner must have authorized this via auth).
        // Use try_invoke_contract so we can return FeeTransferFailed instead of trapping.
        let fee_i128 = fee as i128;
        if fee_i128 > 0 {
            let args: Vec<Val> = vec![
                &env,
                owner.clone().into_val(&env),
                admin.clone().into_val(&env),
                fee_i128.into_val(&env),
            ];
            let res = env.try_invoke_contract::<(), InvokeError>(
                &token,
                &symbol_short!("transfer"),
                args,
            );
            if res.is_err() {
                return Err(InheritanceError::FeeTransferFailed);
            }
        }

        // Transfer net amount to this contract (escrow for the plan).
        // Same: catch failure and return FeeTransferFailed.
        let contract_id = env.current_contract_address();
        let net_i128 = net_amount as i128;
        let net_args: Vec<Val> = vec![
            &env,
            owner.clone().into_val(&env),
            contract_id.clone().into_val(&env),
            net_i128.into_val(&env),
        ];
        let net_res = env.try_invoke_contract::<(), InvokeError>(
            &token,
            &symbol_short!("transfer"),
            net_args,
        );
        if net_res.is_err() {
            return Err(InheritanceError::FeeTransferFailed);
        }

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

        // Create the inheritance plan with net amount (user input minus 2% fee)
        let plan = InheritancePlan {
            plan_name,
            description,
            asset_type: Symbol::new(&env, "USDC"),
            total_amount: net_amount,
            distribution_method,
            beneficiaries,
            total_allocation_bp,
            owner: owner.clone(),
            created_at: env.ledger().timestamp(),
            is_active: true,
            is_lendable,
            total_loaned: 0,
        };

        // Store the plan and get the plan ID
        let plan_id = Self::increment_plan_id(&env);
        Self::store_plan(&env, plan_id, &plan);

        // Add to user's plan list
        Self::add_plan_to_user(&env, owner.clone(), plan_id);

        log!(&env, "Inheritance plan created with ID: {}", plan_id);

        Ok(plan_id)
    }

    pub fn set_lendable(
        env: Env,
        owner: Address,
        plan_id: u64,
        is_lendable: bool,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();
        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        plan.is_lendable = is_lendable;
        Self::store_plan(&env, plan_id, &plan);

        env.events().publish(
            (symbol_short!("VAULT"), symbol_short!("LENDABLE")),
            VaultLendableChangedEvent {
                plan_id,
                is_lendable,
            },
        );
        log!(&env, "Vault {} lendable set to {}", plan_id, is_lendable);
        Ok(())
    }

    pub fn deposit(
        env: Env,
        caller: Address,
        token: Address,
        plan_id: u64,
        amount: u64,
    ) -> Result<(), InheritanceError> {
        caller.require_auth();
        if amount == 0 {
            return Err(InheritanceError::InvalidTotalAmount);
        }

        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Authorization check: owner only
        if plan.owner != caller {
            return Err(InheritanceError::Unauthorized);
        }

        if !plan.is_active {
            return Err(InheritanceError::PlanNotActive);
        }

        let token_client = token::Client::new(&env, &token);
        let balance = token_client.balance(&caller);
        let required = amount as i128;
        if balance < required {
            return Err(InheritanceError::InsufficientBalance);
        }

        let contract_id = env.current_contract_address();
        let args: Vec<Val> = vec![
            &env,
            caller.clone().into_val(&env),
            contract_id.clone().into_val(&env),
            required.into_val(&env),
        ];
        let res =
            env.try_invoke_contract::<(), InvokeError>(&token, &symbol_short!("transfer"), args);
        if res.is_err() {
            return Err(InheritanceError::FeeTransferFailed);
        }

        plan.total_amount += amount;
        Self::store_plan(&env, plan_id, &plan);

        env.events().publish(
            (symbol_short!("VAULT"), symbol_short!("DEPOSIT")),
            VaultDepositEvent { plan_id, amount },
        );
        log!(&env, "Deposited {} into plan {}", amount, plan_id);
        Ok(())
    }

    pub fn withdraw(
        env: Env,
        caller: Address,
        token: Address,
        plan_id: u64,
        amount: u64,
    ) -> Result<(), InheritanceError> {
        caller.require_auth();
        if amount == 0 {
            return Err(InheritanceError::InvalidTotalAmount);
        }
        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Authorization check: owner only
        if plan.owner != caller {
            return Err(InheritanceError::Unauthorized);
        }

        // Emergency Guard: Limit withdrawal if emergency access was recently activated
        if Self::is_emergency_active(&env, plan_id) {
            let limit = (plan.total_amount as u128)
                .checked_mul(EMERGENCY_TRANSFER_LIMIT_BP as u128)
                .and_then(|v| v.checked_div(10000))
                .unwrap_or(0) as u64;

            if amount > limit {
                return Err(InheritanceError::EmergencyCooldownActive);
            }
        }

        // Emergency Guard: Limit withdrawal if emergency access was recently activated
        if Self::is_emergency_active(&env, plan_id) {
            let limit = (plan.total_amount as u128)
                .checked_mul(EMERGENCY_TRANSFER_LIMIT_BP as u128)
                .and_then(|v| v.checked_div(10000))
                .unwrap_or(0) as u64;

            if amount > limit {
                return Err(InheritanceError::EmergencyCooldownActive);
            }
        }

        let available = plan.total_amount.saturating_sub(plan.total_loaned);
        if amount > available {
            return Err(InheritanceError::InsufficientLiquidity);
        }

        let contract_id = env.current_contract_address();
        let required = amount as i128;
        let args: Vec<Val> = vec![
            &env,
            contract_id.clone().into_val(&env),
            caller.clone().into_val(&env),
            required.into_val(&env),
        ];
        let res =
            env.try_invoke_contract::<(), InvokeError>(&token, &symbol_short!("transfer"), args);
        if res.is_err() {
            return Err(InheritanceError::FeeTransferFailed);
        }

        plan.total_amount -= amount;
        Self::store_plan(&env, plan_id, &plan);

        env.events().publish(
            (symbol_short!("VAULT"), symbol_short!("WITHDRAW")),
            VaultWithdrawEvent { plan_id, amount },
        );
        log!(&env, "Withdrew {} from plan {}", amount, plan_id);
        Ok(())
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
        claimer: Address,
        email: String,
        claim_code: u32,
    ) -> Result<(), InheritanceError> {
        // Require claimer authorization
        claimer.require_auth();

        // Check KYC approval - only approved users can claim plans
        Self::check_kyc_approved(&env, &claimer)?;

        // Fetch the plan
        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Check if plan is active
        if !plan.is_active {
            return Err(InheritanceError::PlanNotActive);
        }

        // When inheritance is triggered, bypass the time-based check so
        // that inheritance execution cannot be blocked.
        let triggered = Self::get_trigger_info(&env, plan_id).is_some();
        if !triggered && !Self::is_claim_time_valid(&env, &plan) {
            return Err(InheritanceError::ClaimNotAllowedYet);
        }

        // Hash email and claim code
        let hashed_email = Self::hash_string(&env, email.clone());
        let hashed_claim_code = Self::hash_claim_code(&env, claim_code)?;

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

        // --- Payout Logic ---
        let beneficiary = plan.beneficiaries.get(index).unwrap();

        // Calculate the base payout
        let base_payout = (plan.total_amount as u128)
            .checked_mul(beneficiary.allocation_bp as u128)
            .and_then(|v| v.checked_div(10000))
            .unwrap_or(0) as u64;

        // Emergency Guard: Limit claim if emergency access was recently activated
        if Self::is_emergency_active(&env, plan_id) {
            let limit = (plan.total_amount as u128)
                .checked_mul(EMERGENCY_TRANSFER_LIMIT_BP as u128)
                .and_then(|v| v.checked_div(10000))
                .unwrap_or(0) as u64;

            if base_payout > limit {
                return Err(InheritanceError::EmergencyCooldownActive);
            }
        }

        // If plan is lendable and funds are loaned, we might have yield or need to recall funds.
        // For MVP priority logic: if we don't have enough liquid funds (amount - total_loaned < base_payout)
        // we'd recall from LendingContract.
        // Since we don't store the LendingContract address in InheritanceContract yet,
        // we assume the funds are sitting in the contract (vault) or we are authorized to pull them.
        let available_liquidity = plan.total_amount.saturating_sub(plan.total_loaned);

        // In a full implementation, we would call LendingClient::withdraw_priority
        // if base_payout > available_liquidity.
        // For now, we simulate the priority payout directly if liquid funds are sufficient,
        // or fail with InsufficientLiquidity if not (which a later migration would fix by linking contracts).
        // When inheritance is triggered, bypass the liquidity check so that
        // beneficiary claims are never blocked by outstanding loans.
        if !triggered && base_payout > available_liquidity {
            return Err(InheritanceError::InsufficientLiquidity);
        }

        // Transfer funds to beneficiary
        // Note: For fiat (bank_account), this would typically emit an event for off-chain processing.
        // Here, we'll try to transfer USDC if an address can be derived, or just emit an event.
        // As a simplification, we'll emit the event first.

        // Update plan balances
        let mut updated_plan = plan.clone();
        updated_plan.total_amount = updated_plan.total_amount.saturating_sub(base_payout);
        Self::store_plan(&env, plan_id, &updated_plan);

        // Mark plan as claimed
        Self::add_plan_to_claimed(&env, plan.owner.clone(), plan_id);

        // Emit claim event
        env.events().publish(
            (symbol_short!("CLAIM"), symbol_short!("SUCCESS")),
            (plan_id, hashed_email, base_payout),
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

    /// Activate emergency access for a trusted contact on a vault/plan.
    /// Only the plan owner can activate emergency access.
    pub fn activate_emergency_access(
        env: Env,
        owner: Address,
        plan_id: u64,
        trusted_contact: Address,
    ) -> Result<(), InheritanceError> {
        // Require owner authorization
        owner.require_auth();

        // Get the plan
        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Verify caller is the plan owner
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        // Check if emergency access is already activated
        let key = DataKey::EmergencyAccess(plan_id);
        if env.storage().persistent().has(&key) {
            return Err(InheritanceError::EmergencyAccessAlreadyActive);
        }

        // Record the activation timestamp
        let now = env.ledger().timestamp();

        // Create emergency access record
        let emergency_access = EmergencyAccessRecord {
            plan_id,
            trusted_contact: trusted_contact.clone(),
            activated_at: now,
        };

        // Store the emergency access record
        env.storage().persistent().set(&key, &emergency_access);

        // Emit event
        env.events().publish(
            (symbol_short!("EMERG"), symbol_short!("ACTIV")),
            EmergencyAccessActivatedEvent {
                plan_id,
                trusted_contact,
                activated_at: now,
            },
        );

        log!(
            &env,
            "Emergency access activated for plan {} at timestamp {}",
            plan_id,
            now
        );

        Ok(())
    }

    pub fn set_guardians(
        env: Env,
        owner: Address,
        plan_id: u64,
        guardians: Vec<Address>,
        threshold: u32,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();
        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }
        if threshold == 0 || guardians.len() < threshold {
            return Err(InheritanceError::InvalidGuardianThreshold);
        }
        let config = GuardianConfig {
            guardians,
            threshold,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Guardians(plan_id), &config);
        Ok(())
    }

    /// Add an emergency contact to a vault/plan.
    /// Emergency contacts can later request emergency access with guardian approval.
    pub fn add_emergency_contact(
        env: Env,
        owner: Address,
        plan_id: u64,
        contact: Address,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        let key = DataKey::EmergencyContacts(plan_id);
        let mut contacts: Vec<Address> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        // Check for duplicates
        for c in contacts.iter() {
            if c == contact {
                return Err(InheritanceError::EmergencyContactAlreadyExists);
            }
        }

        // Limit to 10 emergency contacts per plan
        if contacts.len() >= 10 {
            return Err(InheritanceError::TooManyEmergencyContacts);
        }

        contacts.push_back(contact.clone());
        env.storage().persistent().set(&key, &contacts);

        env.events().publish(
            (symbol_short!("EMERG"), symbol_short!("CON_ADD")),
            EmergencyContactAddedEvent {
                plan_id,
                contact: contact.clone(),
            },
        );

        log!(&env, "Emergency contact added to plan {}", plan_id);

        Ok(())
    }

    /// Remove an emergency contact from a vault/plan.
    pub fn remove_emergency_contact(
        env: Env,
        owner: Address,
        plan_id: u64,
        contact: Address,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        let key = DataKey::EmergencyContacts(plan_id);
        let mut contacts: Vec<Address> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        // Find and remove the contact
        let mut found_index: Option<u32> = None;
        for i in 0..contacts.len() {
            if contacts.get(i).unwrap() == contact {
                found_index = Some(i);
                break;
            }
        }

        let index = found_index.ok_or(InheritanceError::EmergencyContactNotFound)?;

        // Swap-remove for efficiency
        let last_index = contacts.len() - 1;
        if index != last_index {
            let last = contacts.get(last_index).unwrap();
            contacts.set(index, last);
        }
        contacts.pop_back();

        env.storage().persistent().set(&key, &contacts);

        env.events().publish(
            (symbol_short!("EMERG"), symbol_short!("CON_REM")),
            EmergencyContactRemovedEvent {
                plan_id,
                contact: contact.clone(),
            },
        );

        log!(&env, "Emergency contact removed from plan {}", plan_id);

        Ok(())
    }

    /// Get all emergency contacts for a vault/plan.
    pub fn get_emergency_contacts(env: Env, plan_id: u64) -> Vec<Address> {
        let key = DataKey::EmergencyContacts(plan_id);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env))
    }

    pub fn approve_emergency_access(
        env: Env,
        guardian: Address,
        plan_id: u64,
        trusted_contact: Address,
    ) -> Result<(), InheritanceError> {
        guardian.require_auth();
        let _plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        let key_access = DataKey::EmergencyAccess(plan_id);
        if env.storage().persistent().has(&key_access) {
            return Err(InheritanceError::EmergencyAccessAlreadyActive);
        }

        let config: GuardianConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Guardians(plan_id))
            .ok_or(InheritanceError::GuardianNotFound)?;

        // Check if guardian is in the list
        let mut is_guardian = false;
        for g in config.guardians.iter() {
            if g == guardian {
                is_guardian = true;
                break;
            }
        }
        if !is_guardian {
            return Err(InheritanceError::Unauthorized);
        }

        let key_approvals = DataKey::EmergencyApprovals(plan_id, trusted_contact.clone());
        let mut approvals: Vec<Address> = env
            .storage()
            .persistent()
            .get(&key_approvals)
            .unwrap_or(Vec::new(&env));

        let mut already_approved = false;
        for a in approvals.iter() {
            if a == guardian {
                already_approved = true;
                break;
            }
        }
        if already_approved {
            return Err(InheritanceError::AlreadyApproved);
        }

        approvals.push_back(guardian.clone());
        env.storage().persistent().set(&key_approvals, &approvals);

        env.events().publish(
            (symbol_short!("EMERG"), symbol_short!("APPROVE")),
            EmergencyAccessApprovedEvent {
                plan_id,
                trusted_contact: trusted_contact.clone(),
                guardian,
                approvals_count: approvals.len(),
            },
        );

        if approvals.len() >= config.threshold {
            let now = env.ledger().timestamp();
            let emergency_access = EmergencyAccessRecord {
                plan_id,
                trusted_contact: trusted_contact.clone(),
                activated_at: now,
            };
            env.storage()
                .persistent()
                .set(&key_access, &emergency_access);

            env.events().publish(
                (symbol_short!("EMERG"), symbol_short!("ACTIV")),
                EmergencyAccessActivatedEvent {
                    plan_id,
                    trusted_contact,
                    activated_at: now,
                },
            );
            log!(
                &env,
                "Emergency access activated for plan {} at timestamp {}",
                plan_id,
                now
            );
        }
        Ok(())
    }

    /// Query the emergency access record for a plan.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `plan_id` - The ID of the plan
    ///
    /// # Returns
    /// The EmergencyAccessRecord if emergency access is active, None otherwise
    pub fn get_emergency_access(env: Env, plan_id: u64) -> Option<EmergencyAccessRecord> {
        if Self::check_and_expire_emergency_access(&env, plan_id) {
            let key = DataKey::EmergencyAccess(plan_id);
            env.storage().persistent().get(&key)
        } else {
            None
        }
    }

    /// Check if emergency access is active and within the cooldown period for a plan.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `plan_id` - The ID of the plan
    ///
    /// # Returns
    /// True if emergency access was activated within the last 24 hours
    pub fn is_emergency_active(env: &Env, plan_id: u64) -> bool {
        if let Some(record) = env
            .storage()
            .persistent()
            .get::<DataKey, EmergencyAccessRecord>(&DataKey::EmergencyAccess(plan_id))
        {
            let now = env.ledger().timestamp();
            let elapsed = now.saturating_sub(record.activated_at);
            return elapsed < EMERGENCY_COOLDOWN_PERIOD;
        }
        false
    }

    /// Deactivate emergency access for a plan.
    /// Only the plan owner can deactivate emergency access.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `owner` - The plan owner (must authorize this call)
    /// * `plan_id` - The ID of the plan to deactivate emergency access for
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - Unauthorized: If caller is not the plan owner
    /// - PlanNotFound: If plan_id doesn't exist
    pub fn deactivate_emergency_access(
        env: Env,
        owner: Address,
        plan_id: u64,
    ) -> Result<(), InheritanceError> {
        // Require owner authorization
        owner.require_auth();

        // Get the plan
        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        // Verify caller is the plan owner
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        // Remove the emergency access record
        let key = DataKey::EmergencyAccess(plan_id);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().remove(&key);

            // Emit revocation event
            env.events().publish(
                (symbol_short!("EMERG"), symbol_short!("REVOK")),
                EmergencyAccessRevocationEvent {
                    plan_id,
                    revoked_at: env.ledger().timestamp(),
                },
            );

            log!(&env, "Emergency access deactivated for plan {}", plan_id);
        }

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
    // Loan Recall on Inheritance Trigger
    // ───────────────────────────────────────────

    fn get_trigger_info(env: &Env, plan_id: u64) -> Option<InheritanceTriggerInfo> {
        let key = DataKey::InheritanceTrigger(plan_id);
        env.storage().persistent().get(&key)
    }

    fn set_trigger_info(env: &Env, plan_id: u64, info: &InheritanceTriggerInfo) {
        let key = DataKey::InheritanceTrigger(plan_id);
        env.storage().persistent().set(&key, info);
    }

    /// Trigger inheritance for a plan. This freezes new loans and initiates
    /// the loan recall process.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `admin` - The admin address (must be the initialized admin)
    /// * `plan_id` - The ID of the plan to trigger inheritance for
    ///
    /// # Effects
    /// - Sets `is_lendable = false` to freeze new loans against this plan
    /// - Records the trigger info for tracking recall/liquidation state
    /// - Emits `INHERIT/TRIGGER` and `LOAN/FREEZE` events
    ///
    /// # Errors
    /// - `PlanNotFound` if plan_id doesn't exist
    /// - `PlanNotActive` if plan is not active
    /// - `InheritanceAlreadyTriggered` if inheritance was already triggered
    pub fn trigger_inheritance(
        env: Env,
        caller: Address,
        plan_id: u64,
    ) -> Result<(), InheritanceError> {
        // Authorization check: Admin OR Owner OR Trusted Contact with active emergency access
        let mut is_authorized = false;

        // 1. Admin check
        if let Some(admin) = Self::get_admin(&env) {
            if admin == caller {
                caller.require_auth();
                is_authorized = true;
            }
        }

        // 2. Plan check (Owner or Generic Emergency Access)
        if !is_authorized {
            let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;
            if plan.owner == caller {
                caller.require_auth();
                is_authorized = true;
            } else if let Some(record) = Self::get_emergency_access(env.clone(), plan_id) {
                if record.trusted_contact == caller {
                    caller.require_auth();
                    is_authorized = true;
                }
            }
        }

        if !is_authorized {
            return Err(InheritanceError::Unauthorized);
        }

        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        if !plan.is_active {
            return Err(InheritanceError::PlanNotActive);
        }

        // Check if already triggered
        if Self::get_trigger_info(&env, plan_id).is_some() {
            return Err(InheritanceError::InheritanceAlreadyTriggered);
        }

        let now = env.ledger().timestamp();

        // Freeze new loans by setting is_lendable to false
        plan.is_lendable = false;
        Self::store_plan(&env, plan_id, &plan);

        // Create trigger info
        let trigger_info = InheritanceTriggerInfo {
            triggered_at: now,
            loan_freeze_active: true,
            recall_attempted: false,
            liquidation_triggered: false,
            original_loaned: plan.total_loaned,
            recalled_amount: 0,
            settled_amount: 0,
        };
        Self::set_trigger_info(&env, plan_id, &trigger_info);

        // Emit events
        env.events().publish(
            (symbol_short!("INHERIT"), symbol_short!("TRIGGER")),
            InheritanceTriggeredEvent {
                plan_id,
                triggered_at: now,
                outstanding_loans: plan.total_loaned,
            },
        );

        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("FREEZE")),
            LoanFreezeEvent {
                plan_id,
                frozen_at: now,
            },
        );

        log!(
            &env,
            "Inheritance triggered for plan {} — loans frozen, outstanding: {}",
            plan_id,
            plan.total_loaned
        );

        Ok(())
    }

    /// Attempt to recall loaned funds back to the plan.
    /// Called by admin after loan repayment has been collected off-chain
    /// or via cross-contract calls to lending/borrowing contracts.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `admin` - The admin address
    /// * `plan_id` - The plan ID
    /// * `recall_amount` - Amount of loaned funds being recalled
    ///
    /// # Effects
    /// - Reduces `total_loaned` by the recalled amount
    /// - Updates trigger info with recall progress
    /// - Emits `LOAN/RECALL` event
    ///
    /// # Errors
    /// - `InheritanceNotTriggered` if inheritance hasn't been triggered
    /// - `NoOutstandingLoans` if there are no loans to recall
    /// - `LoanRecallFailed` if recall_amount exceeds outstanding loans
    pub fn recall_loan(
        env: Env,
        admin: Address,
        plan_id: u64,
        recall_amount: u64,
    ) -> Result<(), InheritanceError> {
        Self::require_admin(&env, &admin)?;

        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        let mut trigger_info = Self::get_trigger_info(&env, plan_id)
            .ok_or(InheritanceError::InheritanceNotTriggered)?;

        if plan.total_loaned == 0 {
            return Err(InheritanceError::NoOutstandingLoans);
        }

        if recall_amount == 0 || recall_amount > plan.total_loaned {
            return Err(InheritanceError::LoanRecallFailed);
        }

        // Reduce the loaned amount
        plan.total_loaned -= recall_amount;
        Self::store_plan(&env, plan_id, &plan);

        // Update trigger info
        trigger_info.recall_attempted = true;
        trigger_info.recalled_amount += recall_amount;
        Self::set_trigger_info(&env, plan_id, &trigger_info);

        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("RECALL")),
            LoanRecallEvent {
                plan_id,
                recalled_amount: recall_amount,
                remaining_loaned: plan.total_loaned,
            },
        );

        log!(
            &env,
            "Recalled {} from plan {} loans — {} remaining",
            recall_amount,
            plan_id,
            plan.total_loaned
        );

        Ok(())
    }

    /// Trigger liquidation fallback when loans cannot be fully recalled.
    /// This writes off unrecoverable loaned amounts so that inheritance
    /// execution cannot be blocked by outstanding loans.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `admin` - The admin address
    /// * `plan_id` - The plan ID
    ///
    /// # Effects
    /// - Writes off remaining `total_loaned` from `total_amount`
    /// - Sets `total_loaned` to 0
    /// - Records liquidation in trigger info
    /// - Emits `LOAN/LIQUIDATE` event
    ///
    /// # Errors
    /// - `InheritanceNotTriggered` if inheritance hasn't been triggered
    /// - `NoOutstandingLoans` if there are no loans to liquidate
    pub fn liquidation_fallback(
        env: Env,
        admin: Address,
        plan_id: u64,
    ) -> Result<(), InheritanceError> {
        Self::require_admin(&env, &admin)?;

        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        let mut trigger_info = Self::get_trigger_info(&env, plan_id)
            .ok_or(InheritanceError::InheritanceNotTriggered)?;

        if plan.total_loaned == 0 {
            return Err(InheritanceError::NoOutstandingLoans);
        }

        let unrecoverable = plan.total_loaned;

        // Write off the unrecoverable loaned amount from the plan's total
        plan.total_amount = plan.total_amount.saturating_sub(unrecoverable);
        plan.total_loaned = 0;
        Self::store_plan(&env, plan_id, &plan);

        // Update trigger info
        trigger_info.liquidation_triggered = true;
        trigger_info.settled_amount += unrecoverable;
        Self::set_trigger_info(&env, plan_id, &trigger_info);

        env.events().publish(
            (symbol_short!("LOAN"), symbol_short!("LIQUIDAT")),
            LiquidationFallbackEvent {
                plan_id,
                settled_amount: unrecoverable,
                claimable_amount: plan.total_amount,
            },
        );

        log!(
            &env,
            "Liquidation fallback for plan {}: wrote off {}, claimable: {}",
            plan_id,
            unrecoverable,
            plan.total_amount
        );

        Ok(())
    }

    /// Query the inheritance trigger status for a plan.
    pub fn get_inheritance_trigger(env: Env, plan_id: u64) -> Option<InheritanceTriggerInfo> {
        Self::get_trigger_info(&env, plan_id)
    }

    /// Calculate the claimable amount for a plan, accounting for outstanding loans.
    /// Returns the amount available to beneficiaries after any loan deductions.
    pub fn get_claimable_amount(env: Env, plan_id: u64) -> Result<u64, InheritanceError> {
        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;
        Ok(plan.total_amount.saturating_sub(plan.total_loaned))
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

    // ── Will Management System (Issues #314–#317) ──

    /// Store a SHA-256 hash of a will document on-chain, mapped to a plan_id.
    pub fn store_will_hash(
        env: Env,
        owner: Address,
        plan_id: u64,
        will_hash: BytesN<32>,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        let key = DataKey::WillHash(plan_id);
        if env
            .storage()
            .persistent()
            .get::<_, BytesN<32>>(&key)
            .is_some()
        {
            return Err(InheritanceError::WillHashAlreadyStored);
        }

        env.storage().persistent().set(&key, &will_hash);

        env.events().publish(
            (symbol_short!("WILL"), symbol_short!("STORED")),
            WillHashStoredEvent { plan_id, will_hash },
        );

        Ok(())
    }

    /// Retrieve the stored will hash for a plan.
    pub fn get_will_hash(env: Env, plan_id: u64) -> Option<BytesN<32>> {
        let key = DataKey::WillHash(plan_id);
        env.storage().persistent().get(&key)
    }

    /// Link a will document hash to a vault (plan). Prevents re-linking unless
    /// the will versioning system is used (create_will_version updates VaultWill).
    pub fn link_will_to_vault(
        env: Env,
        owner: Address,
        plan_id: u64,
        will_hash: BytesN<32>,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::VaultNotFound)?;
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        let key = DataKey::VaultWill(plan_id);
        if env
            .storage()
            .persistent()
            .get::<_, BytesN<32>>(&key)
            .is_some()
        {
            return Err(InheritanceError::WillAlreadyLinked);
        }

        env.storage().persistent().set(&key, &will_hash);

        env.events().publish(
            (symbol_short!("WILL"), symbol_short!("LINKED")),
            WillLinkedToVaultEvent { plan_id, will_hash },
        );

        Ok(())
    }

    /// Retrieve the will hash linked to a vault.
    pub fn get_vault_will(env: Env, plan_id: u64) -> Option<BytesN<32>> {
        let key = DataKey::VaultWill(plan_id);
        env.storage().persistent().get(&key)
    }

    /// Verify that the beneficiaries in a will document match those stored in the plan.
    /// Takes a list of (hashed_email, allocation_bp) pairs and compares against the plan.
    pub fn verify_beneficiaries(
        env: Env,
        plan_id: u64,
        will_beneficiaries: Vec<(BytesN<32>, u32)>,
    ) -> Result<bool, InheritanceError> {
        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        let plan_bens = &plan.beneficiaries;
        let mut status = true;

        // Check count matches
        if will_beneficiaries.len() != plan_bens.len() {
            status = false;
        } else {
            // For each will beneficiary, find a matching plan beneficiary
            for i in 0..will_beneficiaries.len() {
                let (ref wh_email, w_alloc) = will_beneficiaries.get(i).unwrap();
                let mut found = false;
                for j in 0..plan_bens.len() {
                    let pb = plan_bens.get(j).unwrap();
                    if pb.hashed_email == *wh_email && pb.allocation_bp == w_alloc {
                        found = true;
                        break;
                    }
                }
                if !found {
                    status = false;
                    break;
                }
            }
        }

        // Store verification result
        let ver_key = DataKey::BeneficiaryVerification(plan_id);
        env.storage().persistent().set(&ver_key, &status);

        env.events().publish(
            (symbol_short!("WILL"), symbol_short!("VERIFY")),
            BeneficiariesVerifiedEvent { plan_id, status },
        );

        Ok(status)
    }

    /// Get the last beneficiary verification status for a plan.
    pub fn get_verification_status(env: Env, plan_id: u64) -> Option<bool> {
        let key = DataKey::BeneficiaryVerification(plan_id);
        env.storage().persistent().get(&key)
    }

    /// Create a new will version for a plan. Auto-increments version number and
    /// deactivates the previously active version. Also updates the VaultWill link.
    pub fn create_will_version(
        env: Env,
        owner: Address,
        plan_id: u64,
        will_hash: BytesN<32>,
    ) -> Result<u32, InheritanceError> {
        owner.require_auth();

        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        // Block creating a new version if the currently active version is finalized
        let active_key = DataKey::ActiveWillVersion(plan_id);
        if let Some(active_ver_num) = env.storage().persistent().get::<_, u32>(&active_key) {
            let fin_key = DataKey::WillFinalized(plan_id, active_ver_num);
            if env
                .storage()
                .persistent()
                .get::<_, bool>(&fin_key)
                .unwrap_or(false)
            {
                return Err(InheritanceError::WillAlreadyFinalized);
            }
        }

        // Get and increment version count
        let count_key = DataKey::WillVersionCount(plan_id);
        let current_count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let new_version = current_count + 1;
        env.storage().persistent().set(&count_key, &new_version);

        // Deactivate previously active version if any
        let active_key = DataKey::ActiveWillVersion(plan_id);
        if let Some(prev_ver_num) = env.storage().persistent().get::<_, u32>(&active_key) {
            let prev_key = DataKey::WillVersion(plan_id, prev_ver_num);
            if let Some(mut prev_ver) = env
                .storage()
                .persistent()
                .get::<_, WillVersionInfo>(&prev_key)
            {
                prev_ver.is_active = false;
                env.storage().persistent().set(&prev_key, &prev_ver);
            }
        }

        // Store new version
        let version_info = WillVersionInfo {
            version: new_version,
            will_hash: will_hash.clone(),
            created_at: env.ledger().timestamp(),
            is_active: true,
        };
        let ver_key = DataKey::WillVersion(plan_id, new_version);
        env.storage().persistent().set(&ver_key, &version_info);

        // Set as active
        env.storage().persistent().set(&active_key, &new_version);

        // Update VaultWill link to point to latest will hash
        let vault_will_key = DataKey::VaultWill(plan_id);
        env.storage().persistent().set(&vault_will_key, &will_hash);

        env.events().publish(
            (symbol_short!("WILL"), symbol_short!("VERSION")),
            WillVersionCreatedEvent {
                plan_id,
                version: new_version,
            },
        );

        env.events().publish(
            (symbol_short!("WILL"), symbol_short!("ACTIVE")),
            WillVersionActivatedEvent {
                plan_id,
                version: new_version,
            },
        );

        Ok(new_version)
    }

    /// Get a specific will version for a plan.
    pub fn get_will_version(env: Env, plan_id: u64, version: u32) -> Option<WillVersionInfo> {
        let key = DataKey::WillVersion(plan_id, version);
        env.storage().persistent().get(&key)
    }

    /// Get the currently active will version for a plan.
    pub fn get_active_will_version(env: Env, plan_id: u64) -> Option<WillVersionInfo> {
        let active_key = DataKey::ActiveWillVersion(plan_id);
        if let Some(active_ver) = env.storage().persistent().get::<_, u32>(&active_key) {
            let key = DataKey::WillVersion(plan_id, active_ver);
            env.storage().persistent().get(&key)
        } else {
            None
        }
    }

    /// Get the total number of will versions for a plan.
    pub fn get_will_version_count(env: Env, plan_id: u64) -> u32 {
        let key = DataKey::WillVersionCount(plan_id);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    // ── Will Signature Verification (Issue #318) ──

    /// Record that the vault owner has approved and signed a will.
    ///
    /// The caller must be the plan owner. A composite sig_hash is derived from
    /// (vault_id, will_hash) to bind the signature to a specific will version and
    /// prevent replay across different vaults or will documents.
    pub fn sign_will(
        env: Env,
        owner: Address,
        vault_id: u64,
        will_hash: BytesN<32>,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        // Verify the plan exists and caller is the owner
        let plan = Self::get_plan(&env, vault_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        // Derive a deterministic sig_hash from (vault_id, will_hash) for replay protection
        let mut sig_input = Bytes::new(&env);
        for b in vault_id.to_be_bytes() {
            sig_input.push_back(b);
        }
        for b in will_hash.to_array() {
            sig_input.push_back(b);
        }
        let sig_hash: BytesN<32> = env.crypto().sha256(&sig_input).into();

        // Replay protection: reject if this (vault_id, will_hash) pair was already signed
        let used_key = DataKey::SignatureUsed(sig_hash.clone());
        if env
            .storage()
            .persistent()
            .get::<_, bool>(&used_key)
            .unwrap_or(false)
        {
            return Err(InheritanceError::WillAlreadyFinalized);
        }

        // Mark signature as used
        env.storage().persistent().set(&used_key, &true);

        // Store the signature proof
        let proof = WillSignatureProof {
            vault_id,
            will_hash,
            signer: owner.clone(),
            sig_hash,
            signed_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::WillSignature(vault_id), &proof);

        // Emit WillSigned event
        env.events().publish(
            (symbol_short!("WILL"), symbol_short!("SIGNED")),
            WillSignedEvent {
                vault_id,
                signer: owner,
            },
        );

        Ok(())
    }

    /// Retrieve the stored will signature proof for a vault.
    pub fn get_will_signature(env: Env, vault_id: u64) -> Option<WillSignatureProof> {
        env.storage()
            .persistent()
            .get(&DataKey::WillSignature(vault_id))
    }

    /// Create a new legacy message with metadata stored on-chain
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `creator` - The address of the message creator (must be vault owner)
    /// * `params` - Message creation parameters including hash and unlock timestamp
    ///
    /// # Requirements
    /// - Creator must be the vault owner
    /// - Unlock timestamp must be in the future
    /// - Vault/plan must exist
    pub fn create_legacy_message(
        env: Env,
        creator: Address,
        params: CreateLegacyMessageParams,
    ) -> Result<u64, InheritanceError> {
        // Verify vault/plan exists and creator is the owner
        let plan = Self::get_plan(&env, params.vault_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != creator {
            return Err(InheritanceError::Unauthorized);
        }

        // Validate unlock timestamp is in the future
        let current_timestamp = env.ledger().timestamp();
        if params.unlock_timestamp <= current_timestamp {
            return Err(InheritanceError::InvalidClaimCode); // Reuse for invalid timestamp
        }

        // Generate unique message ID
        let message_id = env
            .storage()
            .persistent()
            .get(&DataKey::NextMessageId)
            .unwrap_or(0u64);

        // Create message metadata
        let message = LegacyMessageMetadata {
            vault_id: params.vault_id,
            message_id,
            message_hash: params.message_hash,
            creator: creator.clone(),
            key_reference: params.key_reference,
            unlock_timestamp: params.unlock_timestamp,
            is_unlocked: false,
            is_finalized: false,
            created_at: current_timestamp,
        };

        // Store message metadata
        env.storage()
            .persistent()
            .set(&DataKey::LegacyMessage(message_id), &message);

        // Add message to vault's message list
        let mut vault_messages: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::VaultMessages(params.vault_id))
            .unwrap_or_else(|| vec![&env]);
        vault_messages.push_back(message_id);
        env.storage()
            .persistent()
            .set(&DataKey::VaultMessages(params.vault_id), &vault_messages);

        // Increment next message ID
        env.storage()
            .persistent()
            .set(&DataKey::NextMessageId, &(message_id + 1));

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "message_created"), params.vault_id),
            MessageCreatedEvent {
                vault_id: params.vault_id,
                message_id,
                timestamp: current_timestamp,
            },
        );

        Ok(message_id)
    }

    pub fn update_legacy_message(
        env: Env,
        creator: Address,
        message_id: u64,
        params: CreateLegacyMessageParams,
    ) -> Result<(), InheritanceError> {
        creator.require_auth();

        let mut message = env
            .storage()
            .persistent()
            .get::<_, LegacyMessageMetadata>(&DataKey::LegacyMessage(message_id))
            .ok_or(InheritanceError::PlanNotFound)?;

        if message.creator != creator {
            return Err(InheritanceError::Unauthorized);
        }

        if message.is_finalized {
            return Err(InheritanceError::WillAlreadyFinalized);
        }

        if message.is_unlocked {
            return Err(InheritanceError::AlreadyClaimed);
        }

        message.message_hash = params.message_hash;
        message.unlock_timestamp = params.unlock_timestamp;
        message.key_reference = params.key_reference;

        env.storage()
            .persistent()
            .set(&DataKey::LegacyMessage(message_id), &message);

        env.events().publish(
            (Symbol::new(&env, "message_updated"), message.vault_id),
            MessageUpdatedEvent {
                vault_id: message.vault_id,
                message_id,
                timestamp: env.ledger().timestamp(),
            },
        );
        Ok(())
    }

    pub fn finalize_legacy_message(
        env: Env,
        creator: Address,
        message_id: u64,
    ) -> Result<(), InheritanceError> {
        creator.require_auth();

        let mut message = env
            .storage()
            .persistent()
            .get::<_, LegacyMessageMetadata>(&DataKey::LegacyMessage(message_id))
            .ok_or(InheritanceError::PlanNotFound)?;

        if message.creator != creator {
            return Err(InheritanceError::Unauthorized);
        }

        if message.is_finalized {
            return Err(InheritanceError::WillAlreadyFinalized);
        }

        message.is_finalized = true;

        env.storage()
            .persistent()
            .set(&DataKey::LegacyMessage(message_id), &message);

        env.events().publish(
            (Symbol::new(&env, "message_finalized"), message.vault_id),
            MessageFinalizedEvent {
                vault_id: message.vault_id,
                message_id,
                timestamp: env.ledger().timestamp(),
            },
        );
        Ok(())
    }

    /// Get metadata for a specific legacy message
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `message_id` - The unique message identifier
    pub fn get_legacy_message(env: Env, message_id: u64) -> Option<LegacyMessageMetadata> {
        env.storage()
            .persistent()
            .get(&DataKey::LegacyMessage(message_id))
    }

    /// Get all message IDs for a specific vault
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `vault_id` - The vault/plan ID
    pub fn get_vault_messages(env: Env, vault_id: u64) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::VaultMessages(vault_id))
            .unwrap_or_else(|| vec![&env])
    }

    /// Delete a legacy message before it has been finalized.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `owner` - The vault owner requesting deletion
    /// * `message_id` - The message to delete
    ///
    /// # Errors
    /// - `PlanNotFound` if the message does not exist
    /// - `Unauthorized` if caller is not the message creator
    /// - `WillAlreadyFinalized` if the message has been finalized
    pub fn delete_legacy_message(
        env: Env,
        owner: Address,
        message_id: u64,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let message: LegacyMessageMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::LegacyMessage(message_id))
            .ok_or(InheritanceError::PlanNotFound)?;

        if message.creator != owner {
            return Err(InheritanceError::Unauthorized);
        }

        if message.is_finalized {
            return Err(InheritanceError::WillAlreadyFinalized);
        }

        // Remove message metadata
        env.storage()
            .persistent()
            .remove(&DataKey::LegacyMessage(message_id));

        // Remove from vault's message list
        let vault_messages: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::VaultMessages(message.vault_id))
            .unwrap_or_else(|| vec![&env]);
        let mut updated: Vec<u64> = vec![&env];
        for id in vault_messages.iter() {
            if id != message_id {
                updated.push_back(id);
            }
        }
        env.storage()
            .persistent()
            .set(&DataKey::VaultMessages(message.vault_id), &updated);

        env.events().publish(
            (Symbol::new(&env, "message_deleted"), message.vault_id),
            MessageDeletedEvent {
                vault_id: message.vault_id,
                message_id,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Access a legacy message (returns metadata if accessible)
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `caller` - The address requesting access
    /// * `message_id` - The message ID to access
    ///
    /// # Requirements
    /// - Caller must be a verified beneficiary of the vault
    /// - Message must be unlocked (either by timestamp or inheritance trigger)
    pub fn access_legacy_message(
        env: Env,
        caller: Address,
        message_id: u64,
    ) -> Result<LegacyMessageMetadata, InheritanceError> {
        // Get message metadata
        let mut message: LegacyMessageMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::LegacyMessage(message_id))
            .ok_or(InheritanceError::PlanNotFound)?; // Reuse PlanNotFound for MessageNotFound

        // Check if already unlocked
        if !message.is_unlocked {
            let current_timestamp = env.ledger().timestamp();

            // Check if unlock timestamp has been reached
            if current_timestamp >= message.unlock_timestamp {
                // Unlock by timestamp
                message.is_unlocked = true;
                env.storage()
                    .persistent()
                    .set(&DataKey::LegacyMessage(message_id), &message);

                // Emit unlock event
                env.events().publish(
                    (Symbol::new(&env, "message_unlocked"), message.vault_id),
                    MessageUnlockedEvent {
                        vault_id: message.vault_id,
                        message_id,
                        timestamp: current_timestamp,
                    },
                );
            } else {
                // Check if inheritance has been triggered
                let inheritance_triggered: bool = env
                    .storage()
                    .persistent()
                    .get(&DataKey::InheritanceTrigger(message.vault_id))
                    .map(|info: InheritanceTriggerInfo| info.triggered_at > 0)
                    .unwrap_or(false);

                if inheritance_triggered {
                    // Unlock by inheritance trigger
                    message.is_unlocked = true;
                    env.storage()
                        .persistent()
                        .set(&DataKey::LegacyMessage(message_id), &message);

                    // Emit unlock event
                    env.events().publish(
                        (Symbol::new(&env, "message_unlocked"), message.vault_id),
                        MessageUnlockedEvent {
                            vault_id: message.vault_id,
                            message_id,
                            timestamp: current_timestamp,
                        },
                    );
                } else {
                    // Message still locked
                    return Err(InheritanceError::ClaimNotAllowedYet); // Reuse for locked message
                }
            }
        }

        // Verify caller is a beneficiary of this vault
        let plan = Self::get_plan(&env, message.vault_id).ok_or(InheritanceError::PlanNotFound)?;

        // Hash the caller's address to check against beneficiaries
        let caller_bytes = Bytes::from_val(&env, &caller.to_val());
        let caller_hash: BytesN<32> = env.crypto().sha256(&caller_bytes).into();
        let mut is_beneficiary = false;

        for i in 0..plan.beneficiaries.len() {
            let beneficiary = plan
                .beneficiaries
                .get(i)
                .ok_or(InheritanceError::BeneficiaryNotFound)?;
            // Check if caller matches any beneficiary hashed email
            if beneficiary.hashed_email == caller_hash {
                is_beneficiary = true;
                break;
            }
        }

        if !is_beneficiary {
            return Err(InheritanceError::Unauthorized);
        }

        // Emit access event
        env.events().publish(
            (Symbol::new(&env, "message_accessed"), message.vault_id),
            MessageAccessedEvent {
                vault_id: message.vault_id,
                message_id,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(message)
    }

    /// Manually unlock a message when inheritance is triggered
    /// This can be called during the inheritance trigger process
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `vault_id` - The vault/plan ID for which inheritance was triggered
    pub fn unlock_messages_on_inheritance(env: Env, vault_id: u64) -> Result<(), InheritanceError> {
        // Verify inheritance was triggered
        let trigger_info: InheritanceTriggerInfo = env
            .storage()
            .persistent()
            .get(&DataKey::InheritanceTrigger(vault_id))
            .ok_or(InheritanceError::InheritanceNotTriggered)?;

        if trigger_info.triggered_at == 0 {
            return Err(InheritanceError::InheritanceNotTriggered);
        }

        // Get all messages for this vault
        let messages = Self::get_vault_messages(env.clone(), vault_id);
        let current_timestamp = env.ledger().timestamp();

        // Unlock each message
        for message_id in messages.iter() {
            let mut message: LegacyMessageMetadata = match env
                .storage()
                .persistent()
                .get(&DataKey::LegacyMessage(message_id))
            {
                Some(m) => m,
                None => continue, // Skip if message doesn't exist
            };

            if !message.is_unlocked {
                message.is_unlocked = true;
                env.storage()
                    .persistent()
                    .set(&DataKey::LegacyMessage(message_id), &message);

                // Emit unlock event
                env.events().publish(
                    (Symbol::new(&env, "message_unlocked"), vault_id),
                    MessageUnlockedEvent {
                        vault_id,
                        message_id,
                        timestamp: current_timestamp,
                    },
                );
            }
        }

        Ok(())
    }

    // ── Will Finalization (Issue #319) ──

    /// Finalize a specific will version, permanently locking it.
    ///
    /// Requirements:
    /// - Caller must be the plan owner.
    /// - The will version must exist.
    /// - The owner must have signed the will (WillSignature must exist).
    /// - If witnesses are assigned, all must have signed.
    /// - Cannot finalize an already-finalized version.
    pub fn finalize_will(
        env: Env,
        owner: Address,
        vault_id: u64,
        version: u32,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let plan = Self::get_plan(&env, vault_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        // Version must exist
        let ver_key = DataKey::WillVersion(vault_id, version);
        env.storage()
            .persistent()
            .get::<_, WillVersionInfo>(&ver_key)
            .ok_or(InheritanceError::WillVersionNotFound)?;

        // Already finalized?
        let fin_key = DataKey::WillFinalized(vault_id, version);
        if env
            .storage()
            .persistent()
            .get::<_, bool>(&fin_key)
            .unwrap_or(false)
        {
            return Err(InheritanceError::WillAlreadyFinalized);
        }

        // Owner must have signed the will
        if env
            .storage()
            .persistent()
            .get::<_, WillSignatureProof>(&DataKey::WillSignature(vault_id))
            .is_none()
        {
            return Err(InheritanceError::WillNotVerified);
        }

        // All assigned witnesses must have signed
        let witnesses_key = DataKey::WillWitnesses(vault_id);
        let witnesses: Vec<Address> = env
            .storage()
            .persistent()
            .get(&witnesses_key)
            .unwrap_or_else(|| Vec::new(&env));

        for i in 0..witnesses.len() {
            let w = witnesses.get(i).unwrap();
            let wsig_key = DataKey::WitnessSignature(vault_id, w);
            if env
                .storage()
                .persistent()
                .get::<_, u64>(&wsig_key)
                .is_none()
            {
                return Err(InheritanceError::MissingRequiredField);
            }
        }

        let finalized_at = env.ledger().timestamp();
        env.storage().persistent().set(&fin_key, &true);
        env.storage()
            .persistent()
            .set(&DataKey::WillFinalizedAt(vault_id, version), &finalized_at);

        env.events().publish(
            (symbol_short!("WILL"), symbol_short!("FINAL")),
            WillFinalizedEvent {
                vault_id,
                version,
                finalized_at,
            },
        );

        Ok(())
    }

    /// Check whether a specific will version is finalized.
    pub fn is_will_finalized(env: Env, vault_id: u64, version: u32) -> bool {
        env.storage()
            .persistent()
            .get::<_, bool>(&DataKey::WillFinalized(vault_id, version))
            .unwrap_or(false)
    }

    /// Get the finalization timestamp for a will version (None if not finalized).
    pub fn get_will_finalized_at(env: Env, vault_id: u64, version: u32) -> Option<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::WillFinalizedAt(vault_id, version))
    }

    // ── Legal Witness Verification (Issue #320) ──

    /// Assign a witness address to a vault's will. Only the plan owner can add witnesses.
    pub fn add_witness(
        env: Env,
        owner: Address,
        vault_id: u64,
        witness: Address,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let plan = Self::get_plan(&env, vault_id).ok_or(InheritanceError::PlanNotFound)?;
        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }

        let key = DataKey::WillWitnesses(vault_id);
        let mut witnesses: Vec<Address> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));

        // Prevent duplicates
        for i in 0..witnesses.len() {
            if witnesses.get(i).unwrap() == witness {
                return Err(InheritanceError::EmergencyContactAlreadyExists);
            }
        }

        witnesses.push_back(witness.clone());
        env.storage().persistent().set(&key, &witnesses);

        env.events().publish(
            (symbol_short!("WILL"), symbol_short!("WITNESS")),
            WitnessAddedEvent { vault_id, witness },
        );

        Ok(())
    }

    /// Record a witness signature for a vault's will.
    ///
    /// The caller must be a registered witness for this vault.
    pub fn sign_as_witness(
        env: Env,
        witness: Address,
        vault_id: u64,
    ) -> Result<(), InheritanceError> {
        witness.require_auth();

        // Vault must exist
        Self::get_plan(&env, vault_id).ok_or(InheritanceError::PlanNotFound)?;

        // Witness must be in the registered list
        let key = DataKey::WillWitnesses(vault_id);
        let witnesses: Vec<Address> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));

        let mut found = false;
        for i in 0..witnesses.len() {
            if witnesses.get(i).unwrap() == witness {
                found = true;
                break;
            }
        }
        if !found {
            return Err(InheritanceError::EmergencyContactNotFound);
        }

        // Prevent double-signing
        let wsig_key = DataKey::WitnessSignature(vault_id, witness.clone());
        if env
            .storage()
            .persistent()
            .get::<_, u64>(&wsig_key)
            .is_some()
        {
            return Err(InheritanceError::AlreadyApproved);
        }

        let signed_at = env.ledger().timestamp();
        env.storage().persistent().set(&wsig_key, &signed_at);

        env.events().publish(
            (symbol_short!("WILL"), symbol_short!("WSIGN")),
            WitnessSignedEvent { vault_id, witness },
        );

        Ok(())
    }

    /// Get all registered witnesses for a vault.
    pub fn get_witnesses(env: Env, vault_id: u64) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::WillWitnesses(vault_id))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get the timestamp at which a witness signed, or None if not yet signed.
    pub fn get_witness_signature(env: Env, vault_id: u64, witness: Address) -> Option<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::WitnessSignature(vault_id, witness))
    }

    // ---------- Beneficiary Update Functions ----------

    /// Update a beneficiary's allocation percentage.
    ///
    /// # Arguments
    /// * `owner` - The plan owner (must authorize)
    /// * `plan_id` - The ID of the plan
    /// * `index` - The 0-based beneficiary index
    /// * `new_allocation_bp` - New allocation in basis points (1-10000)
    ///
    /// # Errors
    /// - `Unauthorized`: Not the plan owner
    /// - `PlanNotFound`: Plan does not exist
    /// - `PlanNotActive`: Plan is inactive or inheritance already triggered
    /// - `InvalidBeneficiaryIndex`: Index out of range
    /// - `InvalidAllocation`: New allocation is 0
    /// - `AllocationExceedsLimit`: New total would exceed 10000 bp
    pub fn update_beneficiary_allocation(
        env: Env,
        owner: Address,
        plan_id: u64,
        index: u32,
        new_allocation_bp: u32,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }
        if !plan.is_active || Self::get_trigger_info(&env, plan_id).is_some() {
            return Err(InheritanceError::PlanNotActive);
        }
        if index >= plan.beneficiaries.len() {
            return Err(InheritanceError::InvalidBeneficiaryIndex);
        }
        if new_allocation_bp == 0 {
            return Err(InheritanceError::InvalidAllocation);
        }

        let mut beneficiary = plan.beneficiaries.get(index).unwrap();
        let old_allocation = beneficiary.allocation_bp;

        let new_total = plan
            .total_allocation_bp
            .saturating_sub(old_allocation)
            .saturating_add(new_allocation_bp);
        if new_total > 10000 {
            return Err(InheritanceError::AllocationExceedsLimit);
        }

        beneficiary.allocation_bp = new_allocation_bp;
        plan.beneficiaries.set(index, beneficiary);
        plan.total_allocation_bp = new_total;

        Self::store_plan(&env, plan_id, &plan);

        env.events().publish(
            (symbol_short!("BENEFIC"), symbol_short!("UPDATE")),
            BeneficiaryUpdatedEvent {
                plan_id,
                index,
                field: Symbol::new(&env, "alloc"),
            },
        );

        Ok(())
    }

    /// Update a beneficiary's bank account details.
    ///
    /// # Arguments
    /// * `owner` - The plan owner (must authorize)
    /// * `plan_id` - The ID of the plan
    /// * `index` - The 0-based beneficiary index
    /// * `new_bank_account` - New bank account bytes (must be non-empty)
    ///
    /// # Errors
    /// - `Unauthorized`: Not the plan owner
    /// - `PlanNotFound`: Plan does not exist
    /// - `PlanNotActive`: Plan is inactive or inheritance already triggered
    /// - `InvalidBeneficiaryIndex`: Index out of range
    /// - `InvalidBeneficiaryData`: Bank account is empty
    pub fn update_beneficiary_bank_account(
        env: Env,
        owner: Address,
        plan_id: u64,
        index: u32,
        new_bank_account: Bytes,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }
        if !plan.is_active || Self::get_trigger_info(&env, plan_id).is_some() {
            return Err(InheritanceError::PlanNotActive);
        }
        if index >= plan.beneficiaries.len() {
            return Err(InheritanceError::InvalidBeneficiaryIndex);
        }
        if new_bank_account.is_empty() {
            return Err(InheritanceError::InvalidBeneficiaryData);
        }

        let mut beneficiary = plan.beneficiaries.get(index).unwrap();
        beneficiary.bank_account = new_bank_account;
        plan.beneficiaries.set(index, beneficiary);

        Self::store_plan(&env, plan_id, &plan);

        env.events().publish(
            (symbol_short!("BENEFIC"), symbol_short!("UPDATE")),
            BeneficiaryUpdatedEvent {
                plan_id,
                index,
                field: Symbol::new(&env, "bank"),
            },
        );

        Ok(())
    }

    /// Update a beneficiary's claim code.
    ///
    /// # Arguments
    /// * `owner` - The plan owner (must authorize)
    /// * `plan_id` - The ID of the plan
    /// * `index` - The 0-based beneficiary index
    /// * `new_claim_code` - New 6-digit claim code (0-999999)
    ///
    /// # Errors
    /// - `Unauthorized`: Not the plan owner
    /// - `PlanNotFound`: Plan does not exist
    /// - `PlanNotActive`: Plan is inactive or inheritance already triggered
    /// - `InvalidBeneficiaryIndex`: Index out of range
    /// - `InvalidClaimCodeRange`: Claim code > 999999
    pub fn update_beneficiary_claim_code(
        env: Env,
        owner: Address,
        plan_id: u64,
        index: u32,
        new_claim_code: u32,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }
        if !plan.is_active || Self::get_trigger_info(&env, plan_id).is_some() {
            return Err(InheritanceError::PlanNotActive);
        }
        if index >= plan.beneficiaries.len() {
            return Err(InheritanceError::InvalidBeneficiaryIndex);
        }

        let new_hashed_claim_code = Self::hash_claim_code(&env, new_claim_code)?;

        let mut beneficiary = plan.beneficiaries.get(index).unwrap();
        beneficiary.hashed_claim_code = new_hashed_claim_code;
        plan.beneficiaries.set(index, beneficiary);

        Self::store_plan(&env, plan_id, &plan);

        env.events().publish(
            (symbol_short!("BENEFIC"), symbol_short!("UPDATE")),
            BeneficiaryUpdatedEvent {
                plan_id,
                index,
                field: Symbol::new(&env, "code"),
            },
        );

        Ok(())
    }

    /// Update a beneficiary's email (stored as a hash).
    ///
    /// # Arguments
    /// * `owner` - The plan owner (must authorize)
    /// * `plan_id` - The ID of the plan
    /// * `index` - The 0-based beneficiary index
    /// * `new_email` - New email string (must be non-empty)
    ///
    /// # Errors
    /// - `Unauthorized`: Not the plan owner
    /// - `PlanNotFound`: Plan does not exist
    /// - `PlanNotActive`: Plan is inactive or inheritance already triggered
    /// - `InvalidBeneficiaryIndex`: Index out of range
    /// - `InvalidBeneficiaryData`: Email is empty
    pub fn update_beneficiary_email(
        env: Env,
        owner: Address,
        plan_id: u64,
        index: u32,
        new_email: String,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }
        if !plan.is_active || Self::get_trigger_info(&env, plan_id).is_some() {
            return Err(InheritanceError::PlanNotActive);
        }
        if index >= plan.beneficiaries.len() {
            return Err(InheritanceError::InvalidBeneficiaryIndex);
        }
        if new_email.is_empty() {
            return Err(InheritanceError::InvalidBeneficiaryData);
        }

        let new_hashed_email = Self::hash_string(&env, new_email);

        let mut beneficiary = plan.beneficiaries.get(index).unwrap();
        beneficiary.hashed_email = new_hashed_email;
        plan.beneficiaries.set(index, beneficiary);

        Self::store_plan(&env, plan_id, &plan);

        env.events().publish(
            (symbol_short!("BENEFIC"), symbol_short!("UPDATE")),
            BeneficiaryUpdatedEvent {
                plan_id,
                index,
                field: Symbol::new(&env, "email"),
            },
        );

        Ok(())
    }

    /// Swap the positions of two beneficiaries.
    ///
    /// # Arguments
    /// * `owner` - The plan owner (must authorize)
    /// * `plan_id` - The ID of the plan
    /// * `index_a` - First beneficiary index
    /// * `index_b` - Second beneficiary index
    ///
    /// # Errors
    /// - `Unauthorized`: Not the plan owner
    /// - `PlanNotFound`: Plan does not exist
    /// - `PlanNotActive`: Plan is inactive or inheritance already triggered
    /// - `InvalidBeneficiaryIndex`: Either index is out of range
    pub fn swap_beneficiary_order(
        env: Env,
        owner: Address,
        plan_id: u64,
        index_a: u32,
        index_b: u32,
    ) -> Result<(), InheritanceError> {
        owner.require_auth();

        let mut plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;

        if plan.owner != owner {
            return Err(InheritanceError::Unauthorized);
        }
        if !plan.is_active || Self::get_trigger_info(&env, plan_id).is_some() {
            return Err(InheritanceError::PlanNotActive);
        }

        let len = plan.beneficiaries.len();
        if index_a >= len || index_b >= len {
            return Err(InheritanceError::InvalidBeneficiaryIndex);
        }

        if index_a == index_b {
            return Ok(());
        }

        let beneficiary_a = plan.beneficiaries.get(index_a).unwrap();
        let beneficiary_b = plan.beneficiaries.get(index_b).unwrap();
        plan.beneficiaries.set(index_a, beneficiary_b);
        plan.beneficiaries.set(index_b, beneficiary_a);

        Self::store_plan(&env, plan_id, &plan);

        env.events().publish(
            (symbol_short!("BENEFIC"), symbol_short!("UPDATE")),
            BeneficiaryUpdatedEvent {
                plan_id,
                index: index_a,
                field: Symbol::new(&env, "swap"),
            },
        );

        Ok(())
    }

    /// Find a beneficiary by their email hash.
    ///
    /// # Arguments
    /// * `plan_id` - The ID of the plan to search
    /// * `email` - The plain-text email to look up
    ///
    /// # Returns
    /// `Ok((index, Beneficiary))` on match, or `Err(BeneficiaryNotFound)` if not found.
    ///
    /// # Errors
    /// - `PlanNotFound`: Plan does not exist
    /// - `BeneficiaryNotFound`: No beneficiary with that email exists in the plan
    pub fn get_beneficiary_by_email(
        env: Env,
        plan_id: u64,
        email: String,
    ) -> Result<(u32, Beneficiary), InheritanceError> {
        let plan = Self::get_plan(&env, plan_id).ok_or(InheritanceError::PlanNotFound)?;
        let hashed_email = Self::hash_string(&env, email);

        for i in 0..plan.beneficiaries.len() {
            let b = plan.beneficiaries.get(i).unwrap();
            if b.hashed_email == hashed_email {
                return Ok((i, b));
            }
        }

        Err(InheritanceError::BeneficiaryNotFound)
    }
}

#[cfg(test)]
#[allow(clippy::duplicated_attributes)]
mod message_test;
mod test;

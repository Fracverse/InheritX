#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Bytes, BytesN, Env, String, Vec};

// ─── Enums ──────────────────────────────────────────────────────────────────

/// Health-based conditions that can serve as inheritance triggers.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeneticCondition {
    /// A named hereditary disease has been diagnosed.
    HereditaryDisease(String),
    /// A life-expectancy marker has been detected.
    LifeExpectancyMarker,
    /// Carrier status for a named condition has been confirmed.
    CarrierStatus(String),
    /// A risk score (0–100) has been computed for the individual.
    HealthRiskFactor(u32),
    /// An age-based condition with a trigger age.
    AgeRelatedCondition(u32),
}

/// Current state of a DNA verification request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DNAVerificationStatus {
    Pending,
    Verified,
    Rejected,
    PartialMatch,
    RequiresRetest,
}

/// The kind of event that activates a genetic trigger.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeneticTriggerType {
    HealthConditionDetected,
    AgeThresholdReached,
    CarrierStatusConfirmed,
    RiskFactorExceeded,
    LifeExpectancyReduced,
}

/// Privacy level for DNA data.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrivacyLevel {
    Public, // Basic family verification only
    Protected, // Health conditions hidden
    Private, // Full genetic privacy
    Medical, // Medical professional access only
}

/// Hashing algorithm used for DNA hashing.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HashAlgorithm {
    Sha256,
    HmacSha256,
}

/// Genetic marker types.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeneticMarker {
    IdentityMarker(String), // For lineage verification
    HealthMarker(String), // For health condition detection
    TraitMarker(String), // For trait verification
    AncestryMarker(String), // For ancestry verification
}

/// Risk level for detected conditions.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Errors for the genetic verification contract.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneticVerificationError {
    InvalidInput = 1,
    HashGenerationFailed = 2,
    VerificationFailed = 3,
    InvalidSimilarityThreshold = 4,
}

// ─── Core Structs ─────────────────────────────────────────────────────────────

/// Links a DNA hash to an inheritance plan and its verification state.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneticInheritance {
    /// SHA-256 hash of (raw_dna_data || cryptographic_salt).
    pub dna_hash: BytesN<32>,
    /// Whether the lineage has been verified by an authority.
    pub verified_lineage: bool,
    /// Genetic conditions attached to this plan.
    pub genetic_triggers: Vec<GeneticCondition>,
    /// Identifier of the associated family tree.
    pub family_tree_id: u64,
    /// Ledger timestamp of the last verification.
    pub verification_timestamp: u64,
    /// Address of the entity that performed verification.
    pub verifying_authority: Address,
}

/// One node in the family tree with its DNA hash and relationships.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineageRecord {
    pub person_id: u64,
    /// SHA-256 hash of (raw_dna_data || cryptographic_salt).
    pub dna_hash: BytesN<32>,
    pub parent_ids: Vec<u64>,
    pub children_ids: Vec<u64>,
    /// 1 = parent/child, 2 = grandparent/grandchild, etc.
    pub relationship_degree: u32,
    pub verification_status: DNAVerificationStatus,
}

/// Configuration for a single genetic trigger.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneticTriggerConfig {
    pub trigger_type: GeneticTriggerType,
    /// Human-readable label for the condition being monitored.
    pub condition_name: String,
    /// Numeric threshold (e.g. risk score, age) that activates the trigger.
    pub threshold_value: u32,
    /// When true, a medical authority must attest before the trigger fires.
    pub requires_medical_confirmation: bool,
    /// Days of grace period before the inheritance is actually released.
    pub grace_period_days: u32,
}

/// A verified relationship between two people.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedRelationship {
    pub person1_id: u64,
    pub person2_id: u64,
    pub relationship_type: RelationshipType,
    /// Confidence score 0–100.
    pub confidence_score: u32,
    pub verified_by: Address,
    pub verification_date: u64,
}

/// A relative whose relationship is still being established.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingRelative {
    pub person_id: u64,
    pub dna_hash: BytesN<32>,
    pub proposed_relationship: RelationshipType,
}

/// Biological / legal relationship categories.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationshipType {
    Parent,
    Child,
    Sibling,
    Grandparent,
    Grandchild,
    Spouse,
    Other,
}

/// The complete family tree for one root person.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FamilyTree {
    pub tree_id: u64,
    pub root_person: u64,
    pub all_members: Vec<LineageRecord>,
    pub verified_relationships: Vec<VerifiedRelationship>,
    pub pending_discoveries: Vec<PendingRelative>,
}

/// DNA hash configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DNAHashConfig {
    pub privacy_level: PrivacyLevel,
    pub hash_algorithm: HashAlgorithm,
    pub salt_complexity: u32,
    pub selective_markers: Vec<GeneticMarker>,
}

/// Detected genetic condition.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetectedCondition {
    pub condition_name: String,
    pub confidence_level: u32, // 0-100
    pub risk_level: RiskLevel,
    pub age_of_onset: Option<u32>,
    pub requires_confirmation: bool,
}

// ─── Helper Functions ─────────────────────────────────────────────────────────

/// Returns true when `degree` is a valid relationship degree (1-based, ≥ 1).
pub fn is_valid_relationship_degree(degree: u32) -> bool {
    degree >= 1
}

/// Returns true when `score` is within the valid 0–100 range.
pub fn is_valid_confidence_score(score: u32) -> bool {
    score <= 100
}

/// Returns true when `risk` is within the valid 0–100 range.
pub fn is_valid_risk_score(risk: u32) -> bool {
    risk <= 100
}

#[contract]
pub struct GeneticVerificationContract;

#[contractimpl]
impl GeneticVerificationContract {
    /// Generate a DNA hash from DNA data, salt, and privacy level.
    pub fn generate_dna_hash(
        env: &Env,
        dna_data: Bytes,
        salt: BytesN<32>,
        _privacy_level: PrivacyLevel,
    ) -> Result<BytesN<32>, GeneticVerificationError> {
        if dna_data.is_empty() {
            return Err(GeneticVerificationError::InvalidInput);
        }

        let mut data = Bytes::new(env);
        for b in salt.to_array().iter() {
            data.push_back(*b);
        }
        for b in dna_data.iter() {
            data.push_back(b);
        }

        Ok(env.crypto().sha256(&data).into())
    }

    /// Verify if two DNA hashes match within a similarity threshold.
    pub fn verify_dna_match(
        env: &Env,
        claimed_hash: BytesN<32>,
        reference_hash: BytesN<32>,
        similarity_threshold: u32,
    ) -> Result<bool, GeneticVerificationError> {
        if similarity_threshold > 100 {
            return Err(GeneticVerificationError::InvalidSimilarityThreshold);
        }

        let similarity = Self::calculate_genetic_similarity(env, claimed_hash, reference_hash)?;
        Ok(similarity >= similarity_threshold)
    }

    /// Calculate genetic similarity score (0-100) between two DNA hashes.
    pub fn calculate_genetic_similarity(
        _env: &Env,
        dna_hash1: BytesN<32>,
        dna_hash2: BytesN<32>,
    ) -> Result<u32, GeneticVerificationError> {
        let arr1 = dna_hash1.to_array();
        let arr2 = dna_hash2.to_array();

        let mut matching_bytes = 0;
        for i in 0..32 {
            if arr1[i] == arr2[i] {
                matching_bytes += 1;
            }
        }

        Ok((matching_bytes * 100) / 32)
    }

    /// Generate a genetic salt for a user.
    pub fn generate_genetic_salt(env: &Env, _user_address: Address) -> BytesN<32> {
        // Simple salt based on ledger sequence and a fixed constant
        let sequence = env.ledger().sequence();
        let mut data = Bytes::new(env);
        // Add sequence bytes
        let mut seq_val = sequence;
        let mut count: u32 = 0;
        while count < 8 {
            data.push_back((seq_val & 0xFF) as u8);
            seq_val >>= 8;
            count += 1;
        }
        // Add a genetic salt prefix
        data.push_back(0x47);
        data.push_back(0x45);
        data.push_back(0x4E);
        data.push_back(0x45);
        data.push_back(0x54);
        data.push_back(0x49);
        data.push_back(0x43);
        env.crypto().sha256(&data).into()
    }

    /// Validate genetic integrity.
    pub fn validate_genetic_integrity(
        env: &Env,
        dna_hash: BytesN<32>,
        integrity_proof: Bytes,
    ) -> Result<bool, GeneticVerificationError> {
        let calculated_hash = env.crypto().sha256(&integrity_proof);
        // Compare hash bytes with dna_hash bytes
        let hash_arr = calculated_hash.to_array();
        let dna_arr = dna_hash.to_array();
        let mut matching = 0u32;
        for i in 0..32 {
            if hash_arr[i] == dna_arr[i] {
                matching += 1;
            }
        }
        Ok(matching == 32)
    }

    /// Detect genetic conditions (placeholder implementation).
    pub fn detect_genetic_conditions(
        _env: &Env,
        _dna_hash: BytesN<32>,
        _condition_markers: Vec<GeneticMarker>,
    ) -> Result<Vec<DetectedCondition>, GeneticVerificationError> {
        Ok(Vec::new(_env))
    }

    /// Calculate health risk score (placeholder implementation).
    pub fn calculate_health_risk_score(
        _env: &Env,
        _dna_hash: BytesN<32>,
        _age: u32,
        _lifestyle_factors: Vec<String>,
    ) -> Result<u32, GeneticVerificationError> {
        Ok(0)
    }
}

mod test;

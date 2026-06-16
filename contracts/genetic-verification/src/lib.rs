#![no_std]
use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

// ─── Enums ────────────────────────────────────────────────────────────────────

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

mod test;

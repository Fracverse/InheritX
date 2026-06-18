#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env, String};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn test_dna_hash(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

fn test_address(env: &Env) -> Address {
    Address::generate(env)
}

// ─── GeneticInheritance ───────────────────────────────────────────────────────

#[test]
fn test_genetic_inheritance_creation() {
    let env = Env::default();
    let authority = test_address(&env);
    let gi = GeneticInheritance {
        dna_hash: test_dna_hash(&env, 1),
        verified_lineage: false,
        genetic_triggers: vec![&env],
        family_tree_id: 42,
        verification_timestamp: 1_000_000,
        verifying_authority: authority,
    };
    assert!(!gi.verified_lineage);
    assert_eq!(gi.family_tree_id, 42);
    assert_eq!(gi.genetic_triggers.len(), 0);
}

#[test]
fn test_genetic_inheritance_with_conditions() {
    let env = Env::default();
    let authority = test_address(&env);
    let conditions = vec![
        &env,
        GeneticCondition::HereditaryDisease(String::from_str(&env, "Huntington's")),
        GeneticCondition::HealthRiskFactor(75),
        GeneticCondition::AgeRelatedCondition(65),
    ];
    let gi = GeneticInheritance {
        dna_hash: test_dna_hash(&env, 2),
        verified_lineage: true,
        genetic_triggers: conditions,
        family_tree_id: 1,
        verification_timestamp: 2_000_000,
        verifying_authority: authority,
    };
    assert!(gi.verified_lineage);
    assert_eq!(gi.genetic_triggers.len(), 3);
}

// ─── GeneticCondition ─────────────────────────────────────────────────────────

#[test]
fn test_genetic_condition_variants() {
    let env = Env::default();

    let hd = GeneticCondition::HereditaryDisease(String::from_str(&env, "BRCA1"));
    let lm = GeneticCondition::LifeExpectancyMarker;
    let cs = GeneticCondition::CarrierStatus(String::from_str(&env, "Cystic Fibrosis"));
    let hrf = GeneticCondition::HealthRiskFactor(80);
    let arc = GeneticCondition::AgeRelatedCondition(70);

    // Each variant is distinct
    assert_ne!(hd, lm);
    assert_ne!(cs, hrf);
    assert_ne!(hrf, arc);

    // Same value equals itself
    assert_eq!(
        GeneticCondition::LifeExpectancyMarker,
        GeneticCondition::LifeExpectancyMarker
    );
    assert_eq!(
        GeneticCondition::HealthRiskFactor(80),
        GeneticCondition::HealthRiskFactor(80)
    );
    assert_ne!(
        GeneticCondition::HealthRiskFactor(80),
        GeneticCondition::HealthRiskFactor(50)
    );
}

// ─── DNAVerificationStatus ────────────────────────────────────────────────────

#[test]
fn test_dna_verification_status_variants() {
    assert_eq!(
        DNAVerificationStatus::Pending,
        DNAVerificationStatus::Pending
    );
    assert_eq!(
        DNAVerificationStatus::Verified,
        DNAVerificationStatus::Verified
    );
    assert_ne!(
        DNAVerificationStatus::Pending,
        DNAVerificationStatus::Verified
    );
    assert_ne!(
        DNAVerificationStatus::Rejected,
        DNAVerificationStatus::PartialMatch
    );
    assert_ne!(
        DNAVerificationStatus::RequiresRetest,
        DNAVerificationStatus::Verified
    );
}

// ─── LineageRecord ────────────────────────────────────────────────────────────

#[test]
fn test_lineage_record_creation() {
    let env = Env::default();
    let record = LineageRecord {
        person_id: 1,
        dna_hash: test_dna_hash(&env, 10),
        parent_ids: vec![&env, 0u64],
        children_ids: vec![&env, 2u64, 3u64],
        relationship_degree: 1,
        verification_status: DNAVerificationStatus::Verified,
    };
    assert_eq!(record.person_id, 1);
    assert_eq!(record.parent_ids.len(), 1);
    assert_eq!(record.children_ids.len(), 2);
    assert_eq!(record.relationship_degree, 1);
    assert_eq!(record.verification_status, DNAVerificationStatus::Verified);
}

#[test]
fn test_relationship_degree_validation() {
    assert!(is_valid_relationship_degree(1));
    assert!(is_valid_relationship_degree(2));
    assert!(is_valid_relationship_degree(10));
    assert!(!is_valid_relationship_degree(0)); // 0 is invalid
}

// ─── GeneticTriggerType ───────────────────────────────────────────────────────

#[test]
fn test_genetic_trigger_type_variants() {
    assert_eq!(
        GeneticTriggerType::HealthConditionDetected,
        GeneticTriggerType::HealthConditionDetected
    );
    assert_ne!(
        GeneticTriggerType::AgeThresholdReached,
        GeneticTriggerType::RiskFactorExceeded
    );
    assert_ne!(
        GeneticTriggerType::CarrierStatusConfirmed,
        GeneticTriggerType::LifeExpectancyReduced
    );
}

// ─── GeneticTriggerConfig ─────────────────────────────────────────────────────

#[test]
fn test_genetic_trigger_config() {
    let env = Env::default();
    let config = GeneticTriggerConfig {
        trigger_type: GeneticTriggerType::RiskFactorExceeded,
        condition_name: String::from_str(&env, "High cardiac risk"),
        threshold_value: 85,
        requires_medical_confirmation: true,
        grace_period_days: 30,
    };
    assert_eq!(config.threshold_value, 85);
    assert!(config.requires_medical_confirmation);
    assert_eq!(config.grace_period_days, 30);
    assert_eq!(config.trigger_type, GeneticTriggerType::RiskFactorExceeded);
}

// ─── FamilyTree ───────────────────────────────────────────────────────────────

#[test]
fn test_family_tree_creation() {
    let env = Env::default();
    let authority = test_address(&env);

    let root = LineageRecord {
        person_id: 0,
        dna_hash: test_dna_hash(&env, 0),
        parent_ids: vec![&env],
        children_ids: vec![&env, 1u64],
        relationship_degree: 1,
        verification_status: DNAVerificationStatus::Verified,
    };
    let child = LineageRecord {
        person_id: 1,
        dna_hash: test_dna_hash(&env, 1),
        parent_ids: vec![&env, 0u64],
        children_ids: vec![&env],
        relationship_degree: 1,
        verification_status: DNAVerificationStatus::Pending,
    };
    let rel = VerifiedRelationship {
        person1_id: 0,
        person2_id: 1,
        relationship_type: RelationshipType::Parent,
        confidence_score: 99,
        verified_by: authority,
        verification_date: 1_000_000,
    };
    let tree = FamilyTree {
        tree_id: 1,
        root_person: 0,
        all_members: vec![&env, root, child],
        verified_relationships: vec![&env, rel],
        pending_discoveries: vec![&env],
    };
    assert_eq!(tree.tree_id, 1);
    assert_eq!(tree.all_members.len(), 2);
    assert_eq!(tree.verified_relationships.len(), 1);
    assert_eq!(tree.pending_discoveries.len(), 0);
}

// ─── VerifiedRelationship ─────────────────────────────────────────────────────

#[test]
fn test_confidence_score_validation() {
    assert!(is_valid_confidence_score(0));
    assert!(is_valid_confidence_score(100));
    assert!(is_valid_confidence_score(50));
    assert!(!is_valid_confidence_score(101));
}

#[test]
fn test_verified_relationship_types() {
    let env = Env::default();
    let authority = test_address(&env);

    let make_rel = |rt: RelationshipType| VerifiedRelationship {
        person1_id: 0,
        person2_id: 1,
        relationship_type: rt,
        confidence_score: 95,
        verified_by: authority.clone(),
        verification_date: 0,
    };

    let parent_rel = make_rel(RelationshipType::Parent);
    let sibling_rel = make_rel(RelationshipType::Sibling);
    assert_ne!(parent_rel.relationship_type, sibling_rel.relationship_type);
    assert_eq!(parent_rel.confidence_score, 95);
}

// ─── Privacy helpers ──────────────────────────────────────────────────────────

#[test]
fn test_risk_score_validation() {
    assert!(is_valid_risk_score(0));
    assert!(is_valid_risk_score(100));
    assert!(!is_valid_risk_score(101));
}

#[test]
fn test_dna_hash_uniqueness() {
    let env = Env::default();
    let hash_a = test_dna_hash(&env, 0xAA);
    let hash_b = test_dna_hash(&env, 0xBB);
    assert_ne!(hash_a, hash_b);
}

#[test]
fn test_pending_relative_creation() {
    let env = Env::default();
    let pending = PendingRelative {
        person_id: 99,
        dna_hash: test_dna_hash(&env, 99),
        proposed_relationship: RelationshipType::Grandchild,
    };
    assert_eq!(pending.person_id, 99);
    assert_eq!(pending.proposed_relationship, RelationshipType::Grandchild);
}

// ─── Contract Tests ───────────────────────────────────────────────────────────

#[test]
fn test_generate_dna_hash() {
    let env = Env::default();
    let dna_data = Bytes::from_array(&env, &[1, 2, 3, 4, 5]);
    let salt = BytesN::from_array(&env, &[0xAA; 32]);
    let privacy_level = PrivacyLevel::Private;

    let result = GeneticVerificationContract::generate_dna_hash(&env, dna_data, salt, privacy_level);
    assert!(result.is_ok());
}

#[test]
fn test_generate_dna_hash_empty_data() {
    let env = Env::default();
    let dna_data = Bytes::new(&env);
    let salt = BytesN::from_array(&env, &[0xAA; 32]);
    let privacy_level = PrivacyLevel::Private;

    let result = GeneticVerificationContract::generate_dna_hash(&env, dna_data, salt, privacy_level);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GeneticVerificationError::InvalidInput);
}

#[test]
fn test_calculate_genetic_similarity() {
    let env = Env::default();
    let hash1 = BytesN::from_array(&env, &[0xAA; 32]);
    let hash2 = BytesN::from_array(&env, &[0xAA; 32]);

    let similarity = GeneticVerificationContract::calculate_genetic_similarity(&env, hash1.clone(), hash2);
    assert_eq!(similarity.unwrap(), 100);

    let hash3 = BytesN::from_array(&env, &[0xBB; 32]);
    let similarity2 = GeneticVerificationContract::calculate_genetic_similarity(&env, hash1, hash3);
    assert_eq!(similarity2.unwrap(), 0);
}

#[test]
fn test_verify_dna_match() {
    let env = Env::default();
    let hash1 = BytesN::from_array(&env, &[0xAA; 32]);
    let hash2 = BytesN::from_array(&env, &[0xAA; 32]);

    let result = GeneticVerificationContract::verify_dna_match(&env, hash1, hash2, 50);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_verify_dna_match_invalid_threshold() {
    let env = Env::default();
    let hash1 = BytesN::from_array(&env, &[0xAA; 32]);
    let hash2 = BytesN::from_array(&env, &[0xAA; 32]);

    let result = GeneticVerificationContract::verify_dna_match(&env, hash1, hash2, 101);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GeneticVerificationError::InvalidSimilarityThreshold);
}

#[test]
fn test_generate_genetic_salt() {
    let env = Env::default();
    let user = test_address(&env);
    let salt = GeneticVerificationContract::generate_genetic_salt(&env, user);
    assert_eq!(salt.len(), 32);
}

#[test]
fn test_validate_genetic_integrity() {
    let env = Env::default();
    let data = Bytes::from_array(&env, &[1, 2, 3, 4, 5]);
    let hash: BytesN<32> = env.crypto().sha256(&data).into();
    let result = GeneticVerificationContract::validate_genetic_integrity(&env, hash, data);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_detect_genetic_conditions() {
    let env = Env::default();
    let hash = BytesN::from_array(&env, &[0xAA; 32]);
    let markers = vec![&env];

    let result = GeneticVerificationContract::detect_genetic_conditions(&env, hash, markers);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_calculate_health_risk_score() {
    let env = Env::default();
    let hash = BytesN::from_array(&env, &[0xAA; 32]);
    let factors = vec![&env];

    let result = GeneticVerificationContract::calculate_health_risk_score(&env, hash, 30, factors);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

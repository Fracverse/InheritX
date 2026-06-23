//! Family tree construction and relationship mapping.

use crate::{DataKey, InheritanceError};
use genetic_verification::GeneticVerificationContract;
use soroban_sdk::{contracttype, Address, BytesN, Env, String, Vec};

const MAX_INITIAL_RELATIVES: u32 = 10;
const MAX_SEARCH_RADIUS: u32 = 10;
const MIN_MERGE_CONFIDENCE: u32 = 50;
const NEXT_TREE_ID_KEY: u32 = 8000;
const ALL_TREE_IDS_KEY: u32 = 8001;
const TREE_RECORD_KEY: u32 = 8010;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationshipType {
    Parent,
    Child,
    Sibling,
    HalfSibling,
    Grandparent,
    Grandchild,
    Uncle,
    Aunt,
    Nephew,
    Niece,
    Cousin,
    SecondCousin,
    Spouse,
    StepRelative,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelativeInput {
    pub address: Address,
    pub dna_hash: BytesN<32>,
    pub claimed_relationship: RelationshipType,
    pub supporting_documents: Vec<BytesN<32>>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredRelative {
    pub person_id: u64,
    pub relationship_type: RelationshipType,
    pub genetic_confidence: u32,
    pub relationship_degree: u32,
    pub discovery_method: DiscoveryMethod,
    pub requires_confirmation: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveryMethod {
    GeneticMatching,
    DocumentCrossReference,
    SocialGraphAnalysis,
    HistoricalRecords,
    ThirdPartyDatabases,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationResult {
    pub is_verified: bool,
    pub actual_degree: u32,
    pub confidence: u32,
    pub genetic_similarity: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneticMatch {
    pub person_id: u64,
    pub dna_hash: BytesN<32>,
    pub similarity: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMatch {
    pub document_hash: BytesN<32>,
    pub confidence: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergePoint {
    pub person1_tree1: u64,
    pub person2_tree2: u64,
    pub relationship: RelationshipType,
    pub confidence: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeConflict {
    pub person_id: u64,
    pub conflict_type: ConflictType,
    pub description: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConflictType {
    DuplicatePerson,
    InconsistentRelationship,
    MissingParent,
    CircularReference,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    pub conflict_id: u64,
    pub resolution_action: ResolutionAction,
    pub resolved_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionAction {
    MergeRecords,
    PreferTree1,
    PreferTree2,
    ManualReview,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InheritancePath {
    pub total_degree: u32,
    pub is_valid: bool,
    pub path_steps: Vec<PathStep>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathStep {
    pub from_person_id: u64,
    pub to_person_id: u64,
    pub relationship: RelationshipType,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InheritanceRight {
    pub person_id: u64,
    pub relationship_degree: u32,
    pub share_bp: u32,
    pub priority: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonAtDepth {
    pub person_id: u64,
    pub depth: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredDocument {
    pub document_hash: BytesN<32>,
    pub document_type: DocumentType,
    pub person_id: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentType {
    BirthCertificate,
    DeathCertificate,
    MarriageCertificate,
    AdoptionRecord,
    CourtOrder,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonRecord {
    pub person_id: u64,
    pub address: Address,
    pub dna_hash: BytesN<32>,
    pub supporting_documents: Vec<BytesN<32>>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipEdge {
    pub from_person_id: u64,
    pub to_person_id: u64,
    pub relationship: RelationshipType,
    pub confidence: u32,
    pub verified: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FamilyTree {
    pub tree_id: u64,
    pub root_person_id: u64,
    pub members: Vec<PersonRecord>,
    pub relationships: Vec<RelationshipEdge>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
struct PathCandidate {
    pub person_id: u64,
    pub depth: u32,
    pub steps: Vec<PathStep>,
}

fn tree_key(tree_id: u64) -> DataKey {
    DataKey::PlanMetadata(tree_id, TREE_RECORD_KEY)
}

fn next_tree_id(env: &Env) -> u64 {
    let id = env
        .storage()
        .persistent()
        .get(&DataKey::PlanMetadata(0, NEXT_TREE_ID_KEY))
        .unwrap_or(1u64);
    env.storage()
        .persistent()
        .set(&DataKey::PlanMetadata(0, NEXT_TREE_ID_KEY), &(id + 1));
    id
}

fn remember_tree_id(env: &Env, tree_id: u64) {
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::PlanMetadata(0, ALL_TREE_IDS_KEY))
        .unwrap_or(Vec::new(env));
    for id in ids.iter() {
        if id == tree_id {
            return;
        }
    }
    ids.push_back(tree_id);
    env.storage()
        .persistent()
        .set(&DataKey::PlanMetadata(0, ALL_TREE_IDS_KEY), &ids);
}

fn load_tree(env: &Env, tree_id: u64) -> Result<FamilyTree, InheritanceError> {
    env.storage()
        .persistent()
        .get(&tree_key(tree_id))
        .ok_or(InheritanceError::PlanNotFound)
}

fn store_tree(env: &Env, tree: &FamilyTree) {
    env.storage()
        .persistent()
        .set(&tree_key(tree.tree_id), tree);
    remember_tree_id(env, tree.tree_id);
}

fn is_zero_hash(env: &Env, hash: &BytesN<32>) -> bool {
    *hash == BytesN::from_array(env, &[0u8; 32])
}

fn relationship_degree(relationship: &RelationshipType) -> u32 {
    match relationship {
        RelationshipType::Parent
        | RelationshipType::Child
        | RelationshipType::Spouse
        | RelationshipType::StepRelative => 1,
        RelationshipType::Sibling
        | RelationshipType::HalfSibling
        | RelationshipType::Grandparent
        | RelationshipType::Grandchild
        | RelationshipType::Uncle
        | RelationshipType::Aunt
        | RelationshipType::Nephew
        | RelationshipType::Niece => 2,
        RelationshipType::Cousin => 3,
        RelationshipType::SecondCousin => 4,
    }
}

fn inverse_relationship(relationship: &RelationshipType) -> RelationshipType {
    match relationship {
        RelationshipType::Parent => RelationshipType::Child,
        RelationshipType::Child => RelationshipType::Parent,
        RelationshipType::Grandparent => RelationshipType::Grandchild,
        RelationshipType::Grandchild => RelationshipType::Grandparent,
        RelationshipType::Uncle => RelationshipType::Nephew,
        RelationshipType::Aunt => RelationshipType::Niece,
        RelationshipType::Nephew => RelationshipType::Uncle,
        RelationshipType::Niece => RelationshipType::Aunt,
        other => other.clone(),
    }
}

fn edge_between(tree: &FamilyTree, from: u64, to: u64) -> Option<PathStep> {
    for edge in tree.relationships.iter() {
        if edge.from_person_id == from && edge.to_person_id == to {
            return Some(PathStep {
                from_person_id: from,
                to_person_id: to,
                relationship: edge.relationship,
            });
        }
        if edge.from_person_id == to && edge.to_person_id == from {
            return Some(PathStep {
                from_person_id: from,
                to_person_id: to,
                relationship: inverse_relationship(&edge.relationship),
            });
        }
    }
    None
}

fn has_person(tree: &FamilyTree, person_id: u64) -> bool {
    for person in tree.members.iter() {
        if person.person_id == person_id {
            return true;
        }
    }
    false
}

fn contains_id(ids: &Vec<u64>, id: u64) -> bool {
    for existing in ids.iter() {
        if existing == id {
            return true;
        }
    }
    false
}

fn similarity(env: &Env, a: BytesN<32>, b: BytesN<32>) -> Result<u32, InheritanceError> {
    GeneticVerificationContract::calculate_genetic_similarity(env, a, b)
        .map_err(|_| InheritanceError::InvalidBeneficiaryData)
}

fn infer_relationship_from_similarity(score: u32) -> RelationshipType {
    if score >= 45 {
        RelationshipType::Parent
    } else if score >= 40 {
        RelationshipType::Sibling
    } else if score >= 20 {
        RelationshipType::HalfSibling
    } else if score >= 10 {
        RelationshipType::Cousin
    } else if score >= 5 {
        RelationshipType::SecondCousin
    } else {
        RelationshipType::StepRelative
    }
}

pub fn build_family_tree_internal(
    env: &Env,
    root_person: Address,
    initial_relatives: Vec<RelativeInput>,
) -> Result<u64, InheritanceError> {
    if initial_relatives.len() > MAX_INITIAL_RELATIVES {
        return Err(InheritanceError::TooManyBeneficiaries);
    }

    let tree_id = next_tree_id(env);
    let created_at = env.ledger().timestamp();
    let mut members = Vec::new(env);
    let mut relationships = Vec::new(env);
    let empty_documents = Vec::new(env);
    let root_hash = BytesN::from_array(env, &[0u8; 32]);

    members.push_back(PersonRecord {
        person_id: 1,
        address: root_person,
        dna_hash: root_hash,
        supporting_documents: empty_documents,
    });

    let mut next_person_id = 2u64;
    for relative in initial_relatives.iter() {
        if is_zero_hash(env, &relative.dna_hash) {
            return Err(InheritanceError::InvalidBeneficiaryData);
        }

        members.push_back(PersonRecord {
            person_id: next_person_id,
            address: relative.address,
            dna_hash: relative.dna_hash,
            supporting_documents: relative.supporting_documents,
        });
        relationships.push_back(RelationshipEdge {
            from_person_id: 1,
            to_person_id: next_person_id,
            relationship: relative.claimed_relationship,
            confidence: 80,
            verified: false,
        });
        next_person_id = next_person_id.saturating_add(1);
    }

    let tree = FamilyTree {
        tree_id,
        root_person_id: 1,
        members,
        relationships,
        created_at,
        updated_at: created_at,
    };
    store_tree(env, &tree);
    Ok(tree_id)
}

pub fn discover_relatives_internal(
    env: &Env,
    person_dna_hash: BytesN<32>,
    search_radius: u32,
) -> Result<Vec<DiscoveredRelative>, InheritanceError> {
    if search_radius == 0 || search_radius > MAX_SEARCH_RADIUS {
        return Err(InheritanceError::InvalidAllocation);
    }
    if is_zero_hash(env, &person_dna_hash) {
        return Err(InheritanceError::InvalidBeneficiaryData);
    }

    let mut discovered = Vec::new(env);
    let ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::PlanMetadata(0, ALL_TREE_IDS_KEY))
        .unwrap_or(Vec::new(env));

    for tree_id in ids.iter() {
        let tree = load_tree(env, tree_id)?;
        for person in tree.members.iter() {
            if is_zero_hash(env, &person.dna_hash) || person.dna_hash == person_dna_hash {
                continue;
            }
            let score = similarity(env, person_dna_hash.clone(), person.dna_hash.clone())?;
            let relationship_type = infer_relationship_from_similarity(score);
            let degree = relationship_degree(&relationship_type);
            if degree <= search_radius && score >= 5 {
                discovered.push_back(DiscoveredRelative {
                    person_id: person.person_id,
                    relationship_type,
                    genetic_confidence: score,
                    relationship_degree: degree,
                    discovery_method: DiscoveryMethod::GeneticMatching,
                    requires_confirmation: score < 40,
                });
            }
        }
    }

    Ok(discovered)
}

pub fn scan_genetic_database(
    env: &Env,
    target_dna_hash: BytesN<32>,
    similarity_threshold: u32,
) -> Result<Vec<GeneticMatch>, InheritanceError> {
    if similarity_threshold > 100 {
        return Err(InheritanceError::InvalidAllocation);
    }
    if is_zero_hash(env, &target_dna_hash) {
        return Err(InheritanceError::InvalidBeneficiaryData);
    }

    let mut matches = Vec::new(env);
    let ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::PlanMetadata(0, ALL_TREE_IDS_KEY))
        .unwrap_or(Vec::new(env));

    for tree_id in ids.iter() {
        let tree = load_tree(env, tree_id)?;
        for person in tree.members.iter() {
            if is_zero_hash(env, &person.dna_hash) || person.dna_hash == target_dna_hash {
                continue;
            }
            let score = similarity(env, target_dna_hash.clone(), person.dna_hash.clone())?;
            if score >= similarity_threshold {
                matches.push_back(GeneticMatch {
                    person_id: person.person_id,
                    dna_hash: person.dna_hash,
                    similarity: score,
                });
            }
        }
    }

    Ok(matches)
}

pub fn cross_reference_documents(
    env: &Env,
    person_documents: Vec<BytesN<32>>,
    database_documents: Vec<StoredDocument>,
) -> Result<Vec<DocumentMatch>, InheritanceError> {
    let mut matches = Vec::new(env);

    for document in person_documents.iter() {
        for stored in database_documents.iter() {
            if document == stored.document_hash {
                let confidence = match stored.document_type {
                    DocumentType::BirthCertificate | DocumentType::AdoptionRecord => 90,
                    DocumentType::MarriageCertificate | DocumentType::CourtOrder => 85,
                    DocumentType::DeathCertificate => 70,
                };
                matches.push_back(DocumentMatch {
                    document_hash: document.clone(),
                    confidence,
                });
            }
        }
    }

    Ok(matches)
}

pub fn verify_relationship_degree_internal(
    env: &Env,
    person1_hash: BytesN<32>,
    person2_hash: BytesN<32>,
    claimed_degree: u32,
) -> Result<VerificationResult, InheritanceError> {
    if claimed_degree == 0 || claimed_degree > MAX_SEARCH_RADIUS {
        return Err(InheritanceError::InvalidAllocation);
    }
    if is_zero_hash(env, &person1_hash) || is_zero_hash(env, &person2_hash) {
        return Err(InheritanceError::InvalidBeneficiaryData);
    }

    let genetic_similarity = similarity(env, person1_hash, person2_hash)?;
    let relationship = infer_relationship_from_similarity(genetic_similarity);
    let actual_degree = relationship_degree(&relationship);
    let (min, max) = get_relationship_threshold(&relationship);
    let confidence = if genetic_similarity >= min && genetic_similarity <= max {
        95
    } else if genetic_similarity >= min.saturating_sub(5) {
        75
    } else {
        genetic_similarity
    };

    Ok(VerificationResult {
        is_verified: actual_degree == claimed_degree && confidence >= 75,
        actual_degree,
        confidence,
        genetic_similarity,
    })
}

pub fn get_relationship_threshold(relationship: &RelationshipType) -> (u32, u32) {
    match relationship {
        RelationshipType::Parent | RelationshipType::Child => (45, 55),
        RelationshipType::Sibling => (40, 60),
        RelationshipType::HalfSibling => (20, 30),
        RelationshipType::Grandparent | RelationshipType::Grandchild => (20, 30),
        RelationshipType::Uncle
        | RelationshipType::Aunt
        | RelationshipType::Nephew
        | RelationshipType::Niece => (20, 30),
        RelationshipType::Cousin => (10, 18),
        RelationshipType::SecondCousin => (5, 10),
        RelationshipType::Spouse | RelationshipType::StepRelative => (0, 100),
    }
}

pub fn merge_family_trees_internal(
    env: &Env,
    tree1_id: u64,
    tree2_id: u64,
    merge_point: MergePoint,
) -> Result<u64, InheritanceError> {
    if tree1_id == tree2_id || merge_point.confidence < MIN_MERGE_CONFIDENCE {
        return Err(InheritanceError::InvalidAllocation);
    }

    let tree1 = load_tree(env, tree1_id)?;
    let tree2 = load_tree(env, tree2_id)?;
    if !has_person(&tree1, merge_point.person1_tree1)
        || !has_person(&tree2, merge_point.person2_tree2)
    {
        return Err(InheritanceError::BeneficiaryNotFound);
    }

    let merged_tree_id = next_tree_id(env);
    let mut members = Vec::new(env);
    let mut relationships = Vec::new(env);
    let mut id_map: Vec<(u64, u64)> = Vec::new(env);
    let mut next_person_id = 1u64;

    for person in tree1.members.iter() {
        members.push_back(PersonRecord {
            person_id: next_person_id,
            address: person.address,
            dna_hash: person.dna_hash,
            supporting_documents: person.supporting_documents,
        });
        id_map.push_back((person.person_id, next_person_id));
        next_person_id = next_person_id.saturating_add(1);
    }

    let tree2_offset = 1_000_000u64;
    for person in tree2.members.iter() {
        members.push_back(PersonRecord {
            person_id: next_person_id,
            address: person.address,
            dna_hash: person.dna_hash,
            supporting_documents: person.supporting_documents,
        });
        id_map.push_back((tree2_offset + person.person_id, next_person_id));
        next_person_id = next_person_id.saturating_add(1);
    }

    for edge in tree1.relationships.iter() {
        relationships.push_back(RelationshipEdge {
            from_person_id: remap_id(&id_map, edge.from_person_id)?,
            to_person_id: remap_id(&id_map, edge.to_person_id)?,
            relationship: edge.relationship,
            confidence: edge.confidence,
            verified: edge.verified,
        });
    }
    for edge in tree2.relationships.iter() {
        relationships.push_back(RelationshipEdge {
            from_person_id: remap_id(&id_map, tree2_offset + edge.from_person_id)?,
            to_person_id: remap_id(&id_map, tree2_offset + edge.to_person_id)?,
            relationship: edge.relationship,
            confidence: edge.confidence,
            verified: edge.verified,
        });
    }

    relationships.push_back(RelationshipEdge {
        from_person_id: remap_id(&id_map, merge_point.person1_tree1)?,
        to_person_id: remap_id(&id_map, tree2_offset + merge_point.person2_tree2)?,
        relationship: merge_point.relationship,
        confidence: merge_point.confidence,
        verified: merge_point.confidence >= 80,
    });

    let now = env.ledger().timestamp();
    let tree = FamilyTree {
        tree_id: merged_tree_id,
        root_person_id: 1,
        members,
        relationships,
        created_at: now,
        updated_at: now,
    };
    store_tree(env, &tree);
    Ok(merged_tree_id)
}

fn remap_id(id_map: &Vec<(u64, u64)>, old_id: u64) -> Result<u64, InheritanceError> {
    for item in id_map.iter() {
        if item.0 == old_id {
            return Ok(item.1);
        }
    }
    Err(InheritanceError::BeneficiaryNotFound)
}

pub fn resolve_tree_conflicts(
    env: &Env,
    tree_id: u64,
    conflicts: Vec<TreeConflict>,
) -> Result<Vec<Resolution>, InheritanceError> {
    let _tree = load_tree(env, tree_id)?;
    let mut resolutions = Vec::new(env);
    let mut conflict_id = 1u64;

    for conflict in conflicts.iter() {
        let resolution_action = match conflict.conflict_type {
            ConflictType::DuplicatePerson => ResolutionAction::MergeRecords,
            ConflictType::InconsistentRelationship => ResolutionAction::ManualReview,
            ConflictType::MissingParent => ResolutionAction::PreferTree1,
            ConflictType::CircularReference => ResolutionAction::ManualReview,
        };
        resolutions.push_back(Resolution {
            conflict_id,
            resolution_action,
            resolved_at: env.ledger().timestamp(),
        });
        conflict_id = conflict_id.saturating_add(1);
    }

    Ok(resolutions)
}

pub fn calculate_inheritance_rights_internal(
    env: &Env,
    tree_id: u64,
    deceased_person_id: u64,
) -> Result<Vec<InheritanceRight>, InheritanceError> {
    let tree = load_tree(env, tree_id)?;
    if !has_person(&tree, deceased_person_id) {
        return Err(InheritanceError::BeneficiaryNotFound);
    }

    let people = traverse_tree_breadth_first(env, &tree, deceased_person_id, 4)?;
    let mut rights = Vec::new(env);
    let mut eligible_count = 0u32;
    for person in people.iter() {
        if person.depth > 0 && person.depth <= 4 {
            eligible_count = eligible_count.saturating_add(1);
        }
    }
    if eligible_count == 0 {
        return Ok(rights);
    }

    let share_bp = 10000u32 / eligible_count;
    for person in people.iter() {
        if person.depth > 0 && person.depth <= 4 {
            rights.push_back(InheritanceRight {
                person_id: person.person_id,
                relationship_degree: person.depth,
                share_bp,
                priority: person.depth,
            });
        }
    }

    Ok(rights)
}

pub fn find_inheritance_path_internal(
    env: &Env,
    tree_id: u64,
    deceased_person: u64,
    potential_heir: u64,
) -> Result<InheritancePath, InheritanceError> {
    let tree = load_tree(env, tree_id)?;
    if !has_person(&tree, deceased_person) || !has_person(&tree, potential_heir) {
        return Err(InheritanceError::BeneficiaryNotFound);
    }
    if deceased_person == potential_heir {
        return Ok(InheritancePath {
            total_degree: 0,
            is_valid: false,
            path_steps: Vec::new(env),
        });
    }

    let mut queue = Vec::new(env);
    let mut visited = Vec::new(env);
    queue.push_back(PathCandidate {
        person_id: deceased_person,
        depth: 0,
        steps: Vec::new(env),
    });
    visited.push_back(deceased_person);

    let mut index = 0u32;
    while index < queue.len() {
        let candidate = queue.get(index).ok_or(InheritanceError::PlanNotFound)?;
        if candidate.depth >= MAX_SEARCH_RADIUS {
            index = index.saturating_add(1);
            continue;
        }

        for edge in tree.relationships.iter() {
            let neighbor = if edge.from_person_id == candidate.person_id {
                edge.to_person_id
            } else if edge.to_person_id == candidate.person_id {
                edge.from_person_id
            } else {
                continue;
            };
            if contains_id(&visited, neighbor) {
                continue;
            }

            let step = edge_between(&tree, candidate.person_id, neighbor)
                .ok_or(InheritanceError::PlanNotFound)?;
            let mut steps = candidate.steps.clone();
            steps.push_back(step);
            let depth = candidate.depth.saturating_add(1);
            if neighbor == potential_heir {
                return Ok(InheritancePath {
                    total_degree: depth,
                    is_valid: depth <= 4,
                    path_steps: steps,
                });
            }
            visited.push_back(neighbor);
            queue.push_back(PathCandidate {
                person_id: neighbor,
                depth,
                steps,
            });
        }
        index = index.saturating_add(1);
    }

    Ok(InheritancePath {
        total_degree: 0,
        is_valid: false,
        path_steps: Vec::new(env),
    })
}

pub fn traverse_tree_breadth_first(
    env: &Env,
    tree: &FamilyTree,
    start_person: u64,
    max_depth: u32,
) -> Result<Vec<PersonAtDepth>, InheritanceError> {
    if max_depth == 0 || max_depth > MAX_SEARCH_RADIUS {
        return Err(InheritanceError::InvalidAllocation);
    }
    if !has_person(tree, start_person) {
        return Err(InheritanceError::BeneficiaryNotFound);
    }

    let mut people = Vec::new(env);
    let mut visited = Vec::new(env);
    people.push_back(PersonAtDepth {
        person_id: start_person,
        depth: 0,
    });
    visited.push_back(start_person);

    let mut index = 0u32;
    while index < people.len() {
        let current = people.get(index).ok_or(InheritanceError::PlanNotFound)?;
        if current.depth >= max_depth {
            index = index.saturating_add(1);
            continue;
        }

        for edge in tree.relationships.iter() {
            let neighbor = if edge.from_person_id == current.person_id {
                edge.to_person_id
            } else if edge.to_person_id == current.person_id {
                edge.from_person_id
            } else {
                continue;
            };
            if !contains_id(&visited, neighbor) {
                visited.push_back(neighbor);
                people.push_back(PersonAtDepth {
                    person_id: neighbor,
                    depth: current.depth.saturating_add(1),
                });
            }
        }
        index = index.saturating_add(1);
    }

    Ok(people)
}

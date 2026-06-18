use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Enums ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    Public,
    Protected,
    Private,
    Medical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariantSignificance {
    Benign,
    LikelyBenign,
    Uncertain,
    LikelyPathogenic,
    Pathogenic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    Parent,
    Child,
    Sibling,
    Grandparent,
    Grandchild,
    Spouse,
    Other,
}

// ─── Core DNA Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNPVariant {
    pub rsid: String,
    pub chromosome: u8,
    pub position: u64,
    pub genotype: String,
    pub significance: VariantSignificance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticMarker {
    pub marker_id: String,
    pub marker_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMarker {
    pub rsid: String,
    pub condition: String,
    pub risk_allele: String,
    pub carrier_status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AncestryBreakdown {
    pub populations: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedDNAData {
    pub profile_id: String,
    pub genetic_markers: HashMap<String, GeneticMarker>,
    pub snp_data: Vec<SNPVariant>,
    pub ancestry_composition: AncestryBreakdown,
    pub health_markers: Vec<HealthMarker>,
    pub privacy_level: PrivacyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DNAProfile {
    pub profile_id: String,
    pub snp_data: Vec<SNPVariant>,
    pub genetic_markers: HashMap<String, GeneticMarker>,
}

impl From<&ProcessedDNAData> for DNAProfile {
    fn from(data: &ProcessedDNAData) -> Self {
        Self {
            profile_id: data.profile_id.clone(),
            snp_data: data.snp_data.clone(),
            genetic_markers: data.genetic_markers.clone(),
        }
    }
}

// ─── Health & Risk Types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCondition {
    pub condition_name: String,
    pub risk_level: RiskLevel,
    pub confidence: f64,
    pub genetic_variants: Vec<String>,
    pub age_of_onset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseRisk {
    pub condition_name: String,
    pub risk_level: RiskLevel,
    pub lifetime_risk: f64,
    pub population_risk: f64,
    pub relative_risk: f64,
    pub confidence_interval: (f64, f64),
    pub genetic_variants: Vec<String>,
    pub age_of_onset_prediction: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PGSScore {
    pub trait_name: String,
    pub score: f64,
    pub percentile: f64,
    pub effect_size: f64,
    pub contributing_variants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugResponse {
    pub drug_name: String,
    pub response_type: String,
    pub recommendation: String,
    pub relevant_variants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitPrediction {
    pub trait_name: String,
    pub predicted_value: String,
    pub confidence: f64,
    pub contributing_variants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifestyleRecommendation {
    pub category: String,
    pub recommendation: String,
    pub priority: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreeningRecommendation {
    pub condition: String,
    pub screening_type: String,
    pub recommended_age: u32,
    pub frequency_years: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub condition_name: String,
    pub trigger_type: String,
    pub threshold: f64,
    pub current_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_health_score: f64,
    pub disease_risks: Vec<DiseaseRisk>,
    pub polygenic_scores: Vec<PGSScore>,
    pub lifestyle_recommendations: Vec<LifestyleRecommendation>,
    pub screening_recommendations: Vec<ScreeningRecommendation>,
    pub inheritance_trigger_conditions: Vec<TriggerCondition>,
}

// ─── Similarity Types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEstimate {
    pub most_likely_relationship: RelationshipType,
    pub confidence: f64,
    pub alternative_relationships: Vec<(RelationshipType, f64)>,
    pub shared_centimorgans: f64,
}

// ─── Database Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantInfo {
    pub rsid: String,
    pub significance: VariantSignificance,
    pub clinical_significance: String,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseAssociation {
    pub rsid: String,
    pub disease_name: String,
    pub odds_ratio: f64,
    pub p_value: f64,
}

// ─── Privacy Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateProfile {
    pub profile_id: String,
    pub hashed_markers: HashMap<String, String>,
    pub privacy_level: PrivacyLevel,
    pub redacted_snp_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureComparisonResult {
    pub similarity_score: f64,
    pub shared_marker_count: usize,
    pub comparison_valid: bool,
}

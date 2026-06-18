use super::database::{CompositeGeneticDatabaseClient, GeneticDatabaseClient};
use super::dna_processor::DNAProcessor;
use super::errors::GeneticError;
use super::health::HealthConditionAnalyzer;
use super::privacy::GeneticPrivacyEngine;
use super::similarity::GeneticSimilarityCalculator;
use super::types::*;
use std::sync::Arc;

/// Comprehensive genetic data processing and analysis service.
pub struct GeneticAnalysisService {
    pub dna_processor: DNAProcessor,
    pub health_analyzer: HealthConditionAnalyzer,
    pub similarity_calculator: GeneticSimilarityCalculator,
    pub external_db_client: Arc<dyn GeneticDatabaseClient>,
    pub privacy_engine: GeneticPrivacyEngine,
}

impl Default for GeneticAnalysisService {
    fn default() -> Self {
        Self::new()
    }
}

impl GeneticAnalysisService {
    pub fn new() -> Self {
        Self {
            dna_processor: DNAProcessor,
            health_analyzer: HealthConditionAnalyzer::new(),
            similarity_calculator: GeneticSimilarityCalculator,
            external_db_client: Arc::new(CompositeGeneticDatabaseClient::new()),
            privacy_engine: GeneticPrivacyEngine,
        }
    }

    pub fn with_db_client(client: Arc<dyn GeneticDatabaseClient>) -> Self {
        Self {
            dna_processor: DNAProcessor,
            health_analyzer: HealthConditionAnalyzer::new(),
            similarity_calculator: GeneticSimilarityCalculator,
            external_db_client: client,
            privacy_engine: GeneticPrivacyEngine,
        }
    }

    /// Process raw DNA data into a structured genetic profile.
    pub async fn process_raw_dna_data(
        &self,
        raw_data: Vec<u8>,
        privacy_level: PrivacyLevel,
    ) -> Result<ProcessedDNAData, GeneticError> {
        let mut processed = self.dna_processor.process(&raw_data, privacy_level)?;

        // Enrich SNP significance from external databases
        for snp in &mut processed.snp_data {
            if let Ok(variant_info) = self
                .external_db_client
                .query_variant_significance(&snp.rsid)
                .await
            {
                snp.significance = variant_info.significance;
            }
        }

        Ok(processed)
    }

    /// Calculate genetic similarity between two DNA profiles (IBS score 0.0–1.0).
    pub async fn calculate_genetic_similarity(
        &self,
        dna1: &DNAProfile,
        dna2: &DNAProfile,
    ) -> Result<f64, GeneticError> {
        if dna1.snp_data.is_empty() || dna2.snp_data.is_empty() {
            return Err(GeneticError::InsufficientData);
        }

        Ok(self
            .similarity_calculator
            .calculate_identity_by_state(dna1, dna2))
    }

    /// Detect health conditions from genetic markers in a DNA profile.
    pub async fn detect_health_conditions(
        &self,
        dna_profile: &DNAProfile,
    ) -> Result<Vec<HealthCondition>, GeneticError> {
        let disease_risks = self
            .health_analyzer
            .screen_for_diseases(dna_profile)
            .await?;

        let conditions = disease_risks
            .into_iter()
            .map(|risk| HealthCondition {
                condition_name: risk.condition_name,
                risk_level: risk.risk_level,
                confidence: 1.0 - (risk.confidence_interval.1 - risk.confidence_interval.0),
                genetic_variants: risk.genetic_variants,
                age_of_onset: risk.age_of_onset_prediction,
            })
            .collect();

        Ok(conditions)
    }

    /// Perform comprehensive genetic risk assessment for an individual.
    pub async fn assess_genetic_risks(
        &self,
        dna_profile: &DNAProfile,
        age: u32,
    ) -> Result<RiskAssessment, GeneticError> {
        let disease_risks = self
            .health_analyzer
            .screen_for_diseases(dna_profile)
            .await?;
        let polygenic_scores = self
            .health_analyzer
            .calculate_polygenic_risk_scores(dna_profile)
            .await?;

        let overall_health_score = Self::compute_overall_health_score(&disease_risks, age);
        let lifestyle_recommendations = Self::generate_lifestyle_recommendations(&disease_risks);
        let screening_recommendations =
            Self::generate_screening_recommendations(&disease_risks, age);
        let inheritance_trigger_conditions =
            Self::identify_inheritance_triggers(&disease_risks, &polygenic_scores);

        Ok(RiskAssessment {
            overall_health_score,
            disease_risks,
            polygenic_scores,
            lifestyle_recommendations,
            screening_recommendations,
            inheritance_trigger_conditions,
        })
    }

    /// Calculate detailed relationship estimate between two profiles.
    pub async fn estimate_relationship(
        &self,
        dna1: &DNAProfile,
        dna2: &DNAProfile,
    ) -> Result<RelationshipEstimate, GeneticError> {
        if dna1.snp_data.is_empty() || dna2.snp_data.is_empty() {
            return Err(GeneticError::InsufficientData);
        }

        let ibs = self
            .similarity_calculator
            .calculate_identity_by_state(dna1, dna2);
        let cm = self
            .similarity_calculator
            .calculate_centimorgan_sharing(dna1, dna2);
        let mut estimate = self
            .similarity_calculator
            .estimate_relationship_coefficient(ibs);
        estimate.shared_centimorgans = cm;
        Ok(estimate)
    }

    /// Enrich a processed profile with external database annotations.
    pub async fn enrich_with_external_data(
        &self,
        processed: &ProcessedDNAData,
    ) -> Result<Vec<DiseaseAssociation>, GeneticError> {
        let rsids: Vec<String> = processed.snp_data.iter().map(|s| s.rsid.clone()).collect();
        Ok(self
            .external_db_client
            .lookup_disease_associations(&rsids)
            .await?)
    }

    fn compute_overall_health_score(disease_risks: &[DiseaseRisk], age: u32) -> f64 {
        if disease_risks.is_empty() {
            return 85.0;
        }

        let risk_penalty: f64 = disease_risks
            .iter()
            .map(|r| match r.risk_level {
                RiskLevel::Critical => 25.0,
                RiskLevel::High => 15.0,
                RiskLevel::Medium => 8.0,
                RiskLevel::Low => 3.0,
            })
            .sum();

        let age_factor = if age > 65 { 0.9 } else { 1.0 };
        (100.0 - risk_penalty).clamp(0.0, 100.0) * age_factor
    }

    fn generate_lifestyle_recommendations(
        disease_risks: &[DiseaseRisk],
    ) -> Vec<LifestyleRecommendation> {
        let mut recommendations = Vec::new();

        for risk in disease_risks {
            match risk.condition_name.as_str() {
                name if name.contains("Diabetes") => {
                    recommendations.push(LifestyleRecommendation {
                        category: "Nutrition".into(),
                        recommendation: "Maintain low glycemic diet and regular exercise".into(),
                        priority: risk.risk_level,
                    });
                }
                name if name.contains("Coronary") => {
                    recommendations.push(LifestyleRecommendation {
                        category: "Cardiovascular".into(),
                        recommendation:
                            "Heart-healthy diet, regular aerobic exercise, avoid smoking".into(),
                        priority: risk.risk_level,
                    });
                }
                name if name.contains("Alzheimer") => {
                    recommendations.push(LifestyleRecommendation {
                        category: "Cognitive Health".into(),
                        recommendation: "Mental stimulation, social engagement, Mediterranean diet"
                            .into(),
                        priority: risk.risk_level,
                    });
                }
                _ => {}
            }
        }

        if recommendations.is_empty() {
            recommendations.push(LifestyleRecommendation {
                category: "General".into(),
                recommendation: "Maintain balanced diet and regular physical activity".into(),
                priority: RiskLevel::Low,
            });
        }

        recommendations
    }

    fn generate_screening_recommendations(
        disease_risks: &[DiseaseRisk],
        age: u32,
    ) -> Vec<ScreeningRecommendation> {
        disease_risks
            .iter()
            .filter_map(|risk| {
                risk.age_of_onset_prediction
                    .map(|onset| ScreeningRecommendation {
                        condition: risk.condition_name.clone(),
                        screening_type: format!("Genetic screening for {}", risk.condition_name),
                        recommended_age: onset.saturating_sub(10).max(age),
                        frequency_years: match risk.risk_level {
                            RiskLevel::Critical | RiskLevel::High => 1,
                            RiskLevel::Medium => 2,
                            RiskLevel::Low => 5,
                        },
                    })
            })
            .collect()
    }

    fn identify_inheritance_triggers(
        disease_risks: &[DiseaseRisk],
        pgs_scores: &[PGSScore],
    ) -> Vec<TriggerCondition> {
        let mut triggers = Vec::new();

        for risk in disease_risks {
            if matches!(risk.risk_level, RiskLevel::Critical | RiskLevel::High) {
                triggers.push(TriggerCondition {
                    condition_name: risk.condition_name.clone(),
                    trigger_type: "health_condition_detected".into(),
                    threshold: risk.population_risk * 2.0,
                    current_value: risk.lifetime_risk,
                });
            }
        }

        for pgs in pgs_scores {
            if pgs.percentile >= 90.0 {
                triggers.push(TriggerCondition {
                    condition_name: pgs.trait_name.clone(),
                    trigger_type: "risk_factor_exceeded".into(),
                    threshold: 90.0,
                    current_value: pgs.percentile,
                });
            }
        }

        triggers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_dna_text() -> Vec<u8> {
        b"rs429358\t19\t45411941\tCC\n\
          rs7412\t19\t45412079\tCT\n\
          rs1801133\t1\t11796321\tTT\n\
          rs7903146\t10\t114758349\tTT\n\
          rs1333049\t9\t22125503\tCC\n"
            .to_vec()
    }

    fn sibling_dna_text() -> Vec<u8> {
        b"rs429358\t19\t45411941\tCT\n\
          rs7412\t19\t45412079\tCT\n\
          rs1801133\t1\t11796321\tCT\n\
          rs7903146\t10\t114758349\tTC\n\
          rs1333049\t9\t22125503\tCG\n"
            .to_vec()
    }

    #[tokio::test]
    async fn test_process_raw_dna_data() {
        let service = GeneticAnalysisService::new();
        let result = service
            .process_raw_dna_data(synthetic_dna_text(), PrivacyLevel::Protected)
            .await
            .unwrap();

        assert_eq!(result.snp_data.len(), 5);
        assert!(!result.health_markers.is_empty());
        assert_eq!(result.privacy_level, PrivacyLevel::Protected);
    }

    #[tokio::test]
    async fn test_calculate_genetic_similarity() {
        let service = GeneticAnalysisService::new();
        let data1 = service
            .process_raw_dna_data(synthetic_dna_text(), PrivacyLevel::Public)
            .await
            .unwrap();
        let data2 = service
            .process_raw_dna_data(synthetic_dna_text(), PrivacyLevel::Public)
            .await
            .unwrap();

        let profile1 = DNAProfile::from(&data1);
        let profile2 = DNAProfile::from(&data2);

        let similarity = service
            .calculate_genetic_similarity(&profile1, &profile2)
            .await
            .unwrap();
        assert!((similarity - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_detect_health_conditions() {
        let service = GeneticAnalysisService::new();
        let data = service
            .process_raw_dna_data(synthetic_dna_text(), PrivacyLevel::Protected)
            .await
            .unwrap();
        let profile = DNAProfile::from(&data);

        let conditions = service.detect_health_conditions(&profile).await.unwrap();
        assert!(!conditions.is_empty());
    }

    #[tokio::test]
    async fn test_assess_genetic_risks() {
        let service = GeneticAnalysisService::new();
        let data = service
            .process_raw_dna_data(synthetic_dna_text(), PrivacyLevel::Private)
            .await
            .unwrap();
        let profile = DNAProfile::from(&data);

        let assessment = service.assess_genetic_risks(&profile, 45).await.unwrap();
        assert!(assessment.overall_health_score >= 0.0 && assessment.overall_health_score <= 100.0);
        assert!(!assessment.polygenic_scores.is_empty());
    }

    #[tokio::test]
    async fn test_partial_similarity_between_profiles() {
        let service = GeneticAnalysisService::new();
        let data1 = service
            .process_raw_dna_data(synthetic_dna_text(), PrivacyLevel::Public)
            .await
            .unwrap();
        let data2 = service
            .process_raw_dna_data(sibling_dna_text(), PrivacyLevel::Public)
            .await
            .unwrap();

        let profile1 = DNAProfile::from(&data1);
        let profile2 = DNAProfile::from(&data2);

        let similarity = service
            .calculate_genetic_similarity(&profile1, &profile2)
            .await
            .unwrap();
        assert!(similarity > 0.0 && similarity < 1.0);
    }

    #[tokio::test]
    async fn test_enrich_with_external_data() {
        let service = GeneticAnalysisService::new();
        let data = service
            .process_raw_dna_data(synthetic_dna_text(), PrivacyLevel::Public)
            .await
            .unwrap();

        let associations = service.enrich_with_external_data(&data).await.unwrap();
        assert!(!associations.is_empty());
    }

    #[tokio::test]
    async fn test_privacy_engine_integration() {
        let service = GeneticAnalysisService::new();
        let data = service
            .process_raw_dna_data(synthetic_dna_text(), PrivacyLevel::Private)
            .await
            .unwrap();

        let private = service
            .privacy_engine
            .create_privacy_preserving_profile(&data, PrivacyLevel::Private);
        assert!(private.redacted_snp_count > 0);
    }
}

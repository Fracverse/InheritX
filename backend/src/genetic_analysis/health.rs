use super::errors::AnalysisError;
use super::types::*;
use std::collections::HashMap;

struct DiseaseMarkerDef {
    condition: &'static str,
    risk_allele: &'static str,
    population_risk: f64,
    odds_ratio: f64,
    age_of_onset: Option<u32>,
}

struct PGSDefinition {
    trait_name: &'static str,
    variants: Vec<(&'static str, f64)>, // (rsid, weight)
    population_mean: f64,
    population_std: f64,
}

struct PharmacogenomicsEntry {
    drug: &'static str,
    rsid: &'static str,
    response_allele: &'static str,
    response_type: &'static str,
    recommendation: &'static str,
}

/// Health condition detection and risk scoring engine.
pub struct HealthConditionAnalyzer {
    disease_markers: HashMap<String, DiseaseMarkerDef>,
    pgs_definitions: Vec<PGSDefinition>,
    pharmacogenomics: Vec<PharmacogenomicsEntry>,
}

impl Default for HealthConditionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthConditionAnalyzer {
    pub fn new() -> Self {
        let disease_markers: HashMap<String, DiseaseMarkerDef> = HashMap::from([
            (
                "rs429358".into(),
                DiseaseMarkerDef {
                    condition: "Alzheimer's Disease",
                    risk_allele: "C",
                    population_risk: 0.10,
                    odds_ratio: 3.5,
                    age_of_onset: Some(65),
                },
            ),
            (
                "rs7412".into(),
                DiseaseMarkerDef {
                    condition: "Alzheimer's Disease",
                    risk_allele: "C",
                    population_risk: 0.10,
                    odds_ratio: 0.5,
                    age_of_onset: Some(65),
                },
            ),
            (
                "rs1801133".into(),
                DiseaseMarkerDef {
                    condition: "MTHFR Deficiency",
                    risk_allele: "T",
                    population_risk: 0.12,
                    odds_ratio: 2.0,
                    age_of_onset: None,
                },
            ),
            (
                "rs6025".into(),
                DiseaseMarkerDef {
                    condition: "Factor V Leiden Thrombophilia",
                    risk_allele: "A",
                    population_risk: 0.05,
                    odds_ratio: 5.0,
                    age_of_onset: Some(30),
                },
            ),
            (
                "rs7903146".into(),
                DiseaseMarkerDef {
                    condition: "Type 2 Diabetes",
                    risk_allele: "T",
                    population_risk: 0.15,
                    odds_ratio: 1.4,
                    age_of_onset: Some(45),
                },
            ),
            (
                "rs1333049".into(),
                DiseaseMarkerDef {
                    condition: "Coronary Artery Disease",
                    risk_allele: "C",
                    population_risk: 0.20,
                    odds_ratio: 1.3,
                    age_of_onset: Some(55),
                },
            ),
        ]);

        let pgs_definitions = vec![
            PGSDefinition {
                trait_name: "Coronary Artery Disease",
                variants: vec![
                    ("rs1333049", 0.15),
                    ("rs4977574", 0.12),
                    ("rs10757274", 0.10),
                ],
                population_mean: 0.0,
                population_std: 1.0,
            },
            PGSDefinition {
                trait_name: "Type 2 Diabetes",
                variants: vec![("rs7903146", 0.20), ("rs1801282", 0.08), ("rs5219", 0.06)],
                population_mean: 0.0,
                population_std: 1.0,
            },
            PGSDefinition {
                trait_name: "Breast Cancer",
                variants: vec![("rs2981582", 0.12), ("rs3803662", 0.10), ("rs889312", 0.08)],
                population_mean: 0.0,
                population_std: 1.0,
            },
        ];

        let pharmacogenomics = vec![
            PharmacogenomicsEntry {
                drug: "Warfarin",
                rsid: "rs9923231",
                response_allele: "A",
                response_type: "Increased Sensitivity",
                recommendation: "Consider lower initial dose and INR monitoring",
            },
            PharmacogenomicsEntry {
                drug: "Clopidogrel",
                rsid: "rs4244285",
                response_allele: "A",
                response_type: "Reduced Efficacy",
                recommendation: "Consider alternative antiplatelet therapy",
            },
            PharmacogenomicsEntry {
                drug: "Codeine",
                rsid: "rs1065852",
                response_allele: "A",
                response_type: "Poor Metabolizer",
                recommendation: "Avoid codeine; use alternative analgesic",
            },
        ];

        Self {
            disease_markers,
            pgs_definitions,
            pharmacogenomics,
        }
    }

    pub async fn screen_for_diseases(
        &self,
        dna_profile: &DNAProfile,
    ) -> Result<Vec<DiseaseRisk>, AnalysisError> {
        if dna_profile.snp_data.is_empty() {
            return Err(AnalysisError::NoMarkers);
        }

        let snp_map: HashMap<&str, &SNPVariant> = dna_profile
            .snp_data
            .iter()
            .map(|s| (s.rsid.as_str(), s))
            .collect();

        let mut condition_risks: HashMap<String, DiseaseRisk> = HashMap::new();

        for (rsid, marker) in &self.disease_markers {
            if let Some(snp) = snp_map.get(rsid.as_str()) {
                let has_risk_allele = snp.genotype.contains(marker.risk_allele);
                if !has_risk_allele {
                    continue;
                }

                let lifetime_risk = (marker.population_risk * marker.odds_ratio).min(0.95);
                let relative_risk = marker.odds_ratio;
                let ci_low = (lifetime_risk * 0.8).max(0.0);
                let ci_high = (lifetime_risk * 1.2).min(1.0);

                let risk_level = Self::classify_risk(relative_risk);

                condition_risks
                    .entry(marker.condition.to_string())
                    .and_modify(|existing| {
                        existing.lifetime_risk = existing.lifetime_risk.max(lifetime_risk);
                        existing.relative_risk = existing.relative_risk.max(relative_risk);
                        existing.genetic_variants.push(rsid.clone());
                        existing.risk_level = Self::classify_risk(existing.relative_risk);
                    })
                    .or_insert(DiseaseRisk {
                        condition_name: marker.condition.to_string(),
                        risk_level,
                        lifetime_risk,
                        population_risk: marker.population_risk,
                        relative_risk,
                        confidence_interval: (ci_low, ci_high),
                        genetic_variants: vec![rsid.clone()],
                        age_of_onset_prediction: marker.age_of_onset,
                    });
            }
        }

        Ok(condition_risks.into_values().collect())
    }

    pub async fn calculate_polygenic_risk_scores(
        &self,
        dna_profile: &DNAProfile,
    ) -> Result<Vec<PGSScore>, AnalysisError> {
        if dna_profile.snp_data.is_empty() {
            return Err(AnalysisError::NoMarkers);
        }

        let snp_map: HashMap<&str, &SNPVariant> = dna_profile
            .snp_data
            .iter()
            .map(|s| (s.rsid.as_str(), s))
            .collect();

        let mut scores = Vec::new();

        for pgs in &self.pgs_definitions {
            let mut score = 0.0_f64;
            let mut contributing = Vec::new();

            for (rsid, weight) in &pgs.variants {
                if let Some(snp) = snp_map.get(rsid) {
                    let allele_count = snp.genotype.chars().filter(|c| *c != '-').count() as f64;
                    score += weight * allele_count;
                    contributing.push(rsid.to_string());
                }
            }

            let z_score = if pgs.population_std > 0.0 {
                (score - pgs.population_mean) / pgs.population_std
            } else {
                score
            };

            let percentile = Self::z_to_percentile(z_score);

            scores.push(PGSScore {
                trait_name: pgs.trait_name.to_string(),
                score,
                percentile,
                effect_size: z_score,
                contributing_variants: contributing,
            });
        }

        Ok(scores)
    }

    pub async fn analyze_drug_responses(
        &self,
        dna_profile: &DNAProfile,
    ) -> Result<Vec<DrugResponse>, AnalysisError> {
        let snp_map: HashMap<&str, &SNPVariant> = dna_profile
            .snp_data
            .iter()
            .map(|s| (s.rsid.as_str(), s))
            .collect();

        let responses: Vec<DrugResponse> = self
            .pharmacogenomics
            .iter()
            .filter_map(|entry| {
                snp_map.get(entry.rsid).and_then(|snp| {
                    if snp.genotype.contains(entry.response_allele) {
                        Some(DrugResponse {
                            drug_name: entry.drug.to_string(),
                            response_type: entry.response_type.to_string(),
                            recommendation: entry.recommendation.to_string(),
                            relevant_variants: vec![entry.rsid.to_string()],
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(responses)
    }

    pub async fn predict_trait_expressions(
        &self,
        dna_profile: &DNAProfile,
    ) -> Result<Vec<TraitPrediction>, AnalysisError> {
        let trait_markers: HashMap<&str, (&str, &str, &str)> = HashMap::from([
            ("rs6152", ("Eye Color", "Brown", "G")),
            ("rs12913832", ("Eye Color", "Blue", "G")),
            ("rs17822931", ("Hair Texture", "Straight", "T")),
            ("rs4988235", ("Lactose Tolerance", "Intolerant", "C")),
        ]);

        let snp_map: HashMap<&str, &SNPVariant> = dna_profile
            .snp_data
            .iter()
            .map(|s| (s.rsid.as_str(), s))
            .collect();

        let predictions: Vec<TraitPrediction> = trait_markers
            .iter()
            .filter_map(|(rsid, (trait_name, value, allele))| {
                snp_map.get(rsid).map(|snp| {
                    let matches = snp.genotype.contains(allele);
                    TraitPrediction {
                        trait_name: trait_name.to_string(),
                        predicted_value: if matches {
                            value.to_string()
                        } else {
                            "Other".to_string()
                        },
                        confidence: if matches { 0.75 } else { 0.55 },
                        contributing_variants: vec![rsid.to_string()],
                    }
                })
            })
            .collect();

        Ok(predictions)
    }

    fn classify_risk(relative_risk: f64) -> RiskLevel {
        if relative_risk >= 3.0 {
            RiskLevel::Critical
        } else if relative_risk >= 2.0 {
            RiskLevel::High
        } else if relative_risk >= 1.3 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    fn z_to_percentile(z: f64) -> f64 {
        // Approximate normal CDF using logistic function
        let percentile = 1.0 / (1.0 + (-1.702 * z).exp());
        (percentile * 100.0).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn profile_with_snps(snps: Vec<SNPVariant>) -> DNAProfile {
        DNAProfile {
            profile_id: "test".into(),
            snp_data: snps,
            genetic_markers: HashMap::new(),
        }
    }

    fn snp(rsid: &str, genotype: &str) -> SNPVariant {
        SNPVariant {
            rsid: rsid.into(),
            chromosome: 1,
            position: 100,
            genotype: genotype.into(),
            significance: VariantSignificance::Uncertain,
        }
    }

    #[tokio::test]
    async fn test_disease_screening_detects_apoe_risk() {
        let analyzer = HealthConditionAnalyzer::new();
        let profile = profile_with_snps(vec![snp("rs429358", "CC")]);
        let risks = analyzer.screen_for_diseases(&profile).await.unwrap();
        assert!(!risks.is_empty());
        assert!(risks.iter().any(|r| r.condition_name.contains("Alzheimer")));
    }

    #[tokio::test]
    async fn test_pgs_calculation() {
        let analyzer = HealthConditionAnalyzer::new();
        let profile = profile_with_snps(vec![snp("rs1333049", "CC"), snp("rs7903146", "TT")]);
        let scores = analyzer
            .calculate_polygenic_risk_scores(&profile)
            .await
            .unwrap();
        assert!(!scores.is_empty());
        assert!(scores[0].percentile >= 0.0 && scores[0].percentile <= 100.0);
    }

    #[tokio::test]
    async fn test_empty_profile_returns_error() {
        let analyzer = HealthConditionAnalyzer::new();
        let profile = profile_with_snps(vec![]);
        assert!(analyzer.screen_for_diseases(&profile).await.is_err());
    }
}

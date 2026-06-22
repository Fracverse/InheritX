use super::errors::PredictionError;
use super::types::*;
use std::collections::HashMap;

pub struct HealthPredictionModel {
    pub genetic_risk_models: Vec<GeneticRiskModel>,
    pub lifestyle_models: Vec<LifestyleModel>,
    pub environmental_models: Vec<EnvironmentalModel>,
    pub interaction_models: Vec<InteractionModel>,
}

impl Default for HealthPredictionModel {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthPredictionModel {
    pub fn new() -> Self {
        Self {
            genetic_risk_models: vec![
                GeneticRiskModel {
                    condition: "Type 2 Diabetes".into(),
                    variants: vec!["rs7903146".into(), "rs1801282".into()],
                    weights: vec![0.20, 0.08],
                    baseline_risk: 0.10,
                },
                GeneticRiskModel {
                    condition: "Coronary Artery Disease".into(),
                    variants: vec!["rs1333049".into(), "rs4977574".into()],
                    weights: vec![0.15, 0.12],
                    baseline_risk: 0.15,
                },
                GeneticRiskModel {
                    condition: "Alzheimer's Disease".into(),
                    variants: vec!["rs429358".into(), "rs7412".into()],
                    weights: vec![0.35, 0.15],
                    baseline_risk: 0.08,
                },
                GeneticRiskModel {
                    condition: "Breast Cancer".into(),
                    variants: vec!["rs2981582".into(), "rs3803662".into()],
                    weights: vec![0.12, 0.10],
                    baseline_risk: 0.07,
                },
            ],
            lifestyle_models: vec![
                LifestyleModel {
                    factor_name: "Smoking".into(),
                    effect_size: 2.5,
                    direction: "increases".into(),
                },
                LifestyleModel {
                    factor_name: "Sedentary Lifestyle".into(),
                    effect_size: 1.8,
                    direction: "increases".into(),
                },
                LifestyleModel {
                    factor_name: "Mediterranean Diet".into(),
                    effect_size: 0.7,
                    direction: "decreases".into(),
                },
                LifestyleModel {
                    factor_name: "High Stress".into(),
                    effect_size: 1.4,
                    direction: "increases".into(),
                },
            ],
            environmental_models: vec![
                EnvironmentalModel {
                    factor_name: "Air Pollution".into(),
                    risk_multiplier: 1.3,
                    exposure_threshold: 50.0,
                },
                EnvironmentalModel {
                    factor_name: "Occupational Hazards".into(),
                    risk_multiplier: 1.5,
                    exposure_threshold: 1.0,
                },
            ],
            interaction_models: vec![InteractionModel {
                model_name: "Gene-Lifestyle Interaction".into(),
                gene_environment_interactions: vec![
                    ("rs7903146".into(), "Sedentary Lifestyle".into(), 1.6),
                    ("rs429358".into(), "High Stress".into(), 1.4),
                ],
            }],
        }
    }

    pub fn predict_disease_onset(
        &self,
        genetic_profile: &GeneticProfile,
        lifestyle_factors: &LifestyleFactors,
        environmental_factors: &EnvironmentalFactors,
    ) -> Result<DiseaseOnsetPrediction, PredictionError> {
        if genetic_profile.genetic_mutations.is_empty() {
            return Err(PredictionError::InsufficientData(
                "No genetic data for prediction".into(),
            ));
        }

        let mut predictions: Vec<(String, f64, Vec<String>)> = Vec::new();

        for model in &self.genetic_risk_models {
            let mut genetic_risk = model.baseline_risk;

            for (i, variant) in model.variants.iter().enumerate() {
                if let Some(predisposition) = genetic_profile.genetic_mutations.get(variant) {
                    genetic_risk += predisposition * model.weights[i];
                }
            }

            let lifestyle_multiplier =
                self.compute_lifestyle_multiplier(lifestyle_factors, &model.condition);

            let environmental_multiplier =
                self.compute_environmental_multiplier(environmental_factors, &model.condition);

            let interaction_boost = self.compute_interaction_boost(
                genetic_profile,
                lifestyle_factors,
                &model.condition,
            );

            let final_risk = (genetic_risk * lifestyle_multiplier * environmental_multiplier
                + interaction_boost)
                .min(0.95);

            let mut risk_factors = Vec::new();
            if lifestyle_factors.smoking_status {
                risk_factors.push("Smoking".into());
            }
            if lifestyle_factors.stress_level > 0.7 {
                risk_factors.push("High stress".into());
            }
            if genetic_risk > model.baseline_risk * 1.5 {
                risk_factors.push("Elevated genetic risk".into());
            }

            predictions.push((model.condition.clone(), final_risk, risk_factors));
        }

        predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if predictions.is_empty() {
            return Err(PredictionError::ModelError(
                "No predictions could be generated".into(),
            ));
        }

        let (top_condition, top_risk, top_factors) = predictions.remove(0);

        let onset_age = self.estimate_onset_age(&top_condition, top_risk);

        let mut prob_by_age = Vec::new();
        for age in (30..=90).step_by(10) {
            let age_factor = if age >= onset_age.unwrap_or(60) as i32 {
                1.0 + ((age - onset_age.unwrap_or(60) as i32) as f64 * 0.05)
            } else {
                1.0
            };
            prob_by_age.push((age as u32, (top_risk * age_factor).min(0.95)));
        }

        let ci_low = (top_risk * 0.7).max(0.0);
        let ci_high = (top_risk * 1.3).min(1.0);

        Ok(DiseaseOnsetPrediction {
            condition_name: top_condition,
            predicted_onset_age: onset_age,
            probability_by_age: prob_by_age,
            confidence_interval: (ci_low, ci_high),
            key_risk_factors: top_factors,
        })
    }

    fn compute_lifestyle_multiplier(&self, lifestyle: &LifestyleFactors, _condition: &str) -> f64 {
        let mut multiplier = 1.0;

        if lifestyle.smoking_status {
            multiplier *= 2.5;
        }
        if lifestyle.exercise_frequency < 2.0 {
            multiplier *= 1.8;
        }
        if lifestyle.diet_quality < 0.4 {
            multiplier *= 1.3;
        } else if lifestyle.diet_quality > 0.8 {
            multiplier *= 0.7;
        }
        if lifestyle.alcohol_consumption > 3.0 {
            multiplier *= 1.3;
        }
        if lifestyle.stress_level > 0.8 {
            multiplier *= 1.4;
        }
        if lifestyle.sleep_hours < 6.0 {
            multiplier *= 1.2;
        }

        multiplier
    }

    fn compute_environmental_multiplier(
        &self,
        environmental: &EnvironmentalFactors,
        _condition: &str,
    ) -> f64 {
        let mut multiplier = 1.0;

        if let Some(aqi) = environmental.air_quality_index {
            if aqi > 50.0 {
                multiplier *= 1.3;
            }
        }
        if !environmental.occupational_hazards.is_empty() {
            multiplier *= 1.5;
        }
        if environmental.social_support_score < 0.3 {
            multiplier *= 1.2;
        }

        multiplier
    }

    fn compute_interaction_boost(
        &self,
        genetic_profile: &GeneticProfile,
        lifestyle: &LifestyleFactors,
        _condition: &str,
    ) -> f64 {
        let mut boost = 0.0;

        for model in &self.interaction_models {
            for (gene, env_factor, effect) in &model.gene_environment_interactions {
                let has_gene = genetic_profile
                    .genetic_mutations
                    .get(gene)
                    .copied()
                    .unwrap_or(0.0)
                    > 0.3;

                let has_env = match env_factor.as_str() {
                    "Sedentary Lifestyle" => lifestyle.exercise_frequency < 2.0,
                    "High Stress" => lifestyle.stress_level > 0.7,
                    _ => false,
                };

                if has_gene && has_env {
                    boost += effect;
                }
            }
        }

        boost
    }

    fn estimate_onset_age(&self, _condition: &str, risk: f64) -> Option<u32> {
        if risk < 0.1 {
            return None;
        }
        let base_age: u32 = match _condition {
            "Type 2 Diabetes" => 45,
            "Coronary Artery Disease" => 55,
            "Alzheimer's Disease" => 65,
            "Breast Cancer" => 50,
            _ => 60,
        };
        let reduction = ((risk - 0.1) * 20.0) as u32;
        Some(base_age.saturating_sub(reduction).max(20))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_genetic_profile() -> GeneticProfile {
        let mut mutations = HashMap::new();
        mutations.insert("rs7903146".into(), 0.75);
        mutations.insert("rs429358".into(), 0.45);

        GeneticProfile {
            profile_id: "test".into(),
            genetic_mutations: mutations,
            polygenic_risk_scores: HashMap::new(),
            carrier_status: HashMap::new(),
            pharmacogenomic_markers: HashMap::new(),
        }
    }

    fn sample_lifestyle() -> LifestyleFactors {
        LifestyleFactors {
            diet_quality: 0.6,
            exercise_frequency: 3.0,
            smoking_status: false,
            alcohol_consumption: 1.0,
            stress_level: 0.5,
            sleep_hours: 7.0,
        }
    }

    fn sample_environmental() -> EnvironmentalFactors {
        EnvironmentalFactors {
            air_quality_index: Some(30.0),
            occupational_hazards: vec![],
            geographical_risks: vec![],
            social_support_score: 0.8,
        }
    }

    #[test]
    fn test_predict_disease_onset() {
        let model = HealthPredictionModel::new();
        let profile = sample_genetic_profile();
        let lifestyle = sample_lifestyle();
        let environmental = sample_environmental();

        let prediction = model
            .predict_disease_onset(&profile, &lifestyle, &environmental)
            .unwrap();

        assert!(!prediction.condition_name.is_empty());
        assert!(prediction.predicted_onset_age.is_some());
        assert!(!prediction.probability_by_age.is_empty());
        assert!(prediction.confidence_interval.0 <= prediction.confidence_interval.1);
    }

    #[test]
    fn test_predict_empty_profile_returns_err() {
        let model = HealthPredictionModel::new();
        let profile = GeneticProfile {
            profile_id: "empty".into(),
            genetic_mutations: HashMap::new(),
            polygenic_risk_scores: HashMap::new(),
            carrier_status: HashMap::new(),
            pharmacogenomic_markers: HashMap::new(),
        };

        assert!(model
            .predict_disease_onset(&profile, &sample_lifestyle(), &sample_environmental())
            .is_err());
    }

    #[test]
    fn test_lifestyle_multiplier_smoking() {
        let model = HealthPredictionModel::new();
        let mut lifestyle = sample_lifestyle();
        lifestyle.smoking_status = true;

        let multiplier = model.compute_lifestyle_multiplier(&lifestyle, "any");
        assert!(multiplier > 1.0);
    }

    #[test]
    fn test_lifestyle_multiplier_healthy() {
        let model = HealthPredictionModel::new();
        let lifestyle = sample_lifestyle();
        let multiplier = model.compute_lifestyle_multiplier(&lifestyle, "any");
        assert!(multiplier < 2.0);
    }

    #[test]
    fn test_environmental_multiplier_pollution() {
        let model = HealthPredictionModel::new();
        let mut env = sample_environmental();
        env.air_quality_index = Some(100.0);

        let multiplier = model.compute_environmental_multiplier(&env, "any");
        assert!(multiplier > 1.0);
    }

    #[test]
    fn test_environmental_multiplier_clean() {
        let model = HealthPredictionModel::new();
        let multiplier = model.compute_environmental_multiplier(&sample_environmental(), "any");
        assert!((multiplier - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interaction_boost_gene_lifestyle() {
        let model = HealthPredictionModel::new();
        let profile = sample_genetic_profile();
        let mut lifestyle = sample_lifestyle();
        lifestyle.exercise_frequency = 1.0;

        let boost = model.compute_interaction_boost(&profile, &lifestyle, "any");
        assert!(boost > 0.0);
    }

    #[test]
    fn test_estimate_onset_age_high_risk() {
        let model = HealthPredictionModel::new();
        let age = model.estimate_onset_age("Type 2 Diabetes", 0.85);
        assert!(age.is_some());
        assert!(age.unwrap() <= 45);
    }

    #[test]
    fn test_estimate_onset_age_low_risk() {
        let model = HealthPredictionModel::new();
        let age = model.estimate_onset_age("Type 2 Diabetes", 0.05);
        assert!(age.is_none());
    }

    #[test]
    fn test_prediction_probability_by_age_increases() {
        let model = HealthPredictionModel::new();
        let profile = sample_genetic_profile();
        let prediction = model
            .predict_disease_onset(&profile, &sample_lifestyle(), &sample_environmental())
            .unwrap();

        let probs: Vec<f64> = prediction
            .probability_by_age
            .iter()
            .map(|(_, p)| *p)
            .collect();

        for i in 1..probs.len() {
            assert!(
                probs[i] >= probs[i - 1] - 0.01,
                "Probability should increase with age: {} < {}",
                probs[i],
                probs[i - 1]
            );
        }
    }
}

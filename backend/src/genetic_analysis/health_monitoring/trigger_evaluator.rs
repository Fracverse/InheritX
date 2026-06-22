use super::errors::{EthicsError, TriggerError};
use super::types::*;
use std::collections::HashMap;

pub struct HealthTriggerEvaluator;

impl HealthTriggerEvaluator {
    pub async fn evaluate_genetic_triggers(
        &self,
        genetic_profile: &GeneticProfile,
        health_records: &[MedicalRecord],
    ) -> Result<Vec<EvaluatedTrigger>, TriggerError> {
        if genetic_profile.genetic_mutations.is_empty() {
            return Err(TriggerError::InsufficientData(
                "Genetic profile has no mutations".into(),
            ));
        }

        let mut triggers = Vec::new();

        for (condition, predisposition) in &genetic_profile.genetic_mutations {
            if *predisposition >= 0.7 {
                let has_confirmation = health_records.iter().any(|r| {
                    r.diagnosis_codes
                        .iter()
                        .any(|code| code.to_lowercase().contains(&condition.to_lowercase()))
                });

                triggers.push(EvaluatedTrigger {
                    trigger_id: format!("gen_{}_{}", condition, now_secs()),
                    condition_name: condition.clone(),
                    trigger_type: HealthTriggerType::GeneticConditionDetected(condition.clone()),
                    confidence_level: *predisposition,
                    urgency_level: if has_confirmation {
                        UrgencyLevel::High
                    } else {
                        UrgencyLevel::Medium
                    },
                    estimated_timeframe: Some(TimeFrame {
                        min_days: if has_confirmation { 0 } else { 90 },
                        max_days: if has_confirmation { 30 } else { 365 },
                        description: if has_confirmation {
                            "Condition confirmed - immediate action recommended".into()
                        } else {
                            "Monitor for early symptoms".into()
                        },
                    }),
                    supporting_evidence: vec![Evidence {
                        source: "Genetic Analysis".into(),
                        description: format!(
                            "Genetic predisposition for {}: {:.1}%",
                            condition,
                            predisposition * 100.0
                        ),
                        value: serde_json::json!({
                            "predisposition": predisposition,
                            "confirmed_by_records": has_confirmation
                        }),
                        confidence: *predisposition,
                    }],
                });
            }
        }

        Ok(triggers)
    }

    pub async fn assess_disease_progression(
        &self,
        condition: &MonitoredCondition,
        recent_records: &[MedicalRecord],
    ) -> Result<ProgressionAssessment, TriggerError> {
        let previous_status = condition.current_status;

        let worsening_markers: Vec<&ProgressionMarker> = condition
            .progression_markers
            .iter()
            .filter(|m| {
                m.trend == ProgressionTrend::Worsening
                    || m.trend == ProgressionTrend::RapidlyWorsening
            })
            .collect();

        let new_diagnoses: Vec<&MedicalRecord> = recent_records
            .iter()
            .filter(|r| {
                r.diagnosis_codes.iter().any(|code| {
                    code.to_lowercase()
                        .contains(&condition.condition_name.to_lowercase())
                })
            })
            .collect();

        let current_status = if !new_diagnoses.is_empty() {
            match condition.current_status {
                ConditionStatus::EarlyMarkers | ConditionStatus::Developing => {
                    ConditionStatus::Diagnosed
                }
                _ => condition.current_status,
            }
        } else if worsening_markers.len() as f64 >= condition.progression_markers.len() as f64 * 0.5
        {
            match condition.current_status {
                ConditionStatus::Diagnosed => ConditionStatus::Progressive,
                ConditionStatus::Progressive => ConditionStatus::Terminal,
                _ => condition.current_status,
            }
        } else {
            condition.current_status
        };

        let progression_rate = if condition.progression_markers.is_empty() {
            0.0
        } else {
            worsening_markers.len() as f64 / condition.progression_markers.len() as f64
        };

        let markers_changed: Vec<String> = worsening_markers
            .iter()
            .map(|m| m.marker_name.clone())
            .collect();

        let estimated_time_to_next_stage = match current_status {
            ConditionStatus::Developing => Some(TimeFrame {
                min_days: 30,
                max_days: 180,
                description: "Estimated time to clinical diagnosis".into(),
            }),
            ConditionStatus::Diagnosed => Some(TimeFrame {
                min_days: 90,
                max_days: 365,
                description: "Estimated time to progressive stage".into(),
            }),
            ConditionStatus::Progressive => Some(TimeFrame {
                min_days: 180,
                max_days: 730,
                description: "Estimated time based on progression rate".into(),
            }),
            _ => None,
        };

        Ok(ProgressionAssessment {
            condition_name: condition.condition_name.clone(),
            previous_status,
            current_status,
            progression_rate,
            markers_changed,
            estimated_time_to_next_stage,
            confidence: 1.0 - (progression_rate * 0.3),
        })
    }

    pub async fn predict_health_outcomes(
        &self,
        genetic_profile: &GeneticProfile,
        current_health: &HealthStatus,
    ) -> Result<HealthPrediction, TriggerError> {
        let mut target_condition = String::new();
        let mut highest_predisposition = 0.0_f64;

        for (condition, predisposition) in &genetic_profile.genetic_mutations {
            if *predisposition > highest_predisposition
                && !current_health.active_conditions.contains(condition)
            {
                highest_predisposition = *predisposition;
                target_condition = condition.clone();
            }
        }

        if highest_predisposition == 0.0 {
            return Err(TriggerError::InsufficientData(
                "No significant genetic predispositions found".into(),
            ));
        }

        let probability = (highest_predisposition * 100.0).min(95.0);
        let years_estimate = ((1.0 - highest_predisposition) * 20.0 + 5.0) as u32;

        Ok(HealthPrediction {
            prediction_id: format!("pred_{}", now_secs()),
            condition_name: target_condition,
            predicted_onset: Some(TimeFrame {
                min_days: years_estimate.saturating_sub(2) * 365,
                max_days: (years_estimate + 2) * 365,
                description: format!(
                    "Estimated onset in {}±2 years based on genetic profile",
                    years_estimate
                ),
            }),
            probability,
            contributing_factors: vec![
                format!(
                    "Genetic predisposition: {:.1}%",
                    highest_predisposition * 100.0
                ),
                "Age-related risk accumulation".into(),
            ],
            recommended_actions: vec![
                "Increase monitoring frequency".into(),
                "Schedule preventive screening".into(),
                "Consult with specialist".into(),
            ],
        })
    }

    pub fn evaluate_trigger_urgency(&self, trigger: &EvaluatedTrigger) -> UrgencyLevel {
        match trigger.trigger_type {
            HealthTriggerType::TerminalDiagnosis(_) => UrgencyLevel::Critical,
            HealthTriggerType::LifeExpectancyReduction(months) if months < 6 => {
                UrgencyLevel::Critical
            }
            HealthTriggerType::CognitiveDecline(CognitionLevel::Severe) => UrgencyLevel::High,
            HealthTriggerType::DiseaseProgression(ref c) if trigger.confidence_level >= 0.9 => {
                UrgencyLevel::High
            }
            HealthTriggerType::PhysicalIncapacitation(_) => UrgencyLevel::High,
            HealthTriggerType::TreatmentFailure(_) => UrgencyLevel::High,
            HealthTriggerType::LifeExpectancyReduction(_) => UrgencyLevel::Medium,
            HealthTriggerType::QualityOfLifeDecline(v) if v <= 0.2 => UrgencyLevel::Critical,
            HealthTriggerType::QualityOfLifeDecline(v) if v <= 0.4 => UrgencyLevel::High,
            HealthTriggerType::GeneticConditionDetected(_) => UrgencyLevel::Medium,
            HealthTriggerType::CognitiveDecline(_) => UrgencyLevel::Medium,
            _ => UrgencyLevel::Low,
        }
    }

    pub fn validate_ethics_compliance(
        &self,
        trigger_request: &TriggerRequest,
    ) -> Result<EthicsCompliance, EthicsError> {
        if trigger_request.supporting_data.is_empty() {
            return Err(EthicsError::MedicalNecessityNotEstablished(
                "No supporting medical data provided".into(),
            ));
        }

        let has_genetic_consent = trigger_request.supporting_data.iter().any(|e| {
            e.source.to_lowercase().contains("consent")
                || e.description.to_lowercase().contains("consent")
        });

        let data_minimization = trigger_request.supporting_data.len() <= 10;
        if !data_minimization {
            return Err(EthicsError::DataMinimizationViolation(
                "Excessive supporting data provided for trigger request".into(),
            ));
        }

        Ok(EthicsCompliance {
            informed_consent: has_genetic_consent,
            data_minimization,
            purpose_limitation: true,
            medical_necessity: !trigger_request.supporting_data.is_empty(),
        })
    }

    pub fn prioritize_triggers(triggers: Vec<EvaluatedTrigger>) -> Vec<EvaluatedTrigger> {
        let mut triggers = triggers;
        triggers.sort_by(|a, b| {
            let urgency_order = |u: &UrgencyLevel| -> u8 {
                match u {
                    UrgencyLevel::Emergency => 0,
                    UrgencyLevel::Critical => 1,
                    UrgencyLevel::High => 2,
                    UrgencyLevel::Medium => 3,
                    UrgencyLevel::Low => 4,
                }
            };
            urgency_order(&a.urgency_level)
                .cmp(&urgency_order(&b.urgency_level))
                .then_with(|| {
                    b.confidence_level
                        .partial_cmp(&a.confidence_level)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        triggers
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_genetic_profile() -> GeneticProfile {
        let mut mutations = HashMap::new();
        mutations.insert("Type 2 Diabetes".into(), 0.75);
        mutations.insert("Alzheimer's Disease".into(), 0.45);
        mutations.insert("Coronary Artery Disease".into(), 0.60);

        GeneticProfile {
            profile_id: "profile-001".into(),
            genetic_mutations: mutations,
            polygenic_risk_scores: HashMap::new(),
            carrier_status: HashMap::new(),
            pharmacogenomic_markers: HashMap::new(),
        }
    }

    fn sample_health_records() -> Vec<MedicalRecord> {
        vec![MedicalRecord {
            record_id: "rec-001".into(),
            patient_id: "pat-001".into(),
            record_type: RecordType::Consultation,
            diagnosis_codes: vec!["E11".into()],
            procedures: vec![],
            medications: vec![],
            vital_signs: VitalSigns {
                temperature: None,
                heart_rate: None,
                blood_pressure_systolic: None,
                blood_pressure_diastolic: None,
                respiratory_rate: None,
                oxygen_saturation: None,
                pain_level: None,
            },
            timestamp: 1700000000,
            provider_info: ProviderInfo {
                provider_id: "prov-001".into(),
                provider_name: "Dr. Smith".into(),
                facility: "General Hospital".into(),
                specialty: None,
            },
        }]
    }

    #[tokio::test]
    async fn test_evaluate_genetic_triggers() {
        let evaluator = HealthTriggerEvaluator;
        let profile = sample_genetic_profile();
        let records = sample_health_records();

        let triggers = evaluator
            .evaluate_genetic_triggers(&profile, &records)
            .await
            .unwrap();

        assert!(!triggers.is_empty());
        assert!(triggers
            .iter()
            .any(|t| t.condition_name == "Type 2 Diabetes"));
    }

    #[tokio::test]
    async fn test_evaluate_genetic_triggers_empty_profile() {
        let evaluator = HealthTriggerEvaluator;
        let profile = GeneticProfile {
            profile_id: "empty".into(),
            genetic_mutations: HashMap::new(),
            polygenic_risk_scores: HashMap::new(),
            carrier_status: HashMap::new(),
            pharmacogenomic_markers: HashMap::new(),
        };

        let result = evaluator.evaluate_genetic_triggers(&profile, &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_assess_disease_progression() {
        let evaluator = HealthTriggerEvaluator;
        let condition = MonitoredCondition {
            condition_name: "Type 2 Diabetes".into(),
            genetic_predisposition: 0.75,
            current_status: ConditionStatus::Diagnosed,
            progression_markers: vec![ProgressionMarker {
                marker_name: "hba1c".into(),
                measured_value: 8.5,
                baseline_value: 6.5,
                threshold: 7.0,
                trend: ProgressionTrend::Worsening,
                measured_at: now_secs(),
            }],
            trigger_criteria: TriggerCriteria {
                min_predisposition: 0.5,
                required_status: vec![ConditionStatus::Progressive],
                progression_threshold: 0.8,
                marker_thresholds: HashMap::new(),
                time_window_seconds: 0,
            },
            last_assessment: now_secs(),
        };

        let assessment = evaluator
            .assess_disease_progression(&condition, &[])
            .await
            .unwrap();

        assert_eq!(assessment.previous_status, ConditionStatus::Diagnosed);
        assert!(assessment.progression_rate > 0.0);
    }

    #[tokio::test]
    async fn test_assess_disease_progression_with_new_diagnosis() {
        let evaluator = HealthTriggerEvaluator;
        let condition = MonitoredCondition {
            condition_name: "Hypertension".into(),
            genetic_predisposition: 0.6,
            current_status: ConditionStatus::EarlyMarkers,
            progression_markers: vec![],
            trigger_criteria: TriggerCriteria {
                min_predisposition: 0.5,
                required_status: vec![ConditionStatus::Diagnosed],
                progression_threshold: 0.7,
                marker_thresholds: HashMap::new(),
                time_window_seconds: 0,
            },
            last_assessment: now_secs(),
        };

        let records = vec![MedicalRecord {
            record_id: "rec-hbp".into(),
            patient_id: "pat-001".into(),
            record_type: RecordType::Consultation,
            diagnosis_codes: vec!["I10".into()],
            procedures: vec![],
            medications: vec![],
            vital_signs: VitalSigns {
                temperature: None,
                heart_rate: None,
                blood_pressure_systolic: Some(150),
                blood_pressure_diastolic: Some(95),
                respiratory_rate: None,
                oxygen_saturation: None,
                pain_level: None,
            },
            timestamp: now_secs(),
            provider_info: ProviderInfo {
                provider_id: "prov-001".into(),
                provider_name: "Dr. Smith".into(),
                facility: "Clinic".into(),
                specialty: None,
            },
        }];

        let assessment = evaluator
            .assess_disease_progression(&condition, &records)
            .await
            .unwrap();
        assert_eq!(assessment.current_status, ConditionStatus::Diagnosed);
    }

    #[tokio::test]
    async fn test_predict_health_outcomes() {
        let evaluator = HealthTriggerEvaluator;
        let profile = sample_genetic_profile();
        let health = HealthStatus {
            age: 50,
            active_conditions: vec![],
            current_medications: vec![],
            recent_procedures: vec![],
            overall_wellness_score: 75.0,
            assessed_at: now_secs(),
        };

        let prediction = evaluator
            .predict_health_outcomes(&profile, &health)
            .await
            .unwrap();

        assert!(prediction.probability > 0.0);
        assert!(prediction.predicted_onset.is_some());
    }

    #[test]
    fn test_trigger_urgency_terminal_is_critical() {
        let evaluator = HealthTriggerEvaluator;
        let trigger = EvaluatedTrigger {
            trigger_id: "t1".into(),
            condition_name: "Cancer".into(),
            trigger_type: HealthTriggerType::TerminalDiagnosis("Stage IV Cancer".into()),
            confidence_level: 0.95,
            urgency_level: UrgencyLevel::Low,
            estimated_timeframe: None,
            supporting_evidence: vec![],
        };
        assert_eq!(
            evaluator.evaluate_trigger_urgency(&trigger),
            UrgencyLevel::Critical
        );
    }

    #[test]
    fn test_prioritize_triggers() {
        let evaluator = HealthTriggerEvaluator;
        let triggers = vec![
            EvaluatedTrigger {
                trigger_id: "low".into(),
                condition_name: "Low".into(),
                trigger_type: HealthTriggerType::GeneticConditionDetected("Low".into()),
                confidence_level: 0.5,
                urgency_level: UrgencyLevel::Low,
                estimated_timeframe: None,
                supporting_evidence: vec![],
            },
            EvaluatedTrigger {
                trigger_id: "critical".into(),
                condition_name: "Critical".into(),
                trigger_type: HealthTriggerType::TerminalDiagnosis("Critical".into()),
                confidence_level: 0.9,
                urgency_level: UrgencyLevel::Critical,
                estimated_timeframe: None,
                supporting_evidence: vec![],
            },
        ];

        let prioritized = HealthTriggerEvaluator::prioritize_triggers(triggers);
        assert_eq!(prioritized[0].trigger_id, "critical");
        assert_eq!(prioritized[1].trigger_id, "low");
    }

    #[test]
    fn test_validate_ethics_compliance() {
        let evaluator = HealthTriggerEvaluator;
        let request = TriggerRequest {
            user_id: 1,
            plan_id: 1,
            condition_name: "Test".into(),
            trigger_type: HealthTriggerType::GeneticConditionDetected("Test".into()),
            supporting_data: vec![Evidence {
                source: "consent_form".into(),
                description: "Patient consent obtained".into(),
                value: serde_json::json!({"consented": true}),
                confidence: 1.0,
            }],
            requested_by: "Dr. Smith".into(),
        };

        let compliance = evaluator.validate_ethics_compliance(&request).unwrap();
        assert!(compliance.informed_consent);
        assert!(compliance.data_minimization);
    }

    #[test]
    fn test_validate_ethics_missing_consent() {
        let evaluator = HealthTriggerEvaluator;
        let request = TriggerRequest {
            user_id: 1,
            plan_id: 1,
            condition_name: "Test".into(),
            trigger_type: HealthTriggerType::GeneticConditionDetected("Test".into()),
            supporting_data: vec![Evidence {
                source: "lab_report".into(),
                description: "Lab results".into(),
                value: serde_json::json!({"value": 100}),
                confidence: 1.0,
            }],
            requested_by: "Dr. Smith".into(),
        };

        let compliance = evaluator.validate_ethics_compliance(&request).unwrap();
        assert!(!compliance.informed_consent);
        assert!(compliance.medical_necessity);
    }

    #[test]
    fn test_validate_ethics_excessive_data() {
        let evaluator = HealthTriggerEvaluator;
        let supporting_data: Vec<Evidence> = (0..15)
            .map(|i| Evidence {
                source: format!("source_{}", i),
                description: format!("evidence {}", i),
                value: serde_json::json!({"idx": i}),
                confidence: 1.0,
            })
            .collect();

        let request = TriggerRequest {
            user_id: 1,
            plan_id: 1,
            condition_name: "Test".into(),
            trigger_type: HealthTriggerType::GeneticConditionDetected("Test".into()),
            supporting_data,
            requested_by: "Dr. Smith".into(),
        };

        assert!(evaluator.validate_ethics_compliance(&request).is_err());
    }

    #[test]
    fn test_evaluate_urgency_life_expectancy_reduction() {
        let evaluator = HealthTriggerEvaluator;
        let trigger = EvaluatedTrigger {
            trigger_id: "t1".into(),
            condition_name: "Test".into(),
            trigger_type: HealthTriggerType::LifeExpectancyReduction(3),
            confidence_level: 0.8,
            urgency_level: UrgencyLevel::Low,
            estimated_timeframe: None,
            supporting_evidence: vec![],
        };
        assert_eq!(
            evaluator.evaluate_trigger_urgency(&trigger),
            UrgencyLevel::Critical
        );
    }
}

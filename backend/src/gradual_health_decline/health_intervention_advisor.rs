use super::errors::*;
use super::types::*;
use chrono::Utc;

#[derive(Debug, Clone, Default)]
pub struct HealthInterventionAdvisor;

impl HealthInterventionAdvisor {
    pub async fn generate_recommendations(
        &self,
        assessment: &HealthDeclineAssessment,
    ) -> Result<Vec<HealthIntervention>, RecommendationError> {
        let mut recommendations = Vec::new();

        for system in &assessment.affected_systems {
            let (intervention_type, description, evidence) = match system.system_type {
                HealthSystemType::Cardiovascular => (
                    InterventionType::MedicalTreatment,
                    format!(
                        "Cardiovascular monitoring and treatment for {:.0}% decline",
                        system.decline_percentage
                    ),
                    "Clinical guidelines for cardiovascular decline management".to_string(),
                ),
                HealthSystemType::Cognitive => (
                    InterventionType::CognitiveTraining,
                    format!(
                        "Cognitive training and assessment for {:.0}% decline",
                        system.decline_percentage
                    ),
                    "Neuroplasticity-based cognitive rehabilitation evidence".to_string(),
                ),
                HealthSystemType::Mobility => (
                    InterventionType::PhysicalTherapy,
                    format!(
                        "Physical therapy and mobility support for {:.0}% decline",
                        system.decline_percentage
                    ),
                    "Physical therapy outcomes for functional decline".to_string(),
                ),
                HealthSystemType::Sleep => (
                    InterventionType::LifestyleModification,
                    format!(
                        "Sleep hygiene and behavioral intervention for {:.0}% decline",
                        system.decline_percentage
                    ),
                    "Sleep medicine clinical guidelines".to_string(),
                ),
                HealthSystemType::Mental => (
                    InterventionType::LifestyleModification,
                    "Mental health support and counseling".to_string(),
                    "Psychiatric intervention evidence base".to_string(),
                ),
                HealthSystemType::Respiratory => (
                    InterventionType::MedicalTreatment,
                    "Respiratory therapy and medication management".to_string(),
                    "Pulmonary rehabilitation guidelines".to_string(),
                ),
                HealthSystemType::Metabolic => (
                    InterventionType::MedicationAdjustment,
                    "Metabolic monitoring and medication review".to_string(),
                    "Endocrinology clinical protocols".to_string(),
                ),
                HealthSystemType::Overall => (
                    InterventionType::PreventiveCare,
                    "Comprehensive preventive care review".to_string(),
                    "General preventive medicine guidelines".to_string(),
                ),
            };

            let priority = match system.severity {
                DeclineSeverity::Critical => Priority::Urgent,
                DeclineSeverity::Severe => Priority::High,
                DeclineSeverity::Moderate => Priority::Medium,
                DeclineSeverity::Mild => Priority::Low,
            };

            let expected_impact = ExpectedImpact {
                estimated_decline_reduction_percentage: system.decline_percentage * 0.3,
                estimated_quality_of_life_improvement: system.decline_percentage * 0.2,
                time_to_effect_days: 30,
                sustainability: "Medium".to_string(),
            };

            recommendations.push(HealthIntervention {
                intervention_type,
                priority_level: priority,
                expected_impact,
                implementation_difficulty: Difficulty::Moderate,
                cost_estimate: None,
                provider_referral_needed: matches!(
                    intervention_type,
                    InterventionType::MedicalTreatment
                        | InterventionType::PhysicalTherapy
                        | InterventionType::SurgicalIntervention
                ),
                description,
                evidence_basis: evidence,
            });
        }

        if recommendations.is_empty() {
            return Err(RecommendationError::NoInterventions);
        }

        Ok(recommendations)
    }

    pub async fn adjust_inheritance_timeline(
        &self,
        current_timeline: &InheritanceTimeline,
        interventions: &[ImplementedIntervention],
    ) -> Result<AdjustedTimeline, AdjustmentError> {
        if current_timeline.projected_stages.is_empty() {
            return Err(AdjustmentError::InvalidTimeline);
        }

        if interventions.is_empty() {
            return Err(AdjustmentError::NoInterventions);
        }

        let total_adherence: f64 = interventions.iter().map(|i| i.adherence_score).sum();
        let avg_adherence = total_adherence / interventions.len() as f64;
        let delay_months = ((1.0 - avg_adherence) * 6.0).round() as i32;

        let adjusted_stages = current_timeline.projected_stages.clone();
        let adjusted_completion = current_timeline.estimated_completion
            + (delay_months.max(0) as u64) * 30 * 24 * 60 * 60;

        Ok(AdjustedTimeline {
            original_timeline: current_timeline.clone(),
            adjusted_timeline: InheritanceTimeline {
                current_stage: current_timeline.current_stage,
                projected_stages: adjusted_stages,
                estimated_completion: adjusted_completion,
            },
            adjustment_reasons: vec![
                format!("Intervention adherence: {:.0}%", avg_adherence * 100.0),
                format!("Timeline adjusted by {} months", delay_months),
            ],
            projected_impact_months: delay_months,
        })
    }

    pub async fn notify_healthcare_providers(
        &self,
        _profile: &IntegratedHealthProfile,
        _consent: &HealthcareConsent,
    ) -> Result<ProviderNotificationResult, NotificationError> {
        if !_consent.provider_notification_consent {
            return Err(NotificationError::ConsentNotGranted);
        }

        let recipients = _profile
            .physician_assessments
            .iter()
            .filter_map(|a| {
                if a.functional_capacity_score.is_some() {
                    Some(a.physician_id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(ProviderNotificationResult {
            success: true,
            notifications_sent: recipients.len() as u32,
            failed_notifications: Vec::new(),
            sent_at: Utc::now().timestamp_millis() as u64,
        })
    }
}

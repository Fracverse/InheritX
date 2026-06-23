use super::errors::*;
use super::types::*;
use chrono::Utc;

#[derive(Debug, Clone, Default)]
pub struct HealthTrendPredictor;

impl HealthTrendPredictor {
    pub async fn predict_health_trajectory(
        &self,
        current_health: &HealthDeclineAssessment,
        prediction_months: u32,
    ) -> Result<HealthTrajectoryPrediction, PredictionError> {
        if prediction_months == 0 {
            return Err(PredictionError::InsufficientData(
                "Prediction horizon must be > 0".to_string(),
            ));
        }

        let mut predicted_decline_curve = Vec::new();
        let mut confidence_intervals = Vec::new();
        let monthly_decline = current_health.decline_velocity / 30.0;

        for month in 1..=prediction_months {
            let predicted_score = (current_health.overall_decline_score + monthly_decline * month as f64)
                .clamp(0.0, 100.0);

            let uncertainty = 0.05 * month as f64;
            predicted_decline_curve.push(HealthPoint {
                timestamp_months: month,
                predicted_score,
                confidence_low: (predicted_score * (1.0 - uncertainty)).max(0.0),
                confidence_high: (predicted_score * (1.0 + uncertainty)).min(100.0),
            });

            confidence_intervals.push(ConfidenceInterval {
                timestamp_months: month,
                lower_bound: (predicted_score * (1.0 - uncertainty)).max(0.0),
                upper_bound: (predicted_score * (1.0 + uncertainty)).min(100.0),
                confidence_level: 0.95,
            });
        }

        let mut key_milestone_predictions = Vec::new();
        for system in &current_health.affected_systems {
            if system.decline_percentage > 40.0 && key_milestone_predictions.len() < 3 {
                key_milestone_predictions.push(HealthMilestone {
                    description: format!("{:?} decline reaches critical threshold", system.system_type),
                    predicted_date: chrono::Utc::now().timestamp_millis() as u64
                        + (prediction_months / 2) as u64 * 30 * 24 * 60 * 60,
                    confidence: 0.75,
                    severity: system.severity,
                });
            }
        }

        let uncertainty_factors = vec![
            "Treatment adherence variability".to_string(),
            "Genetic predisposition".to_string(),
            "Environmental factors".to_string(),
        ];

        Ok(HealthTrajectoryPrediction {
            prediction_horizon_months: prediction_months,
            predicted_decline_curve,
            confidence_intervals,
            key_milestone_predictions,
            uncertainty_factors,
        })
    }

    pub async fn estimate_functional_decline_timeline(
        &self,
        decline_assessment: &HealthDeclineAssessment,
    ) -> Result<FunctionalDeclineTimeline, EstimationError> {
        let mobility_system = decline_assessment
            .affected_systems
            .iter()
            .find(|s| s.system_type == HealthSystemType::Mobility);

        let cognitive_system = decline_assessment
            .affected_systems
            .iter()
            .find(|s| s.system_type == HealthSystemType::Cognitive);

        let months_to_significant = if let Some(m) = mobility_system {
            (m.decline_percentage / 10.0).round() as u32
        } else {
            prediction_months_from_score(decline_assessment.overall_decline_score)
        };

        let months_to_severe = if let Some(c) = cognitive_system {
            (c.decline_percentage / 15.0).round() as u32 * 2
        } else {
            months_to_significant * 2
        };

        let mut affected_capabilities = Vec::new();
        if let Some(m) = mobility_system {
            affected_capabilities.push(format!(
                "Mobility: {:.0}% decline over {} months",
                m.decline_percentage, m.decline_duration_months
            ));
        }
        if let Some(c) = cognitive_system {
            affected_capabilities.push(format!(
                "Cognitive function: {:.0}% decline over {} months",
                c.decline_percentage, c.decline_duration_months
            ));
        }

        let support_recommendations = vec![
            "Consider home safety modifications".to_string(),
            "Arrange regular health assessments".to_string(),
            "Update emergency contacts".to_string(),
        ];

        Ok(FunctionalDeclineTimeline {
            current_functional_score: (100.0 - decline_assessment.overall_decline_score).max(0.0),
            predicted_months_to_significant_impact: months_to_significant,
            predicted_months_to_severe_impact: months_to_severe,
            affected_capabilities,
            support_recommendations,
        })
    }

    pub async fn calculate_inheritance_optimal_timing(
        &self,
        health_trajectory: &HealthTrajectoryPrediction,
        _inheritance_plan: &InheritanceTimeline,
    ) -> Result<OptimalInheritanceTiming, CalculationError> {
        let current_score = health_trajectory
            .predicted_decline_curve
            .first()
            .map(|p| p.predicted_score)
            .unwrap_or(50.0);

        let earliest_safe = chrono::Utc::now().timestamp_millis() as u64;
        let latest_optimal = chrono::Utc::now().timestamp_millis() as u64
            + (health_trajectory.prediction_horizon_months / 2) as u64 * 30 * 24 * 60 * 60;

        let recommended_stages = vec![InheritanceStage {
            stage_number: 1,
            release_percentage: 30.0,
            health_threshold: HealthThreshold {
                overall_health_score_max: current_score + 15.0,
                decline_velocity_min: 2.0,
                affected_systems_count: 2,
                functional_independence_score: 70.0,
                quality_of_life_score: 60.0,
            },
            trigger_conditions: vec![TriggerCondition::HealthScoreBelow(current_score + 15.0)],
            confirmation_required: true,
            medical_verification_needed: true,
            beneficiary_notification: NotificationConfig {
                channels: vec![NotificationChannel::InApp, NotificationChannel::Email],
                recipients: vec![],
                immediate: true,
            },
        }];

        Ok(OptimalInheritanceTiming {
            recommended_stages,
            earliest_safe_release: earliest_safe,
            latest_optimal_release: latest_optimal,
            reasoning: format!(
                "Based on predicted trajectory over {} months, staged release recommended starting at {:.0}% decline",
                health_trajectory.prediction_horizon_months,
                current_score
            ),
            confidence_score: 0.8,
        })
    }
}

fn prediction_months_from_score(score: f64) -> u32 {
    ((100.0 - score) / 10.0).max(1.0).round() as u32
}

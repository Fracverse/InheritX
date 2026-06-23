use super::errors::*;
use super::types::*;
use crate::genetic_analysis::health_monitoring::types::MedicalRecord as HealthMedicalRecord;
use crate::fitbit_integration::types::{ActivityData, HeartRateReading};
use chrono::Utc;
use serde_json;

use super::health_decline_analyzer::HealthDeclineAnalyzer;
use super::health_trend_predictor::HealthTrendPredictor;
use super::inheritance_stage_manager::InheritanceStageManager;
use super::medical_integrator::MedicalDataIntegrator;
use super::quality_of_life_assessor::QualityOfLifeAssessor;
use super::types::{HealthBaselineCalculator, HealthInterventionAdvisor, UserActivityData};

// ─── Gradual Health Decline Service ──────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct GradualHealthDeclineService {
    pub decline_analyzer: HealthDeclineAnalyzer,
    pub trend_predictor: HealthTrendPredictor,
    pub staging_manager: InheritanceStageManager,
    pub baseline_calculator: HealthBaselineCalculator,
    pub intervention_advisor: HealthInterventionAdvisor,
}

impl GradualHealthDeclineService {
    pub async fn establish_health_baseline(
        &self,
        user_id: &str,
        monitoring_period_days: u32,
    ) -> Result<HealthBaseline, BaselineError> {
        if monitoring_period_days < 7 {
            return Err(BaselineError::InvalidPeriod(monitoring_period_days));
        }

        let data_points = Vec::new();

        self.baseline_calculator
            .calculate_comprehensive_baseline(user_id, &data_points, monitoring_period_days)
            .await
            .map_err(|e| BaselineError::CalculationFailed(e.to_string()))
    }

    pub async fn monitor_health_decline(
        &self,
        user_id: &str,
    ) -> Result<HealthDeclineAssessment, MonitoringError> {
        let heart_rate_history: Vec<HeartRateReading> = Vec::new();
        let activity_history: Vec<ActivityData> = Vec::new();

        let baseline = HealthBaseline {
            user_id: user_id.to_string(),
            established_date: chrono::Utc::now().timestamp_millis() as u64,
            cardiovascular_baseline: CardiovascularBaseline {
                resting_heart_rate: 70.0,
                heart_rate_variability: 50.0,
                blood_pressure_systolic: 120.0,
                blood_pressure_diastolic: 80.0,
                vo2_max: Some(40.0),
            },
            activity_baseline: ActivityBaseline {
                average_daily_steps: 8000.0,
                average_active_minutes: 30.0,
                exercise_capacity: 70.0,
                mobility_index: 0.8,
            },
            sleep_baseline: SleepBaseline {
                average_duration_hours: 7.5,
                sleep_efficiency: 85.0,
                deep_sleep_percentage: 20.0,
                rem_sleep_percentage: 22.0,
                wake_frequency: 2.0,
            },
            cognitive_baseline: CognitiveBaseline {
                memory_score: 80.0,
                processing_speed: 75.0,
                attention_span: 78.0,
                executive_function: 80.0,
                language_ability: 82.0,
            },
            overall_health_score: 75.0,
            age_adjustment_factor: 1.0,
        };

        let cv_analysis = self
            .decline_analyzer
            .detect_cardiovascular_decline(&heart_rate_history, &baseline.cardiovascular_baseline)
            .await
            .map_err(|e| MonitoringError::AnalysisError(e.to_string()))?;

        let activity_analysis = self
            .decline_analyzer
            .analyze_activity_decline(&activity_history, &baseline.activity_baseline, 6)
            .await
            .map_err(|e| MonitoringError::AnalysisError(e.to_string()))?;

        let mut affected_systems = vec![HealthSystemDecline {
            system_type: HealthSystemType::Cardiovascular,
            decline_percentage: cv_analysis.fitness_decline_percentage,
            decline_duration_months: 3,
            severity: cv_analysis.severity,
            contributing_factors: cv_analysis.contributing_factors.clone(),
        }];

        if activity_analysis.decline_percentage > 10.0 {
            affected_systems.push(HealthSystemDecline {
                system_type: HealthSystemType::Mobility,
                decline_percentage: activity_analysis.decline_percentage,
                decline_duration_months: 3,
                severity: DeclineSeverity::Mild,
                contributing_factors: vec!["Reduced daily activity".to_string()],
            });
        }

        let overall_decline_score = affected_systems
            .iter()
            .map(|s| match s.severity {
                DeclineSeverity::Critical => 80.0,
                DeclineSeverity::Severe => 60.0,
                DeclineSeverity::Moderate => 40.0,
                DeclineSeverity::Mild => 20.0,
            })
            .sum::<f64>()
            / affected_systems.len().max(1) as f64;

        let decline_velocity = overall_decline_score / 30.0;

        let predictive_timeline = HealthTimeline {
            estimated_decline_months: 12,
            critical_threshold_date: Some(
                chrono::Utc::now().timestamp_millis() as u64 + 365 * 24 * 60 * 60 * 1000,
            ),
            milestone_dates: vec![],
            confidence: 0.75,
        };

        let recommended_stages = vec![InheritanceStage {
            stage_number: 1,
            release_percentage: 30.0,
            health_threshold: HealthThreshold {
                overall_health_score_max: 80.0,
                decline_velocity_min: 2.0,
                affected_systems_count: 2,
                functional_independence_score: 70.0,
                quality_of_life_score: 60.0,
            },
            trigger_conditions: vec![TriggerCondition::HealthScoreBelow(80.0)],
            confirmation_required: true,
            medical_verification_needed: false,
            beneficiary_notification: NotificationConfig {
                channels: vec![NotificationChannel::InApp, NotificationChannel::Email],
                recipients: vec![],
                immediate: true,
            },
        }];

        Ok(HealthDeclineAssessment {
            user_id: user_id.to_string(),
            assessment_date: chrono::Utc::now().timestamp_millis() as u64,
            overall_decline_score,
            decline_velocity,
            affected_systems,
            predictive_timeline,
            recommended_inheritance_stages: recommended_stages,
        })
    }

    pub async fn calculate_inheritance_stages(
        &self,
        decline_assessment: &HealthDeclineAssessment,
        plan_id: u64,
    ) -> Result<InheritanceStages, StagingError> {
        let preferences = InheritancePreferences {
            plan_id,
            primary_beneficiary: "primary".to_string(),
            total_allocation_percentage: 100.0,
            auto_release_enabled: true,
            medical_verification_required: false,
        };

        let design = self
            .staging_manager
            .design_inheritance_stages(decline_assessment, &preferences)
            .await
            .map_err(|e| StagingError::DesignFailed(e.to_string()))?;

        Ok(InheritanceStages {
            plan_id,
            stages: design.designed_stages,
            total_stages: design.designed_stages.len() as u32,
            estimated_completion_date: design
                .projected_release_dates
                .last()
                .copied()
                .unwrap_or(0),
        })
    }

    pub async fn trigger_staged_release(
        &self,
        plan_id: u64,
        current_stage: u32,
    ) -> Result<StageReleaseResult, ReleaseError> {
        if current_stage == 0 {
            return Err(ReleaseError::InvalidStage(current_stage));
        }

        let stage = InheritanceStage {
            stage_number: current_stage,
            release_percentage: 30.0,
            health_threshold: HealthThreshold {
                overall_health_score_max: 80.0,
                decline_velocity_min: 2.0,
                affected_systems_count: 2,
                functional_independence_score: 70.0,
                quality_of_life_score: 60.0,
            },
            trigger_conditions: vec![TriggerCondition::HealthScoreBelow(80.0)],
            confirmation_required: true,
            medical_verification_needed: false,
            beneficiary_notification: NotificationConfig {
                channels: vec![NotificationChannel::InApp, NotificationChannel::Email],
                recipients: vec![],
                immediate: true,
            },
        };

        let evidence = HealthEvidence {
            evidence_type: "automated".to_string(),
            source: "health_monitor".to_string(),
            recorded_at: chrono::Utc::now().timestamp_millis() as u64,
            data: serde_json::json!({"stage": current_stage}),
            verified: true,
        };

        let execution = self
            .staging_manager
            .execute_stage_release(plan_id, &stage, &evidence)
            .await
            .map_err(|e| ReleaseError::ExecutionFailed(e.to_string()))?;

        Ok(StageReleaseResult {
            success: true,
            plan_id,
            released_stage: execution.stage_number,
            released_percentage: execution.amount_released,
            remaining_percentage: 70.0,
            message: format!(
                "Stage {} released successfully ({:.0}%)",
                execution.stage_number, execution.amount_released
            ),
        })
    }
}

use super::errors::*;
use super::types::*;
use chrono::Utc;

#[derive(Debug, Clone, Default)]
pub struct InheritanceStageManager;

impl InheritanceStageManager {
    pub async fn design_inheritance_stages(
        &self,
        health_decline: &HealthDeclineAssessment,
        plan_preferences: &InheritancePreferences,
    ) -> Result<InheritanceStageDesign, DesignError> {
        if health_decline.affected_systems.is_empty() {
            return Err(DesignError::NoDeclineDetected);
        }

        let affected_count = health_decline.affected_systems.len() as u32;
        let mut stages = Vec::new();
        let total_percentage = plan_preferences.total_allocation_percentage;
        let stage_count = affected_count.min(4).max(1);
        let stage_percentage = total_percentage / stage_count;

        let mut current_threshold = 100.0;
        for i in 1..=stage_count {
            current_threshold -= health_decline.decline_velocity * (i as f64 * 2.0);
            stages.push(InheritanceStage {
                stage_number: i,
                release_percentage: stage_percentage,
                health_threshold: HealthThreshold {
                    overall_health_score_max: current_threshold,
                    decline_velocity_min: health_decline.decline_velocity * (i as f64),
                    affected_systems_count: affected_count,
                    functional_independence_score: (100.0 - current_threshold).clamp(0.0, 100.0),
                    quality_of_life_score: (100.0 - current_threshold * 0.8).clamp(0.0, 100.0),
                },
                trigger_conditions: vec![TriggerCondition::HealthScoreBelow(current_threshold)],
                confirmation_required: i > 1,
                medical_verification_needed: i >= stage_count - 1,
                beneficiary_notification: NotificationConfig {
                    channels: vec![NotificationChannel::InApp, NotificationChannel::Email],
                    recipients: vec![plan_preferences.primary_beneficiary.clone()],
                    immediate: true,
                },
            });
        }

        let design_rationale = format!(
            "Designed {} stages based on {} affected health systems with {:.0}% total allocation",
            stage_count, affected_count, total_percentage
        );

        let projected_release_dates: Vec<u64> = stages
            .iter()
            .enumerate()
            .map(|(i, _)| {
                chrono::Utc::now().timestamp_millis() as u64
                    + ((i + 1) * 6) as u64 * 30 * 24 * 60 * 60
            })
            .collect();

        Ok(InheritanceStageDesign {
            plan_id: plan_preferences.plan_id,
            designed_stages: stages,
            design_rationale,
            projected_release_dates,
        })
    }

    pub async fn evaluate_stage_trigger(
        &self,
        current_health: &HealthSnapshot,
        stage_criteria: &StageCriteria,
    ) -> Result<StageTriggerEvaluation, EvaluationError> {
        let triggered = current_health.overall_score <= stage_criteria.min_health_decline_score
            && current_health.mobility_score.unwrap_or(100.0)
                <= stage_criteria.max_functional_independence;

        let readiness_score = if triggered {
            1.0
        } else {
            let score_gap = stage_criteria.min_health_decline_score - current_health.overall_score;
            (1.0 - (score_gap / 50.0)).clamp(0.0, 1.0)
        };

        let mut blocking_factors = Vec::new();
        if !triggered {
            blocking_factors.push(format!(
                "Health score {:.0} above threshold {:.0}",
                current_health.overall_score, stage_criteria.min_health_decline_score
            ));
        }

        let recommended_actions = if triggered {
            vec!["Stage trigger conditions met. Proceed with release.".to_string()]
        } else {
            vec!["Continue monitoring health metrics.".to_string()]
        };

        Ok(StageTriggerEvaluation {
            current_health: current_health.clone(),
            stage_criteria: stage_criteria.clone(),
            triggered,
            readiness_score,
            blocking_factors,
            recommended_actions,
        })
    }

    pub async fn execute_stage_release(
        &self,
        plan_id: u64,
        stage: &InheritanceStage,
        health_evidence: &HealthEvidence,
    ) -> Result<StageReleaseExecution, ExecutionError> {
        if !health_evidence.verified {
            return Err(ExecutionError::VerificationFailed(
                "Health evidence not verified".to_string(),
            ));
        }

        Ok(StageReleaseExecution {
            executed_at: chrono::Utc::now().timestamp_millis() as u64,
            stage_number: stage.stage_number,
            amount_released: stage.release_percentage,
            beneficiary_notified: true,
            medical_verification_attached: stage.medical_verification_needed,
            transaction_reference: Some(format!("txn_{}_{}", plan_id, stage.stage_number)),
        })
    }
}

use super::errors::HealthError;
use super::health_condition_tracker::HealthConditionTracker;
use super::medical_integrator::MedicalRecordIntegrator;
use super::privacy_guardian::HealthPrivacyGuardian;
use super::trigger_evaluator::HealthTriggerEvaluator;
use super::types::*;

pub struct HealthNotificationService {
    pub notification_channels: Vec<String>,
}

impl HealthNotificationService {
    pub fn new() -> Self {
        Self {
            notification_channels: vec!["email".into(), "sms".into(), "in_app".into()],
        }
    }

    pub async fn send_alert(&self, user_id: u64, alert: &HealthAlert) -> Result<(), HealthError> {
        tracing::info!(
            "Health alert for user {user_id}: [{}] {} - {}",
            alert.severity as u8,
            alert.condition_name,
            alert.message
        );
        Ok(())
    }

    pub async fn send_trigger_notification(
        &self,
        user_id: u64,
        trigger: &TriggeredCondition,
    ) -> Result<(), HealthError> {
        tracing::info!(
            "Trigger notification for user {user_id}: {} ({})",
            trigger.condition_name,
            trigger.trigger_id
        );
        Ok(())
    }

    pub async fn send_escalation(
        &self,
        user_id: u64,
        escalation_level: UrgencyLevel,
        message: &str,
    ) -> Result<(), HealthError> {
        tracing::info!(
            "Escalation for user {user_id} to {:?}: {message}",
            escalation_level
        );
        Ok(())
    }
}

impl Default for HealthNotificationService {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GeneticHealthMonitorService {
    pub health_tracker: HealthConditionTracker,
    pub medical_integrator: MedicalRecordIntegrator,
    pub trigger_evaluator: HealthTriggerEvaluator,
    pub notification_service: HealthNotificationService,
    pub privacy_guardian: HealthPrivacyGuardian,
}

impl Default for GeneticHealthMonitorService {
    fn default() -> Self {
        Self::new()
    }
}

impl GeneticHealthMonitorService {
    pub fn new() -> Self {
        Self {
            health_tracker: HealthConditionTracker::new(),
            medical_integrator: MedicalRecordIntegrator::new(),
            trigger_evaluator: HealthTriggerEvaluator,
            notification_service: HealthNotificationService::new(),
            privacy_guardian: HealthPrivacyGuardian::new(),
        }
    }

    pub async fn monitor_health_conditions(
        &self,
        user_id: u64,
    ) -> Result<HealthMonitoringStatus, HealthError> {
        let status = self.health_tracker.assess_all_conditions();

        for alert in &status.active_alerts {
            self.notification_service.send_alert(user_id, alert).await?;
        }

        Ok(HealthMonitoringStatus { user_id, ..status })
    }

    pub async fn evaluate_trigger_conditions(
        &self,
        _user_id: u64,
    ) -> Result<Vec<TriggeredCondition>, HealthError> {
        let mut triggered = Vec::new();

        for (name, condition) in &self.health_tracker.monitored_conditions {
            if condition.genetic_predisposition >= condition.trigger_criteria.min_predisposition {
                let meets_status = condition.trigger_criteria.required_status.is_empty()
                    || condition
                        .trigger_criteria
                        .required_status
                        .contains(&condition.current_status);

                if meets_status {
                    let progression_ratio =
                        HealthConditionTracker::calculate_progression_ratio(condition);

                    let urgency = if condition.current_status == ConditionStatus::Terminal {
                        UrgencyLevel::Critical
                    } else if condition.current_status == ConditionStatus::Progressive
                        || progression_ratio >= condition.trigger_criteria.progression_threshold
                    {
                        UrgencyLevel::High
                    } else {
                        UrgencyLevel::Medium
                    };

                    triggered.push(TriggeredCondition {
                        trigger_id: format!("trigger_{}_{}", name, chrono_now()),
                        condition_name: name.clone(),
                        trigger_type: HealthTriggerType::DiseaseProgression(name.clone()),
                        confidence: condition.genetic_predisposition,
                        urgency,
                        evidence: vec![Evidence {
                            source: "HealthMonitor".into(),
                            description: format!(
                                "Condition '{}' is at status {:?} with predisposition {:.1}%",
                                name,
                                condition.current_status,
                                condition.genetic_predisposition * 100.0
                            ),
                            value: serde_json::json!({
                                "predisposition": condition.genetic_predisposition,
                                "status": format!("{:?}", condition.current_status),
                                "progression_ratio": progression_ratio
                            }),
                            confidence: condition.genetic_predisposition,
                        }],
                        triggered_at: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    });
                }
            }
        }

        Ok(triggered)
    }

    pub async fn integrate_medical_records(
        &self,
        user_id: u64,
        records: Vec<MedicalRecord>,
    ) -> Result<(), HealthError> {
        let patient_id = user_id.to_string();

        let mut updated_conditions = Vec::new();

        for record in &records {
            for code in &record.diagnosis_codes {
                for (name, condition) in &self.health_tracker.monitored_conditions {
                    if code
                        .to_lowercase()
                        .contains(&condition.condition_name.to_lowercase())
                        || condition
                            .condition_name
                            .to_lowercase()
                            .contains(&code.to_lowercase())
                    {
                        let matching_markers: Vec<String> = condition
                            .progression_markers
                            .iter()
                            .map(|m| m.marker_name.clone())
                            .collect();

                        updated_conditions.push((name.clone(), matching_markers));
                    }
                }
            }
        }

        tracing::info!(
            "Integrated {} medical records for patient {patient_id}",
            records.len()
        );

        Ok(())
    }

    pub async fn assess_inheritance_triggers(
        &self,
        plan_id: u64,
    ) -> Result<TriggerAssessment, HealthError> {
        let triggered = self.evaluate_trigger_conditions(0).await?;

        let mut evaluated_triggers = Vec::new();

        for t in &triggered {
            let urgency = self
                .trigger_evaluator
                .evaluate_trigger_urgency(&EvaluatedTrigger {
                    trigger_id: t.trigger_id.clone(),
                    condition_name: t.condition_name.clone(),
                    trigger_type: t.trigger_type.clone(),
                    confidence_level: t.confidence,
                    urgency_level: t.urgency,
                    estimated_timeframe: None,
                    supporting_evidence: t.evidence.clone(),
                });

            evaluated_triggers.push(EvaluatedTrigger {
                trigger_id: t.trigger_id.clone(),
                condition_name: t.condition_name.clone(),
                trigger_type: t.trigger_type.clone(),
                confidence_level: t.confidence,
                urgency_level: urgency,
                estimated_timeframe: Some(TimeFrame {
                    min_days: match urgency {
                        UrgencyLevel::Critical => 0,
                        UrgencyLevel::High => 7,
                        UrgencyLevel::Medium => 30,
                        _ => 90,
                    },
                    max_days: match urgency {
                        UrgencyLevel::Critical => 7,
                        UrgencyLevel::High => 30,
                        UrgencyLevel::Medium => 90,
                        _ => 365,
                    },
                    description: format!(
                        "Inheritance trigger assessment based on {:?} urgency",
                        urgency
                    ),
                }),
                supporting_evidence: t.evidence.clone(),
            });
        }

        let prioritized = HealthTriggerEvaluator::prioritize_triggers(evaluated_triggers);

        let requires_immediate = prioritized
            .iter()
            .any(|t| t.urgency_level == UrgencyLevel::Critical);

        Ok(TriggerAssessment {
            plan_id,
            assessed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            active_triggers: prioritized,
            pending_triggers: Vec::new(),
            requires_immediate_action: requires_immediate,
        })
    }
}

fn chrono_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_monitor_service() -> GeneticHealthMonitorService {
        let mut service = GeneticHealthMonitorService::new();
        let condition = MonitoredCondition {
            condition_name: "Type 2 Diabetes".into(),
            genetic_predisposition: 0.75,
            current_status: ConditionStatus::Diagnosed,
            progression_markers: vec![],
            trigger_criteria: TriggerCriteria {
                min_predisposition: 0.5,
                required_status: vec![ConditionStatus::Diagnosed],
                progression_threshold: 0.7,
                marker_thresholds: std::collections::HashMap::new(),
                time_window_seconds: 0,
            },
            last_assessment: 0,
        };
        service.health_tracker.register_condition(condition);
        service
    }

    #[tokio::test]
    async fn test_monitor_health_conditions() {
        let service = sample_monitor_service();
        let status = service.monitor_health_conditions(1).await.unwrap();
        assert_eq!(status.user_id, 1);
    }

    #[tokio::test]
    async fn test_evaluate_trigger_conditions() {
        let service = sample_monitor_service();
        let triggers = service.evaluate_trigger_conditions(1).await.unwrap();
        assert!(!triggers.is_empty());
    }

    #[tokio::test]
    async fn test_integrate_medical_records() {
        let service = sample_monitor_service();
        let records = vec![];
        let result = service.integrate_medical_records(1, records).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_assess_inheritance_triggers() {
        let service = sample_monitor_service();
        let assessment = service.assess_inheritance_triggers(42).await.unwrap();
        assert_eq!(assessment.plan_id, 42);
    }

    #[tokio::test]
    async fn test_notification_service_send_alert() {
        let notif = HealthNotificationService::new();
        let alert = HealthAlert {
            alert_id: "alert-001".into(),
            condition_name: "Test".into(),
            alert_type: HealthTriggerType::GeneticConditionDetected("Test".into()),
            severity: UrgencyLevel::High,
            message: "Test alert".into(),
            triggered_at: 0,
        };
        assert!(notif.send_alert(1, &alert).await.is_ok());
    }

    #[tokio::test]
    async fn test_full_monitoring_cycle() {
        let service = sample_monitor_service();

        let status = service.monitor_health_conditions(1).await.unwrap();
        assert_eq!(status.monitored_conditions.len(), 1);

        let triggers = service.evaluate_trigger_conditions(1).await.unwrap();
        assert!(!triggers.is_empty());

        let assessment = service.assess_inheritance_triggers(42).await.unwrap();
        assert_eq!(assessment.plan_id, 42);
    }

    #[test]
    fn test_health_notification_service_default_channels() {
        let notif = HealthNotificationService::new();
        assert_eq!(notif.notification_channels.len(), 3);
    }

    #[test]
    fn test_service_default_creates_with_all_components() {
        let service = GeneticHealthMonitorService::new();
        assert!(service.health_tracker.monitored_conditions.is_empty());
    }
}

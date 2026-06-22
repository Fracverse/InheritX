use super::errors::HealthError;
use super::types::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct HealthConditionTracker {
    pub monitored_conditions: HashMap<String, MonitoredCondition>,
    pub baseline_health_data: HealthBaseline,
    pub tracking_frequency: TrackingFrequency,
    pub alert_thresholds: AlertThresholds,
}

impl Default for HealthConditionTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthConditionTracker {
    pub fn new() -> Self {
        Self {
            monitored_conditions: HashMap::new(),
            baseline_health_data: HealthBaseline {
                age: 0,
                bmi: None,
                blood_pressure_systolic: None,
                blood_pressure_diastolic: None,
                resting_heart_rate: None,
                cholesterol_ldl: None,
                cholesterol_hdl: None,
                fasting_glucose: None,
                established_at: now_secs(),
            },
            tracking_frequency: TrackingFrequency::Weekly,
            alert_thresholds: AlertThresholds {
                predisposition_warning: 0.3,
                predisposition_critical: 0.7,
                progression_warning: 0.4,
                progression_critical: 0.8,
                urgency_escalation_delay_seconds: 86400,
            },
        }
    }

    pub fn register_condition(&mut self, condition: MonitoredCondition) {
        self.monitored_conditions
            .insert(condition.condition_name.clone(), condition);
    }

    pub fn update_condition_status(
        &mut self,
        condition_name: &str,
        new_status: ConditionStatus,
    ) -> Result<(), HealthError> {
        let condition = self
            .monitored_conditions
            .get_mut(condition_name)
            .ok_or_else(|| HealthError::ConditionNotFound(condition_name.to_string()))?;

        condition.current_status = new_status;
        condition.last_assessment = now_secs();
        Ok(())
    }

    pub fn add_progression_marker(
        &mut self,
        condition_name: &str,
        marker: ProgressionMarker,
    ) -> Result<(), HealthError> {
        let condition = self
            .monitored_conditions
            .get_mut(condition_name)
            .ok_or_else(|| HealthError::ConditionNotFound(condition_name.to_string()))?;

        condition.progression_markers.push(marker);
        condition.last_assessment = now_secs();
        Ok(())
    }

    pub fn evaluate_condition_status(
        &self,
        condition_name: &str,
    ) -> Result<(ConditionStatus, Vec<HealthAlert>), HealthError> {
        let condition = self
            .monitored_conditions
            .get(condition_name)
            .ok_or_else(|| HealthError::ConditionNotFound(condition_name.to_string()))?;

        let mut alerts = Vec::new();

        if condition.genetic_predisposition >= self.alert_thresholds.predisposition_critical {
            alerts.push(HealthAlert {
                alert_id: format!("pred_crit_{}", condition_name),
                condition_name: condition_name.to_string(),
                alert_type: HealthTriggerType::GeneticConditionDetected(condition_name.to_string()),
                severity: UrgencyLevel::Critical,
                message: format!(
                    "Critical genetic predisposition detected for {}: {:.1}%",
                    condition_name,
                    condition.genetic_predisposition * 100.0
                ),
                triggered_at: now_secs(),
            });
        } else if condition.genetic_predisposition >= self.alert_thresholds.predisposition_warning {
            alerts.push(HealthAlert {
                alert_id: format!("pred_warn_{}", condition_name),
                condition_name: condition_name.to_string(),
                alert_type: HealthTriggerType::GeneticConditionDetected(condition_name.to_string()),
                severity: UrgencyLevel::Medium,
                message: format!(
                    "Elevated genetic predisposition for {}: {:.1}%",
                    condition_name,
                    condition.genetic_predisposition * 100.0
                ),
                triggered_at: now_secs(),
            });
        }

        let progression_ratio = Self::calculate_progression_ratio(condition);
        if progression_ratio >= self.alert_thresholds.progression_critical {
            alerts.push(HealthAlert {
                alert_id: format!("prog_crit_{}", condition_name),
                condition_name: condition_name.to_string(),
                alert_type: HealthTriggerType::DiseaseProgression(condition_name.to_string()),
                severity: UrgencyLevel::High,
                message: format!(
                    "Critical disease progression detected for {}",
                    condition_name
                ),
                triggered_at: now_secs(),
            });
        } else if progression_ratio >= self.alert_thresholds.progression_warning {
            alerts.push(HealthAlert {
                alert_id: format!("prog_warn_{}", condition_name),
                condition_name: condition_name.to_string(),
                alert_type: HealthTriggerType::DiseaseProgression(condition_name.to_string()),
                severity: UrgencyLevel::Medium,
                message: format!("Disease progression warning for {}", condition_name),
                triggered_at: now_secs(),
            });
        }

        let escalated_status = self.escalate_status(condition, progression_ratio);
        Ok((escalated_status, alerts))
    }

    pub fn assess_all_conditions(&self) -> HealthMonitoringStatus {
        let mut alerts = Vec::new();
        let mut conditions = Vec::new();

        for (name, condition) in &self.monitored_conditions {
            if let Ok((status, condition_alerts)) = self.evaluate_condition_status(name) {
                let mut c = condition.clone();
                c.current_status = status;
                conditions.push(c);
                alerts.extend(condition_alerts);
            }
        }

        let overall = if alerts.iter().any(|a| a.severity == UrgencyLevel::Critical) {
            MonitoringOverallStatus::Critical
        } else if alerts.iter().any(|a| a.severity == UrgencyLevel::High) {
            MonitoringOverallStatus::High
        } else if alerts.iter().any(|a| a.severity == UrgencyLevel::Medium) {
            MonitoringOverallStatus::Elevated
        } else {
            MonitoringOverallStatus::Normal
        };

        HealthMonitoringStatus {
            user_id: 0,
            monitored_conditions: conditions,
            overall_status: overall,
            last_check: now_secs(),
            active_alerts: alerts,
        }
    }

    pub fn update_baseline(&mut self, baseline: HealthBaseline) {
        self.baseline_health_data = baseline;
    }

    pub fn calculate_progression_ratio(condition: &MonitoredCondition) -> f64 {
        if condition.progression_markers.is_empty() {
            return 0.0;
        }

        let above_threshold = condition
            .progression_markers
            .iter()
            .filter(|m| m.measured_value >= m.threshold)
            .count();

        above_threshold as f64 / condition.progression_markers.len() as f64
    }

    fn escalate_status(
        &self,
        condition: &MonitoredCondition,
        progression_ratio: f64,
    ) -> ConditionStatus {
        match condition.current_status {
            ConditionStatus::NoSymptoms => {
                if progression_ratio >= 0.3 {
                    ConditionStatus::EarlyMarkers
                } else {
                    ConditionStatus::NoSymptoms
                }
            }
            ConditionStatus::EarlyMarkers => {
                if progression_ratio >= 0.6 {
                    ConditionStatus::Developing
                } else {
                    ConditionStatus::EarlyMarkers
                }
            }
            ConditionStatus::Developing => {
                if progression_ratio >= 0.8 {
                    ConditionStatus::Diagnosed
                } else {
                    ConditionStatus::Developing
                }
            }
            other => other,
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_monitored_condition() -> MonitoredCondition {
        MonitoredCondition {
            condition_name: "Type 2 Diabetes".to_string(),
            genetic_predisposition: 0.45,
            current_status: ConditionStatus::EarlyMarkers,
            progression_markers: vec![
                ProgressionMarker {
                    marker_name: "fasting_glucose".to_string(),
                    measured_value: 110.0,
                    baseline_value: 90.0,
                    threshold: 126.0,
                    trend: ProgressionTrend::Worsening,
                    measured_at: now_secs(),
                },
                ProgressionMarker {
                    marker_name: "hba1c".to_string(),
                    measured_value: 5.8,
                    baseline_value: 5.2,
                    threshold: 6.5,
                    trend: ProgressionTrend::Worsening,
                    measured_at: now_secs(),
                },
            ],
            trigger_criteria: TriggerCriteria {
                min_predisposition: 0.3,
                required_status: vec![ConditionStatus::Developing, ConditionStatus::Diagnosed],
                progression_threshold: 0.7,
                marker_thresholds: HashMap::from([
                    ("fasting_glucose".to_string(), 126.0),
                    ("hba1c".to_string(), 6.5),
                ]),
                time_window_seconds: 7776000,
            },
            last_assessment: now_secs(),
        }
    }

    #[test]
    fn test_register_and_find_condition() {
        let mut tracker = HealthConditionTracker::new();
        let condition = sample_monitored_condition();
        tracker.register_condition(condition);
        assert!(tracker.monitored_conditions.contains_key("Type 2 Diabetes"));
    }

    #[test]
    fn test_update_condition_status() {
        let mut tracker = HealthConditionTracker::new();
        tracker.register_condition(sample_monitored_condition());

        tracker
            .update_condition_status("Type 2 Diabetes", ConditionStatus::Developing)
            .unwrap();
        assert_eq!(
            tracker.monitored_conditions["Type 2 Diabetes"].current_status,
            ConditionStatus::Developing
        );
    }

    #[test]
    fn test_update_nonexistent_condition_returns_error() {
        let mut tracker = HealthConditionTracker::new();
        assert!(tracker
            .update_condition_status("Nonexistent", ConditionStatus::Diagnosed)
            .is_err());
    }

    #[test]
    fn test_add_progression_marker() {
        let mut tracker = HealthConditionTracker::new();
        tracker.register_condition(sample_monitored_condition());

        let marker = ProgressionMarker {
            marker_name: "new_marker".to_string(),
            measured_value: 1.0,
            baseline_value: 0.5,
            threshold: 2.0,
            trend: ProgressionTrend::Stable,
            measured_at: now_secs(),
        };

        tracker
            .add_progression_marker("Type 2 Diabetes", marker)
            .unwrap();
        assert_eq!(
            tracker.monitored_conditions["Type 2 Diabetes"]
                .progression_markers
                .len(),
            3
        );
    }

    #[test]
    fn test_evaluate_condition_generates_alerts() {
        let mut tracker = HealthConditionTracker::new();
        let mut condition = sample_monitored_condition();
        condition.genetic_predisposition = 0.85;
        tracker.register_condition(condition);

        let (_, alerts) = tracker
            .evaluate_condition_status("Type 2 Diabetes")
            .unwrap();
        assert!(alerts.iter().any(|a| a.severity == UrgencyLevel::Critical));
    }

    #[test]
    fn test_assess_all_conditions() {
        let mut tracker = HealthConditionTracker::new();
        tracker.register_condition(sample_monitored_condition());

        let status = tracker.assess_all_conditions();
        assert_eq!(status.monitored_conditions.len(), 1);
        assert!(status.last_check > 0);
    }

    #[test]
    fn test_status_escalation_no_symptoms_to_early() {
        let mut condition = sample_monitored_condition();
        condition.current_status = ConditionStatus::NoSymptoms;
        let tracker = HealthConditionTracker::new();
        let escalated = tracker.escalate_status(&condition, 0.4);
        assert_eq!(escalated, ConditionStatus::EarlyMarkers);
    }

    #[test]
    fn test_calculate_progression_ratio_no_markers() {
        let mut condition = sample_monitored_condition();
        condition.progression_markers = vec![];
        let tracker = HealthConditionTracker::new();
        assert_eq!(tracker.calculate_progression_ratio(&condition), 0.0);
    }

    #[test]
    fn test_update_baseline() {
        let mut tracker = HealthConditionTracker::new();
        let baseline = HealthBaseline {
            age: 45,
            bmi: Some(27.5),
            blood_pressure_systolic: Some(130),
            blood_pressure_diastolic: Some(85),
            resting_heart_rate: Some(72),
            cholesterol_ldl: Some(130.0),
            cholesterol_hdl: Some(45.0),
            fasting_glucose: Some(100.0),
            established_at: now_secs(),
        };
        tracker.update_baseline(baseline);
        assert_eq!(tracker.baseline_health_data.age, 45);
    }

    #[test]
    fn test_default_tracking_frequency() {
        let tracker = HealthConditionTracker::new();
        assert_eq!(tracker.tracking_frequency, TrackingFrequency::Weekly);
    }
}

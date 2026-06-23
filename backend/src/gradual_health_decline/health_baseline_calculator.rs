use super::errors::*;
use super::types::*;
use chrono::Utc;

#[derive(Debug, Clone, Default)]
pub struct HealthBaselineCalculator;

impl HealthBaselineCalculator {
    pub async fn calculate_comprehensive_baseline(
        &self,
        user_id: &str,
        data_points: &[HealthDataPoint],
        monitoring_days: u32,
    ) -> Result<HealthBaseline, BaselineError> {
        if monitoring_days < 7 {
            return Err(BaselineError::InsufficientData(
                "Minimum 7 days of data required".to_string(),
            ));
        }

        let mut cv_values: Vec<f64> = Vec::new();
        let mut activity_values: Vec<f64> = Vec::new();
        let mut sleep_values: Vec<f64> = Vec::new();
        let mut cognitive_values: Vec<f64> = Vec::new();

        for point in data_points {
            match point.system_type {
                HealthSystemType::Cardiovascular => cv_values.push(point.value),
                HealthSystemType::Mobility => activity_values.push(point.value),
                HealthSystemType::Sleep => sleep_values.push(point.value),
                HealthSystemType::Cognitive => cognitive_values.push(point.value),
                HealthSystemType::Overall => {}
                HealthSystemType::Respiratory => {}
                HealthSystemType::Mental => {}
                HealthSystemType::Metabolic => {}
            }
        }

        let cardiovascular = if cv_values.is_empty() {
            CardiovascularBaseline {
                resting_heart_rate: 70.0,
                heart_rate_variability: 50.0,
                blood_pressure_systolic: 120.0,
                blood_pressure_diastolic: 80.0,
                vo2_max: Some(40.0),
            }
        } else {
            let avg = cv_values.iter().sum::<f64>() / cv_values.len() as f64;
            CardiovascularBaseline {
                resting_heart_rate: avg,
                heart_rate_variability: avg * 0.7,
                blood_pressure_systolic: 120.0 + avg * 0.1,
                blood_pressure_diastolic: 80.0 + avg * 0.05,
                vo2_max: Some(avg),
            }
        };

        let activity = if activity_values.is_empty() {
            ActivityBaseline {
                average_daily_steps: 8000.0,
                average_active_minutes: 30.0,
                exercise_capacity: 70.0,
                mobility_index: 0.8,
            }
        } else {
            let avg = activity_values.iter().sum::<f64>() / activity_values.len() as f64;
            ActivityBaseline {
                average_daily_steps: avg * 1000.0,
                average_active_minutes: avg,
                exercise_capacity: avg * 0.7,
                mobility_index: (avg / 10000.0).clamp(0.0, 1.0),
            }
        };

        let sleep = if sleep_values.is_empty() {
            SleepBaseline {
                average_duration_hours: 7.5,
                sleep_efficiency: 85.0,
                deep_sleep_percentage: 20.0,
                rem_sleep_percentage: 22.0,
                wake_frequency: 2.0,
            }
        } else {
            let avg = sleep_values.iter().sum::<f64>() / sleep_values.len() as f64;
            SleepBaseline {
                average_duration_hours: avg,
                sleep_efficiency: 85.0,
                deep_sleep_percentage: avg * 0.2,
                rem_sleep_percentage: avg * 0.22,
                wake_frequency: 2.0,
            }
        };

        let cognitive = if cognitive_values.is_empty() {
            CognitiveBaseline {
                memory_score: 80.0,
                processing_speed: 75.0,
                attention_span: 78.0,
                executive_function: 80.0,
                language_ability: 82.0,
            }
        } else {
            let avg = cognitive_values.iter().sum::<f64>() / cognitive_values.len() as f64;
            CognitiveBaseline {
                memory_score: avg,
                processing_speed: avg * 0.9,
                attention_span: avg * 0.95,
                executive_function: avg,
                language_ability: avg * 1.02,
            }
        };

        let overall_health_score =
            (cardiovascular.resting_heart_rate / 100.0 * 25.0
                + activity.mobility_index * 25.0
                + sleep.sleep_efficiency * 0.25
                + cognitive.memory_score * 0.25)
                .clamp(0.0, 100.0);

        let age_adjustment_factor = 1.0;

        Ok(HealthBaseline {
            user_id: user_id.to_string(),
            established_date: Utc::now().timestamp_millis() as u64,
            cardiovascular_baseline: cardiovascular,
            activity_baseline: activity,
            sleep_baseline: sleep,
            cognitive_baseline: cognitive,
            overall_health_score,
            age_adjustment_factor,
        })
    }

    pub async fn compare_baseline(
        &self,
        baseline: &HealthBaseline,
        current_snapshot: &HealthSnapshot,
    ) -> Result<BaselineComparison, ComparisonError> {
        let baseline_score = baseline.overall_health_score;
        let current_score = current_snapshot.overall_score;
        let score_delta = current_score - baseline_score;

        let mut declining_systems = Vec::new();
        let mut improving_systems = Vec::new();
        let mut stable_systems = Vec::new();

        macro_rules! check_system {
            ($current:expr, $prev:expr, $sys:expr) => {
                if $current < $prev - 5.0 {
                    declining_systems.push($sys);
                } else if $current > $prev + 5.0 {
                    improving_systems.push($sys);
                } else {
                    stable_systems.push($sys);
                }
            };
        }

        if let Some(cv) = current_snapshot.cardiovascular_score {
            check_system!(
                cv,
                baseline.cardiovascular_baseline.resting_heart_rate,
                HealthSystemType::Cardiovascular
            );
        }
        if let Some(act) = current_snapshot.mobility_score {
            check_system!(act, baseline.activity_baseline.mobility_index * 100.0, HealthSystemType::Mobility);
        }
        if let Some(sl) = current_snapshot.sleep_score {
            check_system!(sl, baseline.sleep_baseline.sleep_efficiency, HealthSystemType::Sleep);
        }
        if let Some(cog) = current_snapshot.cognitive_score {
            check_system!(cog, baseline.cognitive_baseline.memory_score, HealthSystemType::Cognitive);
        }

        Ok(BaselineComparison {
            user_id: baseline.user_id.clone(),
            baseline_score,
            current_score,
            score_delta,
            declining_systems,
            improving_systems,
            stable_systems,
        })
    }

    pub async fn update_baseline(
        &self,
        baseline: &HealthBaseline,
        new_data_points: &[HealthDataPoint],
        _days_since_update: u32,
    ) -> Result<UpdatedBaseline, UpdateError> {
        if new_data_points.is_empty() {
            return Err(UpdateError::NoChanges);
        }

        let updated_baseline = HealthBaseline {
            user_id: baseline.user_id.clone(),
            established_date: Utc::now().timestamp_millis() as u64,
            cardiovascular_baseline: baseline.cardiovascular_baseline.clone(),
            activity_baseline: baseline.activity_baseline.clone(),
            sleep_baseline: baseline.sleep_baseline.clone(),
            cognitive_baseline: baseline.cognitive_baseline.clone(),
            overall_health_score: baseline.overall_health_score,
            age_adjustment_factor: baseline.age_adjustment_factor,
        };

        Ok(UpdatedBaseline {
            baseline: updated_baseline,
            days_since_establishment: _days_since_update,
            score_change: 0.0,
            updated_fields: vec!["all".to_string()],
        })
    }
}

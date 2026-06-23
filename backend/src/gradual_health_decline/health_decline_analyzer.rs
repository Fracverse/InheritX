use super::errors::*;
use super::types::*;

// ─── Health Decline Analyzer ──────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct HealthDeclineAnalyzer;

impl HealthDeclineAnalyzer {
    pub async fn detect_cardiovascular_decline(
        &self,
        heart_rate_history: &[HeartRateReading],
        baseline: &CardiovascularBaseline,
    ) -> Result<CardiovascularDeclineAnalysis, AnalysisError> {
        if heart_rate_history.len() < 2 {
            return Err(AnalysisError::InsufficientData(
                "At least 2 heart rate readings required".to_string(),
            ));
        }

        let mut values = Vec::new();
        for reading in heart_rate_history {
            if let Ok(v) = reading.bpm.to_string().parse::<f64>() {
                values.push(v);
            }
        }

        if values.len() < 2 {
            return Err(AnalysisError::InsufficientData(
                "Insufficient numeric heart rate data".to_string(),
            ));
        }

        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let hr_change = avg - baseline.resting_heart_rate;

        let trend = if hr_change > 10.0 {
            TrendDirection::RapidlyDeclining
        } else if hr_change > 5.0 {
            TrendDirection::Declining
        } else if hr_change < -3.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        };

        let variability_decline = if baseline.heart_rate_variability > 0.0 {
            ((baseline.heart_rate_variability - avg * 0.5).max(0.0) / baseline.heart_rate_variability) * 100.0
        } else {
            0.0
        };

        let fitness_decline = if let Some(vo2) = baseline.vo2_max {
            if vo2 > 0.0 {
                ((vo2 - avg * 0.3).max(0.0) / vo2) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let mut contributing_factors = Vec::new();
        if avg > baseline.resting_heart_rate + 5.0 {
            contributing_factors.push("Elevated resting heart rate".to_string());
        }
        if variability_decline > 20.0 {
            contributing_factors.push("Reduced HRV".to_string());
        }

        let severity = match trend {
            TrendDirection::RapidlyDeclining if fitness_decline > 30.0 => DeclineSeverity::Severe,
            TrendDirection::RapidlyDeclining => DeclineSeverity::Moderate,
            TrendDirection::Declining if fitness_decline > 15.0 => DeclineSeverity::Moderate,
            TrendDirection::Declining => DeclineSeverity::Mild,
            _ => DeclineSeverity::Mild,
        };

        Ok(CardiovascularDeclineAnalysis {
            heart_rate_trend: trend,
            variability_decline,
            blood_pressure_trend: TrendDirection::Stable,
            fitness_decline_percentage: fitness_decline,
            severity,
            contributing_factors,
        })
    }

    pub async fn analyze_activity_decline(
        &self,
        activity_history: &[ActivityData],
        baseline: &ActivityBaseline,
        months: u32,
    ) -> Result<ActivityDeclineAnalysis, AnalysisError> {
        if activity_history.is_empty() {
            return Err(AnalysisError::InsufficientData(
                "No activity data available".to_string(),
            ));
        }

        let current_steps: u32 = activity_history.iter().map(|a| a.steps).sum::<u32>()
            / activity_history.len().max(1) as u32;
        let baseline_steps = baseline.average_daily_steps.round() as u32;

        let decline_pct = if baseline_steps > 0 {
            ((baseline_steps as f64 - current_steps as f64) / baseline_steps as f64 * 100.0)
                .max(0.0)
        } else {
            0.0
        };

        let decline_weeks = (months * 4) as u32;
        let mut mobility_concerns = Vec::new();

        if decline_pct > 50.0 {
            mobility_concerns.push(MobilityConcern {
                concern_type: "Severe activity decline".to_string(),
                severity: DeclineSeverity::Critical,
                description: format!(
                    "Steps decreased by {:.1}% from baseline of {}",
                    decline_pct, baseline_steps
                ),
            });
        } else if decline_pct > 30.0 {
            mobility_concerns.push(MobilityConcern {
                concern_type: "Significant activity decline".to_string(),
                severity: DeclineSeverity::Severe,
                description: format!("Steps decreased by {:.1}% from baseline", decline_pct),
            });
        } else if decline_pct > 15.0 {
            mobility_concerns.push(MobilityConcern {
                concern_type: "Moderate activity decline".to_string(),
                severity: DeclineSeverity::Moderate,
                description: format!("Steps decreased by {:.1}% from baseline", decline_pct),
            });
        }

        let inheritance_trigger_score = (decline_pct / 100.0).clamp(0.0, 1.0);

        Ok(ActivityDeclineAnalysis {
            baseline_average_steps: baseline_steps,
            current_average_steps: current_steps,
            decline_percentage: decline_pct,
            decline_duration_weeks: decline_weeks,
            mobility_concerns,
            inheritance_trigger_score,
        })
    }

    pub async fn assess_cognitive_decline(
        &self,
        cognitive_assessments: &[f64],
        baseline: &CognitiveBaseline,
    ) -> Result<CognitiveDeclineAnalysis, AnalysisError> {
        if cognitive_assessments.is_empty() {
            return Err(AnalysisError::InsufficientData(
                "No cognitive assessments available".to_string(),
            ));
        }

        let current_memory: f64 =
            cognitive_assessments.iter().sum::<f64>() / cognitive_assessments.len() as f64;
        let memory_decline = ((baseline.memory_score - current_memory) / baseline.memory_score * 100.0)
            .max(0.0);

        let processing_change = baseline.processing_speed - (current_memory * 0.8);
        let attention_decline =
            ((baseline.attention_span - current_memory * 0.7) / baseline.attention_span * 100.0)
                .max(0.0);

        let functional_impact = ((memory_decline + attention_decline) / 2.0).clamp(0.0, 100.0);

        let mut contributing_factors = Vec::new();
        if memory_decline > 20.0 {
            contributing_factors.push("Significant memory decline detected".to_string());
        }
        if functional_impact > 30.0 {
            contributing_factors.push("Functional independence impacted".to_string());
        }

        let severity = if functional_impact > 50.0 {
            DeclineSeverity::Critical
        } else if functional_impact > 30.0 {
            DeclineSeverity::Severe
        } else if functional_impact > 15.0 {
            DeclineSeverity::Moderate
        } else {
            DeclineSeverity::Mild
        };

        Ok(CognitiveDeclineAnalysis {
            memory_decline_percentage: memory_decline,
            processing_speed_change: processing_change,
            attention_decline,
            functional_impact_score: functional_impact,
            severity,
            contributing_factors,
        })
    }

    pub async fn evaluate_sleep_deterioration(
        &self,
        sleep_data: &[f64],
        baseline: &SleepBaseline,
    ) -> Result<SleepDeteriorationAnalysis, AnalysisError> {
        if sleep_data.is_empty() {
            return Err(AnalysisError::InsufficientData(
                "No sleep data available".to_string(),
            ));
        }

        let avg_duration: f64 = sleep_data.iter().sum::<f64>() / sleep_data.len() as f64;
        let duration_trend = if avg_duration < baseline.average_duration_hours * 0.8 {
            TrendDirection::Declining
        } else if avg_duration > baseline.average_duration_hours * 1.1 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        };

        let efficiency_decline = if baseline.sleep_efficiency > 0.0 {
            ((baseline.sleep_efficiency - avg_duration / baseline.average_duration_hours * 100.0).max(0.0))
        } else {
            0.0
        };

        let deep_sleep_reduction = baseline.deep_sleep_percentage * 0.15;
        let wake_increase = baseline.wake_frequency * 0.2;

        let mut contributing_factors = Vec::new();
        if efficiency_decline > 10.0 {
            contributing_factors.push("Sleep efficiency decline".to_string());
        }
        if avg_duration < baseline.average_duration_hours * 0.75 {
            contributing_factors.push("Insufficient sleep duration".to_string());
        }

        let severity = if efficiency_decline > 25.0 {
            DeclineSeverity::Severe
        } else if efficiency_decline > 15.0 {
            DeclineSeverity::Moderate
        } else {
            DeclineSeverity::Mild
        };

        Ok(SleepDeteriorationAnalysis {
            duration_trend,
            efficiency_decline,
            deep_sleep_reduction,
            wake_frequency_increase: wake_increase,
            severity,
            contributing_factors,
        })
    }
}

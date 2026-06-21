use super::errors::{AnalysisError, CalculationError};
use super::types::*;

pub struct StressLevelMonitor;

impl StressLevelMonitor {
    pub async fn calculate_daily_stress_score(
        &self,
        hrv_data: &[HRVReading],
        activity: &ActivityData,
    ) -> Result<StressScore, CalculationError> {
        if hrv_data.is_empty() {
            return Err(CalculationError::MissingData(
                "No HRV data available".to_string(),
            ));
        }

        let avg_rmssd: f64 = hrv_data.iter().map(|r| r.rmssd).sum::<f64>() / hrv_data.len() as f64;

        // Lower HRV (RMSSD) indicates higher stress
        let hrv_stress = ((80.0 - avg_rmssd) / 80.0 * 100.0).clamp(0.0, 100.0);

        // High sedentary time and low active minutes increase stress score
        let activity_ratio = if activity.sedentary_minutes > 0 {
            activity.active_minutes as f64 / activity.sedentary_minutes as f64
        } else {
            1.0
        };
        let activity_stress = ((1.0 - activity_ratio) * 30.0).clamp(0.0, 30.0);

        let score = (hrv_stress * 0.7 + activity_stress * 0.3).clamp(0.0, 100.0);

        let level = match score {
            s if s < 25.0 => StressLevel::Low,
            s if s < 50.0 => StressLevel::Moderate,
            s if s < 75.0 => StressLevel::High,
            _ => StressLevel::VeryHigh,
        };

        let mut factors = Vec::new();
        if avg_rmssd < 30.0 {
            factors.push("Very low heart rate variability".to_string());
        }
        if activity.active_minutes < 20 {
            factors.push("Low physical activity".to_string());
        }
        if activity.sedentary_minutes > 600 {
            factors.push("Extended sedentary periods".to_string());
        }

        Ok(StressScore {
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            score,
            level,
            contributing_factors: factors,
        })
    }

    pub async fn detect_chronic_stress_patterns(
        &self,
        stress_history: &[StressScore],
        weeks: u32,
    ) -> Result<ChronicStressAnalysis, AnalysisError> {
        if stress_history.is_empty() {
            return Err(AnalysisError::InsufficientData(
                "No stress history data".to_string(),
            ));
        }
        if weeks == 0 {
            return Err(AnalysisError::InvalidParameter(
                "Weeks must be greater than 0".to_string(),
            ));
        }

        let avg_score: f64 =
            stress_history.iter().map(|s| s.score).sum::<f64>() / stress_history.len() as f64;

        let high_stress_days = stress_history
            .iter()
            .filter(|s| matches!(s.level, StressLevel::High | StressLevel::VeryHigh))
            .count();
        let high_stress_pct = high_stress_days as f64 / stress_history.len() as f64 * 100.0;

        let chronic = high_stress_pct > 40.0 || avg_score > 60.0;

        let trend = calculate_stress_trend(stress_history);

        let mut recommendations = Vec::new();
        if chronic {
            recommendations.push("Consider stress management techniques".to_string());
            recommendations.push("Increase physical activity gradually".to_string());
        }
        if high_stress_pct > 60.0 {
            recommendations.push("Consult a healthcare professional".to_string());
        }
        if avg_score > 50.0 {
            recommendations.push("Practice relaxation exercises daily".to_string());
        }

        Ok(ChronicStressAnalysis {
            average_stress_score: avg_score,
            trend,
            high_stress_days_percentage: high_stress_pct,
            chronic_stress_detected: chronic,
            recommendations,
        })
    }

    pub async fn correlate_stress_with_health_decline(
        &self,
        stress_data: &StressHistory,
        health_metrics: &HealthMetrics,
    ) -> Result<StressHealthCorrelation, AnalysisError> {
        if stress_data.scores.is_empty() {
            return Err(AnalysisError::InsufficientData(
                "No stress data available".to_string(),
            ));
        }

        let avg_stress: f64 = stress_data.scores.iter().map(|s| s.score).sum::<f64>()
            / stress_data.scores.len() as f64;

        let health_avg = (health_metrics.heart_rate_score
            + health_metrics.sleep_score
            + health_metrics.activity_score)
            / 3.0;

        // Negative correlation: high stress correlates with lower health scores
        let correlation = if health_avg > 0.0 {
            ((100.0 - avg_stress) / 100.0 - health_avg / 100.0).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        let mut impact_areas = Vec::new();
        if health_metrics.sleep_score < 60.0 && avg_stress > 50.0 {
            impact_areas.push("Sleep quality degradation".to_string());
        }
        if health_metrics.heart_rate_score < 60.0 && avg_stress > 50.0 {
            impact_areas.push("Elevated resting heart rate".to_string());
        }
        if health_metrics.activity_score < 60.0 && avg_stress > 50.0 {
            impact_areas.push("Reduced physical activity".to_string());
        }

        let risk = match avg_stress {
            s if s > 75.0 => SeverityLevel::Critical,
            s if s > 60.0 => SeverityLevel::High,
            s if s > 40.0 => SeverityLevel::Moderate,
            _ => SeverityLevel::Low,
        };

        Ok(StressHealthCorrelation {
            correlation_coefficient: correlation,
            stress_impact_areas: impact_areas,
            health_decline_risk: risk,
        })
    }

    pub async fn get_stress_level_score(&self, stress_history: &[StressScore]) -> f64 {
        if stress_history.is_empty() {
            return 50.0;
        }

        let avg_stress: f64 =
            stress_history.iter().map(|s| s.score).sum::<f64>() / stress_history.len() as f64;

        // Invert: low stress = high health score
        (100.0 - avg_stress).clamp(0.0, 100.0)
    }
}

fn calculate_stress_trend(scores: &[StressScore]) -> Trend {
    if scores.len() < 2 {
        return Trend::Stable;
    }

    let split = scores.len() / 2;
    let first_avg = scores[..split].iter().map(|s| s.score).sum::<f64>() / split.max(1) as f64;
    let second_avg =
        scores[split..].iter().map(|s| s.score).sum::<f64>() / (scores.len() - split).max(1) as f64;

    let change = second_avg - first_avg;
    match change {
        c if c < -5.0 => Trend::Improving,
        c if c < 3.0 => Trend::Stable,
        c if c < 10.0 => Trend::Declining,
        _ => Trend::RapidDecline,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculate_daily_stress_low() {
        let monitor = StressLevelMonitor;
        let hrv = vec![
            HRVReading {
                timestamp: "2025-01-01T08:00:00Z".to_string(),
                rmssd: 60.0,
                sdnn: Some(55.0),
            },
            HRVReading {
                timestamp: "2025-01-01T12:00:00Z".to_string(),
                rmssd: 65.0,
                sdnn: Some(58.0),
            },
        ];
        let activity = ActivityData {
            steps: 10000,
            active_minutes: 45,
            sedentary_minutes: 300,
            calories_burned: 2200,
        };
        let result = monitor
            .calculate_daily_stress_score(&hrv, &activity)
            .await
            .unwrap();
        assert!(result.score < 50.0);
        assert!(matches!(
            result.level,
            StressLevel::Low | StressLevel::Moderate
        ));
    }

    #[tokio::test]
    async fn test_calculate_daily_stress_high() {
        let monitor = StressLevelMonitor;
        let hrv = vec![HRVReading {
            timestamp: "2025-01-01T08:00:00Z".to_string(),
            rmssd: 15.0,
            sdnn: Some(12.0),
        }];
        let activity = ActivityData {
            steps: 2000,
            active_minutes: 5,
            sedentary_minutes: 700,
            calories_burned: 1500,
        };
        let result = monitor
            .calculate_daily_stress_score(&hrv, &activity)
            .await
            .unwrap();
        assert!(result.score > 50.0);
    }

    #[tokio::test]
    async fn test_calculate_daily_stress_empty_hrv() {
        let monitor = StressLevelMonitor;
        let activity = ActivityData {
            steps: 5000,
            active_minutes: 30,
            sedentary_minutes: 400,
            calories_burned: 1800,
        };
        let result = monitor.calculate_daily_stress_score(&[], &activity).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_detect_chronic_stress() {
        let monitor = StressLevelMonitor;
        let history: Vec<StressScore> = (0..14)
            .map(|i| StressScore {
                date: format!("2025-01-{:02}", i + 1),
                score: 70.0 + (i as f64 % 5.0),
                level: StressLevel::High,
                contributing_factors: vec![],
            })
            .collect();
        let result = monitor
            .detect_chronic_stress_patterns(&history, 2)
            .await
            .unwrap();
        assert!(result.chronic_stress_detected);
        assert!(!result.recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_detect_no_chronic_stress() {
        let monitor = StressLevelMonitor;
        let history: Vec<StressScore> = (0..14)
            .map(|i| StressScore {
                date: format!("2025-01-{:02}", i + 1),
                score: 20.0 + (i as f64 % 5.0),
                level: StressLevel::Low,
                contributing_factors: vec![],
            })
            .collect();
        let result = monitor
            .detect_chronic_stress_patterns(&history, 2)
            .await
            .unwrap();
        assert!(!result.chronic_stress_detected);
    }

    #[tokio::test]
    async fn test_correlate_stress_with_health() {
        let monitor = StressLevelMonitor;
        let stress_data = StressHistory {
            scores: vec![StressScore {
                date: "2025-01-01".to_string(),
                score: 70.0,
                level: StressLevel::High,
                contributing_factors: vec![],
            }],
            period_weeks: 1,
        };
        let health_metrics = HealthMetrics {
            heart_rate_score: 50.0,
            sleep_score: 45.0,
            activity_score: 55.0,
            stress_score: 30.0,
        };
        let result = monitor
            .correlate_stress_with_health_decline(&stress_data, &health_metrics)
            .await
            .unwrap();
        assert!(!result.stress_impact_areas.is_empty());
        assert!(matches!(
            result.health_decline_risk,
            SeverityLevel::High | SeverityLevel::Critical
        ));
    }

    #[tokio::test]
    async fn test_stress_level_score() {
        let monitor = StressLevelMonitor;
        let history = vec![StressScore {
            date: "2025-01-01".to_string(),
            score: 30.0,
            level: StressLevel::Moderate,
            contributing_factors: vec![],
        }];
        let score = monitor.get_stress_level_score(&history).await;
        assert!((score - 70.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_stress_level_score_empty() {
        let monitor = StressLevelMonitor;
        let score = monitor.get_stress_level_score(&[]).await;
        assert!((score - 50.0).abs() < f64::EPSILON);
    }
}

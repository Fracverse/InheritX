use super::errors::{AnalysisError, CalculationError, DetectionError};
use super::types::*;

pub struct HeartRateAnalyzer;

impl HeartRateAnalyzer {
    pub async fn analyze_resting_heart_rate_trend(
        &self,
        hr_data: &[HeartRateReading],
        days: u32,
    ) -> Result<RestingHRTrend, AnalysisError> {
        if hr_data.is_empty() {
            return Err(AnalysisError::InsufficientData(
                "No heart rate data available".to_string(),
            ));
        }
        if days == 0 {
            return Err(AnalysisError::InvalidParameter(
                "Days must be greater than 0".to_string(),
            ));
        }

        let total_readings = hr_data.len();
        let split_point = total_readings / 2;

        let first_half_avg = if split_point > 0 {
            hr_data[..split_point]
                .iter()
                .map(|r| r.bpm as f64)
                .sum::<f64>()
                / split_point as f64
        } else {
            hr_data[0].bpm as f64
        };

        let second_half_avg = if total_readings - split_point > 0 {
            hr_data[split_point..]
                .iter()
                .map(|r| r.bpm as f64)
                .sum::<f64>()
                / (total_readings - split_point) as f64
        } else {
            first_half_avg
        };

        let change = second_half_avg - first_half_avg;
        // For resting HR, increasing is bad — negate so classify_hr_trend maps correctly
        let trend = classify_hr_trend(-change);
        let health_concern = change > 5.0 || second_half_avg > 100.0;

        Ok(RestingHRTrend {
            trend,
            start_resting_hr: first_half_avg,
            end_resting_hr: second_half_avg,
            change_bpm: change,
            period_days: days,
            health_concern,
        })
    }

    pub async fn detect_heart_rate_anomalies(
        &self,
        current_hr: u32,
        baseline: &HeartRateBaseline,
    ) -> Result<Vec<HeartRateAnomaly>, DetectionError> {
        if baseline.measurement_days < 7 {
            return Err(DetectionError::InsufficientHistory(
                "Need at least 7 days of baseline data".to_string(),
            ));
        }

        let mut anomalies = Vec::new();
        let low_threshold = baseline.average_resting_hr - (2.0 * baseline.standard_deviation);
        let high_threshold = baseline.average_resting_hr + (2.0 * baseline.standard_deviation);

        let hr_f64 = current_hr as f64;

        if hr_f64 > high_threshold {
            let deviation = (hr_f64 - baseline.average_resting_hr) / baseline.standard_deviation;
            let severity = if deviation > 3.0 {
                SeverityLevel::Critical
            } else {
                SeverityLevel::High
            };

            anomalies.push(HeartRateAnomaly {
                timestamp: chrono::Utc::now().to_rfc3339(),
                observed_hr: current_hr,
                expected_range_low: low_threshold,
                expected_range_high: high_threshold,
                severity,
                description: format!(
                    "Heart rate {} bpm is {:.1} standard deviations above baseline",
                    current_hr, deviation
                ),
            });
        } else if hr_f64 < low_threshold && current_hr < 40 {
            anomalies.push(HeartRateAnomaly {
                timestamp: chrono::Utc::now().to_rfc3339(),
                observed_hr: current_hr,
                expected_range_low: low_threshold,
                expected_range_high: high_threshold,
                severity: SeverityLevel::High,
                description: format!(
                    "Heart rate {} bpm is significantly below baseline average of {:.1} bpm",
                    current_hr, baseline.average_resting_hr
                ),
            });
        }

        Ok(anomalies)
    }

    pub async fn calculate_cardiovascular_fitness(
        &self,
        hr_data: &HeartRateHistory,
        activity_data: &ActivityHistory,
    ) -> Result<CardiovascularFitnessScore, CalculationError> {
        if hr_data.daily_resting_hr.is_empty() {
            return Err(CalculationError::MissingData(
                "No resting heart rate data".to_string(),
            ));
        }
        if activity_data.daily_activities.is_empty() {
            return Err(CalculationError::MissingData(
                "No activity data".to_string(),
            ));
        }

        let avg_resting_hr: f64 = hr_data
            .daily_resting_hr
            .iter()
            .map(|d| d.resting_hr as f64)
            .sum::<f64>()
            / hr_data.daily_resting_hr.len() as f64;

        let avg_active_minutes: f64 = activity_data
            .daily_activities
            .iter()
            .map(|a| a.active_minutes as f64)
            .sum::<f64>()
            / activity_data.daily_activities.len() as f64;

        // Lower resting HR and higher active minutes = better fitness
        let hr_score = ((100.0 - avg_resting_hr) / 40.0).clamp(0.0, 1.0) * 50.0;
        let activity_score = (avg_active_minutes / 60.0).clamp(0.0, 1.0) * 50.0;
        let score = hr_score + activity_score;

        let vo2_max_estimate = Some(15.3 * (208.0 - 0.7 * avg_resting_hr) / avg_resting_hr);

        let fitness_level = match score {
            s if s >= 80.0 => "Excellent",
            s if s >= 60.0 => "Good",
            s if s >= 40.0 => "Fair",
            _ => "Poor",
        }
        .to_string();

        let rhr_len = hr_data.daily_resting_hr.len();
        let trend = if rhr_len >= 2 {
            let recent = hr_data.daily_resting_hr[rhr_len - 1].resting_hr as f64;
            let older = hr_data.daily_resting_hr[0].resting_hr as f64;
            classify_hr_trend(older - recent) // decreasing RHR = improving
        } else {
            Trend::Stable
        };

        Ok(CardiovascularFitnessScore {
            score,
            trend,
            vo2_max_estimate,
            fitness_level,
        })
    }

    pub async fn get_heart_rate_score(&self, hr_data: &[HeartRateReading]) -> f64 {
        if hr_data.is_empty() {
            return 50.0;
        }

        let avg_hr: f64 = hr_data.iter().map(|r| r.bpm as f64).sum::<f64>() / hr_data.len() as f64;

        // Optimal resting HR is 60-70 bpm
        let score = if (60.0..=70.0).contains(&avg_hr) {
            100.0
        } else if avg_hr < 60.0 {
            (avg_hr / 60.0 * 100.0).max(50.0)
        } else {
            ((140.0 - avg_hr) / 70.0 * 100.0).max(0.0)
        };

        score.clamp(0.0, 100.0)
    }
}

fn classify_hr_trend(change: f64) -> Trend {
    match change {
        c if c > 5.0 => Trend::Improving,
        c if c > -2.0 => Trend::Stable,
        c if c > -8.0 => Trend::Declining,
        _ => Trend::RapidDecline,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_readings(bpms: &[u32]) -> Vec<HeartRateReading> {
        bpms.iter()
            .enumerate()
            .map(|(i, &bpm)| HeartRateReading {
                timestamp: format!("2025-01-{:02}T08:00:00Z", i + 1),
                bpm,
                confidence: 0.95,
            })
            .collect()
    }

    #[tokio::test]
    async fn test_resting_hr_trend_stable() {
        let analyzer = HeartRateAnalyzer;
        let readings = sample_readings(&[65, 66, 64, 65, 66, 64, 65, 66]);
        let result = analyzer
            .analyze_resting_heart_rate_trend(&readings, 8)
            .await
            .unwrap();
        assert_eq!(result.trend, Trend::Stable);
        assert!(!result.health_concern);
    }

    #[tokio::test]
    async fn test_resting_hr_trend_declining() {
        let analyzer = HeartRateAnalyzer;
        let readings = sample_readings(&[65, 66, 64, 65, 72, 75, 78, 80]);
        let result = analyzer
            .analyze_resting_heart_rate_trend(&readings, 8)
            .await
            .unwrap();
        assert!(matches!(
            result.trend,
            Trend::Declining | Trend::RapidDecline
        ));
        assert!(result.health_concern);
    }

    #[tokio::test]
    async fn test_resting_hr_trend_empty_data() {
        let analyzer = HeartRateAnalyzer;
        let result = analyzer.analyze_resting_heart_rate_trend(&[], 7).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_detect_anomalies_high_hr() {
        let analyzer = HeartRateAnalyzer;
        let baseline = HeartRateBaseline {
            average_resting_hr: 65.0,
            standard_deviation: 3.0,
            measurement_days: 30,
        };
        let anomalies = analyzer
            .detect_heart_rate_anomalies(85, &baseline)
            .await
            .unwrap();
        assert!(!anomalies.is_empty());
        assert!(matches!(
            anomalies[0].severity,
            SeverityLevel::High | SeverityLevel::Critical
        ));
    }

    #[tokio::test]
    async fn test_detect_anomalies_normal_hr() {
        let analyzer = HeartRateAnalyzer;
        let baseline = HeartRateBaseline {
            average_resting_hr: 65.0,
            standard_deviation: 3.0,
            measurement_days: 30,
        };
        let anomalies = analyzer
            .detect_heart_rate_anomalies(66, &baseline)
            .await
            .unwrap();
        assert!(anomalies.is_empty());
    }

    #[tokio::test]
    async fn test_detect_anomalies_insufficient_baseline() {
        let analyzer = HeartRateAnalyzer;
        let baseline = HeartRateBaseline {
            average_resting_hr: 65.0,
            standard_deviation: 3.0,
            measurement_days: 3,
        };
        let result = analyzer.detect_heart_rate_anomalies(85, &baseline).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cardiovascular_fitness_score() {
        let analyzer = HeartRateAnalyzer;
        let hr_data = HeartRateHistory {
            readings: vec![],
            daily_resting_hr: vec![
                DailyRestingHR {
                    date: "2025-01-01".to_string(),
                    resting_hr: 62,
                },
                DailyRestingHR {
                    date: "2025-01-02".to_string(),
                    resting_hr: 60,
                },
            ],
        };
        let activity_data = ActivityHistory {
            daily_activities: vec![DailyActivity {
                date: "2025-01-01".to_string(),
                steps: 10000,
                distance_km: 7.5,
                calories_burned: 2200,
                active_minutes: 45,
                sedentary_minutes: 480,
                floors_climbed: 10,
            }],
        };
        let result = analyzer
            .calculate_cardiovascular_fitness(&hr_data, &activity_data)
            .await
            .unwrap();
        assert!(result.score > 0.0);
        assert!(result.vo2_max_estimate.is_some());
    }

    #[tokio::test]
    async fn test_heart_rate_score_optimal() {
        let analyzer = HeartRateAnalyzer;
        let readings = sample_readings(&[65, 65, 65]);
        let score = analyzer.get_heart_rate_score(&readings).await;
        assert!((score - 100.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_heart_rate_score_empty() {
        let analyzer = HeartRateAnalyzer;
        let score = analyzer.get_heart_rate_score(&[]).await;
        assert!((score - 50.0).abs() < f64::EPSILON);
    }
}

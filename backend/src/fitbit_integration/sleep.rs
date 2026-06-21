use super::errors::{AnalysisError, DetectionError};
use super::types::*;

pub struct SleepPatternAnalyzer;

impl SleepPatternAnalyzer {
    pub async fn analyze_sleep_quality(
        &self,
        sleep_data: &[SleepSession],
        weeks: u32,
    ) -> Result<SleepQualityAnalysis, AnalysisError> {
        if sleep_data.is_empty() {
            return Err(AnalysisError::InsufficientData(
                "No sleep data available".to_string(),
            ));
        }
        if weeks == 0 {
            return Err(AnalysisError::InvalidParameter(
                "Weeks must be greater than 0".to_string(),
            ));
        }

        let avg_score: f64 = sleep_data
            .iter()
            .filter_map(|s| s.sleep_score.map(|sc| sc as f64))
            .sum::<f64>()
            / sleep_data
                .iter()
                .filter(|s| s.sleep_score.is_some())
                .count()
                .max(1) as f64;

        let avg_efficiency: f64 =
            sleep_data.iter().map(|s| s.efficiency).sum::<f64>() / sleep_data.len() as f64;

        let total_sleep_minutes: u32 = sleep_data.iter().map(|s| s.duration_minutes).sum();
        let total_deep_minutes: u32 = sleep_data
            .iter()
            .flat_map(|s| &s.stages)
            .filter(|st| st.stage == SleepStage::Deep)
            .map(|st| st.duration_minutes)
            .sum();

        let deep_sleep_pct = if total_sleep_minutes > 0 {
            total_deep_minutes as f64 / total_sleep_minutes as f64 * 100.0
        } else {
            0.0
        };

        let sleep_consistency = calculate_sleep_consistency(sleep_data);
        let efficiency_trend = classify_efficiency_trend(sleep_data);

        let mut indicators = Vec::new();
        if avg_efficiency < 75.0 {
            indicators.push("Low sleep efficiency detected".to_string());
        }
        if deep_sleep_pct < 13.0 {
            indicators.push("Insufficient deep sleep percentage".to_string());
        }
        if sleep_consistency < 60.0 {
            indicators.push("Inconsistent sleep schedule".to_string());
        }

        let avg_wake_count: f64 =
            sleep_data.iter().map(|s| s.wake_count as f64).sum::<f64>() / sleep_data.len() as f64;
        if avg_wake_count > 4.0 {
            indicators.push("Frequent nighttime awakenings".to_string());
        }

        Ok(SleepQualityAnalysis {
            average_sleep_score: avg_score,
            sleep_efficiency_trend: efficiency_trend,
            deep_sleep_percentage: deep_sleep_pct,
            sleep_consistency_score: sleep_consistency,
            health_decline_indicators: indicators,
        })
    }

    pub async fn detect_sleep_disorders(
        &self,
        sleep_patterns: &SleepPatternHistory,
    ) -> Result<Vec<SleepDisorderIndicator>, DetectionError> {
        if sleep_patterns.sessions.is_empty() {
            return Err(DetectionError::InsufficientHistory(
                "No sleep session data".to_string(),
            ));
        }

        let sessions = &sleep_patterns.sessions;
        let mut indicators = Vec::new();

        // Check for insomnia patterns
        let low_efficiency_count = sessions.iter().filter(|s| s.efficiency < 70.0).count();
        let low_efficiency_pct = low_efficiency_count as f64 / sessions.len() as f64 * 100.0;
        if low_efficiency_pct > 30.0 {
            indicators.push(SleepDisorderIndicator {
                disorder_type: "Insomnia".to_string(),
                confidence: (low_efficiency_pct / 100.0).min(0.95),
                evidence: vec![format!(
                    "{:.0}% of nights with sleep efficiency below 70%",
                    low_efficiency_pct
                )],
                severity: if low_efficiency_pct > 50.0 {
                    SeverityLevel::High
                } else {
                    SeverityLevel::Moderate
                },
            });
        }

        // Check for sleep apnea indicators (frequent waking)
        let avg_wake_count: f64 =
            sessions.iter().map(|s| s.wake_count as f64).sum::<f64>() / sessions.len() as f64;
        if avg_wake_count > 5.0 {
            indicators.push(SleepDisorderIndicator {
                disorder_type: "Sleep Apnea Indicator".to_string(),
                confidence: ((avg_wake_count - 5.0) / 10.0).clamp(0.3, 0.8),
                evidence: vec![format!(
                    "Average of {:.1} awakenings per night",
                    avg_wake_count
                )],
                severity: if avg_wake_count > 8.0 {
                    SeverityLevel::High
                } else {
                    SeverityLevel::Moderate
                },
            });
        }

        // Check for insufficient deep sleep
        let total_sleep: u32 = sessions.iter().map(|s| s.duration_minutes).sum();
        let total_deep: u32 = sessions
            .iter()
            .flat_map(|s| &s.stages)
            .filter(|st| st.stage == SleepStage::Deep)
            .map(|st| st.duration_minutes)
            .sum();
        let deep_pct = if total_sleep > 0 {
            total_deep as f64 / total_sleep as f64 * 100.0
        } else {
            0.0
        };

        if deep_pct < 10.0 && !sessions.is_empty() {
            indicators.push(SleepDisorderIndicator {
                disorder_type: "Deep Sleep Deficiency".to_string(),
                confidence: ((10.0 - deep_pct) / 10.0).clamp(0.3, 0.9),
                evidence: vec![format!(
                    "Deep sleep only {:.1}% of total sleep (normal: 13-23%)",
                    deep_pct
                )],
                severity: if deep_pct < 5.0 {
                    SeverityLevel::High
                } else {
                    SeverityLevel::Moderate
                },
            });
        }

        Ok(indicators)
    }

    pub async fn get_sleep_quality_score(&self, sleep_data: &[SleepSession]) -> f64 {
        if sleep_data.is_empty() {
            return 50.0;
        }

        let avg_efficiency: f64 =
            sleep_data.iter().map(|s| s.efficiency).sum::<f64>() / sleep_data.len() as f64;
        let avg_duration: f64 = sleep_data
            .iter()
            .map(|s| s.duration_minutes as f64)
            .sum::<f64>()
            / sleep_data.len() as f64;

        let efficiency_score = (avg_efficiency / 100.0) * 50.0;
        let duration_score = if (420.0..=540.0).contains(&avg_duration) {
            50.0
        } else if avg_duration < 420.0 {
            (avg_duration / 420.0 * 50.0).max(0.0)
        } else {
            ((600.0 - avg_duration) / 60.0 * 50.0).max(0.0)
        };

        (efficiency_score + duration_score).clamp(0.0, 100.0)
    }
}

fn calculate_sleep_consistency(sessions: &[SleepSession]) -> f64 {
    if sessions.len() < 2 {
        return 100.0;
    }

    let durations: Vec<f64> = sessions.iter().map(|s| s.duration_minutes as f64).collect();
    let mean = durations.iter().sum::<f64>() / durations.len() as f64;
    let variance =
        durations.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / durations.len() as f64;
    let std_dev = variance.sqrt();
    let cv = if mean > 0.0 { std_dev / mean } else { 1.0 };

    ((1.0 - cv) * 100.0).clamp(0.0, 100.0)
}

fn classify_efficiency_trend(sessions: &[SleepSession]) -> Trend {
    if sessions.len() < 2 {
        return Trend::Stable;
    }

    let split = sessions.len() / 2;
    let first_avg =
        sessions[..split].iter().map(|s| s.efficiency).sum::<f64>() / split.max(1) as f64;
    let second_avg = sessions[split..].iter().map(|s| s.efficiency).sum::<f64>()
        / (sessions.len() - split).max(1) as f64;

    let change = second_avg - first_avg;
    match change {
        c if c > 3.0 => Trend::Improving,
        c if c > -3.0 => Trend::Stable,
        c if c > -8.0 => Trend::Declining,
        _ => Trend::RapidDecline,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session(
        efficiency: f64,
        duration: u32,
        deep_mins: u32,
        wake_count: u32,
    ) -> SleepSession {
        SleepSession {
            date: "2025-01-01".to_string(),
            start_time: "22:00".to_string(),
            end_time: "06:00".to_string(),
            duration_minutes: duration,
            efficiency,
            stages: vec![
                SleepStageEntry {
                    stage: SleepStage::Deep,
                    duration_minutes: deep_mins,
                },
                SleepStageEntry {
                    stage: SleepStage::Light,
                    duration_minutes: duration - deep_mins - 60,
                },
                SleepStageEntry {
                    stage: SleepStage::Rem,
                    duration_minutes: 60,
                },
            ],
            sleep_score: Some(75),
            wake_count,
        }
    }

    #[tokio::test]
    async fn test_analyze_sleep_quality_good() {
        let analyzer = SleepPatternAnalyzer;
        let sessions = vec![
            make_session(90.0, 480, 100, 1),
            make_session(88.0, 470, 95, 2),
            make_session(92.0, 490, 110, 1),
        ];
        let result = analyzer.analyze_sleep_quality(&sessions, 1).await.unwrap();
        assert!(result.average_sleep_score > 0.0);
        assert!(result.deep_sleep_percentage > 13.0);
        assert!(result.health_decline_indicators.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_sleep_quality_poor() {
        let analyzer = SleepPatternAnalyzer;
        let sessions = vec![
            make_session(60.0, 300, 20, 6),
            make_session(55.0, 280, 15, 7),
        ];
        let result = analyzer.analyze_sleep_quality(&sessions, 1).await.unwrap();
        assert!(!result.health_decline_indicators.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_sleep_quality_empty() {
        let analyzer = SleepPatternAnalyzer;
        let result = analyzer.analyze_sleep_quality(&[], 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_detect_sleep_disorders_insomnia() {
        let analyzer = SleepPatternAnalyzer;
        let history = SleepPatternHistory {
            sessions: vec![
                make_session(60.0, 300, 30, 5),
                make_session(55.0, 280, 25, 6),
                make_session(65.0, 310, 35, 4),
            ],
            period_weeks: 1,
        };
        let indicators = analyzer.detect_sleep_disorders(&history).await.unwrap();
        assert!(!indicators.is_empty());
        assert!(indicators.iter().any(|i| i.disorder_type == "Insomnia"));
    }

    #[tokio::test]
    async fn test_detect_sleep_disorders_healthy() {
        let analyzer = SleepPatternAnalyzer;
        let history = SleepPatternHistory {
            sessions: vec![
                make_session(90.0, 480, 100, 1),
                make_session(88.0, 470, 95, 2),
            ],
            period_weeks: 1,
        };
        let indicators = analyzer.detect_sleep_disorders(&history).await.unwrap();
        assert!(indicators.is_empty());
    }

    #[tokio::test]
    async fn test_sleep_quality_score_good() {
        let analyzer = SleepPatternAnalyzer;
        let sessions = vec![make_session(90.0, 480, 100, 1)];
        let score = analyzer.get_sleep_quality_score(&sessions).await;
        assert!(score > 70.0);
    }

    #[tokio::test]
    async fn test_sleep_quality_score_empty() {
        let analyzer = SleepPatternAnalyzer;
        let score = analyzer.get_sleep_quality_score(&[]).await;
        assert!((score - 50.0).abs() < f64::EPSILON);
    }
}

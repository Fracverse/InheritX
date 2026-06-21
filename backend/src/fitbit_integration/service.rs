use super::activity::ActivityAnalyzer;
use super::client::FitbitWebAPIClient;
use super::errors::FitbitError;
use super::heart_rate::HeartRateAnalyzer;
use super::sleep::SleepPatternAnalyzer;
use super::stress::StressLevelMonitor;
use super::types::*;

pub struct FitbitIntegrationService {
    pub fitbit_client: FitbitWebAPIClient,
    pub heart_rate_analyzer: HeartRateAnalyzer,
    pub sleep_analyzer: SleepPatternAnalyzer,
    pub stress_monitor: StressLevelMonitor,
    pub activity_analyzer: ActivityAnalyzer,
}

impl FitbitIntegrationService {
    pub fn new(fitbit_client: FitbitWebAPIClient) -> Self {
        Self {
            fitbit_client,
            heart_rate_analyzer: HeartRateAnalyzer,
            sleep_analyzer: SleepPatternAnalyzer,
            stress_monitor: StressLevelMonitor,
            activity_analyzer: ActivityAnalyzer,
        }
    }

    pub fn from_env() -> Option<Self> {
        let client = FitbitWebAPIClient::from_env()?;
        Some(Self::new(client))
    }

    pub async fn authenticate_user(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<FitbitAuth, FitbitError> {
        self.fitbit_client
            .exchange_authorization_code(code, redirect_uri)
            .await
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<FitbitAuth, FitbitError> {
        self.fitbit_client.refresh_access_token(refresh_token).await
    }

    pub async fn get_daily_summary(
        &self,
        access_token: &str,
        user_id: &str,
        date: &str,
    ) -> Result<DailySummary, FitbitError> {
        let activity = self
            .fitbit_client
            .get_activity_data(access_token, user_id, date)
            .await?;

        let heart_rate = self
            .fitbit_client
            .get_heart_rate_data(access_token, user_id, date)
            .await?;

        let sleep_sessions = self
            .fitbit_client
            .get_sleep_data(access_token, user_id, date)
            .await?;

        let sleep_efficiency = if !sleep_sessions.is_empty() {
            sleep_sessions.iter().map(|s| s.efficiency).sum::<f64>() / sleep_sessions.len() as f64
        } else {
            0.0
        };

        let hrv_data = self
            .fitbit_client
            .get_hrv_data(access_token, user_id, date)
            .await
            .ok();

        let stress_score = if let Some(hrv) = hrv_data {
            let activity_data = ActivityData {
                steps: activity.steps,
                active_minutes: activity.active_minutes,
                sedentary_minutes: activity.sedentary_minutes,
                calories_burned: activity.calories_burned,
            };
            self.stress_monitor
                .calculate_daily_stress_score(&hrv, &activity_data)
                .await
                .ok()
                .map(|s| s.score as u32)
        } else {
            None
        };

        Ok(DailySummary {
            steps: activity.steps,
            distance_km: activity.distance_km,
            calories_burned: activity.calories_burned,
            active_minutes: activity.active_minutes,
            resting_heart_rate: heart_rate.resting_heart_rate,
            sleep_efficiency,
            stress_score,
            date: date.to_string(),
        })
    }

    pub async fn monitor_heart_rate_trends(
        &self,
        hr_readings: &[HeartRateReading],
        days: u32,
    ) -> Result<HeartRateTrend, FitbitError> {
        let resting_hr_trend = self
            .heart_rate_analyzer
            .analyze_resting_heart_rate_trend(hr_readings, days)
            .await
            .map_err(|e| FitbitError::ApiRequestFailed(e.to_string()))?;

        let hr_data = HeartRateHistory {
            readings: hr_readings.to_vec(),
            daily_resting_hr: vec![],
        };
        let activity_data = ActivityHistory {
            daily_activities: vec![],
        };

        let cardiovascular_fitness = self
            .heart_rate_analyzer
            .calculate_cardiovascular_fitness(&hr_data, &activity_data)
            .await
            .unwrap_or(CardiovascularFitnessScore {
                score: 50.0,
                trend: Trend::Stable,
                vo2_max_estimate: None,
                fitness_level: "Unknown".to_string(),
            });

        Ok(HeartRateTrend {
            resting_hr_trend,
            anomalies: vec![],
            cardiovascular_fitness,
        })
    }

    pub async fn analyze_sleep_quality(
        &self,
        sleep_data: &[SleepSession],
        weeks: u32,
    ) -> Result<SleepQualityAnalysis, FitbitError> {
        self.sleep_analyzer
            .analyze_sleep_quality(sleep_data, weeks)
            .await
            .map_err(|e| FitbitError::ApiRequestFailed(e.to_string()))
    }

    pub async fn calculate_overall_health_score(
        &self,
        hr_readings: &[HeartRateReading],
        sleep_data: &[SleepSession],
        activity_data: &[DailyActivity],
        stress_history: &[StressScore],
    ) -> Result<OverallHealthScore, FitbitError> {
        let heart_rate_score = self
            .heart_rate_analyzer
            .get_heart_rate_score(hr_readings)
            .await;
        let sleep_score = self
            .sleep_analyzer
            .get_sleep_quality_score(sleep_data)
            .await;
        let activity_score = self
            .activity_analyzer
            .get_activity_score(activity_data)
            .await;
        let stress_score = self
            .stress_monitor
            .get_stress_level_score(stress_history)
            .await;

        let composite = (heart_rate_score + sleep_score + activity_score + stress_score) / 4.0;

        let mut decline_indicators = Vec::new();

        if heart_rate_score < 40.0 {
            decline_indicators.push(DeclineIndicator {
                area: "Heart Rate".to_string(),
                severity: SeverityLevel::High,
                description: "Concerning heart rate patterns detected".to_string(),
                trend: Trend::Declining,
            });
        }

        if sleep_score < 40.0 {
            decline_indicators.push(DeclineIndicator {
                area: "Sleep".to_string(),
                severity: SeverityLevel::High,
                description: "Significant sleep quality decline".to_string(),
                trend: Trend::Declining,
            });
        }

        if activity_score < 40.0 {
            decline_indicators.push(DeclineIndicator {
                area: "Activity".to_string(),
                severity: SeverityLevel::High,
                description: "Notable reduction in physical activity".to_string(),
                trend: Trend::Declining,
            });
        }

        if stress_score < 40.0 {
            decline_indicators.push(DeclineIndicator {
                area: "Stress".to_string(),
                severity: SeverityLevel::High,
                description: "Elevated chronic stress levels".to_string(),
                trend: Trend::Declining,
            });
        }

        Ok(OverallHealthScore {
            composite_score: composite,
            individual_scores: IndividualScores {
                heart_rate: heart_rate_score,
                sleep: sleep_score,
                activity: activity_score,
                stress: stress_score,
            },
            decline_indicators,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service() -> FitbitIntegrationService {
        let client = FitbitWebAPIClient::new("test_id".to_string(), "test_secret".to_string());
        FitbitIntegrationService::new(client)
    }

    #[tokio::test]
    async fn test_overall_health_score_healthy() {
        let service = make_service();

        let hr = vec![HeartRateReading {
            timestamp: "2025-01-01T08:00:00Z".to_string(),
            bpm: 65,
            confidence: 0.95,
        }];
        let sleep = vec![SleepSession {
            date: "2025-01-01".to_string(),
            start_time: "22:00".to_string(),
            end_time: "06:00".to_string(),
            duration_minutes: 480,
            efficiency: 90.0,
            stages: vec![],
            sleep_score: Some(85),
            wake_count: 1,
        }];
        let activity = vec![DailyActivity {
            date: "2025-01-01".to_string(),
            steps: 10000,
            distance_km: 7.5,
            calories_burned: 2200,
            active_minutes: 45,
            sedentary_minutes: 480,
            floors_climbed: 10,
        }];
        let stress = vec![StressScore {
            date: "2025-01-01".to_string(),
            score: 25.0,
            level: StressLevel::Low,
            contributing_factors: vec![],
        }];

        let result = service
            .calculate_overall_health_score(&hr, &sleep, &activity, &stress)
            .await
            .unwrap();

        assert!(result.composite_score > 60.0);
        assert!(result.decline_indicators.is_empty());
    }

    #[tokio::test]
    async fn test_overall_health_score_poor() {
        let service = make_service();

        let hr = vec![HeartRateReading {
            timestamp: "2025-01-01T08:00:00Z".to_string(),
            bpm: 120,
            confidence: 0.9,
        }];
        let sleep = vec![SleepSession {
            date: "2025-01-01".to_string(),
            start_time: "01:00".to_string(),
            end_time: "04:00".to_string(),
            duration_minutes: 180,
            efficiency: 50.0,
            stages: vec![],
            sleep_score: Some(30),
            wake_count: 8,
        }];
        let activity = vec![DailyActivity {
            date: "2025-01-01".to_string(),
            steps: 1000,
            distance_km: 0.8,
            calories_burned: 1200,
            active_minutes: 5,
            sedentary_minutes: 800,
            floors_climbed: 0,
        }];
        let stress = vec![StressScore {
            date: "2025-01-01".to_string(),
            score: 85.0,
            level: StressLevel::VeryHigh,
            contributing_factors: vec!["High stress".to_string()],
        }];

        let result = service
            .calculate_overall_health_score(&hr, &sleep, &activity, &stress)
            .await
            .unwrap();

        assert!(result.composite_score < 50.0);
        assert!(!result.decline_indicators.is_empty());
    }

    #[tokio::test]
    async fn test_overall_health_score_empty_data() {
        let service = make_service();
        let result = service
            .calculate_overall_health_score(&[], &[], &[], &[])
            .await
            .unwrap();
        assert!((result.composite_score - 50.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_analyze_sleep_quality_via_service() {
        let service = make_service();
        let sessions = vec![SleepSession {
            date: "2025-01-01".to_string(),
            start_time: "22:00".to_string(),
            end_time: "06:00".to_string(),
            duration_minutes: 480,
            efficiency: 88.0,
            stages: vec![
                SleepStageEntry {
                    stage: SleepStage::Deep,
                    duration_minutes: 100,
                },
                SleepStageEntry {
                    stage: SleepStage::Light,
                    duration_minutes: 300,
                },
                SleepStageEntry {
                    stage: SleepStage::Rem,
                    duration_minutes: 80,
                },
            ],
            sleep_score: Some(80),
            wake_count: 2,
        }];
        let result = service.analyze_sleep_quality(&sessions, 1).await.unwrap();
        assert!(result.average_sleep_score > 0.0);
    }

    #[tokio::test]
    async fn test_monitor_heart_rate_trends() {
        let service = make_service();
        let readings: Vec<HeartRateReading> = (0..14)
            .map(|i| HeartRateReading {
                timestamp: format!("2025-01-{:02}T08:00:00Z", i + 1),
                bpm: 65 + (i % 3) as u32,
                confidence: 0.95,
            })
            .collect();
        let result = service
            .monitor_heart_rate_trends(&readings, 14)
            .await
            .unwrap();
        assert!(matches!(
            result.resting_hr_trend.trend,
            Trend::Stable | Trend::Improving
        ));
    }
}

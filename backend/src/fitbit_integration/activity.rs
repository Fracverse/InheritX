use super::errors::{AnalysisError, AssessmentError};
use super::types::*;

pub struct ActivityAnalyzer;

impl ActivityAnalyzer {
    pub async fn track_activity_decline(
        &self,
        activity_history: &[DailyActivity],
        baseline_days: u32,
    ) -> Result<ActivityDeclineAnalysis, AnalysisError> {
        if activity_history.is_empty() {
            return Err(AnalysisError::InsufficientData(
                "No activity data available".to_string(),
            ));
        }
        if baseline_days == 0 {
            return Err(AnalysisError::InvalidParameter(
                "Baseline days must be greater than 0".to_string(),
            ));
        }

        let baseline_end = (baseline_days as usize).min(activity_history.len());
        let baseline_steps: u32 = activity_history[..baseline_end]
            .iter()
            .map(|a| a.steps)
            .sum::<u32>()
            / baseline_end.max(1) as u32;

        let recent_start = activity_history.len().saturating_sub(baseline_end);
        let current_steps: u32 = activity_history[recent_start..]
            .iter()
            .map(|a| a.steps)
            .sum::<u32>()
            / (activity_history.len() - recent_start).max(1) as u32;

        let decline_pct = if baseline_steps > 0 {
            ((baseline_steps as f64 - current_steps as f64) / baseline_steps as f64 * 100.0)
                .max(0.0)
        } else {
            0.0
        };

        let decline_weeks = if decline_pct > 10.0 {
            (activity_history.len() as u32).saturating_sub(baseline_days) / 7
        } else {
            0
        };

        let mut concerns = Vec::new();
        if decline_pct > 50.0 {
            concerns.push(MobilityConcern {
                concern_type: "Severe activity decline".to_string(),
                severity: SeverityLevel::Critical,
                description: format!(
                    "Steps decreased by {:.1}% from baseline of {}",
                    decline_pct, baseline_steps
                ),
            });
        } else if decline_pct > 30.0 {
            concerns.push(MobilityConcern {
                concern_type: "Significant activity decline".to_string(),
                severity: SeverityLevel::High,
                description: format!("Steps decreased by {:.1}% from baseline", decline_pct),
            });
        } else if decline_pct > 15.0 {
            concerns.push(MobilityConcern {
                concern_type: "Moderate activity decline".to_string(),
                severity: SeverityLevel::Moderate,
                description: format!("Steps decreased by {:.1}% from baseline", decline_pct),
            });
        }

        let avg_active_minutes: f64 = activity_history
            .iter()
            .map(|a| a.active_minutes as f64)
            .sum::<f64>()
            / activity_history.len() as f64;

        if avg_active_minutes < 15.0 {
            concerns.push(MobilityConcern {
                concern_type: "Very low active minutes".to_string(),
                severity: SeverityLevel::High,
                description: format!(
                    "Average active minutes ({:.0}) well below recommended 30 minutes",
                    avg_active_minutes
                ),
            });
        }

        // Inheritance trigger score: higher = more likely health decline
        let trigger_score = (decline_pct / 100.0 * 0.6
            + if avg_active_minutes < 30.0 { 0.3 } else { 0.0 }
            + if current_steps < 3000 { 0.1 } else { 0.0 })
        .clamp(0.0, 1.0);

        Ok(ActivityDeclineAnalysis {
            baseline_average_steps: baseline_steps,
            current_average_steps: current_steps,
            decline_percentage: decline_pct,
            decline_duration_weeks: decline_weeks,
            mobility_concerns: concerns,
            inheritance_trigger_score: trigger_score,
        })
    }

    pub async fn analyze_exercise_capacity(
        &self,
        workout_data: &[WorkoutSession],
        months: u32,
    ) -> Result<ExerciseCapacityTrend, AnalysisError> {
        if workout_data.is_empty() {
            return Err(AnalysisError::InsufficientData(
                "No workout data available".to_string(),
            ));
        }
        if months == 0 {
            return Err(AnalysisError::InvalidParameter(
                "Months must be greater than 0".to_string(),
            ));
        }

        let split = workout_data.len() / 2;
        let (first_half, second_half) = workout_data.split_at(split.max(1));

        let first_avg_duration = first_half
            .iter()
            .map(|w| w.duration_minutes as f64)
            .sum::<f64>()
            / first_half.len().max(1) as f64;
        let second_avg_duration = second_half
            .iter()
            .map(|w| w.duration_minutes as f64)
            .sum::<f64>()
            / second_half.len().max(1) as f64;

        let first_avg_calories = first_half
            .iter()
            .map(|w| w.calories_burned as f64)
            .sum::<f64>()
            / first_half.len().max(1) as f64;
        let second_avg_calories = second_half
            .iter()
            .map(|w| w.calories_burned as f64)
            .sum::<f64>()
            / second_half.len().max(1) as f64;

        let duration_change = second_avg_duration - first_avg_duration;
        let intensity_change = if first_avg_calories > 0.0 {
            (second_avg_calories - first_avg_calories) / first_avg_calories * 100.0
        } else {
            0.0
        };

        let capacity_score = (second_avg_duration / 60.0 * 50.0
            + second_avg_calories / 500.0 * 50.0)
            .clamp(0.0, 100.0);

        let trend = match duration_change {
            c if c > 5.0 => Trend::Improving,
            c if c > -5.0 => Trend::Stable,
            c if c > -15.0 => Trend::Declining,
            _ => Trend::RapidDecline,
        };

        Ok(ExerciseCapacityTrend {
            trend,
            average_workout_duration_change: duration_change,
            average_intensity_change: intensity_change,
            capacity_score,
        })
    }

    pub async fn detect_mobility_issues(
        &self,
        step_data: &[DailySteps],
        weeks: u32,
    ) -> Result<MobilityAssessment, AssessmentError> {
        if step_data.is_empty() {
            return Err(AssessmentError::InsufficientData);
        }
        if weeks == 0 {
            return Err(AssessmentError::Failed(
                "Weeks must be greater than 0".to_string(),
            ));
        }

        let avg_steps: f64 =
            step_data.iter().map(|s| s.steps as f64).sum::<f64>() / step_data.len() as f64;

        let step_values: Vec<f64> = step_data.iter().map(|s| s.steps as f64).collect();
        let variance = step_values
            .iter()
            .map(|s| (s - avg_steps).powi(2))
            .sum::<f64>()
            / step_values.len() as f64;
        let std_dev = variance.sqrt();
        let consistency = if avg_steps > 0.0 {
            ((1.0 - std_dev / avg_steps) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        // Detect decline by comparing recent to older data
        let split = step_data.len() / 2;
        let older_avg = if split > 0 {
            step_data[..split]
                .iter()
                .map(|s| s.steps as f64)
                .sum::<f64>()
                / split as f64
        } else {
            avg_steps
        };
        let recent_avg = step_data[split..]
            .iter()
            .map(|s| s.steps as f64)
            .sum::<f64>()
            / (step_data.len() - split).max(1) as f64;

        let decline_detected = recent_avg < older_avg * 0.8;

        let mobility_score = (avg_steps / 10000.0 * 70.0 + consistency * 0.3).clamp(0.0, 100.0);

        let mut concerns = Vec::new();
        if avg_steps < 3000.0 {
            concerns.push(MobilityConcern {
                concern_type: "Very low daily steps".to_string(),
                severity: SeverityLevel::Critical,
                description: format!(
                    "Average steps ({:.0}) far below healthy threshold",
                    avg_steps
                ),
            });
        } else if avg_steps < 5000.0 {
            concerns.push(MobilityConcern {
                concern_type: "Low daily steps".to_string(),
                severity: SeverityLevel::High,
                description: format!("Average steps ({:.0}) below recommended minimum", avg_steps),
            });
        }

        if decline_detected {
            let decline_pct = ((older_avg - recent_avg) / older_avg * 100.0).max(0.0);
            concerns.push(MobilityConcern {
                concern_type: "Step count decline".to_string(),
                severity: if decline_pct > 40.0 {
                    SeverityLevel::Critical
                } else {
                    SeverityLevel::High
                },
                description: format!("Steps declined by {:.1}% over {} weeks", decline_pct, weeks),
            });
        }

        Ok(MobilityAssessment {
            mobility_score,
            step_consistency: consistency,
            decline_detected,
            concerns,
        })
    }

    pub async fn get_activity_score(&self, activity_data: &[DailyActivity]) -> f64 {
        if activity_data.is_empty() {
            return 50.0;
        }

        let avg_steps: f64 =
            activity_data.iter().map(|a| a.steps as f64).sum::<f64>() / activity_data.len() as f64;
        let avg_active: f64 = activity_data
            .iter()
            .map(|a| a.active_minutes as f64)
            .sum::<f64>()
            / activity_data.len() as f64;

        let step_score = (avg_steps / 10000.0 * 50.0).clamp(0.0, 50.0);
        let active_score = (avg_active / 30.0 * 50.0).clamp(0.0, 50.0);

        (step_score + active_score).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_daily_activity(steps: u32, active_min: u32) -> DailyActivity {
        DailyActivity {
            date: "2025-01-01".to_string(),
            steps,
            distance_km: steps as f64 * 0.0008,
            calories_burned: 1800 + steps / 5,
            active_minutes: active_min,
            sedentary_minutes: 600,
            floors_climbed: 5,
        }
    }

    #[tokio::test]
    async fn test_track_activity_decline_significant() {
        let analyzer = ActivityAnalyzer;
        let mut history = Vec::new();
        // Baseline: high activity
        for _ in 0..14 {
            history.push(make_daily_activity(10000, 45));
        }
        // Recent: low activity
        for _ in 0..14 {
            history.push(make_daily_activity(4000, 15));
        }
        let result = analyzer.track_activity_decline(&history, 14).await.unwrap();
        assert!(result.decline_percentage > 50.0);
        assert!(!result.mobility_concerns.is_empty());
        assert!(result.inheritance_trigger_score > 0.3);
    }

    #[tokio::test]
    async fn test_track_activity_decline_none() {
        let analyzer = ActivityAnalyzer;
        let history: Vec<DailyActivity> = (0..14).map(|_| make_daily_activity(9000, 40)).collect();
        let result = analyzer.track_activity_decline(&history, 7).await.unwrap();
        assert!(result.decline_percentage < 10.0);
    }

    #[tokio::test]
    async fn test_track_activity_decline_empty() {
        let analyzer = ActivityAnalyzer;
        let result = analyzer.track_activity_decline(&[], 7).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_exercise_capacity_declining() {
        let analyzer = ActivityAnalyzer;
        let workouts = vec![
            WorkoutSession {
                date: "2025-01-01".to_string(),
                activity_type: "Running".to_string(),
                duration_minutes: 45,
                calories_burned: 400,
                average_heart_rate: Some(145),
                peak_heart_rate: Some(170),
            },
            WorkoutSession {
                date: "2025-02-01".to_string(),
                activity_type: "Running".to_string(),
                duration_minutes: 20,
                calories_burned: 180,
                average_heart_rate: Some(150),
                peak_heart_rate: Some(175),
            },
        ];
        let result = analyzer
            .analyze_exercise_capacity(&workouts, 2)
            .await
            .unwrap();
        assert!(matches!(
            result.trend,
            Trend::Declining | Trend::RapidDecline
        ));
    }

    #[tokio::test]
    async fn test_analyze_exercise_capacity_empty() {
        let analyzer = ActivityAnalyzer;
        let result = analyzer.analyze_exercise_capacity(&[], 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_detect_mobility_issues_low_steps() {
        let analyzer = ActivityAnalyzer;
        let steps: Vec<DailySteps> = (0..14)
            .map(|i| DailySteps {
                date: format!("2025-01-{:02}", i + 1),
                steps: 2000,
            })
            .collect();
        let result = analyzer.detect_mobility_issues(&steps, 2).await.unwrap();
        assert!(!result.concerns.is_empty());
        assert!(result.mobility_score < 50.0);
    }

    #[tokio::test]
    async fn test_detect_mobility_issues_healthy() {
        let analyzer = ActivityAnalyzer;
        let steps: Vec<DailySteps> = (0..14)
            .map(|i| DailySteps {
                date: format!("2025-01-{:02}", i + 1),
                steps: 9000,
            })
            .collect();
        let result = analyzer.detect_mobility_issues(&steps, 2).await.unwrap();
        assert!(!result.decline_detected);
    }

    #[tokio::test]
    async fn test_detect_mobility_issues_empty() {
        let analyzer = ActivityAnalyzer;
        let result = analyzer.detect_mobility_issues(&[], 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_activity_score_high() {
        let analyzer = ActivityAnalyzer;
        let data = vec![make_daily_activity(12000, 45)];
        let score = analyzer.get_activity_score(&data).await;
        assert!(score > 80.0);
    }

    #[tokio::test]
    async fn test_activity_score_empty() {
        let analyzer = ActivityAnalyzer;
        let score = analyzer.get_activity_score(&[]).await;
        assert!((score - 50.0).abs() < f64::EPSILON);
    }
}

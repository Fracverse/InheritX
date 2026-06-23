//! Unit tests for the Gradual Health Decline Monitoring Service

#[cfg(test)]
mod tests {
    use crate::gradual_health_decline::{
        CardiovascularBaseline, CognitiveBaseline, HealthBaselineCalculator,
        HealthDeclineAnalyzer, HealthDeclineAssessment, HealthSnapshot, HealthSystemDecline,
        HealthSystemType, HealthTrendPredictor, InheritanceStageManager, InheritancePreferences,
        QualityOfLifeAssessor, UserActivityData, HealthBaseline, ActivityBaseline, SleepBaseline,
        GradualHealthDeclineService,
    };
    use crate::fitbit_integration::types::{HeartRateReading, ActivityData};

    fn sample_baseline() -> CardiovascularBaseline {
        CardiovascularBaseline {
            resting_heart_rate: 70.0,
            heart_rate_variability: 50.0,
            blood_pressure_systolic: 120.0,
            blood_pressure_diastolic: 80.0,
            vo2_max: Some(40.0),
        }
    }

    fn sample_cognitive_baseline() -> CognitiveBaseline {
        CognitiveBaseline {
            memory_score: 80.0,
            processing_speed: 75.0,
            attention_span: 78.0,
            executive_function: 80.0,
            language_ability: 82.0,
        }
    }

    fn sample_activity_baseline() -> ActivityBaseline {
        ActivityBaseline {
            average_daily_steps: 8000.0,
            average_active_minutes: 30.0,
            exercise_capacity: 70.0,
            mobility_index: 0.8,
        }
    }

    fn sample_sleep_baseline() -> SleepBaseline {
        SleepBaseline {
            average_duration_hours: 7.5,
            sleep_efficiency: 85.0,
            deep_sleep_percentage: 20.0,
            rem_sleep_percentage: 22.0,
            wake_frequency: 2.0,
        }
    }

    #[tokio::test]
    async fn test_cardiovascular_decline_detection() {
        let analyzer = HealthDeclineAnalyzer::default();
        let baseline = sample_baseline();
        let readings = vec![
            HeartRateReading {
                timestamp: "2024-01-01T08:00:00Z".to_string(),
                bpm: 85,
                confidence: 0.9,
            },
            HeartRateReading {
                timestamp: "2024-01-02T08:00:00Z".to_string(),
                bpm: 88,
                confidence: 0.9,
            },
        ];

        let result = analyzer
            .detect_cardiovascular_decline(&readings, &baseline)
            .await
            .unwrap();

        assert!(result.fitness_decline_percentage >= 0.0);
    }

    #[tokio::test]
    async fn test_activity_decline_detection() {
        let analyzer = HealthDeclineAnalyzer::default();
        let baseline = sample_activity_baseline();
        let history = vec![
            ActivityData {
                steps: 6000,
                active_minutes: 20,
                sedentary_minutes: 300,
                calories_burned: 2000,
            },
            ActivityData {
                steps: 5500,
                active_minutes: 18,
                sedentary_minutes: 320,
                calories_burned: 1900,
            },
        ];

        let result = analyzer
            .analyze_activity_decline(&history, &baseline, 1)
            .await
            .unwrap();

        assert!(result.decline_percentage >= 0.0);
        assert!(result.decline_percentage <= 100.0);
    }

    #[tokio::test]
    async fn test_cognitive_decline_detection() {
        let analyzer = HealthDeclineAnalyzer::default();
        let baseline = sample_cognitive_baseline();
        let assessments = vec![75.0, 73.0, 72.0];

        let result = analyzer
            .assess_cognitive_decline(&assessments, &baseline)
            .await
            .unwrap();

        assert!(result.memory_decline_percentage >= 0.0);
    }

    #[tokio::test]
    async fn test_sleep_deterioration_detection() {
        let analyzer = HealthDeclineAnalyzer::default();
        let baseline = sample_sleep_baseline();
        let sleep_data = vec![6.5, 6.0, 5.8, 6.2];

        let result = analyzer
            .evaluate_sleep_deterioration(&sleep_data, &baseline)
            .await
            .unwrap();

        assert!(result.efficiency_decline >= 0.0);
    }

    #[tokio::test]
    async fn test_health_trajectory_prediction() {
        let predictor = HealthTrendPredictor::default();
        let assessment = HealthDeclineAssessment {
            user_id: "user1".to_string(),
            assessment_date: chrono::Utc::now().timestamp_millis() as u64,
            overall_decline_score: 40.0,
            decline_velocity: 1.0,
            affected_systems: vec![],
            predictive_timeline: crate::gradual_health_decline::HealthTimeline {
                estimated_decline_months: 12,
                critical_threshold_date: None,
                milestone_dates: vec![],
                confidence: 0.75,
            },
            recommended_inheritance_stages: vec![],
        };

        let result = predictor
            .predict_health_trajectory(&assessment, 6)
            .await
            .unwrap();

        assert_eq!(result.prediction_horizon_months, 6);
        assert_eq!(result.predicted_decline_curve.len(), 6);
    }

    #[tokio::test]
    async fn test_inheritance_stage_design() {
        let manager = InheritanceStageManager::default();
        let assessment = HealthDeclineAssessment {
            user_id: "user1".to_string(),
            assessment_date: chrono::Utc::now().timestamp_millis() as u64,
            overall_decline_score: 45.0,
            decline_velocity: 2.0,
            affected_systems: vec![
                HealthSystemDecline {
                    system_type: HealthSystemType::Cardiovascular,
                    decline_percentage: 20.0,
                    decline_duration_months: 6,
                    severity: crate::gradual_health_decline::DeclineSeverity::Moderate,
                    contributing_factors: vec!["Hypertension".to_string()],
                },
            ],
            predictive_timeline: crate::gradual_health_decline::HealthTimeline {
                estimated_decline_months: 12,
                critical_threshold_date: None,
                milestone_dates: vec![],
                confidence: 0.75,
            },
            recommended_inheritance_stages: vec![],
        };

        let preferences = InheritancePreferences {
            plan_id: 1,
            primary_beneficiary: "beneficiary1".to_string(),
            total_allocation_percentage: 100.0,
            auto_release_enabled: true,
            medical_verification_required: false,
        };

        let design = manager
            .design_inheritance_stages(&assessment, &preferences)
            .await
            .unwrap();

        assert!(!design.designed_stages.is_empty());
    }

    #[tokio::test]
    async fn test_quality_of_life_assessment() {
        let assessor = QualityOfLifeAssessor::default();
        let user_data = UserActivityData {
            user_id: "user1".to_string(),
            recorded_at: chrono::Utc::now().timestamp_millis() as u64,
            daily_activities: vec![crate::gradual_health_decline::DailyActivitySummary {
                date: "2024-01-01".to_string(),
                bathing: true,
                dressing: true,
                toileting: true,
                transferring: true,
                feeding: true,
                cooking: true,
                cleaning: true,
                shopping: true,
                medication_management: true,
                transportation: true,
            }],
            assistive_devices_used: vec![],
            home_modifications: vec![],
            care_recipient_status: false,
        };

        let adl = assessor
            .assess_activities_of_daily_living(&user_data)
            .await
            .unwrap();

        assert_eq!(adl.overall_adl_score, 100.0);
    }

    #[tokio::test]
    async fn test_baseline_calculation() {
        let calculator = HealthBaselineCalculator::default();
        let data_points = vec![
            crate::gradual_health_decline::HealthDataPoint {
                timestamp: chrono::Utc::now().timestamp_millis() as u64 - 86400000,
                system_type: HealthSystemType::Cardiovascular,
                value: 72.0,
                source: "wearable".to_string(),
                confidence: 0.9,
            },
            crate::gradual_health_decline::HealthDataPoint {
                timestamp: chrono::Utc::now().timestamp_millis() as u64 - 172800000,
                system_type: HealthSystemType::Cardiovascular,
                value: 70.0,
                source: "wearable".to_string(),
                confidence: 0.9,
            },
        ];

        let result = calculator
            .calculate_comprehensive_baseline("user1", &data_points, 30)
            .await
            .unwrap();

        assert_eq!(result.user_id, "user1");
    }

    #[tokio::test]
    async fn test_gradual_health_decline_service() {
        let service = GradualHealthDeclineService::default();

        let baseline = service
            .establish_health_baseline("user1", 30)
            .await
            .unwrap();

        assert_eq!(baseline.user_id, "user1");

        let assessment = service.monitor_health_decline("user1").await.unwrap();
        assert_eq!(assessment.user_id, "user1");

        let stages = service
            .calculate_inheritance_stages(&assessment, 1)
            .await
            .unwrap();

        assert_eq!(stages.plan_id, 1);

        let release = service.trigger_staged_release(1, 1).await.unwrap();
        assert!(release.success);
    }
}

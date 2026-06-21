use serde::{Deserialize, Serialize};

// ─── Enums ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trend {
    Improving,
    Stable,
    Declining,
    RapidDecline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SleepStage {
    Awake,
    Light,
    Deep,
    Rem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityLevel {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StressLevel {
    Low,
    Moderate,
    High,
    VeryHigh,
}

// ─── Authentication ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitbitAuth {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub expires_at: i64,
    pub scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitbitTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub expires_in: i64,
    pub scope: String,
    pub token_type: String,
}

// ─── Heart Rate ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateZone {
    pub name: String,
    pub min_hr: u32,
    pub max_hr: u32,
    pub minutes: u32,
    pub calories_out: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitbitHeartRateData {
    pub resting_heart_rate: u32,
    pub fat_burn_zone: HeartRateZone,
    pub cardio_zone: HeartRateZone,
    pub peak_zone: HeartRateZone,
    pub heart_rate_variability: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateReading {
    pub timestamp: String,
    pub bpm: u32,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateBaseline {
    pub average_resting_hr: f64,
    pub standard_deviation: f64,
    pub measurement_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateAnomaly {
    pub timestamp: String,
    pub observed_hr: u32,
    pub expected_range_low: f64,
    pub expected_range_high: f64,
    pub severity: SeverityLevel,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestingHRTrend {
    pub trend: Trend,
    pub start_resting_hr: f64,
    pub end_resting_hr: f64,
    pub change_bpm: f64,
    pub period_days: u32,
    pub health_concern: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateHistory {
    pub readings: Vec<HeartRateReading>,
    pub daily_resting_hr: Vec<DailyRestingHR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRestingHR {
    pub date: String,
    pub resting_hr: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardiovascularFitnessScore {
    pub score: f64,
    pub trend: Trend,
    pub vo2_max_estimate: Option<f64>,
    pub fitness_level: String,
}

// ─── Sleep ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepSession {
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub duration_minutes: u32,
    pub efficiency: f64,
    pub stages: Vec<SleepStageEntry>,
    pub sleep_score: Option<u32>,
    pub wake_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepStageEntry {
    pub stage: SleepStage,
    pub duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepQualityAnalysis {
    pub average_sleep_score: f64,
    pub sleep_efficiency_trend: Trend,
    pub deep_sleep_percentage: f64,
    pub sleep_consistency_score: f64,
    pub health_decline_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepPatternHistory {
    pub sessions: Vec<SleepSession>,
    pub period_weeks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepDisorderIndicator {
    pub disorder_type: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub severity: SeverityLevel,
}

// ─── Stress ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HRVReading {
    pub timestamp: String,
    pub rmssd: f64,
    pub sdnn: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressScore {
    pub date: String,
    pub score: f64,
    pub level: StressLevel,
    pub contributing_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicStressAnalysis {
    pub average_stress_score: f64,
    pub trend: Trend,
    pub high_stress_days_percentage: f64,
    pub chronic_stress_detected: bool,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressHistory {
    pub scores: Vec<StressScore>,
    pub period_weeks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub heart_rate_score: f64,
    pub sleep_score: f64,
    pub activity_score: f64,
    pub stress_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressHealthCorrelation {
    pub correlation_coefficient: f64,
    pub stress_impact_areas: Vec<String>,
    pub health_decline_risk: SeverityLevel,
}

// ─── Activity ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityData {
    pub steps: u32,
    pub active_minutes: u32,
    pub sedentary_minutes: u32,
    pub calories_burned: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: String,
    pub steps: u32,
    pub distance_km: f64,
    pub calories_burned: u32,
    pub active_minutes: u32,
    pub sedentary_minutes: u32,
    pub floors_climbed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySteps {
    pub date: String,
    pub steps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutSession {
    pub date: String,
    pub activity_type: String,
    pub duration_minutes: u32,
    pub calories_burned: u32,
    pub average_heart_rate: Option<u32>,
    pub peak_heart_rate: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityHistory {
    pub daily_activities: Vec<DailyActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDeclineAnalysis {
    pub baseline_average_steps: u32,
    pub current_average_steps: u32,
    pub decline_percentage: f64,
    pub decline_duration_weeks: u32,
    pub mobility_concerns: Vec<MobilityConcern>,
    pub inheritance_trigger_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobilityConcern {
    pub concern_type: String,
    pub severity: SeverityLevel,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseCapacityTrend {
    pub trend: Trend,
    pub average_workout_duration_change: f64,
    pub average_intensity_change: f64,
    pub capacity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobilityAssessment {
    pub mobility_score: f64,
    pub step_consistency: f64,
    pub decline_detected: bool,
    pub concerns: Vec<MobilityConcern>,
}

// ─── Daily Summary ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub steps: u32,
    pub distance_km: f64,
    pub calories_burned: u32,
    pub active_minutes: u32,
    pub resting_heart_rate: u32,
    pub sleep_efficiency: f64,
    pub stress_score: Option<u32>,
    pub date: String,
}

// ─── Overall Health Score ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualScores {
    pub heart_rate: f64,
    pub sleep: f64,
    pub activity: f64,
    pub stress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclineIndicator {
    pub area: String,
    pub severity: SeverityLevel,
    pub description: String,
    pub trend: Trend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallHealthScore {
    pub composite_score: f64,
    pub individual_scores: IndividualScores,
    pub decline_indicators: Vec<DeclineIndicator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateTrend {
    pub resting_hr_trend: RestingHRTrend,
    pub anomalies: Vec<HeartRateAnomaly>,
    pub cardiovascular_fitness: CardiovascularFitnessScore,
}

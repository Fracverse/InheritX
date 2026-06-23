use serde::{Deserialize, Serialize};

// ─── Core Enums ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthSystemType {
    Cardiovascular,
    Respiratory,
    Cognitive,
    Mobility,
    Sleep,
    Mental,
    Metabolic,
    Overall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeclineSeverity {
    Mild,
    Moderate,
    Severe,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionType {
    MedicalTreatment,
    LifestyleModification,
    PhysicalTherapy,
    CognitiveTraining,
    MedicationAdjustment,
    SurgicalIntervention,
    PreventiveCare,
    PalliativeCare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Easy,
    Moderate,
    Challenging,
    Complex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerCondition {
    HealthScoreBelow(f64),
    DeclineVelocityAbove(f64),
    SystemCountAbove(u32),
    FunctionalScoreBelow(f64),
    QualityOfLifeBelow(f64),
    CognitiveImpairment,
    MobilityLoss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
    Sms,
    InApp,
    PushNotification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub channels: Vec<NotificationChannel>,
    pub recipients: Vec<String>,
    pub immediate: bool,
}

// ─── Health Data Types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDataPoint {
    pub timestamp: u64,
    pub system_type: HealthSystemType,
    pub value: f64,
    pub source: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub user_id: String,
    pub recorded_at: u64,
    pub cardiovascular_score: Option<f64>,
    pub respiratory_score: Option<f64>,
    pub cognitive_score: Option<f64>,
    pub mobility_score: Option<f64>,
    pub sleep_score: Option<f64>,
    pub mental_score: Option<f64>,
    pub metabolic_score: Option<f64>,
    pub overall_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardiovascularBaseline {
    pub resting_heart_rate: f64,
    pub heart_rate_variability: f64,
    pub blood_pressure_systolic: f64,
    pub blood_pressure_diastolic: f64,
    pub vo2_max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityBaseline {
    pub average_daily_steps: f64,
    pub average_active_minutes: f64,
    pub exercise_capacity: f64,
    pub mobility_index: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepBaseline {
    pub average_duration_hours: f64,
    pub sleep_efficiency: f64,
    pub deep_sleep_percentage: f64,
    pub rem_sleep_percentage: f64,
    pub wake_frequency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveBaseline {
    pub memory_score: f64,
    pub processing_speed: f64,
    pub attention_span: f64,
    pub executive_function: f64,
    pub language_ability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthBaseline {
    pub user_id: String,
    pub established_date: u64,
    pub cardiovascular_baseline: CardiovascularBaseline,
    pub activity_baseline: ActivityBaseline,
    pub sleep_baseline: SleepBaseline,
    pub cognitive_baseline: CognitiveBaseline,
    pub overall_health_score: f64,
    pub age_adjustment_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatedBaseline {
    pub baseline: HealthBaseline,
    pub days_since_establishment: u32,
    pub score_change: f64,
    pub updated_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub user_id: String,
    pub baseline_score: f64,
    pub current_score: f64,
    pub score_delta: f64,
    pub declining_systems: Vec<HealthSystemType>,
    pub improving_systems: Vec<HealthSystemType>,
    pub stable_systems: Vec<HealthSystemType>,
}

// ─── Health Decline Analysis Types ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDeclineAssessment {
    pub user_id: String,
    pub assessment_date: u64,
    pub overall_decline_score: f64,
    pub decline_velocity: f64,
    pub affected_systems: Vec<HealthSystemDecline>,
    pub predictive_timeline: HealthTimeline,
    pub recommended_inheritance_stages: Vec<InheritanceStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSystemDecline {
    pub system_type: HealthSystemType,
    pub decline_percentage: f64,
    pub decline_duration_months: u32,
    pub severity: DeclineSeverity,
    pub contributing_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardiovascularDeclineAnalysis {
    pub heart_rate_trend: TrendDirection,
    pub variability_decline: f64,
    pub blood_pressure_trend: TrendDirection,
    pub fitness_decline_percentage: f64,
    pub severity: DeclineSeverity,
    pub contributing_factors: Vec<String>,
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
    pub severity: DeclineSeverity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveDeclineAnalysis {
    pub memory_decline_percentage: f64,
    pub processing_speed_change: f64,
    pub attention_decline: f64,
    pub functional_impact_score: f64,
    pub severity: DeclineSeverity,
    pub contributing_factors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    RapidlyDeclining,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepDeteriorationAnalysis {
    pub duration_trend: TrendDirection,
    pub efficiency_decline: f64,
    pub deep_sleep_reduction: f64,
    pub wake_frequency_increase: f64,
    pub severity: DeclineSeverity,
    pub contributing_factors: Vec<String>,
}

// ─── Predictive Health Modeling ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrajectoryPrediction {
    pub prediction_horizon_months: u32,
    pub predicted_decline_curve: Vec<HealthPoint>,
    pub confidence_intervals: Vec<ConfidenceInterval>,
    pub key_milestone_predictions: Vec<HealthMilestone>,
    pub uncertainty_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthPoint {
    pub timestamp_months: u32,
    pub predicted_score: f64,
    pub confidence_low: f64,
    pub confidence_high: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub timestamp_months: u32,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub confidence_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMilestone {
    pub description: String,
    pub predicted_date: u64,
    pub confidence: f64,
    pub severity: DeclineSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTimeline {
    pub estimated_decline_months: u32,
    pub critical_threshold_date: Option<u64>,
    pub milestone_dates: Vec<u64>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalDeclineTimeline {
    pub current_functional_score: f64,
    pub predicted_months_to_significant_impact: u32,
    pub predicted_months_to_severe_impact: u32,
    pub affected_capabilities: Vec<String>,
    pub support_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalInheritanceTiming {
    pub recommended_stages: Vec<InheritanceStage>,
    pub earliest_safe_release: u64,
    pub latest_optimal_release: u64,
    pub reasoning: String,
    pub confidence_score: f64,
}

// ─── Staged Inheritance Release ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceStages {
    pub plan_id: u64,
    pub stages: Vec<InheritanceStage>,
    pub total_stages: u32,
    pub estimated_completion_date: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceStage {
    pub stage_number: u32,
    pub release_percentage: f64,
    pub health_threshold: HealthThreshold,
    pub trigger_conditions: Vec<TriggerCondition>,
    pub confirmation_required: bool,
    pub medical_verification_needed: bool,
    pub beneficiary_notification: NotificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThreshold {
    pub overall_health_score_max: f64,
    pub decline_velocity_min: f64,
    pub affected_systems_count: u32,
    pub functional_independence_score: f64,
    pub quality_of_life_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceStageDesign {
    pub plan_id: u64,
    pub designed_stages: Vec<InheritanceStage>,
    pub design_rationale: String,
    pub projected_release_dates: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTriggerEvaluation {
    pub current_health: HealthSnapshot,
    pub stage_criteria: StageCriteria,
    pub triggered: bool,
    pub readiness_score: f64,
    pub blocking_factors: Vec<String>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCriteria {
    pub stage_number: u32,
    pub min_health_decline_score: f64,
    pub max_functional_independence: f64,
    pub required_systems_affected: u32,
    pub medical_verification_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageReleaseResult {
    pub success: bool,
    pub plan_id: u64,
    pub released_stage: u32,
    pub released_percentage: f64,
    pub remaining_percentage: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageReleaseExecution {
    pub executed_at: u64,
    pub stage_number: u32,
    pub amount_released: f64,
    pub beneficiary_notified: bool,
    pub medical_verification_attached: bool,
    pub transaction_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEvidence {
    pub evidence_type: String,
    pub source: String,
    pub recorded_at: u64,
    pub data: serde_json::Value,
    pub verified: bool,
}

// ─── Medical Data Integration ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicianAssessment {
    pub assessment_id: String,
    pub physician_id: String,
    pub physician_name: String,
    pub assessment_date: u64,
    pub diagnoses: Vec<String>,
    pub observations: Vec<String>,
    pub prognosis: String,
    pub recommended_actions: Vec<String>,
    pub functional_capacity_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedHealthProfile {
    pub user_id: String,
    pub medical_records: Vec<MedicalRecord>,
    pub lab_results: Vec<LabResult>,
    pub physician_assessments: Vec<PhysicianAssessment>,
    pub wearable_data: Vec<HealthDataPoint>,
    pub genetic_data: Option<GeneticProfile>,
    pub integrated_at: u64,
    pub completeness_score: f64,
}

// ─── Health Intervention Recommendations ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIntervention {
    pub intervention_type: InterventionType,
    pub priority_level: Priority,
    pub expected_impact: ExpectedImpact,
    pub implementation_difficulty: Difficulty,
    pub cost_estimate: Option<f64>,
    pub provider_referral_needed: bool,
    pub description: String,
    pub evidence_basis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedImpact {
    pub estimated_decline_reduction_percentage: f64,
    pub estimated_quality_of_life_improvement: f64,
    pub time_to_effect_days: u32,
    pub sustainability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementedIntervention {
    pub intervention_type: InterventionType,
    pub start_date: u64,
    pub current_status: String,
    pub adherence_score: f64,
    pub observed_effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustedTimeline {
    pub original_timeline: InheritanceTimeline,
    pub adjusted_timeline: InheritanceTimeline,
    pub adjustment_reasons: Vec<String>,
    pub projected_impact_months: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceTimeline {
    pub current_stage: u32,
    pub projected_stages: Vec<InheritanceStage>,
    pub estimated_completion: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareConsent {
    pub user_id: String,
    pub data_sharing_consent: bool,
    pub provider_notification_consent: bool,
    pub family_notification_consent: bool,
    pub research_participation_consent: bool,
    pub consent_date: u64,
    pub expiry_date: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderNotificationResult {
    pub success: bool,
    pub notifications_sent: u32,
    pub failed_notifications: Vec<String>,
    pub sent_at: u64,
}

// ─── Quality of Life Assessment ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivityData {
    pub user_id: String,
    pub recorded_at: u64,
    pub daily_activities: Vec<DailyActivitySummary>,
    pub assistive_devices_used: Vec<String>,
    pub home_modifications: Vec<String>,
    pub care_recipient_status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivitySummary {
    pub date: String,
    pub bathing: bool,
    pub dressing: bool,
    pub toileting: bool,
    pub transferring: bool,
    pub feeding: bool,
    pub cooking: bool,
    pub cleaning: bool,
    pub shopping: bool,
    pub medication_management: bool,
    pub transportation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ADLAssessment {
    pub user_id: String,
    pub assessed_at: u64,
    pub bathing_independence: f64,
    pub dressing_independence: f64,
    pub toileting_independence: f64,
    pub transferring_independence: f64,
    pub feeding_independence: f64,
    pub overall_adl_score: f64,
    pub assistance_required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IADLAssessment {
    pub user_id: String,
    pub assessed_at: u64,
    pub cooking_ability: f64,
    pub cleaning_ability: f64,
    pub shopping_ability: f64,
    pub medication_management_ability: f64,
    pub transportation_ability: f64,
    pub overall_iadl_score: f64,
    pub assistance_required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityOfLifeScore {
    pub user_id: String,
    pub calculated_at: u64,
    pub adl_score: f64,
    pub iadl_score: f64,
    pub overall_quality_of_life: f64,
    pub domain_scores: Vec<DomainScore>,
    pub improvement_areas: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainScore {
    pub domain: String,
    pub score: f64,
    pub weight: f64,
}

// ─── Shared Data Types (from existing modules) ────────────────────────────────

pub use crate::genetic_analysis::health_monitoring::types::{
    MedicalRecord, LabResult, VitalSigns, ProviderInfo, GeneticProfile, LifestyleFactors,
    EnvironmentalFactors, ConditionStatus, MedicalDataType, RecordType, ProgressionMarker,
    ProgressionTrend, UrgencyLevel, Evidence, TimeFrame,
};

pub use crate::fitbit_integration::types::{
    HeartRateReading, ActivityData, SleepSession, SeverityLevel as FitbitSeverityLevel,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionStatus {
    NoSymptoms,
    EarlyMarkers,
    Developing,
    Diagnosed,
    Progressive,
    Terminal,
    InRemission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthTriggerType {
    GeneticConditionDetected(String),
    DiseaseProgression(String),
    TerminalDiagnosis(String),
    LifeExpectancyReduction(u32),
    QualityOfLifeDecline(f64),
    CognitiveDecline(CognitionLevel),
    PhysicalIncapacitation(IncapacitationType),
    TreatmentFailure(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UrgencyLevel {
    Low,
    Medium,
    High,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    Consultation,
    HospitalAdmission,
    EmergencyVisit,
    LabReport,
    ImagingStudy,
    Prescription,
    Vaccination,
    DischargeSummary,
    SpecialistReferral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncapacitationType {
    Mobility,
    Cognitive,
    Sensory,
    Communication,
    SelfCare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitionLevel {
    Normal,
    MildImpairment,
    ModerateImpairment,
    Severe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MedicalDataType {
    GeneticData,
    LabResults,
    ImagingStudies,
    Prescriptions,
    DiagnosisCodes,
    VitalSigns,
    FamilyHistory,
    ClinicalNotes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackingFrequency {
    RealTime,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredCondition {
    pub condition_name: String,
    pub genetic_predisposition: f64,
    pub current_status: ConditionStatus,
    pub progression_markers: Vec<ProgressionMarker>,
    pub trigger_criteria: TriggerCriteria,
    pub last_assessment: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionMarker {
    pub marker_name: String,
    pub measured_value: f64,
    pub baseline_value: f64,
    pub threshold: f64,
    pub trend: ProgressionTrend,
    pub measured_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressionTrend {
    Stable,
    Improving,
    Worsening,
    RapidlyWorsening,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCriteria {
    pub min_predisposition: f64,
    pub required_status: Vec<ConditionStatus>,
    pub progression_threshold: f64,
    pub marker_thresholds: HashMap<String, f64>,
    pub time_window_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthBaseline {
    pub age: u32,
    pub bmi: Option<f64>,
    pub blood_pressure_systolic: Option<u32>,
    pub blood_pressure_diastolic: Option<u32>,
    pub resting_heart_rate: Option<u32>,
    pub cholesterol_ldl: Option<f64>,
    pub cholesterol_hdl: Option<f64>,
    pub fasting_glucose: Option<f64>,
    pub established_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub predisposition_warning: f64,
    pub predisposition_critical: f64,
    pub progression_warning: f64,
    pub progression_critical: f64,
    pub urgency_escalation_delay_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitoringStatus {
    pub user_id: u64,
    pub monitored_conditions: Vec<MonitoredCondition>,
    pub overall_status: MonitoringOverallStatus,
    pub last_check: u64,
    pub active_alerts: Vec<HealthAlert>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonitoringOverallStatus {
    Normal,
    Elevated,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub alert_id: String,
    pub condition_name: String,
    pub alert_type: HealthTriggerType,
    pub severity: UrgencyLevel,
    pub message: String,
    pub triggered_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredCondition {
    pub trigger_id: String,
    pub condition_name: String,
    pub trigger_type: HealthTriggerType,
    pub confidence: f64,
    pub urgency: UrgencyLevel,
    pub evidence: Vec<Evidence>,
    pub triggered_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub source: String,
    pub description: String,
    pub value: serde_json::Value,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerAssessment {
    pub plan_id: u64,
    pub assessed_at: u64,
    pub active_triggers: Vec<EvaluatedTrigger>,
    pub pending_triggers: Vec<EvaluatedTrigger>,
    pub requires_immediate_action: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatedTrigger {
    pub trigger_id: String,
    pub condition_name: String,
    pub trigger_type: HealthTriggerType,
    pub confidence_level: f64,
    pub urgency_level: UrgencyLevel,
    pub estimated_timeframe: Option<TimeFrame>,
    pub supporting_evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeFrame {
    pub min_days: u32,
    pub max_days: u32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalRecord {
    pub record_id: String,
    pub patient_id: String,
    pub record_type: RecordType,
    pub diagnosis_codes: Vec<String>,
    pub procedures: Vec<ProcedureCode>,
    pub medications: Vec<Medication>,
    pub vital_signs: VitalSigns,
    pub timestamp: u64,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureCode {
    pub code: String,
    pub description: String,
    pub date_performed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub prescribed_date: u64,
    pub prescribing_physician: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalSigns {
    pub temperature: Option<f64>,
    pub heart_rate: Option<u32>,
    pub blood_pressure_systolic: Option<u32>,
    pub blood_pressure_diastolic: Option<u32>,
    pub respiratory_rate: Option<u32>,
    pub oxygen_saturation: Option<f64>,
    pub pain_level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub provider_id: String,
    pub provider_name: String,
    pub facility: String,
    pub specialty: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabResult {
    pub result_id: String,
    pub patient_id: String,
    pub test_name: String,
    pub test_code: String,
    pub value: String,
    pub unit: String,
    pub reference_range: String,
    pub is_abnormal: bool,
    pub performed_at: u64,
    pub ordering_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagingStudy {
    pub study_id: String,
    pub patient_id: String,
    pub study_type: String,
    pub body_region: String,
    pub findings: String,
    pub impression: String,
    pub performed_at: u64,
    pub radiologist: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionAssessment {
    pub condition_name: String,
    pub previous_status: ConditionStatus,
    pub current_status: ConditionStatus,
    pub progression_rate: f64,
    pub markers_changed: Vec<String>,
    pub estimated_time_to_next_stage: Option<TimeFrame>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthPrediction {
    pub prediction_id: String,
    pub condition_name: String,
    pub predicted_onset: Option<TimeFrame>,
    pub probability: f64,
    pub contributing_factors: Vec<String>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub age: u32,
    pub active_conditions: Vec<String>,
    pub current_medications: Vec<String>,
    pub recent_procedures: Vec<String>,
    pub overall_wellness_score: f64,
    pub assessed_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticProfile {
    pub profile_id: String,
    pub genetic_mutations: HashMap<String, f64>,
    pub polygenic_risk_scores: HashMap<String, f64>,
    pub carrier_status: HashMap<String, bool>,
    pub pharmacogenomic_markers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedRecord {
    pub anonymized_id: String,
    pub record_type: RecordType,
    pub diagnosis_codes: Vec<String>,
    pub vital_signs: VitalSigns,
    pub timestamp: u64,
    pub age_range: String,
    pub gender: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsCompliance {
    pub informed_consent: bool,
    pub data_minimization: bool,
    pub purpose_limitation: bool,
    pub medical_necessity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerRequest {
    pub user_id: u64,
    pub plan_id: u64,
    pub condition_name: String,
    pub trigger_type: HealthTriggerType,
    pub supporting_data: Vec<Evidence>,
    pub requested_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifestyleFactors {
    pub diet_quality: f64,
    pub exercise_frequency: f64,
    pub smoking_status: bool,
    pub alcohol_consumption: f64,
    pub stress_level: f64,
    pub sleep_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalFactors {
    pub air_quality_index: Option<f64>,
    pub occupational_hazards: Vec<String>,
    pub geographical_risks: Vec<String>,
    pub social_support_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseOnsetPrediction {
    pub condition_name: String,
    pub predicted_onset_age: Option<u32>,
    pub probability_by_age: Vec<(u32, f64)>,
    pub confidence_interval: (f64, f64),
    pub key_risk_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticRiskModel {
    pub condition: String,
    pub variants: Vec<String>,
    pub weights: Vec<f64>,
    pub baseline_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifestyleModel {
    pub factor_name: String,
    pub effect_size: f64,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalModel {
    pub factor_name: String,
    pub risk_multiplier: f64,
    pub exposure_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionModel {
    pub model_name: String,
    pub gene_environment_interactions: Vec<(String, String, f64)>,
}

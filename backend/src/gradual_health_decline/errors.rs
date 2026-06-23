use thiserror::Error;

#[derive(Debug, Error)]
pub enum BaselineError {
    #[error("Insufficient historical data for baseline calculation: {0}")]
    InsufficientData(String),

    #[error("Invalid monitoring period: {0} days")]
    InvalidPeriod(u32),

    #[error("Baseline calculation failed: {0}")]
    CalculationFailed(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Database error: {0}")]
    Database(String),
}

#[derive(Debug, Error)]
pub enum MonitoringError {
    #[error("Health data retrieval failed: {0}")]
    DataRetrievalFailed(String),

    #[error("Insufficient data points for monitoring: {0}")]
    InsufficientDataPoints(usize),

    #[error("Analysis computation error: {0}")]
    AnalysisError(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Database error: {0}")]
    Database(String),
}

#[derive(Debug, Error)]
pub enum StagingError {
    #[error("Invalid assessment data: {0}")]
    InvalidAssessment(String),

    #[error("No suitable inheritance plan found: {0}")]
    PlanNotFound(u64),

    #[error("Stage design failed: {0}")]
    DesignFailed(String),

    #[error("Database error: {0}")]
    Database(String),
}

#[derive(Debug, Error)]
pub enum ReleaseError {
    #[error("Stage trigger condition not met: {0}")]
    TriggerNotMet(String),

    #[error("Invalid stage number: {0}")]
    InvalidStage(u32),

    #[error("Plan not found: {0}")]
    PlanNotFound(u64),

    #[error("Medical verification required but missing")]
    VerificationRequired,

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Database error: {0}")]
    Database(String),
}

#[derive(Debug, Error)]
pub enum CalculationError {
    #[error("Baseline comparison failed: {0}")]
    BaselineComparisonFailed(String),

    #[error("Invalid health snapshot")]
    InvalidSnapshot,

    #[error("Insufficient historical data")]
    InsufficientHistory,

    #[error("Unsupported health system type")]
    UnsupportedSystem,
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Baseline update failed: {0}")]
    UpdateFailed(String),

    #[error("No changes detected since last baseline")]
    NoChanges,

    #[error("Database error: {0}")]
    Database(String),
}

#[derive(Debug, Error)]
pub enum ComparisonError {
    #[error("Comparison calculation failed: {0}")]
    CalculationFailed(String),

    #[error("Missing baseline data for user: {0}")]
    MissingBaseline(String),

    #[error("Missing current health data")]
    MissingCurrentData,
}

#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("Cardiovascular analysis failed: {0}")]
    CardiovascularError(String),

    #[error("Activity analysis failed: {0}")]
    ActivityError(String),

    #[error("Cognitive analysis failed: {0}")]
    CognitiveError(String),

    #[error("Sleep analysis failed: {0}")]
    SleepError(String),

    #[error("Insufficient data for analysis: {0}")]
    InsufficientData(String),
}

#[derive(Debug, Error)]
pub enum PredictionError {
    #[error("Prediction model error: {0}")]
    ModelError(String),

    #[error("Insufficient data for prediction: {0}")]
    InsufficientData(String),

    #[error("Model not calibrated for population: {0}")]
    NotCalibrated(String),
}

#[derive(Debug, Error)]
pub enum DesignError {
    #[error("Stage design failed: {0}")]
    DesignFailed(String),

    #[error("Invalid plan preferences")]
    InvalidPreferences,

    #[error("No declining systems detected")]
    NoDeclineDetected,
}

#[derive(Debug, Error)]
pub enum EvaluationError {
    #[error("Trigger evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Invalid stage criteria")]
    InvalidCriteria,

    #[error("Insufficient health data for evaluation")]
    InsufficientData,
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Release execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Stage not ready for release")]
    StageNotReady,

    #[error("Verification failed: {0}")]
    VerificationFailed(String),
}

#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("Medical record integration failed: {0}")]
    RecordIntegrationFailed(String),

    #[error("Patient not found: {0}")]
    PatientNotFound(String),

    #[error("Data format error: {0}")]
    DataFormatError(String),
}

#[derive(Debug, Error)]
pub enum IncorporationError {
    #[error("Lab result incorporation failed: {0}")]
    IncorporationFailed(String),

    #[error("Invalid lab result format")]
    InvalidFormat,
}

#[derive(Debug, Error)]
pub enum InclusionError {
    #[error("Assessment inclusion failed: {0}")]
    InclusionFailed(String),

    #[error("Invalid assessment data")]
    InvalidAssessment,
}

#[derive(Debug, Error)]
pub enum RecommendationError {
    #[error("Recommendation generation failed: {0}")]
    GenerationFailed(String),

    #[error("No applicable interventions found")]
    NoInterventions,

    #[error("Insufficient assessment data")]
    InsufficientData,
}

#[derive(Debug, Error)]
pub enum AdjustmentError {
    #[error("Timeline adjustment failed: {0}")]
    AdjustmentFailed(String),

    #[error("No interventions detected for adjustment")]
    NoInterventions,

    #[error("Invalid current timeline")]
    InvalidTimeline,
}

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Provider notification failed: {0}")]
    NotificationFailed(String),

    #[error("Consent not granted for notification")]
    ConsentNotGranted,

    #[error("Invalid recipient data")]
    InvalidRecipient,
}

#[derive(Debug, Error)]
pub enum AssessmentError {
    #[error("Assessment calculation failed: {0}")]
    CalculationFailed(String),

    #[error("Invalid activity data")]
    InvalidData,

    #[error("Insufficient activity history")]
    InsufficientHistory,
}

#[derive(Debug, Error)]
pub enum QualityOfLifeError {
    #[error("Quality of life assessment failed: {0}")]
    AssessmentFailed(String),

    #[error("Invalid user activity data")]
    InvalidData,

    #[error("Insufficient data for calculation")]
    InsufficientData,
}

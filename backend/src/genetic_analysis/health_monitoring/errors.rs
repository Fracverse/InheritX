use thiserror::Error;

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("Health monitoring error: {0}")]
    Monitoring(String),

    #[error("Condition not found: {0}")]
    ConditionNotFound(String),

    #[error("Invalid health data: {0}")]
    InvalidData(String),

    #[error("Medical integration error: {0}")]
    MedicalIntegration(String),

    #[error("Trigger evaluation error: {0}")]
    TriggerEvaluation(String),

    #[error("Privacy violation: {0}")]
    PrivacyViolation(String),

    #[error("Consent not granted for data type: {0}")]
    ConsentDenied(String),

    #[error("External health service unavailable: {0}")]
    ExternalServiceUnavailable(String),

    #[error("Database error: {0}")]
    Database(String),
}

#[derive(Debug, Error)]
pub enum TriggerError {
    #[error("Trigger evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Insufficient health data for trigger evaluation: {0}")]
    InsufficientData(String),

    #[error("Invalid trigger criteria: {0}")]
    InvalidCriteria(String),

    #[error("Conflicting triggers detected: {0}")]
    ConflictingTriggers(String),

    #[error("Ethics validation failed: {0}")]
    EthicsViolation(String),
}

#[derive(Debug, Error)]
pub enum EHRError {
    #[error("EHR client error: {0}")]
    ClientError(String),

    #[error("Patient not found: {0}")]
    PatientNotFound(String),

    #[error("Record not found: {0}")]
    RecordNotFound(String),

    #[error("Authentication failed with EHR provider: {0}")]
    AuthenticationFailed(String),

    #[error("Rate limited by EHR provider: {0}")]
    RateLimited(String),

    #[error("EHR provider unavailable: {0}")]
    ProviderUnavailable(String),
}

#[derive(Debug, Error)]
pub enum PrivacyError {
    #[error("Privacy validation failed: {0}")]
    ValidationFailed(String),

    #[error("Consent not granted: {0}")]
    ConsentDenied(String),

    #[error("Data anonymization failed: {0}")]
    AnonymizationFailed(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),
}

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Audit logging failed: {0}")]
    LoggingFailed(String),

    #[error("Unauthorized access attempt detected: {0}")]
    UnauthorizedAccess(String),

    #[error("Audit trail corrupted: {0}")]
    TrailCorrupted(String),
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
pub enum EthicsError {
    #[error("Missing informed consent")]
    MissingConsent,

    #[error("Data minimization violation: {0}")]
    DataMinimizationViolation(String),

    #[error("Purpose limitation exceeded: {0}")]
    PurposeLimitationExceeded(String),

    #[error("Medical necessity not established: {0}")]
    MedicalNecessityNotEstablished(String),
}

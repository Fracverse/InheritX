use thiserror::Error;

#[derive(Debug, Error)]
pub enum FitbitError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Token expired for user: {0}")]
    TokenExpired(String),

    #[error("API request failed: {0}")]
    ApiRequestFailed(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Insufficient data: {0}")]
    InsufficientData(String),
}

#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("Analysis failed: {0}")]
    Failed(String),

    #[error("Insufficient data for analysis: {0}")]
    InsufficientData(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

#[derive(Debug, Error)]
pub enum SleepError {
    #[error("Sleep data unavailable: {0}")]
    DataUnavailable(String),

    #[error("Sleep analysis failed: {0}")]
    AnalysisFailed(String),
}

#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("Detection failed: {0}")]
    Failed(String),

    #[error("Insufficient history: {0}")]
    InsufficientHistory(String),
}

#[derive(Debug, Error)]
pub enum CalculationError {
    #[error("Calculation failed: {0}")]
    Failed(String),

    #[error("Missing required data: {0}")]
    MissingData(String),
}

#[derive(Debug, Error)]
pub enum AssessmentError {
    #[error("Assessment failed: {0}")]
    Failed(String),

    #[error("Insufficient step data")]
    InsufficientData,
}

impl From<AnalysisError> for FitbitError {
    fn from(err: AnalysisError) -> Self {
        FitbitError::ApiRequestFailed(err.to_string())
    }
}

impl From<SleepError> for FitbitError {
    fn from(err: SleepError) -> Self {
        FitbitError::ApiRequestFailed(err.to_string())
    }
}

impl From<CalculationError> for FitbitError {
    fn from(err: CalculationError) -> Self {
        FitbitError::ApiRequestFailed(err.to_string())
    }
}

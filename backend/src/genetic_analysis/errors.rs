use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeneticError {
    #[error("Invalid DNA data: {0}")]
    InvalidInput(String),

    #[error("DNA processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Privacy error: {0}")]
    Privacy(String),

    #[error("Insufficient SNP data for comparison")]
    InsufficientData,
}

#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("Analysis failed: {0}")]
    Failed(String),

    #[error("No markers found for profile")]
    NoMarkers,
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Variant not found: {0}")]
    VariantNotFound(String),

    #[error("Database query failed: {0}")]
    QueryFailed(String),

    #[error("External service unavailable: {0}")]
    Unavailable(String),
}

impl From<AnalysisError> for GeneticError {
    fn from(err: AnalysisError) -> Self {
        GeneticError::Analysis(err.to_string())
    }
}

impl From<DatabaseError> for GeneticError {
    fn from(err: DatabaseError) -> Self {
        GeneticError::Database(err.to_string())
    }
}

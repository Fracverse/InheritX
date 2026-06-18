//! # Genetic Data Processing and Analysis Service
//!
//! Comprehensive backend service for DNA analysis, health condition detection,
//! genetic similarity calculations, risk assessment, and privacy-preserving analysis.
//!
//! Issue #14 / #745

mod database;
mod dna_processor;
mod errors;
mod health;
mod privacy;
mod service;
mod similarity;
mod types;

pub use database::{
    ClinVarClient, CompositeGeneticDatabaseClient, DbSnpClient, GeneticDatabaseClient,
    GwasCatalogClient,
};
pub use dna_processor::DNAProcessor;
pub use errors::{AnalysisError, DatabaseError, GeneticError};
pub use health::HealthConditionAnalyzer;
pub use privacy::GeneticPrivacyEngine;
pub use service::GeneticAnalysisService;
pub use similarity::GeneticSimilarityCalculator;
pub use types::*;

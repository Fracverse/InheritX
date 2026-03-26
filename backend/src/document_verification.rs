//! # Legal Document Integrity Check
//!
//! Provides verification that a legal will document matches the hash stored on-chain,
//! ensuring the document has not been altered. Supports version-specific verification
//! and enables trustless verification for courts, executors, and beneficiaries.

use crate::api_error::ApiError;
use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVerificationRequest {
    /// Document ID to verify
    pub document_id: Uuid,
    /// Optional: Specific version to verify (defaults to active version)
    pub version: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHashVerificationRequest {
    /// Document ID to verify against
    pub document_id: Uuid,
    /// Hash to verify (hex-encoded SHA256)
    pub provided_hash: String,
    /// Optional: Specific version to verify
    pub version: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContentVerificationRequest {
    /// Document ID to verify against
    pub document_id: Uuid,
    /// Base64-encoded document content to verify
    pub document_content: String,
    /// Optional: Specific version to verify
    pub version: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the document matches the stored hash
    pub is_valid: bool,
    /// Document ID that was verified
    pub document_id: Uuid,
    /// Plan ID associated with the document
    pub plan_id: Uuid,
    /// Version that was verified
    pub version: u32,
    /// Stored hash from database
    pub stored_hash: String,
    /// Computed hash from provided document
    pub computed_hash: Option<String>,
    /// Verification timestamp
    pub verified_at: chrono::DateTime<chrono::Utc>,
    /// Additional verification details
    pub details: VerificationDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDetails {
    /// Whether this is the active version
    pub is_active_version: bool,
    /// Document status (draft/finalized)
    pub status: String,
    /// Template used
    pub template: String,
    /// When the document was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Whether the document is encrypted
    pub is_encrypted: bool,
}

// ─── Service ──────────────────────────────────────────────────────────────────

pub struct DocumentVerificationService;

impl DocumentVerificationService {
    /// Verify a document by retrieving it from the database and comparing hashes.
    /// This is the primary verification method for stored documents.
    pub async fn verify_document(
        db: &PgPool,
        document_id: Uuid,
        version: Option<u32>,
    ) -> Result<VerificationResult, ApiError> {
        let doc = Self::fetch_document(db, document_id, version).await?;

        // Compute hash from stored PDF content
        let computed_hash = Self::compute_document_hash(&doc.pdf_base64)?;

        let is_valid = computed_hash == doc.will_hash;

        Ok(VerificationResult {
            is_valid,
            document_id: doc.id,
            plan_id: doc.plan_id,
            version: doc.version as u32,
            stored_hash: doc.will_hash.clone(),
            computed_hash: Some(computed_hash),
            verified_at: chrono::Utc::now(),
            details: VerificationDetails {
                is_active_version: doc.is_active_version,
                status: doc.status,
                template: doc.template,
                generated_at: doc.generated_at,
                is_encrypted: doc.is_encrypted,
            },
        })
    }

    /// Verify a document by comparing a provided hash with the stored hash.
    /// Useful for quick verification without transferring the full document.
    pub async fn verify_hash(
        db: &PgPool,
        document_id: Uuid,
        provided_hash: String,
        version: Option<u32>,
    ) -> Result<VerificationResult, ApiError> {
        let doc = Self::fetch_document(db, document_id, version).await?;

        // Normalize hashes to lowercase for comparison
        let provided_hash_normalized = provided_hash.to_lowercase();
        let stored_hash_normalized = doc.will_hash.to_lowercase();

        let is_valid = provided_hash_normalized == stored_hash_normalized;

        Ok(VerificationResult {
            is_valid,
            document_id: doc.id,
            plan_id: doc.plan_id,
            version: doc.version as u32,
            stored_hash: doc.will_hash.clone(),
            computed_hash: Some(provided_hash),
            verified_at: chrono::Utc::now(),
            details: VerificationDetails {
                is_active_version: doc.is_active_version,
                status: doc.status,
                template: doc.template,
                generated_at: doc.generated_at,
                is_encrypted: doc.is_encrypted,
            },
        })
    }

    /// Verify a document by computing the hash of provided content and comparing
    /// with the stored hash. This is the most comprehensive verification method.
    pub async fn verify_content(
        db: &PgPool,
        document_id: Uuid,
        document_content: String,
        version: Option<u32>,
    ) -> Result<VerificationResult, ApiError> {
        let doc = Self::fetch_document(db, document_id, version).await?;

        // Compute hash from provided content
        let computed_hash = Self::compute_document_hash(&document_content)?;

        let is_valid = computed_hash == doc.will_hash;

        Ok(VerificationResult {
            is_valid,
            document_id: doc.id,
            plan_id: doc.plan_id,
            version: doc.version as u32,
            stored_hash: doc.will_hash.clone(),
            computed_hash: Some(computed_hash),
            verified_at: chrono::Utc::now(),
            details: VerificationDetails {
                is_active_version: doc.is_active_version,
                status: doc.status,
                template: doc.template,
                generated_at: doc.generated_at,
                is_encrypted: doc.is_encrypted,
            },
        })
    }

    /// Verify all versions of a document for a given plan.
    /// Returns a list of verification results for each version.
    pub async fn verify_all_versions(
        db: &PgPool,
        plan_id: Uuid,
    ) -> Result<Vec<VerificationResult>, ApiError> {
        let versions = Self::fetch_all_versions(db, plan_id).await?;

        let mut results = Vec::new();
        for doc in versions {
            let computed_hash = Self::compute_document_hash(&doc.pdf_base64)?;
            let is_valid = computed_hash == doc.will_hash;

            results.push(VerificationResult {
                is_valid,
                document_id: doc.id,
                plan_id: doc.plan_id,
                version: doc.version as u32,
                stored_hash: doc.will_hash.clone(),
                computed_hash: Some(computed_hash),
                verified_at: chrono::Utc::now(),
                details: VerificationDetails {
                    is_active_version: doc.is_active_version,
                    status: doc.status,
                    template: doc.template,
                    generated_at: doc.generated_at,
                    is_encrypted: doc.is_encrypted,
                },
            });
        }

        Ok(results)
    }

    // ─── Helper Functions ─────────────────────────────────────────────────────

    /// Compute SHA256 hash of document content (base64-encoded PDF)
    fn compute_document_hash(content: &str) -> Result<String, ApiError> {
        let hash_bytes = digest(&SHA256, content.as_bytes());
        Ok(hex::encode(hash_bytes.as_ref()))
    }

    /// Fetch document from database with optional version specification
    async fn fetch_document(
        db: &PgPool,
        document_id: Uuid,
        version: Option<u32>,
    ) -> Result<DocumentRecord, ApiError> {
        let query = if let Some(v) = version {
            sqlx::query_as::<_, DocumentRecord>(
                r#"
                SELECT 
                    d.id,
                    d.plan_id,
                    d.version,
                    d.will_hash,
                    d.pdf_base64,
                    d.status,
                    d.template,
                    d.generated_at,
                    d.is_encrypted,
                    CASE 
                        WHEN d.status = 'finalized' THEN 
                            d.version = (
                                SELECT MAX(version) 
                                FROM will_documents 
                                WHERE plan_id = d.plan_id AND status = 'finalized'
                            )
                        ELSE 
                            d.version = (
                                SELECT MAX(version) 
                                FROM will_documents 
                                WHERE plan_id = d.plan_id
                            )
                    END as is_active_version
                FROM will_documents d
                WHERE d.id = $1 AND d.version = $2
                "#,
            )
            .bind(document_id)
            .bind(v as i32)
            .fetch_optional(db)
            .await?
        } else {
            sqlx::query_as::<_, DocumentRecord>(
                r#"
                SELECT 
                    d.id,
                    d.plan_id,
                    d.version,
                    d.will_hash,
                    d.pdf_base64,
                    d.status,
                    d.template,
                    d.generated_at,
                    d.is_encrypted,
                    true as is_active_version
                FROM will_documents d
                WHERE d.id = $1
                ORDER BY d.version DESC
                LIMIT 1
                "#,
            )
            .bind(document_id)
            .fetch_optional(db)
            .await?
        };

        query.ok_or_else(|| {
            ApiError::NotFound(format!(
                "Document {} {} not found",
                document_id,
                version.map_or(String::new(), |v| format!("version {}", v))
            ))
        })
    }

    /// Fetch all versions of documents for a plan
    async fn fetch_all_versions(
        db: &PgPool,
        plan_id: Uuid,
    ) -> Result<Vec<DocumentRecord>, ApiError> {
        let active_version: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT MAX(version) 
            FROM will_documents 
            WHERE plan_id = $1 AND status = 'finalized'
            "#,
        )
        .bind(plan_id)
        .fetch_optional(db)
        .await?
        .flatten();

        let active_version = active_version.unwrap_or({
            // If no finalized version, use the latest version
            0
        });

        sqlx::query_as::<_, DocumentRecord>(
            r#"
            SELECT 
                id,
                plan_id,
                version,
                will_hash,
                pdf_base64,
                status,
                template,
                generated_at,
                is_encrypted,
                (version = $2) as is_active_version
            FROM will_documents
            WHERE plan_id = $1
            ORDER BY version DESC
            "#,
        )
        .bind(plan_id)
        .bind(active_version)
        .fetch_all(db)
        .await
        .map_err(ApiError::from)
    }
}

// ─── Internal Types ───────────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
struct DocumentRecord {
    id: Uuid,
    plan_id: Uuid,
    version: i32,
    will_hash: String,
    pdf_base64: String,
    status: String,
    template: String,
    generated_at: chrono::DateTime<chrono::Utc>,
    is_encrypted: bool,
    is_active_version: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_document_hash() {
        let content = "test document content";
        let hash = DocumentVerificationService::compute_document_hash(content).unwrap();

        // Verify it's a valid hex string of correct length (64 chars for SHA256)
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_compute_document_hash_consistency() {
        let content = "test document content";
        let hash1 = DocumentVerificationService::compute_document_hash(content).unwrap();
        let hash2 = DocumentVerificationService::compute_document_hash(content).unwrap();

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_document_hash_different_content() {
        let content1 = "test document content";
        let content2 = "different content";
        let hash1 = DocumentVerificationService::compute_document_hash(content1).unwrap();
        let hash2 = DocumentVerificationService::compute_document_hash(content2).unwrap();

        // Different content should produce different hashes
        assert_ne!(hash1, hash2);
    }
}

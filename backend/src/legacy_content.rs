use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::path::PathBuf;
use uuid::Uuid;

const MAX_FILE_SIZE: usize = 524_288_000; // 500MB
const ALLOWED_VIDEO_TYPES: &[&str] = &["video/mp4", "video/mpeg", "video/quicktime", "video/x-msvideo", "video/webm"];
const ALLOWED_AUDIO_TYPES: &[&str] = &["audio/mpeg", "audio/wav", "audio/ogg", "audio/mp4", "audio/webm"];
const ALLOWED_TEXT_TYPES: &[&str] = &["text/plain", "text/markdown", "text/html"];
const ALLOWED_DOCUMENT_TYPES: &[&str] = &[
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyContent {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub content_type: String,
    pub file_size: i64,
    pub storage_path: String,
    pub file_hash: String,
    pub encrypted: bool,
    pub encryption_key_version: Option<i32>,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMetadata {
    pub original_filename: String,
    pub content_type: String,
    pub file_size: usize,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentListFilters {
    pub content_type_prefix: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub struct LegacyContentService;

impl LegacyContentService {
    /// Validate file type
    pub fn validate_content_type(content_type: &str) -> Result<(), ApiError> {
        let allowed_types: Vec<&str> = ALLOWED_VIDEO_TYPES
            .iter()
            .chain(ALLOWED_AUDIO_TYPES.iter())
            .chain(ALLOWED_TEXT_TYPES.iter())
            .chain(ALLOWED_DOCUMENT_TYPES.iter())
            .copied()
            .collect();

        if !allowed_types.contains(&content_type) {
            return Err(ApiError::BadRequest(format!(
                "Unsupported content type: {}. Allowed types: video, audio, text, documents",
                content_type
            )));
        }

        Ok(())
    }

    /// Validate file size
    pub fn validate_file_size(size: usize) -> Result<(), ApiError> {
        if size == 0 {
            return Err(ApiError::BadRequest("File is empty".to_string()));
        }

        if size > MAX_FILE_SIZE {
            return Err(ApiError::BadRequest(format!(
                "File size {} bytes exceeds maximum allowed size of {} bytes (500MB)",
                size, MAX_FILE_SIZE
            )));
        }

        Ok(())
    }

    /// Generate storage path
    pub fn generate_storage_path(user_id: Uuid, filename: &str) -> String {
        let date = Utc::now().format("%Y/%m/%d");
        format!("legacy_content/{}/{}/{}", user_id, date, filename)
    }

    /// Calculate SHA-256 hash of file content
    pub fn calculate_file_hash(content: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Store file metadata in database
    pub async fn create_content_record(
        db: &PgPool,
        owner_user_id: Uuid,
        metadata: &UploadMetadata,
        storage_path: String,
        file_hash: String,
    ) -> Result<LegacyContent, ApiError> {
        let filename = Uuid::new_v4().to_string();
        
        let meta_json = serde_json::json!({
            "description": metadata.description,
            "uploaded_at": Utc::now()
        });

        let record = sqlx::query_as!(
            LegacyContent,
            r#"
            INSERT INTO legacy_content 
            (owner_user_id, filename, original_filename, content_type, file_size, 
             storage_path, file_hash, encrypted, status, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, false, 'active', $8)
            RETURNING 
                id, owner_user_id, filename, original_filename, content_type, 
                file_size, storage_path, file_hash, encrypted, 
                encryption_key_version, status, 
                metadata as "metadata: serde_json::Value",
                created_at, updated_at
            "#,
            owner_user_id,
            filename,
            metadata.original_filename,
            metadata.content_type,
            metadata.file_size as i64,
            storage_path,
            file_hash,
            meta_json
        )
        .fetch_one(db)
        .await?;

        Ok(record)
    }

    /// List user's content
    pub async fn list_user_content(
        db: &PgPool,
        owner_user_id: Uuid,
        filters: &ContentListFilters,
    ) -> Result<Vec<LegacyContent>, ApiError> {
        let limit = filters.limit.unwrap_or(50).min(100);
        let offset = filters.offset.unwrap_or(0);

        let records = if let Some(ref prefix) = filters.content_type_prefix {
            sqlx::query_as!(
                LegacyContent,
                r#"
                SELECT 
                    id, owner_user_id, filename, original_filename, content_type, 
                    file_size, storage_path, file_hash, encrypted, 
                    encryption_key_version, status,
                    metadata as "metadata: serde_json::Value",
                    created_at, updated_at
                FROM legacy_content
                WHERE owner_user_id = $1 
                  AND status = 'active'
                  AND content_type LIKE $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#,
                owner_user_id,
                format!("{}%", prefix),
                limit,
                offset
            )
            .fetch_all(db)
            .await?
        } else {
            sqlx::query_as!(
                LegacyContent,
                r#"
                SELECT 
                    id, owner_user_id, filename, original_filename, content_type, 
                    file_size, storage_path, file_hash, encrypted, 
                    encryption_key_version, status,
                    metadata as "metadata: serde_json::Value",
                    created_at, updated_at
                FROM legacy_content
                WHERE owner_user_id = $1 AND status = 'active'
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                owner_user_id,
                limit,
                offset
            )
            .fetch_all(db)
            .await?
        };

        Ok(records)
    }

    /// Get content by ID
    pub async fn get_content_by_id(
        db: &PgPool,
        content_id: Uuid,
        owner_user_id: Uuid,
    ) -> Result<LegacyContent, ApiError> {
        let record = sqlx::query_as!(
            LegacyContent,
            r#"
            SELECT 
                id, owner_user_id, filename, original_filename, content_type, 
                file_size, storage_path, file_hash, encrypted, 
                encryption_key_version, status,
                metadata as "metadata: serde_json::Value",
                created_at, updated_at
            FROM legacy_content
            WHERE id = $1 AND owner_user_id = $2 AND status = 'active'
            "#,
            content_id,
            owner_user_id
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Content not found".to_string()))?;

        Ok(record)
    }

    /// Delete content (soft delete)
    pub async fn delete_content(
        db: &PgPool,
        content_id: Uuid,
        owner_user_id: Uuid,
    ) -> Result<(), ApiError> {
        let result = sqlx::query!(
            r#"
            UPDATE legacy_content
            SET status = 'deleted', updated_at = NOW()
            WHERE id = $1 AND owner_user_id = $2 AND status = 'active'
            "#,
            content_id,
            owner_user_id
        )
        .execute(db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound("Content not found".to_string()));
        }

        Ok(())
    }

    /// Get storage statistics for user
    pub async fn get_user_storage_stats(
        db: &PgPool,
        owner_user_id: Uuid,
    ) -> Result<StorageStats, ApiError> {
        let stats = sqlx::query_as!(
            StorageStats,
            r#"
            SELECT 
                COUNT(*)::bigint as "total_files!",
                COALESCE(SUM(file_size), 0)::bigint as "total_size!",
                COUNT(CASE WHEN content_type LIKE 'video/%' THEN 1 END)::bigint as "video_count!",
                COUNT(CASE WHEN content_type LIKE 'audio/%' THEN 1 END)::bigint as "audio_count!",
                COUNT(CASE WHEN content_type LIKE 'text/%' THEN 1 END)::bigint as "text_count!",
                COUNT(CASE WHEN content_type LIKE 'application/%' THEN 1 END)::bigint as "document_count!"
            FROM legacy_content
            WHERE owner_user_id = $1 AND status = 'active'
            "#,
            owner_user_id
        )
        .fetch_one(db)
        .await?;

        Ok(stats)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_files: i64,
    pub total_size: i64,
    pub video_count: i64,
    pub audio_count: i64,
    pub text_count: i64,
    pub document_count: i64,
}

/// File storage handler (filesystem-based)
pub struct FileStorageService {
    base_path: PathBuf,
}

impl FileStorageService {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Save file to disk
    pub async fn save_file(&self, storage_path: &str, content: &[u8]) -> Result<(), ApiError> {
        let full_path = self.base_path.join(storage_path);
        
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("Failed to create directory: {}", e))
            })?;
        }

        tokio::fs::write(&full_path, content).await.map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to write file: {}", e))
        })?;

        Ok(())
    }

    /// Read file from disk
    pub async fn read_file(&self, storage_path: &str) -> Result<Vec<u8>, ApiError> {
        let full_path = self.base_path.join(storage_path);
        
        tokio::fs::read(&full_path).await.map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to read file: {}", e))
        })
    }

    /// Delete file from disk
    pub async fn delete_file(&self, storage_path: &str) -> Result<(), ApiError> {
        let full_path = self.base_path.join(storage_path);
        
        tokio::fs::remove_file(&full_path).await.map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to delete file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_video_types() {
        assert!(LegacyContentService::validate_content_type("video/mp4").is_ok());
        assert!(LegacyContentService::validate_content_type("video/webm").is_ok());
    }

    #[test]
    fn test_validate_audio_types() {
        assert!(LegacyContentService::validate_content_type("audio/mpeg").is_ok());
        assert!(LegacyContentService::validate_content_type("audio/wav").is_ok());
    }

    #[test]
    fn test_validate_document_types() {
        assert!(LegacyContentService::validate_content_type("application/pdf").is_ok());
        assert!(LegacyContentService::validate_content_type("application/msword").is_ok());
    }

    #[test]
    fn test_reject_invalid_type() {
        assert!(LegacyContentService::validate_content_type("application/exe").is_err());
        assert!(LegacyContentService::validate_content_type("image/png").is_err());
    }

    #[test]
    fn test_validate_file_size() {
        assert!(LegacyContentService::validate_file_size(1024).is_ok());
        assert!(LegacyContentService::validate_file_size(MAX_FILE_SIZE).is_ok());
        assert!(LegacyContentService::validate_file_size(MAX_FILE_SIZE + 1).is_err());
        assert!(LegacyContentService::validate_file_size(0).is_err());
    }

    #[test]
    fn test_file_hash() {
        let content = b"test content";
        let hash = LegacyContentService::calculate_file_hash(content);
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }
}

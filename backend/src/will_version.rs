//! Will Version Management API
//!
//! Provides endpoints to list, retrieve, and finalize versioned will documents.

use crate::api_error::ApiError;
use crate::will_pdf::GeneratedWillDocument;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ---- Types ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillVersionSummary {
    pub document_id: Uuid,
    pub plan_id: Uuid,
    pub version: u32,
    pub status: String,
    pub template_used: String,
    pub will_hash: String,
    pub filename: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedVersions {
    pub versions: Vec<WillVersionSummary>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

// ---- Service ----

pub struct WillVersionService;

impl WillVersionService {
    /// List all versions for a plan with pagination. Excludes pdf_base64 for efficiency.
    pub async fn get_all_versions(
        db: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<WillVersionSummary>, i64), ApiError> {
        let total: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)::bigint FROM will_documents WHERE plan_id = $1 AND user_id = $2",
        )
        .bind(plan_id)
        .bind(user_id)
        .fetch_one(db)
        .await
        .unwrap_or(0);

        let offset = ((page.saturating_sub(1)) * per_page) as i64;

        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            plan_id: Uuid,
            version: i32,
            status: String,
            template: String,
            will_hash: String,
            filename: String,
            generated_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, plan_id, version, status, template, will_hash, filename, generated_at \
             FROM will_documents \
             WHERE plan_id = $1 AND user_id = $2 \
             ORDER BY version DESC \
             LIMIT $3 OFFSET $4",
        )
        .bind(plan_id)
        .bind(user_id)
        .bind(per_page as i64)
        .bind(offset)
        .fetch_all(db)
        .await?;

        let versions = rows
            .into_iter()
            .map(|r| WillVersionSummary {
                document_id: r.id,
                plan_id: r.plan_id,
                version: r.version as u32,
                status: r.status,
                template_used: r.template,
                will_hash: r.will_hash,
                filename: r.filename,
                generated_at: r.generated_at,
            })
            .collect();

        Ok((versions, total))
    }

    /// Get the active version: latest finalized, or latest draft if none finalized.
    pub async fn get_active_version(
        db: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
    ) -> Result<WillVersionSummary, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            plan_id: Uuid,
            version: i32,
            status: String,
            template: String,
            will_hash: String,
            filename: String,
            generated_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            "SELECT id, plan_id, version, status, template, will_hash, filename, generated_at \
             FROM will_documents \
             WHERE plan_id = $1 AND user_id = $2 \
             ORDER BY CASE WHEN status = 'finalized' THEN 0 ELSE 1 END, version DESC \
             LIMIT 1",
        )
        .bind(plan_id)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound("No will versions found for this plan".to_string()))?;

        Ok(WillVersionSummary {
            document_id: row.id,
            plan_id: row.plan_id,
            version: row.version as u32,
            status: row.status,
            template_used: row.template,
            will_hash: row.will_hash,
            filename: row.filename,
            generated_at: row.generated_at,
        })
    }

    /// Get a specific version by version number, including full PDF content.
    pub async fn get_version(
        db: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
        version_number: u32,
    ) -> Result<GeneratedWillDocument, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            plan_id: Uuid,
            template: String,
            will_hash: String,
            version: i32,
            filename: String,
            pdf_base64: String,
            generated_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            "SELECT id, plan_id, template, will_hash, version, filename, pdf_base64, generated_at \
             FROM will_documents \
             WHERE plan_id = $1 AND user_id = $2 AND version = $3",
        )
        .bind(plan_id)
        .bind(user_id)
        .bind(version_number as i32)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Will version {version_number} not found")))?;

        Ok(GeneratedWillDocument {
            document_id: row.id,
            plan_id: row.plan_id,
            template_used: row.template,
            will_hash: row.will_hash,
            generated_at: row.generated_at,
            version: row.version as u32,
            pdf_base64: row.pdf_base64,
            filename: row.filename,
        })
    }

    /// Mark a draft version as finalized.
    pub async fn finalize_version(
        db: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
        version_number: u32,
    ) -> Result<WillVersionSummary, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            plan_id: Uuid,
            version: i32,
            status: String,
            template: String,
            will_hash: String,
            filename: String,
            generated_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            "SELECT id, plan_id, version, status, template, will_hash, filename, generated_at \
             FROM will_documents \
             WHERE plan_id = $1 AND user_id = $2 AND version = $3",
        )
        .bind(plan_id)
        .bind(user_id)
        .bind(version_number as i32)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Will version {version_number} not found")))?;

        if row.status == "finalized" {
            return Err(ApiError::BadRequest(
                "This version is already finalized".to_string(),
            ));
        }

        sqlx::query(
            "UPDATE will_documents SET status = 'finalized' \
             WHERE plan_id = $1 AND user_id = $2 AND version = $3",
        )
        .bind(plan_id)
        .bind(user_id)
        .bind(version_number as i32)
        .execute(db)
        .await?;

        Ok(WillVersionSummary {
            document_id: row.id,
            plan_id: row.plan_id,
            version: row.version as u32,
            status: "finalized".to_string(),
            template_used: row.template,
            will_hash: row.will_hash,
            filename: row.filename,
            generated_at: row.generated_at,
        })
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    fn clamp_pagination(params: &PaginationParams) -> (u32, u32) {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(10).clamp(1, 100);
        (page, per_page)
    }

    fn validate_status(status: &str) -> Result<(), ApiError> {
        match status {
            "draft" | "finalized" => Ok(()),
            _ => Err(ApiError::BadRequest(format!(
                "Invalid status: {status}. Must be 'draft' or 'finalized'"
            ))),
        }
    }

    #[test]
    fn test_clamp_pagination_defaults() {
        let params = PaginationParams {
            page: None,
            per_page: None,
        };
        let (page, per_page) = clamp_pagination(&params);
        assert_eq!(page, 1);
        assert_eq!(per_page, 10);
    }

    #[test]
    fn test_clamp_pagination_zero_page() {
        let params = PaginationParams {
            page: Some(0),
            per_page: Some(5),
        };
        let (page, per_page) = clamp_pagination(&params);
        assert_eq!(page, 1);
        assert_eq!(per_page, 5);
    }

    #[test]
    fn test_clamp_pagination_exceeds_max() {
        let params = PaginationParams {
            page: Some(3),
            per_page: Some(500),
        };
        let (page, per_page) = clamp_pagination(&params);
        assert_eq!(page, 3);
        assert_eq!(per_page, 100);
    }

    #[test]
    fn test_clamp_pagination_min_per_page() {
        let params = PaginationParams {
            page: Some(1),
            per_page: Some(0),
        };
        let (page, per_page) = clamp_pagination(&params);
        assert_eq!(page, 1);
        assert_eq!(per_page, 1);
    }

    #[test]
    fn test_validate_status_draft() {
        assert!(validate_status("draft").is_ok());
    }

    #[test]
    fn test_validate_status_finalized() {
        assert!(validate_status("finalized").is_ok());
    }

    #[test]
    fn test_validate_status_invalid() {
        assert!(validate_status("pending").is_err());
        assert!(validate_status("").is_err());
        assert!(validate_status("DRAFT").is_err());
    }
}

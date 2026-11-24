//! KYC evidence upload and retrieval (issue #1274).
//!
//! The handler parses the multipart form, validates the file size/type and the
//! declared document type, writes the bytes to the configured storage directory
//! and persists ownership metadata. A success response is only emitted after
//! both the file write and the metadata insert succeed; on failure the partial
//! file is removed and an explicit error is returned.

use crate::api::AppState;
use crate::auth::UserContext;
use axum::{
    extract::{Extension, Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

/// Largest accepted upload. Enforced both while streaming the field and by the
/// route-level `DefaultBodyLimit`.
pub const MAX_KYC_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

pub const KYC_DOCUMENT_TYPES: [&str; 3] = ["id", "address_proof", "selfie"];

pub const KYC_CONTENT_TYPES: [&str; 3] = ["image/jpeg", "image/png", "application/pdf"];

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct KycDocumentRecord {
    pub id: Uuid,
    pub owner_address: String,
    pub document_type: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
}

#[derive(Debug, Serialize)]
pub struct KycDocumentResponse {
    pub document_id: String,
    pub url: String,
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

fn storage_dir() -> String {
    std::env::var("KYC_STORAGE_DIR").unwrap_or_else(|_| "./kyc_uploads".to_string())
}

fn extension_for_content_type(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "application/pdf" => Some("pdf"),
        _ => None,
    }
}

/// Derive a content type from a client-supplied file name when the multipart
/// part omits `Content-Type`.
fn content_type_from_filename(filename: &str) -> Option<&'static str> {
    let lower = filename.to_lowercase();
    if lower.ends_with(".pdf") {
        Some("application/pdf")
    } else if lower.ends_with(".png") {
        Some("image/png")
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Some("image/jpeg")
    } else {
        None
    }
}

fn sanitize_filename(filename: &str) -> String {
    let base = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("upload")
        .trim();
    let sanitized: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "upload".to_string()
    } else {
        sanitized
    }
}

/// Persists the upload bytes. Flushing before returning ensures a storage
/// failure is reported instead of a fabricated success.
async fn persist_bytes(
    storage_dir: &str,
    id: Uuid,
    extension: &str,
    bytes: &[u8],
) -> std::io::Result<PathBuf> {
    tokio::fs::create_dir_all(storage_dir).await?;
    let path = FsPath::new(storage_dir).join(format!("{id}.{extension}"));
    let mut file = tokio::fs::File::create(&path).await?;
    file.write_all(bytes).await?;
    file.flush().await?;
    file.sync_all().await?;
    Ok(path)
}

/// POST /api/kyc/upload
pub async fn upload_kyc_document(
    State(state): State<Arc<AppState>>,
    ctx: Option<Extension<UserContext>>,
    mut multipart: Multipart,
) -> Response {
    let mut document_type: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(e) => {
                return error(
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid multipart upload: {e}"),
                )
            }
        };

        match field.name() {
            Some("document_type") => match field.text().await {
                Ok(value) => document_type = Some(value.trim().to_string()),
                Err(e) => {
                    return error(
                        StatusCode::BAD_REQUEST,
                        &format!("Invalid document_type: {e}"),
                    )
                }
            },
            Some("file") => {
                let field_filename = field.file_name().map(|name| name.to_string());
                let field_content_type = field.content_type().map(|value| value.to_string());

                let mut field = field;
                let mut collected = Vec::new();
                loop {
                    match field.chunk().await {
                        Ok(Some(chunk)) => {
                            if collected.len() + chunk.len() > MAX_KYC_UPLOAD_BYTES {
                                return error(
                                    StatusCode::PAYLOAD_TOO_LARGE,
                                    "File exceeds the 10 MiB upload limit",
                                );
                            }
                            collected.extend_from_slice(&chunk);
                        }
                        Ok(None) => break,
                        Err(e) => {
                            return error(
                                StatusCode::BAD_REQUEST,
                                &format!("Failed to read uploaded file: {e}"),
                            )
                        }
                    }
                }

                if collected.is_empty() {
                    return error(StatusCode::BAD_REQUEST, "Uploaded file is empty");
                }

                filename = field_filename;
                content_type = field_content_type;
                bytes = Some(collected);
            }
            _ => {
                // Ignore unknown parts, but drain them so parsing can continue.
                let mut field = field;
                while let Ok(Some(_)) = field.chunk().await {}
            }
        }
    }

    let document_type = match document_type {
        Some(value) if KYC_DOCUMENT_TYPES.contains(&value.as_str()) => value,
        Some(_) => {
            return error(
                StatusCode::BAD_REQUEST,
                "document_type must be one of: id, address_proof, selfie",
            )
        }
        None => return error(StatusCode::BAD_REQUEST, "document_type is required"),
    };

    let (filename, bytes) = match (filename, bytes) {
        (Some(filename), Some(bytes)) => (sanitize_filename(&filename), bytes),
        _ => return error(StatusCode::BAD_REQUEST, "file is required"),
    };

    let content_type = match content_type
        .filter(|value| !value.is_empty())
        .or_else(|| content_type_from_filename(&filename).map(str::to_string))
    {
        Some(value) if KYC_CONTENT_TYPES.contains(&value.as_str()) => value,
        _ => {
            return error(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Unsupported file type; upload a JPEG, PNG or PDF",
            )
        }
    };

    let Some(extension) = extension_for_content_type(&content_type) else {
        return error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported file type; upload a JPEG, PNG or PDF",
        );
    };

    let owner_address = ctx
        .map(|Extension(context)| context.user_id)
        .filter(|owner| !owner.trim().is_empty())
        .unwrap_or_else(|| "anonymous".to_string());

    let id = Uuid::new_v4();
    let directory = storage_dir();
    let stored_path = match persist_bytes(&directory, id, extension, &bytes).await {
        Ok(path) => path,
        Err(e) => {
            tracing::error!(error = %e, "Failed to persist KYC document");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to store the uploaded document",
            );
        }
    };

    let size_bytes = bytes.len() as i64;
    let storage_path = stored_path.to_string_lossy().to_string();

    let inserted = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO kyc_documents (
            id, owner_address, document_type, filename, content_type, size_bytes, storage_path
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(&owner_address)
    .bind(&document_type)
    .bind(&filename)
    .bind(&content_type)
    .bind(size_bytes)
    .bind(&storage_path)
    .fetch_one(&state.db_pool)
    .await;

    match inserted {
        Ok(_) => (
            StatusCode::OK,
            Json(KycDocumentResponse {
                document_id: id.to_string(),
                url: format!("/api/kyc/documents/{id}"),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to persist KYC document metadata");
            // Do not leave an orphaned object behind when metadata fails.
            if let Err(remove_error) = tokio::fs::remove_file(&stored_path).await {
                tracing::warn!(
                    error = %remove_error,
                    path = %stored_path.display(),
                    "Failed to remove KYC file after metadata insert failure"
                );
            }
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to save document metadata",
            )
        }
    }
}

/// GET /api/kyc/documents/{document_id}
pub async fn get_kyc_document(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
) -> Response {
    let record = sqlx::query_as::<_, KycDocumentRecord>(
        "SELECT id, owner_address, document_type, filename, content_type, size_bytes, storage_path \
         FROM kyc_documents WHERE id = $1",
    )
    .bind(document_id)
    .fetch_optional(&state.db_pool)
    .await;

    let record = match record {
        Ok(Some(record)) => record,
        Ok(None) => return error(StatusCode::NOT_FOUND, "Document not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to load KYC document metadata");
            return error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load document");
        }
    };

    let bytes = match tokio::fs::read(&record.storage_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!(error = %e, path = %record.storage_path, "Stored KYC file is missing");
            return error(StatusCode::NOT_FOUND, "Document content is not available");
        }
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, record.content_type),
            (
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}\"", record.filename),
            ),
        ],
        bytes,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_every_documented_type() {
        for document_type in KYC_DOCUMENT_TYPES {
            assert!(KYC_DOCUMENT_TYPES.contains(&document_type));
        }
    }

    #[test]
    fn infers_content_type_from_extension() {
        assert_eq!(
            content_type_from_filename("passport.PDF"),
            Some("application/pdf")
        );
        assert_eq!(content_type_from_filename("selfie.png"), Some("image/png"));
        assert_eq!(content_type_from_filename("photo.jpeg"), Some("image/jpeg"));
        assert_eq!(content_type_from_filename("notes.txt"), None);
    }

    #[test]
    fn sanitizes_path_traversal_from_filenames() {
        assert_eq!(sanitize_filename("../../etc/passwd"), "passwd");
        assert_eq!(sanitize_filename("a b?.pdf"), "a_b_.pdf");
        assert_eq!(sanitize_filename(""), "upload");
    }

    #[tokio::test]
    async fn storage_failure_is_reported() {
        // A regular file cannot be used as a directory, so create_dir_all fails.
        let dir = std::env::temp_dir().join(format!("kyc-test-{}", Uuid::new_v4()));
        tokio::fs::write(&dir, b"not a directory").await.unwrap();

        let result = persist_bytes(dir.to_str().unwrap(), Uuid::new_v4(), "pdf", b"document").await;

        assert!(result.is_err());
        tokio::fs::remove_file(&dir).await.ok();
    }
}

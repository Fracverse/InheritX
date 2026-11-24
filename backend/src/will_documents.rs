//! Will-document retrieval, integrity verification, version downloads and the
//! document event audit trail (issue #1273).
//!
//! Generation/upload of encrypted documents lives in a separate ticket; these
//! handlers read the persisted rows and compare content against the stored
//! hash. Private routes are wrapped in
//! [`crate::auth::jwt_any_or_signature_auth_middleware`] and additionally
//! enforce that the caller is the plan owner or an admin. Verification is
//! intentionally public.

use crate::api::AppState;
use crate::auth::UserContext;
use axum::{
    extract::{Extension, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::Engine as _;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct WillDocumentRow {
    pub document_id: Uuid,
    pub plan_id: Uuid,
    pub template_used: String,
    pub will_hash: String,
    pub generated_at: DateTime<Utc>,
    pub version: i32,
    pub filename: String,
    pub is_encrypted: bool,
    pub status: String,
    #[serde(skip_serializing)]
    pub content: Option<Vec<u8>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WillEventRow {
    pub event_type: String,
    pub document_id: Option<Uuid>,
    pub plan_id: Uuid,
    pub vault_id: String,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct VerificationDetails {
    pub is_active_version: bool,
    pub status: String,
    pub template: String,
    pub generated_at: DateTime<Utc>,
    pub is_encrypted: bool,
}

#[derive(Debug, Serialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub document_id: String,
    pub plan_id: String,
    pub version: i32,
    pub hash_match: bool,
    pub stored_hash: String,
    pub computed_hash: Option<String>,
    pub message: String,
    pub verified_at: String,
    pub details: VerificationDetails,
}

#[derive(Debug, Serialize)]
pub struct EventStats {
    pub plan_id: String,
    pub will_created_count: i64,
    pub will_updated_count: i64,
    pub will_finalized_count: i64,
    pub will_signed_count: i64,
    pub witness_signed_count: i64,
    pub will_verified_count: i64,
    pub total_events: i64,
    pub first_event_at: Option<String>,
    pub last_event_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VersionQuery {
    pub version: Option<i32>,
    /// The public verification client historically sent `?page=` instead of
    /// `?version=`; accept it as an alias.
    pub page: Option<i32>,
}

impl VersionQuery {
    fn requested_version(&self) -> Option<i32> {
        self.version.or(self.page)
    }
}

#[derive(Debug, Deserialize)]
pub struct VerifyHashRequest {
    pub hash: String,
    pub version: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyContentRequest {
    pub content: String,
    pub version: Option<i32>,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn bad_request(message: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": message }))).into_response()
}

fn not_found(message: &str) -> Response {
    (StatusCode::NOT_FOUND, Json(json!({ "error": message }))).into_response()
}

fn internal_error(action: &str, error: sqlx::Error) -> Response {
    tracing::error!(error = %error, "Failed to {action}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": format!("Failed to {action}") })),
    )
        .into_response()
}

/// True when the authenticated caller may read a plan's private documents.
pub(crate) fn caller_can_access(ctx: &UserContext, owner_address: &str) -> bool {
    if ctx.role == "admin" || ctx.user_id.eq_ignore_ascii_case(owner_address) {
        return true;
    }

    // Signature-authenticated users are identified by their hex public key,
    // while plans store the Stellar `G...` strkey. Accept either form.
    if let Ok(bytes) = hex::decode(ctx.user_id.trim_start_matches("0x")) {
        if let Ok(key) = <[u8; 32]>::try_from(bytes.as_slice()) {
            if stellar_strkey::ed25519::PublicKey(key).to_string() == owner_address {
                return true;
            }
        }
    }

    false
}

async fn fetch_owner(state: &Arc<AppState>, plan_id: Uuid) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>("SELECT owner_address FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_optional(&state.db_pool)
        .await
}

async fn fetch_document(
    state: &Arc<AppState>,
    document_id: Uuid,
    version: Option<i32>,
) -> Result<Option<WillDocumentRow>, sqlx::Error> {
    const COLUMNS: &str = "document_id, plan_id, template_used, will_hash, generated_at, \
                           version, filename, is_encrypted, status, content";

    match version {
        Some(version) => {
            sqlx::query_as::<_, WillDocumentRow>(&format!(
                "SELECT {COLUMNS} FROM will_documents WHERE document_id = $1 AND version = $2"
            ))
            .bind(document_id)
            .bind(version)
            .fetch_optional(&state.db_pool)
            .await
        }
        None => {
            sqlx::query_as::<_, WillDocumentRow>(&format!(
                "SELECT {COLUMNS} FROM will_documents WHERE document_id = $1"
            ))
            .bind(document_id)
            .fetch_optional(&state.db_pool)
            .await
        }
    }
}

fn build_verification(
    document: &WillDocumentRow,
    computed_hash: Option<String>,
    hash_match: bool,
) -> VerificationResult {
    let message = if hash_match {
        "Document hash matches the persisted hash".to_string()
    } else if computed_hash.is_none() {
        "Document content is not stored; the persisted hash cannot be recomputed".to_string()
    } else {
        "Hash mismatch: the supplied content does not match the persisted hash".to_string()
    };

    VerificationResult {
        is_valid: hash_match && document.status == "active",
        document_id: document.document_id.to_string(),
        plan_id: document.plan_id.to_string(),
        version: document.version,
        hash_match,
        stored_hash: document.will_hash.clone(),
        computed_hash,
        message,
        verified_at: Utc::now().to_rfc3339(),
        details: VerificationDetails {
            is_active_version: document.status == "active",
            status: document.status.clone(),
            template: document.template_used.clone(),
            generated_at: document.generated_at,
            is_encrypted: document.is_encrypted,
        },
    }
}

/// GET /api/plans/{plan_id}/will/documents
pub async fn list_documents(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(plan_id): Path<Uuid>,
) -> Response {
    let owner = match fetch_owner(&state, plan_id).await {
        Ok(Some(owner)) => owner,
        Ok(None) => return not_found("Plan not found"),
        Err(e) => return internal_error("load plan", e),
    };

    if !caller_can_access(&ctx, &owner) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You do not have access to this plan's documents" })),
        )
            .into_response();
    }

    let documents = sqlx::query_as::<_, WillDocumentRow>(
        "SELECT document_id, plan_id, template_used, will_hash, generated_at, \
                version, filename, is_encrypted, status, NULL::bytea AS content \
         FROM will_documents WHERE plan_id = $1 ORDER BY version ASC",
    )
    .bind(plan_id)
    .fetch_all(&state.db_pool)
    .await;

    match documents {
        Ok(data) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "data": data })),
        )
            .into_response(),
        Err(e) => internal_error("load will documents", e),
    }
}

/// GET /api/will/documents/{document_id}
pub async fn get_document(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(document_id): Path<Uuid>,
) -> Response {
    let document = match fetch_document(&state, document_id, None).await {
        Ok(Some(document)) => document,
        Ok(None) => return not_found("Document not found"),
        Err(e) => return internal_error("load will document", e),
    };

    let owner = match fetch_owner(&state, document.plan_id).await {
        Ok(Some(owner)) => owner,
        Ok(None) => return not_found("Plan not found"),
        Err(e) => return internal_error("load plan", e),
    };

    if !caller_can_access(&ctx, &owner) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You do not have access to this document" })),
        )
            .into_response();
    }

    let mut data = serde_json::to_value(&document).unwrap_or_else(|_| json!({}));
    if let Some(content) = document.content.as_ref() {
        data["pdf_base64"] = json!(base64::engine::general_purpose::STANDARD.encode(content));
    }

    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "data": data })),
    )
        .into_response()
}

fn document_download_response(document: WillDocumentRow) -> Response {
    let Some(content) = document.content else {
        return not_found("Document content is not available for download");
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", document.filename),
            ),
        ],
        content,
    )
        .into_response()
}

/// GET /api/will/documents/{document_id}/download
pub async fn download_document(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(document_id): Path<Uuid>,
) -> Response {
    let document = match fetch_document(&state, document_id, None).await {
        Ok(Some(document)) => document,
        Ok(None) => return not_found("Document not found"),
        Err(e) => return internal_error("load will document", e),
    };

    let owner = match fetch_owner(&state, document.plan_id).await {
        Ok(Some(owner)) => owner,
        Ok(None) => return not_found("Plan not found"),
        Err(e) => return internal_error("load plan", e),
    };

    if !caller_can_access(&ctx, &owner) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You do not have access to this document" })),
        )
            .into_response();
    }

    document_download_response(document)
}

/// GET /api/plans/{plan_id}/will/documents/{version}/download
pub async fn download_version(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path((plan_id, version)): Path<(Uuid, i32)>,
) -> Response {
    let owner = match fetch_owner(&state, plan_id).await {
        Ok(Some(owner)) => owner,
        Ok(None) => return not_found("Plan not found"),
        Err(e) => return internal_error("load plan", e),
    };

    if !caller_can_access(&ctx, &owner) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You do not have access to this document" })),
        )
            .into_response();
    }

    let document = sqlx::query_as::<_, WillDocumentRow>(
        "SELECT document_id, plan_id, template_used, will_hash, generated_at, \
                version, filename, is_encrypted, status, content \
         FROM will_documents WHERE plan_id = $1 AND version = $2",
    )
    .bind(plan_id)
    .bind(version)
    .fetch_optional(&state.db_pool)
    .await;

    match document {
        Ok(Some(document)) => document_download_response(document),
        Ok(None) => not_found("Document version not found"),
        Err(e) => internal_error("load will document version", e),
    }
}

fn verify_stored_document(document: &WillDocumentRow) -> VerificationResult {
    match document.content.as_ref() {
        Some(content) => {
            let computed = sha256_hex(content);
            let hash_match = computed.eq_ignore_ascii_case(&document.will_hash);
            build_verification(document, Some(computed), hash_match)
        }
        None => build_verification(document, None, false),
    }
}

/// GET /api/will/documents/{document_id}/verify (public)
pub async fn verify_document(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    Query(query): Query<VersionQuery>,
) -> Response {
    let document = match fetch_document(&state, document_id, query.requested_version()).await {
        Ok(Some(document)) => document,
        Ok(None) => return not_found("Document not found"),
        Err(e) => return internal_error("load will document", e),
    };

    let result = verify_stored_document(&document);
    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "data": result })),
    )
        .into_response()
}

/// POST /api/will/documents/{document_id}/verify/hash (public)
pub async fn verify_document_by_hash(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    Json(payload): Json<VerifyHashRequest>,
) -> Response {
    let supplied_hash = payload.hash.trim();
    if supplied_hash.is_empty() {
        return bad_request("hash is required");
    }

    let document = match fetch_document(&state, document_id, payload.version).await {
        Ok(Some(document)) => document,
        Ok(None) => return not_found("Document not found"),
        Err(e) => return internal_error("load will document", e),
    };

    let hash_match = supplied_hash.eq_ignore_ascii_case(&document.will_hash);
    let result = build_verification(&document, Some(supplied_hash.to_string()), hash_match);

    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "data": result })),
    )
        .into_response()
}

/// POST /api/will/documents/{document_id}/verify/content (public)
pub async fn verify_document_by_content(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    Json(payload): Json<VerifyContentRequest>,
) -> Response {
    if payload.content.is_empty() {
        return bad_request("content is required");
    }

    let document = match fetch_document(&state, document_id, payload.version).await {
        Ok(Some(document)) => document,
        Ok(None) => return not_found("Document not found"),
        Err(e) => return internal_error("load will document", e),
    };

    let computed = sha256_hex(payload.content.as_bytes());
    let hash_match = computed.eq_ignore_ascii_case(&document.will_hash);

    let mut result = build_verification(&document, Some(computed), hash_match);

    // When the exact persisted bytes are available, also report a mismatch
    // between the supplied content and the stored object (even in the unlikely
    // event of a hash collision).
    if let Some(stored) = document.content.as_ref() {
        if stored.as_slice() != payload.content.as_bytes() {
            result.hash_match = false;
            result.is_valid = false;
            result.message =
                "Content mismatch: supplied content differs from the persisted document"
                    .to_string();
        }
    }

    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "data": result })),
    )
        .into_response()
}

async fn fetch_plan_events(
    state: &Arc<AppState>,
    plan_id: Uuid,
) -> Result<Vec<WillEventRow>, sqlx::Error> {
    sqlx::query_as::<_, WillEventRow>(
        "SELECT event_type, document_id, plan_id, vault_id, payload, created_at \
         FROM will_events WHERE plan_id = $1 ORDER BY created_at ASC, id ASC",
    )
    .bind(plan_id)
    .fetch_all(&state.db_pool)
    .await
}

fn event_to_json(event: WillEventRow) -> Value {
    json!({
        "event_type": event.event_type,
        "document_id": event.document_id.map(|id| id.to_string()),
        "plan_id": event.plan_id,
        "vault_id": event.vault_id,
        "timestamp": event.created_at,
        "payload": event.payload,
    })
}

async fn authorize_plan(
    state: &Arc<AppState>,
    ctx: &UserContext,
    plan_id: Uuid,
) -> Result<(), Response> {
    match fetch_owner(state, plan_id).await {
        Ok(Some(owner)) if caller_can_access(ctx, &owner) => Ok(()),
        Ok(Some(_)) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You do not have access to this plan's documents" })),
        )
            .into_response()),
        Ok(None) => Err(not_found("Plan not found")),
        Err(e) => Err(internal_error("load plan", e)),
    }
}

/// GET /api/will/documents/{document_id}/events
pub async fn get_document_events(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(document_id): Path<Uuid>,
) -> Response {
    let document = match fetch_document(&state, document_id, None).await {
        Ok(Some(document)) => document,
        Ok(None) => return not_found("Document not found"),
        Err(e) => return internal_error("load will document", e),
    };

    if let Err(response) = authorize_plan(&state, &ctx, document.plan_id).await {
        return response;
    }

    let events = sqlx::query_as::<_, WillEventRow>(
        "SELECT event_type, document_id, plan_id, vault_id, payload, created_at \
         FROM will_events WHERE document_id = $1 ORDER BY created_at ASC, id ASC",
    )
    .bind(document_id)
    .fetch_all(&state.db_pool)
    .await;

    match events {
        Ok(events) => {
            let data: Vec<Value> = events.into_iter().map(event_to_json).collect();
            (
                StatusCode::OK,
                Json(json!({ "status": "ok", "data": data })),
            )
                .into_response()
        }
        Err(e) => internal_error("load document events", e),
    }
}

/// GET /api/plans/{plan_id}/will/events
pub async fn get_plan_events(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(plan_id): Path<Uuid>,
) -> Response {
    if let Err(response) = authorize_plan(&state, &ctx, plan_id).await {
        return response;
    }

    match fetch_plan_events(&state, plan_id).await {
        Ok(events) => {
            let data: Vec<Value> = events.into_iter().map(event_to_json).collect();
            (
                StatusCode::OK,
                Json(json!({ "status": "ok", "data": data })),
            )
                .into_response()
        }
        Err(e) => internal_error("load plan events", e),
    }
}

/// GET /api/plans/{plan_id}/will/events/stats
pub async fn get_plan_event_stats(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(plan_id): Path<Uuid>,
) -> Response {
    if let Err(response) = authorize_plan(&state, &ctx, plan_id).await {
        return response;
    }

    let events = match fetch_plan_events(&state, plan_id).await {
        Ok(events) => events,
        Err(e) => return internal_error("load plan events", e),
    };

    let count = |kind: &str| events.iter().filter(|e| e.event_type == kind).count() as i64;

    let stats = EventStats {
        plan_id: plan_id.to_string(),
        will_created_count: count("will_created"),
        will_updated_count: count("will_updated"),
        will_finalized_count: count("will_finalized"),
        will_signed_count: count("will_signed"),
        witness_signed_count: count("witness_signed"),
        will_verified_count: count("will_verified"),
        total_events: events.len() as i64,
        first_event_at: events.first().map(|e| e.created_at.to_rfc3339()),
        last_event_at: events.last().map(|e| e.created_at.to_rfc3339()),
    };

    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "data": stats })),
    )
        .into_response()
}

/// GET /api/plans/{plan_id}/will/verify-all
pub async fn verify_all_versions(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(plan_id): Path<Uuid>,
) -> Response {
    if let Err(response) = authorize_plan(&state, &ctx, plan_id).await {
        return response;
    }

    let documents = sqlx::query_as::<_, WillDocumentRow>(
        "SELECT document_id, plan_id, template_used, will_hash, generated_at, \
                version, filename, is_encrypted, status, content \
         FROM will_documents WHERE plan_id = $1 ORDER BY version ASC",
    )
    .bind(plan_id)
    .fetch_all(&state.db_pool)
    .await;

    let documents = match documents {
        Ok(documents) => documents,
        Err(e) => return internal_error("load will documents", e),
    };

    let results: Vec<VerificationResult> = documents.iter().map(verify_stored_document).collect();
    let valid_count = results.iter().filter(|r| r.is_valid).count();

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "data": results,
            "count": results.len(),
            "valid_count": valid_count,
            "invalid_count": results.len() - valid_count,
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::UserContext;

    fn document(content: Option<&[u8]>) -> WillDocumentRow {
        WillDocumentRow {
            document_id: Uuid::new_v4(),
            plan_id: Uuid::new_v4(),
            template_used: "standard".to_string(),
            will_hash: sha256_hex(b"will body"),
            generated_at: Utc::now(),
            version: 1,
            filename: "will_v1.pdf".to_string(),
            is_encrypted: false,
            status: "active".to_string(),
            content: content.map(|c| c.to_vec()),
        }
    }

    #[test]
    fn sha256_is_lowercase_hex() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn matching_content_verifies() {
        let doc = document(Some(b"will body"));
        let result = verify_stored_document(&doc);
        assert!(result.is_valid);
        assert!(result.hash_match);
        assert_eq!(
            result.computed_hash.as_deref(),
            Some(doc.will_hash.as_str())
        );
    }

    #[test]
    fn altered_content_reports_explicit_mismatch() {
        let doc = document(Some(b"tampered"));
        let result = verify_stored_document(&doc);
        assert!(!result.is_valid);
        assert!(!result.hash_match);
        assert!(result.message.contains("mismatch"));
    }

    #[test]
    fn missing_content_cannot_be_verified() {
        let doc = document(None);
        let result = verify_stored_document(&doc);
        assert!(!result.is_valid);
        assert!(result.computed_hash.is_none());
        assert!(result.message.contains("not stored"));
    }

    #[test]
    fn access_is_limited_to_owner_and_admin() {
        let owner = "GOWNER";
        assert!(caller_can_access(
            &UserContext {
                user_id: "GOWNER".to_string(),
                role: "user".to_string()
            },
            owner
        ));
        assert!(caller_can_access(
            &UserContext {
                user_id: "someone-else".to_string(),
                role: "admin".to_string()
            },
            owner
        ));
        assert!(!caller_can_access(
            &UserContext {
                user_id: "someone-else".to_string(),
                role: "user".to_string()
            },
            owner
        ));
    }
}

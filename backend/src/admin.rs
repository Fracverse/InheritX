use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::api::AppState;

// --- Request/Response Types ---

#[derive(Debug, Deserialize)]
pub struct UsersQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub kyc_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub wallet_address: String,
    pub kyc_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UsersListResponse {
    pub data: Vec<UserRow>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKycRequest {
    pub kyc_status: String,
    pub admin_notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateKycResponse {
    pub wallet_address: String,
    pub kyc_status: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}

// --- Admin Handlers ---

/// GET /api/admin/users
/// Returns a paginated list of registered users and their KYC statuses
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(query): Query<UsersQuery>,
) -> impl IntoResponse {
    // Pagination defaults
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;

    info!(
        page = page,
        page_size = page_size,
        kyc_status_filter = ?query.kyc_status,
        "Admin: Listing users"
    );

    // Count total users matching the filter
    let total: i64 = match sqlx::query_scalar(
        r#"
        SELECT COUNT(*) 
        FROM users 
        WHERE ($1::text IS NULL OR kyc_status::text = $1)
        "#,
    )
    .bind(query.kyc_status.as_deref())
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(count) => count,
        Err(e) => {
            error!(error = %e, "Admin: Failed to count users");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Failed to retrieve user count".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Fetch paginated users
    let users: Vec<UserRow> = match sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, wallet_address, kyc_status::text as kyc_status, created_at
        FROM users
        WHERE ($1::text IS NULL OR kyc_status::text = $1)
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(query.kyc_status.as_deref())
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!(error = %e, "Admin: Failed to fetch users");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Failed to retrieve users".to_string(),
                }),
            )
                .into_response();
        }
    };

    info!(
        total_users = total,
        returned_count = users.len(),
        "Admin: Successfully retrieved users"
    );

    (
        StatusCode::OK,
        Json(UsersListResponse {
            data: users,
            page,
            page_size,
            total,
        }),
    )
        .into_response()
}

/// PUT /api/admin/users/:address/kyc
/// Allows overriding KYC status (Approve/Reject) directly in PostgreSQL
pub async fn update_user_kyc(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
    Json(payload): Json<UpdateKycRequest>,
) -> impl IntoResponse {
    // Validate KYC status
    let valid_statuses = ["pending", "approved", "rejected", "submitted"];
    if !valid_statuses.contains(&payload.kyc_status.as_str()) {
        error!(
            wallet_address = %address,
            attempted_status = %payload.kyc_status,
            "Admin: Invalid KYC status attempted"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: format!(
                    "Invalid KYC status. Must be one of: {}",
                    valid_statuses.join(", ")
                ),
            }),
        )
            .into_response();
    }

    info!(
        wallet_address = %address,
        new_status = %payload.kyc_status,
        admin_notes = ?payload.admin_notes,
        "Admin: Updating user KYC status"
    );

    // Check if user exists
    let user_exists: bool = match sqlx::query_scalar(
        r#"
        SELECT EXISTS(SELECT 1 FROM users WHERE wallet_address = $1)
        "#,
    )
    .bind(&address)
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(exists) => exists,
        Err(e) => {
            error!(
                wallet_address = %address,
                error = %e,
                "Admin: Database error checking user existence"
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Database error".to_string(),
                }),
            )
                .into_response();
        }
    };

    if !user_exists {
        error!(
            wallet_address = %address,
            "Admin: Attempted to update KYC for non-existent user"
        );
        return (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "User not found".to_string(),
            }),
        )
            .into_response();
    }

    // Update KYC status
    let updated_at = Utc::now();
    match sqlx::query(
        r#"
        UPDATE users 
        SET kyc_status = $1::kyc_status
        WHERE wallet_address = $2
        "#,
    )
    .bind(&payload.kyc_status)
    .bind(&address)
    .execute(&state.db_pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                error!(
                    wallet_address = %address,
                    "Admin: No rows affected when updating KYC status"
                );
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError {
                        error: "Failed to update KYC status".to_string(),
                    }),
                )
                    .into_response();
            }
        }
        Err(e) => {
            error!(
                wallet_address = %address,
                kyc_status = %payload.kyc_status,
                error = %e,
                "Admin: Failed to update KYC status"
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("Failed to update KYC status: {}", e),
                }),
            )
                .into_response();
        }
    }

    // Log the admin action for auditing
    info!(
        wallet_address = %address,
        kyc_status = %payload.kyc_status,
        admin_notes = ?payload.admin_notes,
        updated_at = %updated_at,
        "Admin: Successfully updated user KYC status"
    );

    // Broadcast KYC update event via WebSocket
    let event = crate::ws::KycUpdateEvent {
        wallet_address: address.clone(),
        kyc_status: payload.kyc_status.clone(),
        event_type: "admin_override".to_string(),
    };

    if let Err(e) = state.kyc_tx.send(event) {
        error!(
            error = %e,
            wallet_address = %address,
            "Admin: Failed to broadcast KYC update event"
        );
        // Don't fail the request if broadcast fails
    }

    (
        StatusCode::OK,
        Json(UpdateKycResponse {
            wallet_address: address,
            kyc_status: payload.kyc_status,
            updated_at,
        }),
    )
        .into_response()
}

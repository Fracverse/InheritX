use axum::{extract::{Path, State, Extension}, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::Utc;

use crate::{
    api_error::ApiError, 
    models::{KycRecord, KycStatus},
    service::ServiceContainer
};

#[derive(Debug, Deserialize)]
pub struct KycUpdateRequest {
    pub user_id: String,
}

pub async fn get_kyc_status(
    State(services): State<Arc<ServiceContainer>>,
    Path(user_id): Path<String>,
) -> Result<Json<KycRecord>, ApiError> {
    let client = services.db_pool.get().await?;

    let row = client
        .query_opt(
            "SELECT status, reviewed_by, reviewed_at, created_at FROM kyc_status WHERE user_id = $1",
            &[&user_id],
        )
        .await?;

    match row {
        Some(row) => {
            let status_str: String = row.get("status");
            Ok(Json(KycRecord {
                user_id,
                status: KycStatus::from_str(&status_str),
                reviewed_by: row.get("reviewed_by"),
                reviewed_at: row.get("reviewed_at"),
                created_at: row.get("created_at"),
            }))
        }
        None => {
            Ok(Json(KycRecord {
                user_id,
                status: KycStatus::Pending,
                reviewed_by: None,
                reviewed_at: None,
                created_at: Utc::now(),
            }))
        }
    }
}

pub async fn approve_kyc(
    State(services): State<Arc<ServiceContainer>>,
    Extension(admin_id): Extension<String>,
    Json(payload): Json<KycUpdateRequest>,
) -> Result<Json<KycRecord>, ApiError> {
    update_kyc_status(&services, admin_id, payload.user_id, KycStatus::Approved).await
}

pub async fn reject_kyc(
    State(services): State<Arc<ServiceContainer>>,
    Extension(admin_id): Extension<String>,
    Json(payload): Json<KycUpdateRequest>,
) -> Result<Json<KycRecord>, ApiError> {
    update_kyc_status(&services, admin_id, payload.user_id, KycStatus::Rejected).await
}

async fn update_kyc_status(
    services: &ServiceContainer,
    admin_id: String,
    user_id: String,
    status: KycStatus,
) -> Result<Json<KycRecord>, ApiError> {
    let client = services.db_pool.get().await?;

    let status_str = status.to_string();
    let now = Utc::now();

    let row = client
        .query_one(
            r#"
            INSERT INTO kyc_status (user_id, status, reviewed_by, reviewed_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id) DO UPDATE 
            SET status = EXCLUDED.status, 
                reviewed_by = EXCLUDED.reviewed_by, 
                reviewed_at = EXCLUDED.reviewed_at
            RETURNING created_at
            "#,
            &[&user_id, &status_str, &admin_id, &now, &now],
        )
        .await?;

    let created_at = row.get("created_at");

    Ok(Json(KycRecord {
        user_id,
        status,
        reviewed_by: Some(admin_id),
        reviewed_at: Some(now),
        created_at,
    }))
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_payments: i64,
    pub total_transfers: i64,
    pub total_withdrawals: i64,
    pub active_merchants: i64,
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub database: String,
    pub services: Vec<String>,
}

pub async fn get_dashboard_stats(
    State(_services): State<Arc<ServiceContainer>>,
) -> Result<Json<DashboardStats>, ApiError> {
    // Placeholder implementation
    Ok(Json(DashboardStats {
        total_users: 0,
        total_payments: 0,
        total_transfers: 0,
        total_withdrawals: 0,
        active_merchants: 0,
    }))
}

pub async fn get_transactions(
    State(_services): State<Arc<ServiceContainer>>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    // Placeholder implementation
    Ok(Json(vec![]))
}

pub async fn get_user_activity(
    State(_services): State<Arc<ServiceContainer>>,
    Path(_user_id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    // Placeholder implementation
    Ok(Json(vec![]))
}

pub async fn get_system_health(
    State(_services): State<Arc<ServiceContainer>>,
) -> Result<Json<SystemHealth>, ApiError> {
    // Placeholder implementation
    Ok(Json(SystemHealth {
        database: "healthy".to_string(),
        services: vec!["identity".to_string(), "payment".to_string()],
    }))
}
use crate::api_error::ApiError;
use crate::app::AppState;
use crate::auth::{verify_user_exists, AuthenticatedAdmin, AuthenticatedUser};
use axum::{
    extract::State,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    pub data_type: &'static str,
    pub table_name: &'static str,
    pub retention_days: i64,
    pub action: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveRunResult {
    pub archived_notifications: i64,
    pub archived_action_logs: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDataRequest {
    #[serde(default)]
    pub hard_delete: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserExportPayload {
    pub user: Value,
    pub plans: Vec<Value>,
    pub notifications: Vec<Value>,
    pub action_logs: Vec<Value>,
    pub exported_at: chrono::DateTime<chrono::Utc>,
}

pub struct DataRetentionService;

impl DataRetentionService {
    pub fn policies() -> Vec<RetentionPolicy> {
        vec![
            RetentionPolicy {
                data_type: "notifications",
                table_name: "notifications",
                retention_days: read_i64("RETENTION_NOTIFICATIONS_DAYS", 180),
                action: "archive_then_delete",
            },
            RetentionPolicy {
                data_type: "action_logs",
                table_name: "action_logs",
                retention_days: read_i64("RETENTION_ACTION_LOGS_DAYS", 365),
                action: "archive_then_delete",
            },
            RetentionPolicy {
                data_type: "sessions",
                table_name: "sessions",
                retention_days: read_i64("RETENTION_SESSIONS_DAYS", 30),
                action: "delete",
            },
        ]
    }

    pub async fn run_archive(pool: &PgPool) -> Result<ArchiveRunResult, ApiError> {
        let notifications_days = read_i64("RETENTION_NOTIFICATIONS_DAYS", 180);
        let action_logs_days = read_i64("RETENTION_ACTION_LOGS_DAYS", 365);

        let mut tx = pool.begin().await?;

        let archived_notifications = sqlx::query(
            r#"
            WITH moved AS (
                DELETE FROM notifications
                WHERE created_at < NOW() - make_interval(days => $1)
                RETURNING id, row_to_json(notifications.*)::jsonb AS payload
            )
            INSERT INTO data_archives(source_table, source_id, payload)
            SELECT 'notifications', id::text, payload FROM moved
            "#,
        )
        .bind(notifications_days)
        .execute(&mut *tx)
        .await?
        .rows_affected() as i64;

        let archived_action_logs = sqlx::query(
            r#"
            WITH moved AS (
                DELETE FROM action_logs
                WHERE timestamp < NOW() - make_interval(days => $1)
                RETURNING id, row_to_json(action_logs.*)::jsonb AS payload
            )
            INSERT INTO data_archives(source_table, source_id, payload)
            SELECT 'action_logs', id::text, payload FROM moved
            "#,
        )
        .bind(action_logs_days)
        .execute(&mut *tx)
        .await?
        .rows_affected() as i64;

        // Session hygiene retention.
        let session_days = read_i64("RETENTION_SESSIONS_DAYS", 30);
        let _ = sqlx::query(
            "DELETE FROM sessions WHERE created_at < NOW() - make_interval(days => $1)",
        )
        .bind(session_days)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(ArchiveRunResult {
            archived_notifications,
            archived_action_logs,
        })
    }

    pub fn start_archive_worker(pool: PgPool) {
        let interval_secs = read_u64("RETENTION_ARCHIVE_INTERVAL_SECS", 3600);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                match Self::run_archive(&pool).await {
                    Ok(result) => {
                        tracing::info!(
                            archived_notifications = result.archived_notifications,
                            archived_action_logs = result.archived_action_logs,
                            "Data retention archival cycle completed"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Data retention archival cycle failed");
                    }
                }
            }
        });
    }

    pub async fn export_user_data(
        pool: &PgPool,
        user_id: uuid::Uuid,
    ) -> Result<UserExportPayload, ApiError> {
        let user = sqlx::query_as::<_, (serde_json::Value,)>(
            r#"
            SELECT to_jsonb(u.*) FROM users u WHERE u.id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .map(|v| v.0)
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        let plans = sqlx::query_as::<_, (serde_json::Value,)>(
            r#"
            SELECT to_jsonb(p.*)
            FROM plans p
            WHERE p.user_id = $1
            ORDER BY p.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|v| v.0)
        .collect();

        let notifications = sqlx::query_as::<_, (serde_json::Value,)>(
            r#"
            SELECT to_jsonb(n.*)
            FROM notifications n
            WHERE n.user_id = $1
            ORDER BY n.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|v| v.0)
        .collect();

        let action_logs = sqlx::query_as::<_, (serde_json::Value,)>(
            r#"
            SELECT to_jsonb(a.*)
            FROM action_logs a
            WHERE a.user_id = $1
            ORDER BY a.timestamp DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|v| v.0)
        .collect();

        Ok(UserExportPayload {
            user,
            plans,
            notifications,
            action_logs,
            exported_at: chrono::Utc::now(),
        })
    }

    pub async fn delete_user_data(
        pool: &PgPool,
        user_id: uuid::Uuid,
        hard_delete: bool,
    ) -> Result<(), ApiError> {
        verify_user_exists(pool, &user_id).await?;

        let mut tx = pool.begin().await?;

        if hard_delete {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        } else {
            let anonymised_email = format!("deleted+{}@redacted.local", user_id);

            sqlx::query(
                r#"
                UPDATE users
                SET
                    email = $2,
                    wallet_address = NULL,
                    kyc_reference = NULL,
                    password_hash = 'deleted',
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(user_id)
            .bind(anonymised_email)
            .execute(&mut *tx)
            .await?;

            sqlx::query("DELETE FROM kyc_status WHERE user_id = $1")
                .bind(user_id)
                .execute(&mut *tx)
                .await?;

            sqlx::query("DELETE FROM notifications WHERE user_id = $1")
                .bind(user_id)
                .execute(&mut *tx)
                .await?;

            sqlx::query("DELETE FROM action_logs WHERE user_id = $1")
                .bind(user_id)
                .execute(&mut *tx)
                .await?;

            sqlx::query("DELETE FROM sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

async fn get_policies(
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "status": "success",
        "data": DataRetentionService::policies()
    })))
}

async fn run_archive_now(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let result = DataRetentionService::run_archive(&state.db).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn export_my_data(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let export = DataRetentionService::export_user_data(&state.db, user.user_id).await?;
    Ok(Json(json!({ "status": "success", "data": export })))
}

async fn delete_my_data(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<DeleteDataRequest>,
) -> Result<Json<Value>, ApiError> {
    DataRetentionService::delete_user_data(&state.db, user.user_id, payload.hard_delete).await?;
    Ok(Json(json!({
        "status": "success",
        "message": "Data deletion request processed"
    })))
}

pub fn retention_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/admin/data-retention/policies", get(get_policies))
        .route(
            "/api/admin/data-retention/archive/run",
            post(run_archive_now),
        )
        .route("/api/user/data-export", get(export_my_data))
        .route("/api/user/data", delete(delete_my_data))
}

fn read_i64(name: &str, default: i64) -> i64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(default)
}

fn read_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}

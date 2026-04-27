use crate::api_error::ApiError;
use crate::db::DbPool;
use crate::notifications::AuditLogService;
use axum::{
    extract::{Path, State},
    Json, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{encode};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

type HmacSha256 = Hmac<Sha256>;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_delivery: Option<chrono::DateTime<chrono::Utc>>,
    pub failure_count: i32,
}

#[derive(Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<String>,
}

#[derive(Serialize)]
pub struct WebhookEvent {
    pub id: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct WebhookService {
    db: PgPool,
    client: Client,
    retry_queue: Arc<Mutex<HashMap<Uuid, Vec<WebhookEvent>>>>,
}

impl WebhookService {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            client: Client::new(),
            retry_queue: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_webhook(
        &self,
        user_id: Uuid,
        request: CreateWebhookRequest,
    ) -> Result<Webhook, ApiError> {
        if request.events.is_empty() {
            return Err(ApiError::BadRequest("At least one event type required".to_string()));
        }

        let secret = self.generate_secret();
        let webhook_id = Uuid::new_v4();

        let webhook = sqlx::query_as::<_, Webhook>(
            r#"
            INSERT INTO webhooks (id, user_id, url, secret, events, is_active, created_at, failure_count)
            VALUES ($1, $2, $3, $4, $5, true, NOW(), 0)
            RETURNING *
            "#,
        )
        .bind(webhook_id)
        .bind(user_id)
        .bind(&request.url)
        .bind(&secret)
        .bind(&request.events)
        .fetch_one(&self.db)
        .await?;

        // Audit log
        AuditLogService::log(
            &self.db,
            Some(user_id),
            None,
            "webhook_registered",
            Some(webhook_id),
            "webhook",
            None,
            None,
            Some(format!("Events: {:?}", request.events)),
        )
        .await?;

        Ok(webhook)
    }

    pub async fn get_webhooks(&self, user_id: Uuid) -> Result<Vec<Webhook>, ApiError> {
        let webhooks = sqlx::query_as::<_, Webhook>(
            "SELECT * FROM webhooks WHERE user_id = $1 AND is_active = true",
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(webhooks)
    }

    pub async fn delete_webhook(&self, user_id: Uuid, webhook_id: Uuid) -> Result<(), ApiError> {
        let result = sqlx::query(
            "UPDATE webhooks SET is_active = false WHERE id = $1 AND user_id = $2",
        )
        .bind(webhook_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound("Webhook not found".to_string()));
        }

        AuditLogService::log(
            &self.db,
            Some(user_id),
            None,
            "webhook_deleted",
            Some(webhook_id),
            "webhook",
            None,
            None,
            None,
        )
        .await?;

        Ok(())
    }

    pub async fn deliver_event(&self, event: WebhookEvent) -> Result<(), ApiError> {
        let webhooks = sqlx::query_as::<_, Webhook>(
            "SELECT * FROM webhooks WHERE is_active = true AND $1 = ANY(events)",
        )
        .bind(&event.event_type)
        .fetch_all(&self.db)
        .await?;

        for webhook in webhooks {
            if let Err(e) = self.deliver_to_webhook(&webhook, &event).await {
                warn!("Failed to deliver webhook {}: {}", webhook.id, e);
                self.handle_delivery_failure(webhook.id).await?;
            }
        }

        Ok(())
    }

    async fn deliver_to_webhook(&self, webhook: &Webhook, event: &WebhookEvent) -> Result<(), ApiError> {
        let payload = serde_json::to_string(event)?;
        let signature = self.generate_signature(&webhook.secret, &payload);

        let response = self.client
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", signature)
            .header("X-Webhook-ID", webhook.id.to_string())
            .body(payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ApiError::InternalServerError(format!("Webhook delivery failed: {}", response.status())));
        }

        // Update last delivery
        sqlx::query(
            "UPDATE webhooks SET last_delivery = NOW() WHERE id = $1",
        )
        .bind(webhook.id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn handle_delivery_failure(&self, webhook_id: Uuid) -> Result<(), ApiError> {
        let failure_count: i32 = sqlx::query_scalar(
            "UPDATE webhooks SET failure_count = failure_count + 1 WHERE id = $1 RETURNING failure_count",
        )
        .bind(webhook_id)
        .execute(&self.db)
        .await?
        .unwrap_or(0);

        if failure_count >= 5 {
            // Deactivate webhook after 5 failures
            sqlx::query("UPDATE webhooks SET is_active = false WHERE id = $1")
                .bind(webhook_id)
                .execute(&self.db)
                .await?;
        } else {
            // Add to retry queue
            let mut queue = self.retry_queue.lock().await;
            // For simplicity, just increment failure count
            // In production, implement proper retry logic with exponential backoff
        }

        Ok(())
    }

    fn generate_secret(&self) -> String {
        use rand::Rng;
        let secret: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        secret
    }

    fn generate_signature(&self, secret: &str, payload: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        format!("sha256={}", encode(code_bytes))
    }

    pub async fn retry_failed_deliveries(&self) -> Result<(), ApiError> {
        // Implement retry logic for failed deliveries
        // This would check the retry queue and attempt redelivery
        Ok(())
    }
}

// Event types
pub mod event_types {
    pub const PLAN_CREATED: &str = "plan.created";
    pub const PLAN_CLAIMED: &str = "plan.claimed";
    pub const LOAN_DISBURSED: &str = "loan.disbursed";
    pub const LOAN_REPAID: &str = "loan.repaid";
    pub const KYC_SUBMITTED: &str = "kyc.submitted";
    pub const KYC_APPROVED: &str = "kyc.approved";
    pub const KYC_REJECTED: &str = "kyc.rejected";
}

// API handlers
pub async fn register_webhook(
    State(service): State<Arc<WebhookService>>,
    user_id: crate::auth::AuthenticatedUser,
    Json(request): Json<CreateWebhookRequest>,
) -> Result<Json<Webhook>, (StatusCode, String)> {
    match service.register_webhook(user_id.0, request).await {
        Ok(webhook) => Ok(Json(webhook)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn get_webhooks(
    State(service): State<Arc<WebhookService>>,
    user_id: crate::auth::AuthenticatedUser,
) -> Result<Json<Vec<Webhook>>, (StatusCode, String)> {
    match service.get_webhooks(user_id.0).await {
        Ok(webhooks) => Ok(Json(webhooks)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn delete_webhook(
    State(service): State<Arc<WebhookService>>,
    user_id: crate::auth::AuthenticatedUser,
    Path(webhook_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    match service.delete_webhook(user_id.0, webhook_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
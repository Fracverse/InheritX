use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::Value;
use sha2::Sha256;
use sqlx::{PgPool, Row, Postgres, Transaction};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

type HmacSha256 = Hmac<Sha256>;

pub struct WebhookDispatcherService {
    db: PgPool,
    client: Client,
}

impl WebhookDispatcherService {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            client: Client::new(),
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.run_once().await {
                    error!("Webhook dispatcher run failed: {e}");
                }

                sleep(Duration::from_secs(5)).await;
            }
        });
    }

    async fn run_once(&self) -> Result<(), sqlx::Error> {
        let mut tx: Transaction<'_, Postgres> = self.db.begin().await?;

        // select pending dispatches ready to run
        let rows = sqlx::query(
            "SELECT wd.id, wd.endpoint_id, wd.event_type, wd.payload, wd.attempts, we.url, we.secret
               FROM webhook_dispatches wd
               JOIN webhook_endpoints we ON wd.endpoint_id = we.id
               WHERE wd.status = 'pending' AND (wd.next_attempt_at IS NULL OR wd.next_attempt_at <= NOW())
               ORDER BY wd.created_at ASC
               LIMIT 25 FOR UPDATE SKIP LOCKED",
        )
        .fetch_all(&mut tx)
        .await?;

        for row in rows {
            let id: uuid::Uuid = row.get("id");
            let url: String = row.get("url");
            let secret: Option<String> = row.get("secret");
            let payload: Value = row.get("payload");
            let attempts: i32 = row.get("attempts");
            let event_type: String = row.get("event_type");

            let body = serde_json::to_vec(&payload).unwrap_or_default();

            // compute HMAC-SHA256 signature
            let key = secret.unwrap_or_default();
            let mut mac = HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
            mac.update(&body);
            let signature = hex::encode(mac.finalize().into_bytes());

            let res = self
                .client
                .post(&url)
                .header("X-Event-Type", event_type.clone())
                .header("X-Signature", format!("sha256={}", signature))
                .json(&payload)
                .send()
                .await;

            match res {
                Ok(resp) if resp.status().is_success() => {
                    sqlx::query("UPDATE webhook_dispatches SET status = 'success', updated_at = NOW() WHERE id = $1")
                        .bind(id)
                        .execute(&mut tx)
                        .await?;
                    info!("Webhook dispatch {} succeeded", id);
                }
                Ok(mut resp) => {
                    let status = resp.status().as_u16();
                    let text = resp.text().await.unwrap_or_default();
                    warn!("Webhook dispatch {} returned status {}: {}", id, status, text);
                    Self::handle_failure(&mut tx, id, attempts, Some(format!("status {}: {}", status, text))).await?;
                }
                Err(e) => {
                    warn!("Webhook dispatch {} request error: {}", id, e);
                    Self::handle_failure(&mut tx, id, attempts, Some(e.to_string())).await?;
                }
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn handle_failure(tx: &mut Transaction<'_, Postgres>, id: uuid::Uuid, attempts: i32, last_error: Option<String>) -> Result<(), sqlx::Error> {
        let next_attempts = attempts + 1;
        let max_attempts_row = sqlx::query("SELECT max_attempts FROM webhook_dispatches WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;

        let max_attempts: i32 = if let Some(row) = max_attempts_row {
            row.get::<Option<i32>, _>("max_attempts").unwrap_or(5)
        } else {
            5
        };

        if next_attempts >= max_attempts {
            // mark failed
            sqlx::query("UPDATE webhook_dispatches SET attempts = $1, status = 'failed', last_error = $2, updated_at = NOW() WHERE id = $3")
                .bind(next_attempts)
                .bind(last_error)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        } else {
            let backoff_secs = 2u64.pow(next_attempts as u32);
            sqlx::query("UPDATE webhook_dispatches SET attempts = $1, last_error = $2, next_attempt_at = (NOW() + ($3 || ' seconds')::interval), updated_at = NOW() WHERE id = $4")
                .bind(next_attempts)
                .bind(last_error)
                .bind(backoff_secs as i64)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        Ok(())
    }

    // Enqueue a payload for all active endpoints
    pub async fn enqueue_event(db: &PgPool, event_type: &str, payload: &Value) -> Result<(), sqlx::Error> {
        let mut tx = db.begin().await?;
        let endpoints = sqlx::query("SELECT id FROM webhook_endpoints WHERE is_active = true").fetch_all(&mut tx).await?;
        for ep in endpoints {
            let endpoint_id: uuid::Uuid = ep.get("id");
            sqlx::query("INSERT INTO webhook_dispatches (endpoint_id, event_type, payload) VALUES ($1, $2, $3)")
                .bind(endpoint_id)
                .bind(event_type)
                .bind(payload.clone())
                .execute(&mut tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // light compile-time sanity checks
    use super::*;
    #[test]
    fn smoke() {
        let _ = WebhookDispatcherService::new(sqlx::PgPool::connect_lazy("postgres://localhost").unwrap());
    }
}

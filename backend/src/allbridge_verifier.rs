use crate::allbridge;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::MissedTickBehavior;
use tracing::{error, info, warn};

#[derive(Clone, Debug)]
pub struct AllbridgeVerifierConfig {
    pub poll_interval: Duration,
    pub max_retries: u32,
}

impl AllbridgeVerifierConfig {
    pub fn from_env() -> Self {
        let secs = std::env::var("ALLBRIDGE_VERIFIER_POLL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        let max_retries = std::env::var("ALLBRIDGE_VERIFIER_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        Self {
            poll_interval: Duration::from_secs(secs),
            max_retries,
        }
    }
}

pub struct AllbridgeVerifierService {
    db: PgPool,
    client: Client,
    config: AllbridgeVerifierConfig,
}

impl AllbridgeVerifierService {
    pub fn new(db: PgPool, config: AllbridgeVerifierConfig) -> Self {
        Self {
            db,
            client: Client::new(),
            config,
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.config.poll_interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                interval.tick().await;

                if let Err(e) = self.run_once().await {
                    error!("Allbridge verifier run failed: {}", e);
                }
            }
        });
    }

    async fn run_once(&self) -> Result<(), sqlx::Error> {
        // Acquire advisory lock to avoid parallel workers
        let mut tx = self.db.begin().await?;
        let lock_acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
            .bind(9999i64)
            .fetch_one(&mut *tx)
            .await?;

        if !lock_acquired {
            warn!("Allbridge verifier lock held by another worker; skipping this cycle");
            tx.commit().await?;
            return Ok(());
        }

        // Select a batch of PENDING transfers to poll
        let rows = sqlx::query!(
            r#"SELECT id, origin_tx_hash, status, created_at, updated_at FROM allbridge_transfers
               WHERE status = 'PENDING' OR status = 'PROCESSING'
               ORDER BY created_at ASC
               LIMIT 50 FOR UPDATE SKIP LOCKED"#
        )
        .fetch_all(&mut *tx)
        .await?;

        for r in rows {
            let origin = r.origin_tx_hash.clone();
            match allbridge::fetch_transfer_status_remote(&self.client, &origin).await {
                Ok(status) => {
                    info!("Polled status for {} => {}", origin, status);
                    let new_status = match status.to_uppercase().as_str() {
                        "COMPLETED" | "SUCCESS" | "CONFIRMED" => "CONFIRMED",
                        "FAILED" | "ERROR" => "FAILED",
                        _ => "PROCESSING",
                    };

                    if let Err(e) = sqlx::query!(
                        "UPDATE allbridge_transfers SET status = $1, updated_at = NOW(), last_polled_at = NOW() WHERE id = $2",
                        new_status,
                        r.id
                    ).execute(&mut *tx).await {
                        error!("Failed to update transfer {}: {}", r.id, e);
                    }

                    // If confirmed, additional domain actions could be taken here (credits, notify)
                }
                Err(e) => {
                    error!("Failed to poll Allbridge for {}: {}", origin, e);
                    // leave row for retry; optionally increment retry counter in DB
                }
            }
        }

        tx.commit().await?;
        Ok(())
    }
}

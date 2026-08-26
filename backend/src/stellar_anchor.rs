use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPayoutRequest {
    pub plan_id: Option<Uuid>,
    pub beneficiary_address: String,
    pub beneficiary_name: String,
    pub token: String,
    pub token_amount: f64,
    pub fiat_currency: String,
    pub bank_name: String,
    pub account_number: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AnchorPayoutStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPayout {
    pub id: Uuid,
    pub plan_id: Option<Uuid>,
    pub request: AnchorPayoutRequest,
    pub exchange_rate: f64,
    pub fiat_amount: f64,
    pub anchor_fee_usd: f64,
    pub external_transaction_id: Option<String>,
    pub status: AnchorPayoutStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct AnchorApiResponse {
    id: Option<String>,
    transaction_id: Option<String>,
    status: Option<String>,
    exchange_rate: Option<f64>,
    fiat_amount: Option<f64>,
    fee: Option<f64>,
    message: Option<String>,
}

pub struct AnchorRegistry {
    client: Client,
    api_url: String,
    pool: PgPool,
}

impl AnchorRegistry {
    pub fn new(api_url: String, pool: PgPool) -> Self {
        Self {
            client: Client::new(),
            api_url,
            pool,
        }
    }

    pub async fn create_payout(self: &Arc<Self>, req: AnchorPayoutRequest) -> AnchorPayout {
        let url = format!("{}/transactions/send", self.api_url.trim_end_matches('/'));

        let payload = serde_json::json!({
            "beneficiary_address": req.beneficiary_address,
            "beneficiary_name": req.beneficiary_name,
            "token": req.token,
            "token_amount": req.token_amount,
            "fiat_currency": req.fiat_currency,
            "bank_name": req.bank_name,
            "account_number": req.account_number,
        });

        // Call the Stellar Anchor API
        let (external_tx_id, exchange_rate, fiat_amount, anchor_fee, status) = 
            match self.client.post(&url).json(&payload).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.json::<AnchorApiResponse>().await {
                            Ok(api_resp) => {
                                if let Some(msg) = &api_resp.message {
                                    warn!(message = %msg, "Anchor API response message");
                                }
                                let status = match api_resp.status.as_deref() {
                                    Some("completed") => AnchorPayoutStatus::Completed,
                                    Some("processing") | Some("pending") => {
                                        AnchorPayoutStatus::Processing
                                    }
                                    Some("failed") => AnchorPayoutStatus::Failed,
                                    _ => AnchorPayoutStatus::Processing,
                                };

                                let tx_id = api_resp.id.or(api_resp.transaction_id);
                                (
                                    tx_id,
                                    api_resp.exchange_rate.unwrap_or(1.0),
                                    api_resp.fiat_amount.unwrap_or(0.0),
                                    api_resp.fee.unwrap_or(0.0),
                                    status,
                                )
                            }
                            Err(e) => {
                                warn!(
                                    anchor_url = %url,
                                    error = %e,
                                    "Failed to parse anchor API response"
                                );
                                (None, 1.0, 0.0, 0.0, AnchorPayoutStatus::Failed)
                            }
                        }
                    } else {
                        let status_code = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        error!(
                            anchor_url = %url,
                            status = %status_code,
                            body = %body,
                            "Anchor API returned error"
                        );
                        (None, 1.0, 0.0, 0.0, AnchorPayoutStatus::Failed)
                    }
                }
                Err(e) => {
                    error!(
                        anchor_url = %url,
                        error = %e,
                        "Failed to reach anchor API"
                    );
                    (None, 1.0, 0.0, 0.0, AnchorPayoutStatus::Failed)
                }
            };

        // Convert status to database enum string
        let status_str = match status {
            AnchorPayoutStatus::Pending => "pending",
            AnchorPayoutStatus::Processing => "processing",
            AnchorPayoutStatus::Completed => "completed",
            AnchorPayoutStatus::Failed => "failed",
        };

        // Persist to database
        let payout_id = Uuid::new_v4();
        let amount_dec = Decimal::from_f64_retain(req.token_amount)
            .unwrap_or(Decimal::ZERO);
        let exchange_rate_dec = Decimal::from_f64_retain(exchange_rate)
            .unwrap_or(Decimal::ONE);
        let anchor_fee_dec = Decimal::from_f64_retain(anchor_fee)
            .unwrap_or(Decimal::ZERO);

        let now = chrono::Utc::now();

        match sqlx::query(
            r#"
            INSERT INTO payouts (
                id,
                plan_id,
                beneficiary_address,
                amount,
                payout_type,
                status,
                exchange_rate,
                anchor_fee_usd,
                external_transaction_id,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(payout_id)
        .bind(req.plan_id)
        .bind(&req.beneficiary_address)
        .bind(amount_dec)
        .bind("fiat")
        .bind(status_str)
        .bind(exchange_rate_dec)
        .bind(anchor_fee_dec)
        .bind(external_tx_id.as_ref())
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        {
            Ok(_) => {
                info!(
                    payout_id = %payout_id,
                    beneficiary = %req.beneficiary_address,
                    status = %status_str,
                    "Payout persisted to database"
                );
            }
            Err(e) => {
                error!(
                    payout_id = %payout_id,
                    error = %e,
                    "Failed to persist payout to database"
                );
            }
        }

        AnchorPayout {
            id: payout_id,
            plan_id: req.plan_id,
            request: req,
            exchange_rate,
            fiat_amount,
            anchor_fee_usd: anchor_fee,
            external_transaction_id: external_tx_id,
            status,
            created_at: now,
            updated_at: now,
        }
    }

    pub async fn get_payout(&self, id: &Uuid) -> Option<AnchorPayout> {
        #[derive(sqlx::FromRow)]
        struct PayoutRow {
            id: Uuid,
            plan_id: Option<Uuid>,
            beneficiary_address: String,
            amount: Decimal,
            status: String,
            exchange_rate: Decimal,
            anchor_fee_usd: Decimal,
            external_transaction_id: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, PayoutRow>(
            r#"
            SELECT 
                id, 
                plan_id, 
                beneficiary_address, 
                amount, 
                status, 
                exchange_rate, 
                anchor_fee_usd, 
                external_transaction_id,
                created_at,
                updated_at
            FROM payouts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        let status = match row.status.as_str() {
            "pending" => AnchorPayoutStatus::Pending,
            "processing" => AnchorPayoutStatus::Processing,
            "completed" => AnchorPayoutStatus::Completed,
            "failed" => AnchorPayoutStatus::Failed,
            _ => AnchorPayoutStatus::Pending,
        };

        Some(AnchorPayout {
            id: row.id,
            plan_id: row.plan_id,
            request: AnchorPayoutRequest {
                plan_id: row.plan_id,
                beneficiary_address: row.beneficiary_address.clone(),
                beneficiary_name: String::new(),
                token: String::new(),
                token_amount: row.amount.to_string().parse().unwrap_or(0.0),
                fiat_currency: String::new(),
                bank_name: String::new(),
                account_number: String::new(),
            },
            exchange_rate: row.exchange_rate.to_string().parse().unwrap_or(1.0),
            fiat_amount: 0.0,
            anchor_fee_usd: row.anchor_fee_usd.to_string().parse().unwrap_or(0.0),
            external_transaction_id: row.external_transaction_id,
            status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn list_payouts(&self, address: Option<&str>) -> Vec<AnchorPayout> {
        #[derive(sqlx::FromRow)]
        struct PayoutRow {
            id: Uuid,
            plan_id: Option<Uuid>,
            beneficiary_address: String,
            amount: Decimal,
            status: String,
            exchange_rate: Decimal,
            anchor_fee_usd: Decimal,
            external_transaction_id: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let rows = if let Some(addr) = address {
            sqlx::query_as::<_, PayoutRow>(
                r#"
                SELECT 
                    id, 
                    plan_id, 
                    beneficiary_address, 
                    amount, 
                    status, 
                    exchange_rate, 
                    anchor_fee_usd, 
                    external_transaction_id,
                    created_at,
                    updated_at
                FROM payouts
                WHERE beneficiary_address = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(addr)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PayoutRow>(
                r#"
                SELECT 
                    id, 
                    plan_id, 
                    beneficiary_address, 
                    amount, 
                    status, 
                    exchange_rate, 
                    anchor_fee_usd, 
                    external_transaction_id,
                    created_at,
                    updated_at
                FROM payouts
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await
        };

        rows.unwrap_or_default()
            .into_iter()
            .map(|row| {
                let status = match row.status.as_str() {
                    "pending" => AnchorPayoutStatus::Pending,
                    "processing" => AnchorPayoutStatus::Processing,
                    "completed" => AnchorPayoutStatus::Completed,
                    "failed" => AnchorPayoutStatus::Failed,
                    _ => AnchorPayoutStatus::Pending,
                };

                AnchorPayout {
                    id: row.id,
                    plan_id: row.plan_id,
                    request: AnchorPayoutRequest {
                        plan_id: row.plan_id,
                        beneficiary_address: row.beneficiary_address.clone(),
                        beneficiary_name: String::new(),
                        token: String::new(),
                        token_amount: row.amount.to_string().parse().unwrap_or(0.0),
                        fiat_currency: String::new(),
                        bank_name: String::new(),
                        account_number: String::new(),
                    },
                    exchange_rate: row.exchange_rate.to_string().parse().unwrap_or(1.0),
                    fiat_amount: 0.0,
                    anchor_fee_usd: row.anchor_fee_usd.to_string().parse().unwrap_or(0.0),
                    external_transaction_id: row.external_transaction_id,
                    status,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }
            })
            .collect()
    }
}

/// Background worker that periodically polls Stellar Anchor status endpoints
/// to update payout statuses in the database.
pub async fn spawn_payout_status_poller(
    registry: Arc<AnchorRegistry>,
    polling_interval_secs: u64,
) {
    let interval = Duration::from_secs(polling_interval_secs);
    
    info!(
        interval_secs = polling_interval_secs,
        "Starting payout status polling worker"
    );

    loop {
        tokio::time::sleep(interval).await;

        // Fetch all active payouts (pending or processing)
        let active_payouts = match fetch_active_payouts(&registry.pool).await {
            Ok(payouts) => payouts,
            Err(e) => {
                error!(error = %e, "Failed to fetch active payouts");
                continue;
            }
        };

        if active_payouts.is_empty() {
            continue;
        }

        info!(count = active_payouts.len(), "Polling status for active payouts");

        for payout in active_payouts {
            if let Some(ref external_tx_id) = payout.external_transaction_id {
                // Query the Stellar Anchor's transaction status endpoint
                match query_anchor_status(&registry, external_tx_id).await {
                    Ok(new_status) => {
                        // Only update if status has changed
                        if new_status != payout.status {
                            if let Err(e) = update_payout_status(
                                &registry.pool,
                                payout.id,
                                new_status,
                            )
                            .await
                            {
                                error!(
                                    payout_id = %payout.id,
                                    error = %e,
                                    "Failed to update payout status"
                                );
                            } else {
                                info!(
                                    payout_id = %payout.id,
                                    old_status = ?payout.status,
                                    new_status = ?new_status,
                                    "Payout status updated"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            payout_id = %payout.id,
                            external_tx_id = %external_tx_id,
                            error = %e,
                            "Failed to query anchor status"
                        );
                    }
                }
            } else {
                warn!(
                    payout_id = %payout.id,
                    "Payout has no external_transaction_id, skipping status poll"
                );
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct ActivePayoutRow {
    id: Uuid,
    external_transaction_id: Option<String>,
    status: String,
}

async fn fetch_active_payouts(pool: &PgPool) -> Result<Vec<ActivePayoutRow>, sqlx::Error> {
    sqlx::query_as::<_, ActivePayoutRow>(
        r#"
        SELECT id, external_transaction_id, status
        FROM payouts
        WHERE status IN ('pending', 'processing')
        AND external_transaction_id IS NOT NULL
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await
}

async fn query_anchor_status(
    registry: &Arc<AnchorRegistry>,
    external_tx_id: &str,
) -> Result<String, anyhow::Error> {
    let url = format!(
        "{}/transaction/{}",
        registry.api_url.trim_end_matches('/'),
        external_tx_id
    );

    let resp = registry
        .client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Anchor API returned status {}", resp.status());
    }

    #[derive(Deserialize)]
    struct StatusResponse {
        status: Option<String>,
    }

    let status_resp: StatusResponse = resp.json().await?;
    
    let status = match status_resp.status.as_deref() {
        Some("completed") => "completed",
        Some("processing") | Some("pending") | Some("pending_user") | Some("pending_external") => "processing",
        Some("error") | Some("refunded") | Some("failed") => "failed",
        _ => "processing",
    };

    Ok(status.to_string())
}

async fn update_payout_status(
    pool: &PgPool,
    payout_id: Uuid,
    status: String,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE payouts
        SET status = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(&status)
    .bind(payout_id)
    .execute(pool)
    .await?;

    Ok(())
}

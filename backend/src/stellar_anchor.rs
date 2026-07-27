use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPayoutRequest {
    pub beneficiary_address: String,
    pub beneficiary_name: String,
    pub token: String,
    pub token_amount: f64,
    pub fiat_currency: String,
    pub bank_name: String,
    pub account_number: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnchorPayoutStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl AnchorPayoutStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnchorPayoutStatus::Pending => "pending",
            AnchorPayoutStatus::Processing => "processing",
            AnchorPayoutStatus::Completed => "completed",
            AnchorPayoutStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPayout {
    pub id: String,
    pub request: AnchorPayoutRequest,
    pub exchange_rate: f64,
    pub fiat_amount: f64,
    pub anchor_fee_usd: f64,
    pub status: AnchorPayoutStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnchorTransactionRequest {
    amount: String,
    asset_code: String,
    destination_asset: String,
    sender_id: String,
    fields: TransactionFields,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionFields {
    #[serde(rename = "transaction")]
    transaction: TransactionDetail,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionDetail {
    beneficiary_name: String,
    bank_name: String,
    account_number: String,
}

#[derive(Debug, Deserialize)]
struct AnchorTransactionResponse {
    id: String,
    status: String,
    #[serde(default)]
    amount_in: Option<String>,
    #[serde(default)]
    amount_out: Option<String>,
    #[serde(default)]
    amount_fee: Option<String>,
    #[serde(default)]
    stellar_transaction_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnchorTransactionStatusResponse {
    transaction: AnchorTransactionStatus,
}

#[derive(Debug, Deserialize)]
struct AnchorTransactionStatus {
    id: String,
    status: String,
    #[serde(default)]
    amount_in: Option<String>,
    #[serde(default)]
    amount_out: Option<String>,
    #[serde(default)]
    amount_fee: Option<String>,
    #[serde(default)]
    stellar_account: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnchorTransactionListResponse {
    transactions: Vec<AnchorTransactionListItem>,
}

#[derive(Debug, Deserialize)]
struct AnchorTransactionListItem {
    id: String,
    status: String,
    #[serde(default)]
    amount_in: Option<String>,
    #[serde(default)]
    amount_out: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AnchorError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Anchor returned error status {status}: {body}")]
    Api { status: u16, body: String },
    #[error("Anchor API not configured")]
    NotConfigured,
    #[error("Payout not found")]
    NotFound,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub struct AnchorRegistry {
    client: reqwest::Client,
    api_url: Option<String>,
    api_key: Option<String>,
}

impl AnchorRegistry {
    pub fn new(
        api_url: Option<String>,
        api_key: Option<String>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url,
            api_key,
        }
    }

    pub async fn create_payout(
        self: &Arc<Self>,
        req: AnchorPayoutRequest,
    ) -> Result<AnchorPayout, AnchorError> {
        let api_url = match &self.api_url {
            Some(url) => url.trim_end_matches('/').to_string(),
            None => {
                warn!("Anchor API not configured — returning simulated payout for {}", req.beneficiary_address);
                return Ok(simulated_payout(req));
            }
        };

        let payload = AnchorTransactionRequest {
            amount: format!("{:.7}", req.token_amount),
            asset_code: req.token.clone(),
            destination_asset: req.fiat_currency.clone(),
            sender_id: req.beneficiary_address.clone(),
            fields: TransactionFields {
                transaction: TransactionDetail {
                    beneficiary_name: req.beneficiary_name.clone(),
                    bank_name: req.bank_name.clone(),
                    account_number: req.account_number.clone(),
                },
            },
        };

        let mut request_builder = self
            .client
            .post(format!("{}/transactions", api_url))
            .json(&payload);

        if let Some(key) = &self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = match request_builder.send().await {
            Ok(resp) => resp,
            Err(e) => {
                error!(error = %e, beneficiary = %req.beneficiary_address, "Failed to send create payout request to anchor");
                return Err(AnchorError::Http(e));
            }
        };

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            error!(
                status = %status,
                body = %body,
                beneficiary = %req.beneficiary_address,
                "Anchor API returned error"
            );
            return Err(AnchorError::Api {
                status: status.as_u16(),
                body,
            });
        }

        match response.json::<AnchorTransactionResponse>().await {
            Ok(anchor_resp) => {
                info!(
                    anchor_tx_id = %anchor_resp.id,
                    status = %anchor_resp.status,
                    beneficiary = %req.beneficiary_address,
                    "Anchor payout created successfully"
                );

                let exchange_rate = if let (Some(_in_val), Some(out_val)) =
                    (&anchor_resp.amount_in, &anchor_resp.amount_out)
                {
                    if req.token_amount > 0.0 {
                        out_val.parse::<f64>().unwrap_or(1.0) / req.token_amount
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

                let fiat_amount = anchor_resp
                    .amount_out
                    .as_deref()
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(req.token_amount * exchange_rate);

                let anchor_fee = anchor_resp
                    .amount_fee
                    .as_deref()
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);

                let mapped_status = map_anchor_status(&anchor_resp.status);
                let now = chrono::Utc::now().to_rfc3339();

                Ok(AnchorPayout {
                    id: anchor_resp.id,
                    request: req,
                    exchange_rate,
                    fiat_amount,
                    anchor_fee_usd: anchor_fee,
                    status: mapped_status,
                    created_at: now.clone(),
                    updated_at: now,
                })
            }
            Err(e) => {
                error!(error = %e, "Failed to parse anchor create payout response");
                Err(AnchorError::InvalidResponse(e.to_string()))
            }
        }
    }

    pub async fn get_payout(&self, id: &str) -> Result<AnchorPayout, AnchorError> {
        let api_url = match &self.api_url {
            Some(url) => url.trim_end_matches('/').to_string(),
            None => return Err(AnchorError::NotConfigured),
        };

        let mut request_builder = self
            .client
            .get(format!("{}/transactions/{}", api_url, id));

        if let Some(key) = &self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = match request_builder.send().await {
            Ok(resp) => resp,
            Err(e) => {
                error!(error = %e, anchor_id = %id, "Failed to get payout from anchor");
                return Err(AnchorError::Http(e));
            }
        };

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(AnchorError::NotFound);
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AnchorError::Api {
                status: status.as_u16(),
                body,
            });
        }

        match response.json::<AnchorTransactionStatusResponse>().await {
            Ok(status_resp) => {
                let tx = status_resp.transaction;
                let now = chrono::Utc::now().to_rfc3339();
                Ok(AnchorPayout {
                    id: tx.id,
                    request: AnchorPayoutRequest {
                        beneficiary_address: tx.stellar_account.clone().unwrap_or_default(),
                        beneficiary_name: String::new(),
                        token: String::new(),
                        token_amount: tx
                            .amount_in
                            .as_deref()
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(0.0),
                        fiat_currency: String::new(),
                        bank_name: String::new(),
                        account_number: String::new(),
                    },
                    exchange_rate: 1.0,
                    fiat_amount: tx
                        .amount_out
                        .as_deref()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0.0),
                    anchor_fee_usd: tx
                        .amount_fee
                        .as_deref()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0.0),
                    status: map_anchor_status(&tx.status),
                    created_at: now.clone(),
                    updated_at: tx.updated_at.unwrap_or(now),
                })
            }
            Err(e) => {
                error!(error = %e, anchor_id = %id, "Failed to parse anchor payout status response");
                Err(AnchorError::InvalidResponse(e.to_string()))
            }
        }
    }

    pub async fn list_payouts(
        &self,
        address: Option<String>,
    ) -> Result<Vec<AnchorPayout>, AnchorError> {
        let api_url = match &self.api_url {
            Some(url) => url.trim_end_matches('/').to_string(),
            None => return Err(AnchorError::NotConfigured),
        };

        let mut request_builder = self.client.get(format!("{}/transactions", api_url));

        if let Some(ref addr) = address {
            request_builder = request_builder.query(&[("sender_id", addr.as_str())]);
        }

        if let Some(key) = &self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = match request_builder.send().await {
            Ok(resp) => resp,
            Err(e) => {
                error!(error = %e, "Failed to list payouts from anchor");
                return Err(AnchorError::Http(e));
            }
        };

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AnchorError::Api {
                status: status.as_u16(),
                body,
            });
        }

        match response.json::<AnchorTransactionListResponse>().await {
            Ok(list_resp) => {
                let now = chrono::Utc::now().to_rfc3339();
                let payouts = list_resp
                    .transactions
                    .into_iter()
                    .map(|tx| AnchorPayout {
                        id: tx.id,
                        request: AnchorPayoutRequest {
                            beneficiary_address: String::new(),
                            beneficiary_name: String::new(),
                            token: String::new(),
                            token_amount: tx
                                .amount_in
                                .as_deref()
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0.0),
                            fiat_currency: String::new(),
                            bank_name: String::new(),
                            account_number: String::new(),
                        },
                        exchange_rate: 1.0,
                        fiat_amount: tx
                            .amount_out
                            .as_deref()
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(0.0),
                        anchor_fee_usd: 0.0,
                        status: map_anchor_status(&tx.status),
                        created_at: tx.created_at.unwrap_or_else(|| now.clone()),
                        updated_at: tx.created_at.unwrap_or(now.clone()),
                    })
                    .collect();

                Ok(payouts)
            }
            Err(e) => {
                error!(error = %e, "Failed to parse anchor list payouts response");
                Err(AnchorError::InvalidResponse(e.to_string()))
            }
        }
    }
}

fn map_anchor_status(status: &str) -> AnchorPayoutStatus {
    match status.to_lowercase().as_str() {
        "pending" => AnchorPayoutStatus::Pending,
        "processing" | "in_progress" => AnchorPayoutStatus::Processing,
        "completed" | "success" => AnchorPayoutStatus::Completed,
        "failed" | "error" | "rejected" => AnchorPayoutStatus::Failed,
        _ => {
            warn!(status = %status, "Unknown anchor payout status, defaulting to Pending");
            AnchorPayoutStatus::Pending
        }
    }
}

/// Creates a simulated payout response when the anchor API is not configured.
/// This allows the system to function in development/test mode.
fn simulated_payout(req: AnchorPayoutRequest) -> AnchorPayout {
    let now = chrono::Utc::now().to_rfc3339();
    AnchorPayout {
        id: uuid::Uuid::new_v4().to_string(),
        request: req,
        exchange_rate: 1.0,
        fiat_amount: 0.0,
        anchor_fee_usd: 0.0,
        status: AnchorPayoutStatus::Pending,
        created_at: now.clone(),
        updated_at: now,
    }
}

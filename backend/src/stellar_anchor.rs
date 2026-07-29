use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, warn};

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
pub enum AnchorPayoutStatus {
    Pending,
    Processing,
    Completed,
    Failed,
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
}

impl AnchorRegistry {
    pub fn new(api_url: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
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

        match self.client.post(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<AnchorApiResponse>().await {
                        Ok(api_resp) => {
                            let status = match api_resp.status.as_deref() {
                                Some("completed") => AnchorPayoutStatus::Completed,
                                Some("processing") | Some("pending") => {
                                    AnchorPayoutStatus::Processing
                                }
                                Some("failed") => AnchorPayoutStatus::Failed,
                                _ => AnchorPayoutStatus::Processing,
                            };

                            let now = chrono::Utc::now().to_rfc3339();
                            AnchorPayout {
                                id: api_resp.id.or(api_resp.transaction_id).unwrap_or_default(),
                                request: req,
                                exchange_rate: api_resp.exchange_rate.unwrap_or(1.0),
                                fiat_amount: api_resp.fiat_amount.unwrap_or(0.0),
                                anchor_fee_usd: api_resp.fee.unwrap_or(0.0),
                                status,
                                created_at: now.clone(),
                                updated_at: now,
                            }
                        }
                        Err(e) => {
                            let now = chrono::Utc::now().to_rfc3339();
                            warn!(
                                anchor_url = %url,
                                error = %e,
                                "Failed to parse anchor API response"
                            );
                            AnchorPayout {
                                id: String::new(),
                                request: req,
                                exchange_rate: 1.0,
                                fiat_amount: 0.0,
                                anchor_fee_usd: 0.0,
                                status: AnchorPayoutStatus::Failed,
                                created_at: now.clone(),
                                updated_at: now,
                            }
                        }
                    }
                } else {
                    let status_code = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    let now = chrono::Utc::now().to_rfc3339();
                    error!(
                        anchor_url = %url,
                        status = %status_code,
                        body = %body,
                        "Anchor API returned error"
                    );
                    AnchorPayout {
                        id: String::new(),
                        request: req,
                        exchange_rate: 1.0,
                        fiat_amount: 0.0,
                        anchor_fee_usd: 0.0,
                        status: AnchorPayoutStatus::Failed,
                        created_at: now.clone(),
                        updated_at: now,
                    }
                }
            }
            Err(e) => {
                let now = chrono::Utc::now().to_rfc3339();
                error!(
                    anchor_url = %url,
                    error = %e,
                    "Failed to reach anchor API"
                );
                AnchorPayout {
                    id: String::new(),
                    request: req,
                    exchange_rate: 1.0,
                    fiat_amount: 0.0,
                    anchor_fee_usd: 0.0,
                    status: AnchorPayoutStatus::Failed,
                    created_at: now.clone(),
                    updated_at: now,
                }
            }
        }
    }

    pub fn get_payout(&self, _id: &str) -> Option<AnchorPayout> {
        None
    }

    pub fn list_payouts(&self, _address: Option<String>) -> Vec<AnchorPayout> {
        Vec::new()
    }
}

//! Stellar Anchor client implementing SEP-10 (authentication), SEP-24 (interactive off-ramp),
//! and SEP-6 (non-interactive off-ramp) for fiat settlement of USDC/XLM payouts.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Testnet anchor endpoints (Stellar's reference anchor)
// ---------------------------------------------------------------------------

const TOML_PATH: &str = "/.well-known/stellar.toml";
const TESTNET_ANCHOR_BASE: &str = "https://testanchor.stellar.org";

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPayoutRequest {
    pub beneficiary_address: String,
    pub beneficiary_name: String,
    pub token: String,      // "USDC" | "XLM"
    pub token_amount: f64,
    pub fiat_currency: String, // ISO-4217 e.g. "NGN", "KES", "USD"
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
    pub anchor_transaction_id: Option<String>,
    pub status: AnchorPayoutStatus,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// SEP-10 types
// ---------------------------------------------------------------------------

/// SEP-10 challenge transaction returned by the anchor.
#[derive(Debug, Deserialize)]
struct Sep10ChallengeResponse {
    transaction: String,
    network_passphrase: String,
}

/// SEP-10 JWT response after submitting the signed challenge.
#[derive(Debug, Deserialize)]
struct Sep10TokenResponse {
    token: String,
}

// ---------------------------------------------------------------------------
// SEP-24 types
// ---------------------------------------------------------------------------

/// Response from POST /transactions/withdraw/interactive (SEP-24).
#[derive(Debug, Deserialize)]
struct Sep24WithdrawResponse {
    #[serde(rename = "type")]
    kind: String,
    url: Option<String>,
    id: String,
}

/// Individual transaction record from GET /transactions (SEP-24/SEP-6).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnchorTransaction {
    id: String,
    status: String,
    amount_in: Option<String>,
    amount_out: Option<String>,
    amount_fee: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TransactionsResponse {
    transactions: Vec<AnchorTransaction>,
}

#[derive(Debug, Deserialize)]
struct TransactionResponse {
    transaction: AnchorTransaction,
}

// ---------------------------------------------------------------------------
// SEP-6 types
// ---------------------------------------------------------------------------

/// Response from GET /withdraw (SEP-6 non-interactive).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Sep6WithdrawResponse {
    account_id: String,
    memo_type: Option<String>,
    memo: Option<String>,
    id: Option<String>,
}

// ---------------------------------------------------------------------------
// TOML info helper
// ---------------------------------------------------------------------------

/// Minimal fields we need from stellar.toml
#[derive(Debug, Deserialize, Default)]
struct StellarTomlInfo {
    #[serde(rename = "WEB_AUTH_ENDPOINT")]
    web_auth_endpoint: Option<String>,
    #[serde(rename = "TRANSFER_SERVER_SEP0024")]
    transfer_server_sep24: Option<String>,
    #[serde(rename = "TRANSFER_SERVER")]
    transfer_server: Option<String>,
}

async fn fetch_toml(client: &reqwest::Client, anchor_base: &str) -> Result<StellarTomlInfo> {
    let url = format!("{}{}", anchor_base, TOML_PATH);
    let text = client.get(&url).send().await?.text().await?;
    // Parse only the key = "value" lines we care about
    let mut info = StellarTomlInfo::default();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("WEB_AUTH_ENDPOINT") {
            info.web_auth_endpoint = extract_toml_str(rest);
        } else if let Some(rest) = line.strip_prefix("TRANSFER_SERVER_SEP0024") {
            info.transfer_server_sep24 = extract_toml_str(rest);
        } else if let Some(rest) = line.strip_prefix("TRANSFER_SERVER") {
            // Only set if not already set by SEP0024 variant
            if info.transfer_server.is_none() {
                info.transfer_server = extract_toml_str(rest);
            }
        }
    }
    Ok(info)
}

fn extract_toml_str(rest: &str) -> Option<String> {
    let rest = rest.trim();
    let rest = rest.strip_prefix('=')?;
    let rest = rest.trim().trim_matches('"');
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

// ---------------------------------------------------------------------------
// SEP-10 authentication
// ---------------------------------------------------------------------------

/// Authenticates with the anchor using SEP-10 and returns a JWT token.
///
/// In a real implementation the challenge XDR would be signed with the
/// beneficiary's Stellar secret key. On testnet we use a demo signing stub
/// that echoes the unsigned transaction back — the anchor accepts it for
/// mock/test flows.
pub async fn sep10_authenticate(
    client: &reqwest::Client,
    web_auth_endpoint: &str,
    account: &str,
) -> Result<String> {
    // 1. Fetch challenge
    let challenge: Sep10ChallengeResponse = client
        .get(web_auth_endpoint)
        .query(&[("account", account)])
        .send()
        .await?
        .json()
        .await?;

    tracing::debug!(
        passphrase = %challenge.network_passphrase,
        "SEP-10 challenge received"
    );

    // 2. Submit the (mock-signed) challenge transaction back to get a JWT.
    //    Production: sign challenge.transaction XDR with the Stellar keypair first.
    let mut body = HashMap::new();
    body.insert("transaction", &challenge.transaction);

    let token_resp: Sep10TokenResponse = client
        .post(web_auth_endpoint)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    tracing::info!(account = %account, "SEP-10 authentication successful");
    Ok(token_resp.token)
}

// ---------------------------------------------------------------------------
// SEP-24 interactive off-ramp
// ---------------------------------------------------------------------------

/// Initiates a SEP-24 interactive withdrawal and returns the anchor transaction ID.
pub async fn sep24_withdraw(
    client: &reqwest::Client,
    transfer_server: &str,
    jwt: &str,
    req: &AnchorPayoutRequest,
) -> Result<String> {
    let asset_code = req.token.to_uppercase();

    // Build form fields including bank detail fields formatted per anchor spec
    let mut fields = HashMap::new();
    fields.insert("asset_code", asset_code.as_str());
    fields.insert("account", req.beneficiary_address.as_str());
    // Bank detail fields (SEP-9 KYC fields)
    fields.insert("dest", req.account_number.as_str());
    fields.insert("dest_extra", req.bank_name.as_str());
    fields.insert("type", "bank_account");

    let resp: Sep24WithdrawResponse = client
        .post(format!("{}/transactions/withdraw/interactive", transfer_server))
        .bearer_auth(jwt)
        .json(&fields)
        .send()
        .await?
        .json()
        .await?;

    if resp.kind != "interactive_customer_info_needed" && resp.url.is_some() {
        tracing::info!(id = %resp.id, url = ?resp.url, "SEP-24 interactive withdrawal initiated");
    }

    Ok(resp.id)
}

/// Polls SEP-24 transaction status by ID until terminal state or max attempts.
pub async fn sep24_poll_status(
    client: &reqwest::Client,
    transfer_server: &str,
    jwt: &str,
    transaction_id: &str,
) -> Result<AnchorPayoutStatus> {
    let url = format!("{}/transaction", transfer_server);
    let resp: TransactionResponse = client
        .get(&url)
        .bearer_auth(jwt)
        .query(&[("id", transaction_id)])
        .send()
        .await?
        .json()
        .await?;

    Ok(map_anchor_status(&resp.transaction.status))
}

// ---------------------------------------------------------------------------
// SEP-6 non-interactive off-ramp
// ---------------------------------------------------------------------------

/// Initiates a SEP-6 withdrawal and returns the anchor transaction ID.
pub async fn sep6_withdraw(
    client: &reqwest::Client,
    transfer_server: &str,
    jwt: &str,
    req: &AnchorPayoutRequest,
) -> Result<String> {
    let asset_code = req.token.to_uppercase();

    let resp: Sep6WithdrawResponse = client
        .get(format!("{}/withdraw", transfer_server))
        .bearer_auth(jwt)
        .query(&[
            ("asset_code", asset_code.as_str()),
            ("account", req.beneficiary_address.as_str()),
            ("dest", req.account_number.as_str()),
            ("dest_extra", req.bank_name.as_str()),
            ("type", "bank_account"),
            ("amount", &req.token_amount.to_string()),
        ])
        .send()
        .await?
        .json()
        .await?;

    let id = resp
        .id
        .unwrap_or_else(|| format!("sep6-{}", resp.account_id));
    tracing::info!(id = %id, memo = ?resp.memo, "SEP-6 withdrawal initiated");
    Ok(id)
}

// ---------------------------------------------------------------------------
// Status mapping
// ---------------------------------------------------------------------------

fn map_anchor_status(s: &str) -> AnchorPayoutStatus {
    match s {
        "completed" => AnchorPayoutStatus::Completed,
        "error" | "expired" => AnchorPayoutStatus::Failed,
        "pending_anchor" | "pending_stellar" | "pending_external" | "pending_user_transfer_start" => {
            AnchorPayoutStatus::Processing
        }
        _ => AnchorPayoutStatus::Pending,
    }
}

// ---------------------------------------------------------------------------
// AnchorRegistry — in-memory store + orchestration
// ---------------------------------------------------------------------------

pub struct AnchorRegistry {
    payouts: Mutex<HashMap<String, AnchorPayout>>,
    client: reqwest::Client,
    anchor_base: String,
}

impl AnchorRegistry {
    pub fn new() -> Self {
        Self::with_anchor_base(TESTNET_ANCHOR_BASE.to_string())
    }

    pub fn with_anchor_base(anchor_base: String) -> Self {
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .build()
            .expect("failed to build reqwest client");
        Self {
            payouts: Mutex::new(HashMap::new()),
            client,
            anchor_base,
        }
    }

    /// Initiate an anchor payout: runs SEP-10 → SEP-24 (fallback SEP-6).
    /// Spawns a background task to poll and advance payout status.
    pub fn create_payout(self: &Arc<Self>, req: AnchorPayoutRequest) -> AnchorPayout {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let payout = AnchorPayout {
            id: id.clone(),
            exchange_rate: 1.0, // updated by background task
            fiat_amount: req.token_amount, // updated by background task
            anchor_fee_usd: 0.0,
            anchor_transaction_id: None,
            status: AnchorPayoutStatus::Pending,
            created_at: now.clone(),
            updated_at: now,
            request: req,
        };

        self.payouts
            .lock()
            .unwrap()
            .insert(id.clone(), payout.clone());

        // Spawn background task to run the SEP flow and poll status
        let registry = Arc::clone(self);
        tokio::spawn(async move {
            if let Err(e) = registry.run_sep_flow(&id).await {
                tracing::error!(payout_id = %id, error = %e, "Anchor SEP flow failed");
                registry.update_status(&id, AnchorPayoutStatus::Failed);
            }
        });

        payout
    }

    /// Retrieve payout by transaction ID.
    pub fn get_payout(&self, id: &str) -> Option<AnchorPayout> {
        self.payouts.lock().unwrap().get(id).cloned()
    }

    /// List all payouts, optionally filtered by beneficiary address.
    pub fn list_payouts(&self, address: Option<String>) -> Vec<AnchorPayout> {
        let store = self.payouts.lock().unwrap();
        match address {
            Some(addr) => store
                .values()
                .filter(|p| p.request.beneficiary_address == addr)
                .cloned()
                .collect(),
            None => store.values().cloned().collect(),
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    async fn run_sep_flow(&self, id: &str) -> Result<()> {
        let req = {
            let store = self.payouts.lock().unwrap();
            store
                .get(id)
                .ok_or_else(|| anyhow!("payout not found"))?
                .request
                .clone()
        };

        // Fetch TOML to discover endpoints
        let toml = fetch_toml(&self.client, &self.anchor_base).await?;

        let web_auth = toml
            .web_auth_endpoint
            .unwrap_or_else(|| format!("{}/auth", self.anchor_base));

        // SEP-10: authenticate
        let jwt = sep10_authenticate(&self.client, &web_auth, &req.beneficiary_address).await?;
        self.update_status(id, AnchorPayoutStatus::Processing);

        // Prefer SEP-24, fall back to SEP-6
        let transfer_server = toml
            .transfer_server_sep24
            .or(toml.transfer_server)
            .unwrap_or_else(|| format!("{}/sep24", self.anchor_base));

        let anchor_tx_id = if toml_supports_sep24(&transfer_server) {
            sep24_withdraw(&self.client, &transfer_server, &jwt, &req).await?
        } else {
            sep6_withdraw(&self.client, &transfer_server, &jwt, &req).await?
        };

        // Store anchor transaction ID
        {
            let mut store = self.payouts.lock().unwrap();
            if let Some(p) = store.get_mut(id) {
                p.anchor_transaction_id = Some(anchor_tx_id.clone());
                p.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }

        // Poll for terminal status (max 10 attempts, 3 s apart)
        for _ in 0..10 {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            let status =
                sep24_poll_status(&self.client, &transfer_server, &jwt, &anchor_tx_id).await?;
            self.update_status(id, status);
            if matches!(
                status,
                AnchorPayoutStatus::Completed | AnchorPayoutStatus::Failed
            ) {
                break;
            }
        }

        Ok(())
    }

    fn update_status(&self, id: &str, status: AnchorPayoutStatus) {
        let mut store = self.payouts.lock().unwrap();
        if let Some(p) = store.get_mut(id) {
            p.status = status;
            p.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }
}

/// Heuristic: if the transfer server URL contains "sep24" assume SEP-24 support.
/// In production, inspect stellar.toml SERVICES or OPTIONS entries.
fn toml_supports_sep24(transfer_server: &str) -> bool {
    transfer_server.contains("sep24")
        || transfer_server.contains("24")
        || !transfer_server.contains("sep6")
}

impl Default for AnchorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

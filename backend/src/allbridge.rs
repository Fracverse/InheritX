use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info};

static CACHE: Lazy<RwLock<Option<(u64, AllbridgeTokensResponse)>>> = Lazy::new(|| RwLock::new(None));

#[derive(Debug, Deserialize, Serialize)]
pub struct ChainInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenInfo {
    pub chain: String,
    pub token_address: String,
    pub symbol: String,
    pub decimals: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BridgeRoute {
    pub from_chain: String,
    pub to_chain: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AllbridgeTokensResponse {
    pub chains: Vec<ChainInfo>,
    pub tokens: Vec<TokenInfo>,
    pub routes: Vec<BridgeRoute>,
}

#[derive(Debug, Deserialize)]
pub struct QuoteRequest {
    pub origin_chain: String,
    pub origin_token: String,
    pub target_chain: String,
    pub target_token: String,
    pub amount: String, // string to preserve precision
    pub origin_tx_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QuoteResponse {
    pub origin_chain: String,
    pub origin_token: String,
    pub target_chain: String,
    pub target_token: String,
    pub amount: String,
    pub fees: String,
    pub slippage: f64,
    pub target_amount: String,
}

fn allbridge_base() -> String {
    std::env::var("ALLBRIDGE_API_BASE").unwrap_or_else(|_| "https://api.allbridge.io".to_string())
}

pub async fn fetch_tokens_remote(client: &Client) -> Result<AllbridgeTokensResponse, reqwest::Error> {
    let url = format!("{}/v1/tokens", allbridge_base());
    let resp = client.get(&url).send().await?;
    let parsed = resp.json::<AllbridgeTokensResponse>().await?;
    Ok(parsed)
}

/// Public: get tokens with simple caching (5 minutes)
pub async fn get_cached_tokens(client: &Client) -> Result<AllbridgeTokensResponse, reqwest::Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    {
        let guard = CACHE.read().await;
        if let Some((ts, data)) = &*guard {
            if now.saturating_sub(*ts) < 300 {
                return Ok(data.clone());
            }
        }
    }

    match fetch_tokens_remote(client).await {
        Ok(data) => {
            let mut guard = CACHE.write().await;
            *guard = Some((now, data.clone()));
            Ok(data)
        }
        Err(e) => Err(e),
    }
}

/// Persist a transfer record in DB and return the inserted id
pub async fn persist_transfer(
    pool: &PgPool,
    origin_tx_hash: &str,
    origin_chain: &str,
    origin_token: &str,
    target_chain: &str,
    target_token: &str,
    amount: &str,
    target_amount: &str,
    fees: &str,
) -> Result<uuid::Uuid, sqlx::Error> {
    let row = sqlx::query_scalar(
        r#"INSERT INTO allbridge_transfers (
            origin_tx_hash, origin_chain, origin_token, target_chain, target_token, amount, target_amount, fees, status
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING id"#,
    )
    .bind(origin_tx_hash)
    .bind(origin_chain)
    .bind(origin_token)
    .bind(target_chain)
    .bind(target_token)
    .bind(sqlx::types::Decimal::from_str_exact(amount).unwrap_or(sqlx::types::Decimal::from(0)))
    .bind(sqlx::types::Decimal::from_str_exact(target_amount).unwrap_or(sqlx::types::Decimal::from(0)))
    .bind(sqlx::types::Decimal::from_str_exact(fees).unwrap_or(sqlx::types::Decimal::from(0)))
    .bind("PENDING")
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn fetch_transfer_status_remote(client: &Client, origin_tx_hash: &str) -> Result<String, reqwest::Error> {
    let url = format!("{}/v1/txs/{}", allbridge_base(), origin_tx_hash);
    let resp = client.get(&url).send().await?;
    // Flexible parsing: try to find `status` field
    let v: serde_json::Value = resp.json().await?;
    if let Some(status) = v.get("status").and_then(|s| s.as_str()) {
        Ok(status.to_string())
    } else if let Some(status) = v.get("state").and_then(|s| s.as_str()) {
        Ok(status.to_string())
    } else {
        Ok("UNKNOWN".to_string())
    }
}

pub fn calculate_dummy_fees(amount_str: &str) -> (String, f64, String) {
    // Very small, deterministic fee calc: 0.5% fee, slippage 0.5%
    // amount_str expected to be integer-like string representing smallest unit
    // For simplicity, treat as decimal string
    let amt = rust_decimal::Decimal::from_str_exact(amount_str).unwrap_or(rust_decimal::Decimal::ZERO);
    let fee = (amt * rust_decimal::Decimal::new(5, 3)).round_dp(0); // 0.5% ~ multiply by 0.005
    let target = (amt - fee).max(rust_decimal::Decimal::ZERO);
    (fee.to_string(), 0.005, target.to_string())
}

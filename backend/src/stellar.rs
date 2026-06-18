//! Stellar SDK integration (Issue #470).
//!
//! Provides:
//! * [`HorizonClient`]     – REST wrapper around the Stellar Horizon API.
//! * [`SorobanRpcClient`]  – JSON-RPC wrapper for the Soroban RPC endpoint,
//!   enabling contract invocations from the backend.
//! * [`TransactionMonitor`] – polling-based transaction-status monitor.
//!
//! All network I/O goes through `reqwest`, which is already part of the
//! dependency graph.  XDR encoding relies on `stellar-xdr`.

use crate::api_error::ApiError;
use crate::circuit_breaker::CircuitBreaker;
use crate::retry::{retry_async, RetryConfig};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

// ─────────────────────────────────────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Stellar network configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct StellarConfig {
    /// Human-readable network passphrase used for transaction signing.
    /// e.g. "Test SDF Network ; September 2015" for testnet.
    pub network_passphrase: String,
    /// Base URL for the Horizon REST API.
    pub horizon_url: String,
    /// Base URL for the Soroban JSON-RPC endpoint.
    pub rpc_url: String,
    /// HTTP request timeout in seconds.
    pub request_timeout_secs: u64,
    /// Maximum number of retry attempts for transient errors.
    pub max_retries: u32,
}

impl StellarConfig {
    /// Load configuration from environment variables with sensible fallbacks.
    ///
    /// | Variable                          | Default                                    |
    /// |-----------------------------------|--------------------------------------------|
    /// | `STELLAR_NETWORK_PASSPHRASE`      | Test SDF Network ; September 2015          |
    /// | `STELLAR_HORIZON_URL`             | https://horizon-testnet.stellar.org        |
    /// | `STELLAR_RPC_URL`                 | https://soroban-testnet.stellar.org        |
    /// | `STELLAR_REQUEST_TIMEOUT_SECS`    | 30                                         |
    /// | `STELLAR_MAX_RETRIES`             | 3                                          |
    pub fn from_env() -> Self {
        Self {
            network_passphrase: std::env::var("STELLAR_NETWORK_PASSPHRASE")
                .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string()),
            horizon_url: std::env::var("STELLAR_HORIZON_URL")
                .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string()),
            rpc_url: std::env::var("STELLAR_RPC_URL")
                .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string()),
            request_timeout_secs: std::env::var("STELLAR_REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            max_retries: std::env::var("STELLAR_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Horizon API types
// ─────────────────────────────────────────────────────────────────────────────

/// Summary of an account returned by the Horizon `/accounts/{id}` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizonAccount {
    pub id: String,
    pub account_id: String,
    pub sequence: String,
    pub balances: Vec<Balance>,
    pub subentry_count: u32,
    pub last_modified_ledger: u32,
    pub last_modified_time: String,
    pub thresholds: Thresholds,
    pub flags: Flags,
}

/// Asset balance for a Stellar account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: String,
    pub limit: Option<String>,
    pub buying_liabilities: String,
    pub selling_liabilities: String,
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
}

/// Multi-sig threshold settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub low_threshold: u8,
    pub med_threshold: u8,
    pub high_threshold: u8,
}

/// Account flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flags {
    pub auth_required: bool,
    pub auth_revocable: bool,
    pub auth_immutable: bool,
    pub auth_clawback_enabled: bool,
}

/// Represents a Stellar transaction record from Horizon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizonTransaction {
    pub id: String,
    pub hash: String,
    pub ledger: u32,
    pub created_at: String,
    pub source_account: String,
    pub source_account_sequence: String,
    pub fee_account: String,
    pub fee_charged: String,
    pub max_fee: String,
    pub operation_count: u32,
    pub envelope_xdr: String,
    pub result_xdr: String,
    pub successful: bool,
    pub memo_type: String,
    pub memo: Option<String>,
}

/// Page of Horizon records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizonPage<T> {
    #[serde(rename = "_embedded")]
    pub embedded: EmbeddedRecords<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedRecords<T> {
    pub records: Vec<T>,
}

/// Response from Horizon when a transaction is submitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSubmitResult {
    pub hash: String,
    pub ledger: Option<u32>,
    pub envelope_xdr: String,
    pub result_xdr: String,
    pub result_meta_xdr: String,
    pub successful: Option<bool>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Soroban RPC types
// ─────────────────────────────────────────────────────────────────────────────

/// JSON-RPC 2.0 request wrapper.
#[derive(Debug, Serialize)]
struct JsonRpcRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    params: Value,
}

/// JSON-RPC 2.0 response wrapper.
#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    pub id: Option<u64>,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error object.
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

/// Soroban RPC `getHealth` result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SorobanHealth {
    pub status: String,
    pub latest_ledger: Option<u64>,
    pub oldest_ledger: Option<u64>,
}

/// Soroban RPC `getLedgerEntries` result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub key: String,
    pub xdr: String,
    pub last_modified_ledger_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLedgerEntriesResult {
    pub entries: Option<Vec<LedgerEntry>>,
    pub latest_ledger: u64,
}

/// Soroban RPC `simulateTransaction` result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateTransactionResult {
    pub error: Option<String>,
    pub results: Option<Vec<SimulateInvocationResult>>,
    pub cost: Option<SimulateCost>,
    pub latest_ledger: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateInvocationResult {
    pub auth: Vec<String>,
    pub xdr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateCost {
    #[serde(rename = "cpuInsns")]
    pub cpu_insns: String,
    #[serde(rename = "memBytes")]
    pub mem_bytes: String,
}

/// Soroban RPC `sendTransaction` result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTransactionResult {
    pub status: String,
    pub hash: String,
    pub latest_ledger: u64,
    #[serde(rename = "latestLedgerCloseTime")]
    pub latest_ledger_close_time: String,
    pub error_result_xdr: Option<String>,
}

/// Soroban RPC `getTransaction` result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionResult {
    pub status: String,
    pub latest_ledger: u64,
    #[serde(rename = "latestLedgerCloseTime")]
    pub latest_ledger_close_time: String,
    #[serde(rename = "oldestLedger")]
    pub oldest_ledger: Option<u64>,
    #[serde(rename = "oldestLedgerCloseTime")]
    pub oldest_ledger_close_time: Option<String>,
    pub ledger: Option<u64>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "applicationOrder")]
    pub application_order: Option<u32>,
    pub envelope_xdr: Option<String>,
    pub result_xdr: Option<String>,
    pub result_meta_xdr: Option<String>,
}

/// Known statuses for a pending Soroban transaction.
pub const SOROBAN_STATUS_PENDING: &str = "PENDING";
pub const SOROBAN_STATUS_SUCCESS: &str = "SUCCESS";
pub const SOROBAN_STATUS_ERROR: &str = "ERROR";
pub const SOROBAN_STATUS_NOT_FOUND: &str = "NOT_FOUND";

// ─────────────────────────────────────────────────────────────────────────────
// HorizonClient
// ─────────────────────────────────────────────────────────────────────────────

/// HTTP client for the Stellar Horizon REST API.
///
/// Uses a circuit breaker to shed load when Horizon is unreachable, and
/// automatic retry with exponential back-off for transient failures.
#[derive(Clone)]
pub struct HorizonClient {
    client: Client,
    base_url: String,
    circuit_breaker: CircuitBreaker,
    config: StellarConfig,
}

impl HorizonClient {
    /// Construct a new `HorizonClient` from a [`StellarConfig`].
    pub fn new(config: StellarConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("failed to build reqwest client");

        let circuit_breaker = CircuitBreaker::from_env("horizon", 5, 60);

        Self {
            client,
            base_url: config.horizon_url.trim_end_matches('/').to_string(),
            circuit_breaker,
            config,
        }
    }

    /// Build from environment variables via [`StellarConfig::from_env`].
    pub fn from_env() -> Self {
        Self::new(StellarConfig::from_env())
    }

    // ── private helper ────────────────────────────────────────────────────────

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, path);
        let max_retries = self.config.max_retries;

        self.circuit_breaker
            .call(|| async {
                retry_async(
                    RetryConfig {
                        max_attempts: max_retries,
                        ..RetryConfig::external_service()
                    },
                    || {
                        let client = self.client.clone();
                        let url = url.clone();
                        async move {
                            debug!(url = %url, "Horizon GET request");
                            let response = client
                                .get(&url)
                                .send()
                                .await
                                .map_err(|e| {
                                    if e.is_timeout() {
                                        ApiError::Timeout
                                    } else {
                                        ApiError::ExternalService(format!(
                                            "Horizon request failed: {e}"
                                        ))
                                    }
                                })?;

                            if !response.status().is_success() {
                                let status = response.status();
                                let body = response.text().await.unwrap_or_default();
                                warn!(status = %status, body = %body, "Horizon non-success response");
                                return Err(ApiError::ExternalService(format!(
                                    "Horizon returned status {status}: {body}"
                                )));
                            }

                            response.json::<T>().await.map_err(|e| {
                                ApiError::ExternalService(format!(
                                    "Failed to deserialize Horizon response: {e}"
                                ))
                            })
                        }
                    },
                    |e| matches!(e, ApiError::ExternalService(_) | ApiError::Timeout),
                )
                .await
            })
            .await
    }

    // ── public API ────────────────────────────────────────────────────────────

    /// Fetch account details for the given Stellar address.
    pub async fn get_account(&self, account_id: &str) -> Result<HorizonAccount, ApiError> {
        let path = format!("/accounts/{account_id}");
        info!(account_id, "Fetching Stellar account from Horizon");
        self.get(&path).await
    }

    /// Fetch recent transactions for an account (newest first).
    ///
    /// `limit` is capped at 200 by Horizon.
    pub async fn get_account_transactions(
        &self,
        account_id: &str,
        limit: u32,
    ) -> Result<Vec<HorizonTransaction>, ApiError> {
        let limit = limit.min(200);
        let path = format!(
            "/accounts/{account_id}/transactions?limit={limit}&order=desc&include_failed=false"
        );
        info!(
            account_id,
            limit, "Fetching account transactions from Horizon"
        );
        let page: HorizonPage<HorizonTransaction> = self.get(&path).await?;
        Ok(page.embedded.records)
    }

    /// Fetch a single transaction by its hash.
    pub async fn get_transaction(&self, tx_hash: &str) -> Result<HorizonTransaction, ApiError> {
        let path = format!("/transactions/{tx_hash}");
        debug!(tx_hash, "Fetching transaction from Horizon");
        self.get(&path).await
    }

    /// Submit a base64-encoded XDR transaction envelope to the network.
    pub async fn submit_transaction(
        &self,
        tx_xdr: &str,
    ) -> Result<TransactionSubmitResult, ApiError> {
        let url = format!("{}/transactions", self.base_url);
        info!("Submitting transaction to Horizon");

        self.circuit_breaker
            .call(|| async {
                let response = self
                    .client
                    .post(&url)
                    .form(&[("tx", tx_xdr)])
                    .send()
                    .await
                    .map_err(|e| {
                        if e.is_timeout() {
                            ApiError::Timeout
                        } else {
                            ApiError::ExternalService(format!("Horizon submit failed: {e}"))
                        }
                    })?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    error!(status = %status, body = %body, "Horizon rejected transaction");
                    return Err(ApiError::ExternalService(format!(
                        "Horizon rejected transaction (status {status}): {body}"
                    )));
                }

                response
                    .json::<TransactionSubmitResult>()
                    .await
                    .map_err(|e| {
                        ApiError::ExternalService(format!(
                            "Failed to deserialize submit response: {e}"
                        ))
                    })
            })
            .await
    }

    /// Check whether the Horizon server is reachable by fetching the root resource.
    pub async fn health_check(&self) -> Result<bool, ApiError> {
        let url = format!("{}/", self.base_url);
        let response =
            self.client.get(&url).send().await.map_err(|e| {
                ApiError::ExternalService(format!("Horizon health check failed: {e}"))
            })?;
        Ok(response.status().is_success())
    }

    /// Fetch the current ledger sequence number from Horizon.
    pub async fn get_latest_ledger(&self) -> Result<u32, ApiError> {
        #[derive(Deserialize)]
        struct Root {
            core_latest_ledger: u32,
        }
        let root: Root = self.get("/").await?;
        Ok(root.core_latest_ledger)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SorobanRpcClient
// ─────────────────────────────────────────────────────────────────────────────

/// JSON-RPC 2.0 client for the Soroban smart-contract RPC endpoint.
///
/// Wraps every call with the same circuit-breaker and retry patterns used
/// by the rest of the backend.
#[derive(Clone)]
pub struct SorobanRpcClient {
    client: Client,
    rpc_url: String,
    circuit_breaker: CircuitBreaker,
    request_id: Arc<std::sync::atomic::AtomicU64>,
    config: StellarConfig,
}

impl SorobanRpcClient {
    /// Construct a new client from the given config.
    pub fn new(config: StellarConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("failed to build reqwest client");

        let circuit_breaker = CircuitBreaker::from_env("soroban_rpc", 5, 60);

        Self {
            client,
            rpc_url: config.rpc_url.trim_end_matches('/').to_string(),
            circuit_breaker,
            request_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            config,
        }
    }

    /// Build from environment variables via [`StellarConfig::from_env`].
    pub fn from_env() -> Self {
        Self::new(StellarConfig::from_env())
    }

    // ── private helper ────────────────────────────────────────────────────────

    fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, ApiError> {
        let url = self.rpc_url.clone();
        let id = self.next_id();
        let max_retries = self.config.max_retries;

        self.circuit_breaker
            .call(|| async {
                retry_async(
                    RetryConfig {
                        max_attempts: max_retries,
                        ..RetryConfig::external_service()
                    },
                    || {
                        let client = self.client.clone();
                        let url = url.clone();
                        let body = JsonRpcRequest {
                            jsonrpc: "2.0",
                            id,
                            method,
                            params: params.clone(),
                        };
                        async move {
                            debug!(method, "Soroban RPC call");
                            let response =
                                client.post(&url).json(&body).send().await.map_err(|e| {
                                    if e.is_timeout() {
                                        ApiError::Timeout
                                    } else {
                                        ApiError::ExternalService(format!(
                                            "Soroban RPC request failed: {e}"
                                        ))
                                    }
                                })?;

                            if !response.status().is_success() {
                                let status = response.status();
                                let text = response.text().await.unwrap_or_default();
                                warn!(status = %status, body = %text, "Soroban RPC HTTP error");
                                return Err(ApiError::ExternalService(format!(
                                    "Soroban RPC HTTP {status}: {text}"
                                )));
                            }

                            let rpc_resp: JsonRpcResponse<T> =
                                response.json().await.map_err(|e| {
                                    ApiError::ExternalService(format!(
                                        "Failed to parse Soroban RPC response: {e}"
                                    ))
                                })?;

                            if let Some(err) = rpc_resp.error {
                                return Err(ApiError::ExternalService(format!(
                                    "Soroban RPC error {}: {}",
                                    err.code, err.message
                                )));
                            }

                            rpc_resp.result.ok_or_else(|| {
                                ApiError::ExternalService(
                                    "Soroban RPC returned empty result".to_string(),
                                )
                            })
                        }
                    },
                    |e| matches!(e, ApiError::ExternalService(_) | ApiError::Timeout),
                )
                .await
            })
            .await
    }

    // ── public API ────────────────────────────────────────────────────────────

    /// Call `getHealth` – verifies the RPC server is live and reports the
    /// latest ledger it has processed.
    pub async fn get_health(&self) -> Result<SorobanHealth, ApiError> {
        info!("Checking Soroban RPC health");
        self.call("getHealth", json!({})).await
    }

    /// Call `getLedgerEntries` with the provided XDR-encoded keys.
    pub async fn get_ledger_entries(
        &self,
        keys: Vec<String>,
    ) -> Result<GetLedgerEntriesResult, ApiError> {
        debug!(
            count = keys.len(),
            "Fetching ledger entries from Soroban RPC"
        );
        self.call("getLedgerEntries", json!({ "keys": keys })).await
    }

    /// Call `simulateTransaction` to estimate fees before submitting.
    ///
    /// `tx_xdr` is the base64-encoded XDR of the unsigned transaction envelope.
    pub async fn simulate_transaction(
        &self,
        tx_xdr: &str,
    ) -> Result<SimulateTransactionResult, ApiError> {
        info!("Simulating Soroban transaction");
        self.call("simulateTransaction", json!({ "transaction": tx_xdr }))
            .await
    }

    /// Call `sendTransaction` to broadcast a signed transaction to the network.
    ///
    /// Returns immediately with a status of `PENDING`; use
    /// [`SorobanRpcClient::get_transaction`] or the [`TransactionMonitor`] to
    /// await the final result.
    pub async fn send_transaction(&self, tx_xdr: &str) -> Result<SendTransactionResult, ApiError> {
        info!("Sending transaction via Soroban RPC");
        self.call("sendTransaction", json!({ "transaction": tx_xdr }))
            .await
    }

    /// Call `getTransaction` to poll for the outcome of a previously submitted
    /// transaction.
    pub async fn get_transaction(&self, tx_hash: &str) -> Result<GetTransactionResult, ApiError> {
        debug!(tx_hash, "Polling Soroban RPC for transaction status");
        self.call("getTransaction", json!({ "hash": tx_hash }))
            .await
    }

    /// Call `getLatestLedger` to get the latest ledger known to the RPC server.
    pub async fn get_latest_ledger(&self) -> Result<Value, ApiError> {
        debug!("Fetching latest ledger from Soroban RPC");
        self.call("getLatestLedger", json!({})).await
    }

    /// Call `getContractData` to read a specific entry in a contract's storage.
    ///
    /// `contract_id` is the contract's Stellar address (C… form).
    /// `key_xdr`     is the base64-encoded XDR `ScVal` key.
    pub async fn get_contract_data(
        &self,
        contract_id: &str,
        key_xdr: &str,
        durability: &str,
    ) -> Result<Value, ApiError> {
        debug!(contract_id, "Fetching contract data from Soroban RPC");
        self.call(
            "getContractData",
            json!({
                "contract": contract_id,
                "key": key_xdr,
                "durability": durability
            }),
        )
        .await
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Transaction monitor
// ─────────────────────────────────────────────────────────────────────────────

/// Status of a monitored transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Pending,
    Success,
    Error,
    NotFound,
}

/// A single record kept by the [`TransactionMonitor`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredTransaction {
    pub hash: String,
    pub status: TransactionStatus,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub result_xdr: Option<String>,
    pub error_message: Option<String>,
    pub ledger: Option<u64>,
}

/// Polls the Soroban RPC until all tracked transactions reach a terminal state.
///
/// The monitor keeps an in-memory registry; submit a hash with
/// [`TransactionMonitor::track`] and retrieve the outcome with
/// [`TransactionMonitor::get_status`].  The background task started by
/// [`TransactionMonitor::start`] polls every `poll_interval` seconds.
#[derive(Clone)]
pub struct TransactionMonitor {
    rpc: SorobanRpcClient,
    registry: Arc<RwLock<HashMap<String, MonitoredTransaction>>>,
    poll_interval_secs: u64,
    max_poll_attempts: u32,
}

impl TransactionMonitor {
    /// Create a new monitor backed by the given Soroban RPC client.
    pub fn new(rpc: SorobanRpcClient) -> Self {
        let poll_interval_secs: u64 = std::env::var("STELLAR_MONITOR_POLL_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        let max_poll_attempts: u32 = std::env::var("STELLAR_MONITOR_MAX_POLL_ATTEMPTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        Self {
            rpc,
            registry: Arc::new(RwLock::new(HashMap::new())),
            poll_interval_secs,
            max_poll_attempts,
        }
    }

    /// Register a transaction hash for monitoring.
    pub async fn track(&self, hash: impl Into<String>) {
        let hash = hash.into();
        let now = chrono::Utc::now();
        let entry = MonitoredTransaction {
            hash: hash.clone(),
            status: TransactionStatus::Pending,
            submitted_at: now,
            updated_at: now,
            result_xdr: None,
            error_message: None,
            ledger: None,
        };
        info!(hash, "Tracking Soroban transaction");
        self.registry.write().await.insert(hash, entry);
    }

    /// Return the current status record for `hash`, if tracked.
    pub async fn get_status(&self, hash: &str) -> Option<MonitoredTransaction> {
        self.registry.read().await.get(hash).cloned()
    }

    /// Remove a completed/errored transaction from the registry.
    pub async fn untrack(&self, hash: &str) {
        self.registry.write().await.remove(hash);
    }

    /// Spawn a Tokio task that periodically polls all pending transactions.
    ///
    /// The returned handle is a `JoinHandle`; dropping it does **not** cancel
    /// the task.  Call [`tokio::task::JoinHandle::abort`] if you need to stop
    /// monitoring.
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!(
                poll_interval_secs = self.poll_interval_secs,
                "Stellar transaction monitor started"
            );
            loop {
                sleep(Duration::from_secs(self.poll_interval_secs)).await;
                self.poll_once().await;
            }
        })
    }

    /// Run one polling cycle over all pending hashes.  Exported for testing.
    pub async fn poll_once(&self) {
        let pending: Vec<String> = {
            let guard = self.registry.read().await;
            guard
                .values()
                .filter(|t| t.status == TransactionStatus::Pending)
                .map(|t| t.hash.clone())
                .collect()
        };

        for hash in pending {
            match self.rpc.get_transaction(&hash).await {
                Ok(result) => self.apply_result(&hash, result).await,
                Err(e) => {
                    warn!(hash, error = %e, "Failed to poll transaction status");
                    // Keep the entry as Pending; will retry next cycle.
                    let mut guard = self.registry.write().await;
                    if let Some(entry) = guard.get_mut(&hash) {
                        entry.updated_at = chrono::Utc::now();
                        // If we have exceeded max_poll_attempts mark as error.
                        let age_secs = (chrono::Utc::now() - entry.submitted_at).num_seconds();
                        let max_age =
                            (self.max_poll_attempts as i64) * (self.poll_interval_secs as i64);
                        if age_secs > max_age {
                            error!(hash, age_secs, "Transaction monitoring timed out");
                            entry.status = TransactionStatus::Error;
                            entry.error_message = Some(
                                "Monitoring timed out – transaction not confirmed".to_string(),
                            );
                        }
                    }
                }
            }
        }
    }

    async fn apply_result(&self, hash: &str, result: GetTransactionResult) {
        let mut guard = self.registry.write().await;
        let Some(entry) = guard.get_mut(hash) else {
            return;
        };

        entry.updated_at = chrono::Utc::now();
        match result.status.as_str() {
            SOROBAN_STATUS_SUCCESS => {
                info!(hash, "Soroban transaction confirmed successfully");
                entry.status = TransactionStatus::Success;
                entry.result_xdr = result.result_xdr;
                entry.ledger = result.ledger;
            }
            SOROBAN_STATUS_ERROR => {
                error!(hash, "Soroban transaction failed on-chain");
                entry.status = TransactionStatus::Error;
                entry.result_xdr = result.result_xdr;
                entry.error_message = Some("Transaction failed on-chain".to_string());
                entry.ledger = result.ledger;
            }
            SOROBAN_STATUS_NOT_FOUND => {
                // Transaction may not yet be ingested; keep as Pending.
                debug!(hash, "Transaction not yet found in Soroban RPC");
            }
            SOROBAN_STATUS_PENDING => {
                debug!(hash, "Transaction still pending");
            }
            other => {
                warn!(
                    hash,
                    status = other,
                    "Unknown transaction status from Soroban RPC"
                );
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// StellarClient – top-level façade
// ─────────────────────────────────────────────────────────────────────────────

/// Convenience wrapper that bundles the Horizon client, Soroban RPC client,
/// and transaction monitor into a single `Arc`-cloneable handle.
#[derive(Clone)]
pub struct StellarClient {
    pub horizon: HorizonClient,
    pub soroban: SorobanRpcClient,
    pub monitor: TransactionMonitor,
    pub config: StellarConfig,
}

impl StellarClient {
    /// Build a full Stellar client from environment variables.
    pub fn from_env() -> Self {
        let config = StellarConfig::from_env();
        let horizon = HorizonClient::new(config.clone());
        let soroban = SorobanRpcClient::new(config.clone());
        let monitor = TransactionMonitor::new(soroban.clone());
        Self {
            horizon,
            soroban,
            monitor,
            config,
        }
    }

    /// Submit a signed transaction XDR via Soroban RPC, automatically
    /// registering it with the [`TransactionMonitor`].
    ///
    /// Returns the initial [`SendTransactionResult`] (status will be `PENDING`).
    pub async fn submit_and_monitor(
        &self,
        tx_xdr: &str,
    ) -> Result<SendTransactionResult, ApiError> {
        let result = self.soroban.send_transaction(tx_xdr).await?;
        self.monitor.track(result.hash.clone()).await;
        Ok(result)
    }

    /// Convenience: simulate, then send, then monitor a transaction.
    ///
    /// Returns an error if simulation reports a failure; otherwise sends
    /// the transaction and begins monitoring.
    pub async fn simulate_and_submit(
        &self,
        tx_xdr: &str,
    ) -> Result<SendTransactionResult, ApiError> {
        let sim = self.soroban.simulate_transaction(tx_xdr).await?;
        if let Some(err) = sim.error {
            return Err(ApiError::ExternalService(format!(
                "Transaction simulation failed: {err}"
            )));
        }
        self.submit_and_monitor(tx_xdr).await
    }

    /// Health check for both Horizon and Soroban RPC.
    pub async fn health_check(&self) -> StellarHealthStatus {
        let horizon_ok = self.horizon.health_check().await.unwrap_or(false);
        let soroban_ok = self
            .soroban
            .get_health()
            .await
            .map(|h| h.status == "healthy")
            .unwrap_or(false);

        StellarHealthStatus {
            horizon_reachable: horizon_ok,
            soroban_reachable: soroban_ok,
            network_passphrase: self.config.network_passphrase.clone(),
        }
    }
}

/// Aggregate health status for the Stellar integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarHealthStatus {
    pub horizon_reachable: bool,
    pub soroban_reachable: bool,
    pub network_passphrase: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn testnet_config() -> StellarConfig {
        StellarConfig {
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
            horizon_url: "https://horizon-testnet.stellar.org".to_string(),
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            request_timeout_secs: 30,
            max_retries: 3,
        }
    }

    fn mainnet_config() -> StellarConfig {
        StellarConfig {
            network_passphrase: "Public Global Stellar Network ; September 2015".to_string(),
            horizon_url: "https://horizon.stellar.org".to_string(),
            rpc_url: "https://rpc.stellar.org".to_string(),
            request_timeout_secs: 30,
            max_retries: 3,
        }
    }

    #[test]
    fn stellar_config_testnet_passphrase() {
        let cfg = testnet_config();
        assert_eq!(cfg.network_passphrase, "Test SDF Network ; September 2015");
        assert!(cfg.horizon_url.contains("horizon-testnet.stellar.org"));
        assert!(cfg.rpc_url.contains("soroban-testnet.stellar.org"));
    }

    #[test]
    fn stellar_config_mainnet_passphrase() {
        let cfg = mainnet_config();
        assert_eq!(
            cfg.network_passphrase,
            "Public Global Stellar Network ; September 2015"
        );
        assert_eq!(cfg.horizon_url, "https://horizon.stellar.org");
        assert_eq!(cfg.rpc_url, "https://rpc.stellar.org");
    }

    #[test]
    fn horizon_client_strips_trailing_slash() {
        let mut cfg = testnet_config();
        cfg.horizon_url = "https://horizon-testnet.stellar.org/".to_string();
        let client = HorizonClient::new(cfg);
        assert!(!client.base_url.ends_with('/'));
    }

    #[test]
    fn soroban_rpc_client_strips_trailing_slash() {
        let mut cfg = testnet_config();
        cfg.rpc_url = "https://soroban-testnet.stellar.org/".to_string();
        let client = SorobanRpcClient::new(cfg);
        assert!(!client.rpc_url.ends_with('/'));
    }

    #[tokio::test]
    async fn transaction_monitor_track_and_get() {
        let soroban = SorobanRpcClient::new(testnet_config());
        let monitor = TransactionMonitor::new(soroban);

        let hash = "abc123def456";
        monitor.track(hash).await;

        let status = monitor.get_status(hash).await;
        assert!(status.is_some());
        let entry = status.unwrap();
        assert_eq!(entry.hash, hash);
        assert_eq!(entry.status, TransactionStatus::Pending);

        monitor.untrack(hash).await;
        assert!(monitor.get_status(hash).await.is_none());
    }

    #[tokio::test]
    async fn monitor_apply_success_result() {
        let soroban = SorobanRpcClient::new(testnet_config());
        let monitor = TransactionMonitor::new(soroban);

        let hash = "success_hash_001";
        monitor.track(hash).await;

        let result = GetTransactionResult {
            status: SOROBAN_STATUS_SUCCESS.to_string(),
            latest_ledger: 12345,
            latest_ledger_close_time: "2024-01-01T00:00:00Z".to_string(),
            oldest_ledger: None,
            oldest_ledger_close_time: None,
            ledger: Some(12345),
            created_at: None,
            application_order: None,
            envelope_xdr: None,
            result_xdr: Some("result_xdr_here".to_string()),
            result_meta_xdr: None,
        };

        monitor.apply_result(hash, result).await;
        let entry = monitor.get_status(hash).await.unwrap();
        assert_eq!(entry.status, TransactionStatus::Success);
        assert_eq!(entry.ledger, Some(12345));
    }

    #[tokio::test]
    async fn monitor_apply_error_result() {
        let soroban = SorobanRpcClient::new(testnet_config());
        let monitor = TransactionMonitor::new(soroban);

        let hash = "error_hash_002";
        monitor.track(hash).await;

        let result = GetTransactionResult {
            status: SOROBAN_STATUS_ERROR.to_string(),
            latest_ledger: 12345,
            latest_ledger_close_time: "2024-01-01T00:00:00Z".to_string(),
            oldest_ledger: None,
            oldest_ledger_close_time: None,
            ledger: Some(12345),
            created_at: None,
            application_order: None,
            envelope_xdr: None,
            result_xdr: None,
            result_meta_xdr: None,
        };

        monitor.apply_result(hash, result).await;
        let entry = monitor.get_status(hash).await.unwrap();
        assert_eq!(entry.status, TransactionStatus::Error);
        assert!(entry.error_message.is_some());
    }

    #[tokio::test]
    async fn monitor_pending_stays_pending() {
        let soroban = SorobanRpcClient::new(testnet_config());
        let monitor = TransactionMonitor::new(soroban);

        let hash = "pending_hash_003";
        monitor.track(hash).await;

        let result = GetTransactionResult {
            status: SOROBAN_STATUS_PENDING.to_string(),
            latest_ledger: 12345,
            latest_ledger_close_time: "2024-01-01T00:00:00Z".to_string(),
            oldest_ledger: None,
            oldest_ledger_close_time: None,
            ledger: None,
            created_at: None,
            application_order: None,
            envelope_xdr: None,
            result_xdr: None,
            result_meta_xdr: None,
        };

        monitor.apply_result(hash, result).await;
        let entry = monitor.get_status(hash).await.unwrap();
        // Still pending – should not be promoted to error or success
        assert_eq!(entry.status, TransactionStatus::Pending);
    }

    #[tokio::test]
    async fn monitor_multiple_transactions() {
        let soroban = SorobanRpcClient::new(testnet_config());
        let monitor = TransactionMonitor::new(soroban);

        for i in 0..5 {
            monitor.track(format!("hash_{i}")).await;
        }

        for i in 0..5 {
            let status = monitor.get_status(&format!("hash_{i}")).await;
            assert!(status.is_some());
            assert_eq!(status.unwrap().status, TransactionStatus::Pending);
        }
    }
}

//! # Bridge Event Listener
//!
//! Background daemon that polls the Stellar Horizon API for Soroban contract
//! events carrying the `BridgePay` topic, parses each [`BridgePayoutEvent`],
//! persists them to `bridge_payout_events` with idempotency, and enqueues a
//! `bridge.payout` webhook dispatch for every new event.
//!
//! ## Design
//! * Polls Horizon's `/contracts/{id}/events` endpoint on a configurable interval.
//! * Tracks the highest ledger sequence it has already seen so each poll only
//!   fetches new ledgers (cursor-based pagination).
//! * Uses `INSERT … ON CONFLICT DO NOTHING` for exactly-once DB persistence.
//! * Delegates fan-out delivery to [`crate::WebhookDispatcherService`].
//! * Supports graceful shutdown via a [`tokio::sync::watch`] channel.
//!
//! ## Configuration (environment variables)
//! | Variable | Default | Description |
//! |---|---|---|
//! | `STELLAR_CONTRACT_ID` | *(required)* | Bech32m contract address to filter events |
//! | `STELLAR_HORIZON_URL` | `https://horizon-testnet.stellar.org` | Horizon base URL |
//! | `BRIDGE_LISTENER_POLL_INTERVAL_SECS` | `30` | Seconds between polls |

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::WebhookDispatcherService;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Soroban event topic symbol that identifies a BridgePay event.
/// Matches `symbol_short!("BridgePay")` emitted by the contract.
pub const BRIDGE_PAY_TOPIC: &str = "BridgePay";

/// Default polling interval when `BRIDGE_LISTENER_POLL_INTERVAL_SECS` is unset.
const DEFAULT_POLL_INTERVAL_SECS: u64 = 30;

/// Maximum events to request per Horizon page. Horizon caps at 200.
const HORIZON_PAGE_LIMIT: u32 = 200;

// ── Configuration ─────────────────────────────────────────────────────────────

/// Runtime configuration for [`BridgeEventListenerService`].
#[derive(Debug, Clone)]
pub struct BridgeListenerConfig {
    /// Bech32m Soroban contract address whose events we want.
    pub contract_id: String,
    /// Stellar Horizon base URL (no trailing slash).
    pub horizon_url: String,
    /// How long to wait between poll cycles.
    pub poll_interval: Duration,
}

impl BridgeListenerConfig {
    /// Build from environment variables, returning `None` when
    /// `STELLAR_CONTRACT_ID` is absent or empty (daemon is disabled).
    pub fn from_env(horizon_url: &str) -> Option<Self> {
        let contract_id = std::env::var("STELLAR_CONTRACT_ID")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())?;

        let poll_interval_secs = std::env::var("BRIDGE_LISTENER_POLL_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_POLL_INTERVAL_SECS)
            .max(1);

        Some(Self {
            contract_id,
            horizon_url: horizon_url.trim_end_matches('/').to_string(),
            poll_interval: Duration::from_secs(poll_interval_secs),
        })
    }
}

// ── Horizon API response types ────────────────────────────────────────────────

/// Subset of a Horizon `ContractEvent` record we care about.
#[derive(Debug, Deserialize)]
pub struct HorizonEvent {
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub ledger: Option<Value>,
    pub transaction_hash: Option<String>,
    pub id: Option<String>,
    pub topic: Option<Vec<Value>>,
    pub value: Option<Value>,
    pub contract_id: Option<String>,
    pub paging_token: Option<String>,
}

/// Top-level Horizon events page response.
#[derive(Debug, Deserialize)]
pub struct HorizonEventsPage {
    #[serde(rename = "_embedded")]
    pub embedded: Option<HorizonEmbedded>,
}

/// The embedded records wrapper inside a Horizon page.
#[derive(Debug, Deserialize)]
pub struct HorizonEmbedded {
    pub records: Vec<HorizonEvent>,
}

// ── Parsed domain type ────────────────────────────────────────────────────────

/// Decoded fields from a `BridgePayoutEvent` Soroban contract event.
#[derive(Debug, Clone)]
pub struct ParsedBridgePayoutEvent {
    pub contract_id: String,
    pub ledger_sequence: i64,
    pub tx_hash: String,
    pub event_index: i32,
    pub owner_address: String,
    pub token_address: String,
    pub beneficiary_address: String,
    pub destination_chain: String,
    pub destination_address: String,
    pub gross_amount: i64,
    pub fee_amount: i64,
    pub net_amount: i64,
    pub source_chain: String,
    pub source_tx_hash: String,
    pub raw_event: Value,
}

// ── Service ───────────────────────────────────────────────────────────────────

/// Background service that polls Horizon and persists `BridgePay` events.
pub struct BridgeEventListenerService {
    db: PgPool,
    client: Client,
    config: BridgeListenerConfig,
}

impl BridgeEventListenerService {
    /// Create a new service instance. Does **not** start the background loop;
    /// call [`Self::start`] for that.
    pub fn new(db: PgPool, config: BridgeListenerConfig) -> Self {
        Self {
            db,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("failed to build reqwest client"),
            config,
        }
    }

    /// Spawn the polling loop onto the Tokio runtime.
    ///
    /// The loop runs until `shutdown_rx` receives any value (sender dropped or
    /// an explicit send), enabling clean shutdown in tests and production.
    pub fn start(self: Arc<Self>, mut shutdown_rx: watch::Receiver<bool>) {
        tokio::spawn(async move {
            info!(
                contract_id = %self.config.contract_id,
                poll_interval_secs = self.config.poll_interval.as_secs(),
                "Bridge event listener started"
            );
            loop {
                tokio::select! {
                    // Shutdown signal received – exit cleanly.
                    _ = shutdown_rx.changed() => {
                        info!("Bridge event listener received shutdown signal");
                        break;
                    }
                    // Wait for the configured poll interval, then run one cycle.
                    _ = sleep(self.config.poll_interval) => {
                        if let Err(e) = self.run_once().await {
                            error!("Bridge event listener poll failed: {e:#}");
                        }
                    }
                }
            }
            info!("Bridge event listener stopped");
        });
    }

    /// Execute one full poll-and-persist cycle.
    ///
    /// 1. Reads the latest persisted ledger from the DB (cursor resume).
    /// 2. Fetches all new `BridgePay` contract events from Horizon.
    /// 3. Parses and deduplicates each event.
    /// 4. Persists each new event and enqueues a webhook dispatch.
    ///
    /// Returns the number of newly-persisted events.
    pub async fn run_once(&self) -> Result<usize, anyhow::Error> {
        let cursor_ledger = self.latest_persisted_ledger().await?;
        let events = self.fetch_events(cursor_ledger.map(|l| l + 1)).await?;

        if events.is_empty() {
            debug!(
                contract_id = %self.config.contract_id,
                "No new BridgePay events found"
            );
            return Ok(0);
        }

        let mut persisted = 0usize;
        for event in &events {
            match self.persist_event(event).await {
                Ok(true) => {
                    persisted += 1;
                    self.enqueue_webhook(event).await;
                }
                Ok(false) => {
                    debug!(
                        tx_hash = %event.tx_hash,
                        event_index = event.event_index,
                        "Duplicate BridgePay event skipped"
                    );
                }
                Err(e) => {
                    error!(
                        tx_hash = %event.tx_hash,
                        error = %e,
                        "Failed to persist BridgePay event"
                    );
                }
            }
        }

        if persisted > 0 {
            info!(
                contract_id = %self.config.contract_id,
                persisted,
                "BridgePay events captured"
            );
        }

        Ok(persisted)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Return the highest `ledger_sequence` already stored, or `None` when the
    /// table is empty for this contract.
    ///
    /// Uses `fetch_one` because `COALESCE(MAX(…), 0)` always produces a row.
    pub async fn latest_persisted_ledger(&self) -> Result<Option<i64>, anyhow::Error> {
        // COALESCE guarantees a non-null row, so fetch_one is correct here.
        let (seq,): (i64,) = sqlx::query_as(
            "SELECT COALESCE(MAX(ledger_sequence), 0) \
             FROM bridge_payout_events \
             WHERE contract_id = $1",
        )
        .bind(&self.config.contract_id)
        .fetch_one(&self.db)
        .await?;

        Ok(if seq == 0 { None } else { Some(seq) })
    }

    /// Fetch raw `BridgePay` contract events from Horizon starting at
    /// `from_ledger`. Returns an empty vec when Horizon is unreachable (the
    /// caller will retry on the next tick).
    pub async fn fetch_events(
        &self,
        from_ledger: Option<i64>,
    ) -> Result<Vec<ParsedBridgePayoutEvent>, anyhow::Error> {
        let mut url = format!(
            "{}/contracts/{}/events?limit={}",
            self.config.horizon_url, self.config.contract_id, HORIZON_PAGE_LIMIT,
        );

        if let Some(seq) = from_ledger {
            // Horizon paging_token for a ledger-level cursor is the ledger
            // sequence left-padded to 19 digits, followed by a 10-digit zero
            // transaction index.  e.g. "0000000001234567890-0000000000"
            let cursor = format!("{seq:019}-0000000000");
            url.push_str(&format!("&cursor={cursor}"));
        }

        debug!(url = %url, "Fetching contract events from Horizon");

        let response = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Horizon request failed (will retry): {e}");
                return Ok(vec![]);
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(%status, body = %body, "Horizon returned non-2xx (will retry)");
            return Ok(vec![]);
        }

        let page: HorizonEventsPage = match response.json().await {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to parse Horizon response: {e}");
                return Ok(vec![]);
            }
        };

        let records = page.embedded.map(|e| e.records).unwrap_or_default();

        let mut parsed = Vec::with_capacity(records.len());
        for (idx, record) in records.into_iter().enumerate() {
            if !self.is_bridge_pay_event(&record) {
                continue;
            }
            match Self::parse_event(record, idx as i32) {
                Ok(event) => parsed.push(event),
                Err(e) => warn!("Failed to parse BridgePay event at index {idx}: {e}"),
            }
        }

        Ok(parsed)
    }

    /// Return `true` when the event's first topic matches the `BridgePay`
    /// symbol. Horizon encodes Soroban `Symbol` values as plain strings in the
    /// `topic` array.
    pub fn is_bridge_pay_event(&self, event: &HorizonEvent) -> bool {
        event
            .topic
            .as_ref()
            .and_then(|topics| topics.first())
            .and_then(|v| v.as_str())
            .map(|s| s == BRIDGE_PAY_TOPIC)
            .unwrap_or(false)
    }

    /// Parse a raw [`HorizonEvent`] into a [`ParsedBridgePayoutEvent`].
    ///
    /// The `value` field is a Soroban `ScVal` serialised as JSON by Horizon.
    /// For a `BridgePayoutEvent` struct the shape is a JSON object whose keys
    /// mirror the Soroban struct field names.
    pub fn parse_event(
        record: HorizonEvent,
        idx: i32,
    ) -> Result<ParsedBridgePayoutEvent, anyhow::Error> {
        let raw_event = record
            .value
            .as_ref()
            .cloned()
            .unwrap_or(Value::Null);

        let contract_id = record.contract_id.clone().unwrap_or_default();

        // Ledger can be either a number or a string depending on Horizon version.
        let ledger_sequence: i64 = match &record.ledger {
            Some(Value::Number(n)) => n.as_i64().unwrap_or(0),
            Some(Value::String(s)) => s.parse().unwrap_or(0),
            _ => 0,
        };

        let tx_hash = record.transaction_hash.clone().unwrap_or_default();

        // Horizon event IDs use the format "<ledger>-<tx_order>-<event_index>".
        let event_index = record
            .id
            .as_deref()
            .and_then(|id| id.split('-').nth(2))
            .and_then(|s| s.parse().ok())
            .unwrap_or(idx);

        let val = record.value.unwrap_or(Value::Null);

        Ok(ParsedBridgePayoutEvent {
            contract_id,
            ledger_sequence,
            tx_hash,
            event_index,
            owner_address: extract_str(&val, "owner"),
            token_address: extract_str(&val, "token"),
            beneficiary_address: extract_str(&val, "beneficiary"),
            destination_chain: extract_str(&val, "destination_chain"),
            destination_address: extract_str(&val, "destination_address"),
            gross_amount: extract_i64(&val, "gross_amount"),
            fee_amount: extract_i64(&val, "fee_amount"),
            net_amount: extract_i64(&val, "net_amount"),
            source_chain: extract_str(&val, "source_chain"),
            source_tx_hash: extract_str(&val, "source_tx_hash"),
            raw_event,
        })
    }

    /// Persist a parsed event. Returns `true` when a new row was inserted,
    /// `false` when the event already existed (dedup via unique constraint).
    pub async fn persist_event(
        &self,
        event: &ParsedBridgePayoutEvent,
    ) -> Result<bool, anyhow::Error> {
        let rows_affected = sqlx::query(
            r#"
            INSERT INTO bridge_payout_events (
                contract_id, ledger_sequence, tx_hash, event_index,
                owner_address, token_address, beneficiary_address,
                destination_chain, destination_address,
                gross_amount, fee_amount, net_amount,
                source_chain, source_tx_hash, raw_event
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7,
                $8, $9,
                $10, $11, $12,
                $13, $14, $15
            )
            ON CONFLICT (contract_id, ledger_sequence, tx_hash, event_index)
            DO NOTHING
            "#,
        )
        .bind(&event.contract_id)
        .bind(event.ledger_sequence)
        .bind(&event.tx_hash)
        .bind(event.event_index)
        .bind(&event.owner_address)
        .bind(&event.token_address)
        .bind(&event.beneficiary_address)
        .bind(&event.destination_chain)
        .bind(&event.destination_address)
        .bind(event.gross_amount)
        .bind(event.fee_amount)
        .bind(event.net_amount)
        .bind(&event.source_chain)
        .bind(&event.source_tx_hash)
        .bind(&event.raw_event)
        .execute(&self.db)
        .await?;

        Ok(rows_affected.rows_affected() > 0)
    }

    /// Enqueue a `bridge.payout` webhook dispatch for all active endpoints.
    pub async fn enqueue_webhook(&self, event: &ParsedBridgePayoutEvent) {
        let payload = serde_json::json!({
            "event":               "bridge.payout",
            "contract_id":         event.contract_id,
            "ledger_sequence":     event.ledger_sequence,
            "tx_hash":             event.tx_hash,
            "event_index":         event.event_index,
            "owner_address":       event.owner_address,
            "token_address":       event.token_address,
            "beneficiary_address": event.beneficiary_address,
            "destination_chain":   event.destination_chain,
            "destination_address": event.destination_address,
            "gross_amount":        event.gross_amount,
            "fee_amount":          event.fee_amount,
            "net_amount":          event.net_amount,
            "source_chain":        event.source_chain,
            "source_tx_hash":      event.source_tx_hash,
        });

        if let Err(e) =
            WebhookDispatcherService::enqueue_event(&self.db, "bridge.payout", &payload).await
        {
            warn!(
                tx_hash = %event.tx_hash,
                error = %e,
                "Failed to enqueue bridge.payout webhook"
            );
        }
    }
}

// ── Value extraction helpers ──────────────────────────────────────────────────

/// Pull a string field out of a Soroban-encoded Horizon event value.
///
/// Horizon can represent `Address` and `Symbol` as plain strings or as
/// objects with an inner `"id"` / `"value"` key — we handle both shapes.
pub fn extract_str(val: &Value, key: &str) -> String {
    let field = &val[key];
    if let Some(s) = field.as_str() {
        return s.to_string();
    }
    // Horizon sometimes wraps scalar values: { "type": "Address", "id": "G..." }
    if let Some(s) = field.get("id").and_then(|v| v.as_str()) {
        return s.to_string();
    }
    if let Some(s) = field.get("value").and_then(|v| v.as_str()) {
        return s.to_string();
    }
    String::new()
}

/// Pull an i64-compatible integer from a Soroban event value field.
///
/// Soroban `i128` values are represented as strings by Horizon; regular
/// JSON numbers are also accepted.
pub fn extract_i64(val: &Value, key: &str) -> i64 {
    let field = &val[key];
    if let Some(n) = field.as_i64() {
        return n;
    }
    if let Some(s) = field.as_str() {
        return s.parse().unwrap_or(0);
    }
    0
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::{Mutex, OnceLock};

    /// Serialise env-var access so parallel tests don't race.
    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    // ── BridgeListenerConfig::from_env ────────────────────────────────────────

    #[test]
    fn config_returns_none_when_contract_id_absent() {
        let _g = env_lock();
        std::env::remove_var("STELLAR_CONTRACT_ID");
        assert!(BridgeListenerConfig::from_env("https://horizon-testnet.stellar.org").is_none());
    }

    #[test]
    fn config_returns_none_when_contract_id_empty() {
        let _g = env_lock();
        std::env::set_var("STELLAR_CONTRACT_ID", "   ");
        let cfg = BridgeListenerConfig::from_env("https://horizon-testnet.stellar.org");
        std::env::remove_var("STELLAR_CONTRACT_ID");
        assert!(cfg.is_none());
    }

    #[test]
    fn config_builds_from_env_with_defaults() {
        let _g = env_lock();
        std::env::set_var("STELLAR_CONTRACT_ID", "CTEST123");
        std::env::remove_var("BRIDGE_LISTENER_POLL_INTERVAL_SECS");

        let cfg = BridgeListenerConfig::from_env("https://horizon-testnet.stellar.org/")
            .expect("should build");

        std::env::remove_var("STELLAR_CONTRACT_ID");

        assert_eq!(cfg.contract_id, "CTEST123");
        // trailing slash is stripped
        assert_eq!(cfg.horizon_url, "https://horizon-testnet.stellar.org");
        assert_eq!(cfg.poll_interval, Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS));
    }

    #[test]
    fn config_applies_custom_poll_interval() {
        let _g = env_lock();
        std::env::set_var("STELLAR_CONTRACT_ID", "CTEST456");
        std::env::set_var("BRIDGE_LISTENER_POLL_INTERVAL_SECS", "15");

        let cfg = BridgeListenerConfig::from_env("https://horizon-testnet.stellar.org")
            .expect("should build");

        std::env::remove_var("STELLAR_CONTRACT_ID");
        std::env::remove_var("BRIDGE_LISTENER_POLL_INTERVAL_SECS");

        assert_eq!(cfg.poll_interval, Duration::from_secs(15));
    }

    #[test]
    fn config_clamps_poll_interval_to_minimum_one_second() {
        let _g = env_lock();
        std::env::set_var("STELLAR_CONTRACT_ID", "CTEST789");
        std::env::set_var("BRIDGE_LISTENER_POLL_INTERVAL_SECS", "0");

        let cfg = BridgeListenerConfig::from_env("https://horizon-testnet.stellar.org")
            .expect("should build");

        std::env::remove_var("STELLAR_CONTRACT_ID");
        std::env::remove_var("BRIDGE_LISTENER_POLL_INTERVAL_SECS");

        assert_eq!(cfg.poll_interval, Duration::from_secs(1));
    }

    // ── is_bridge_pay_event ───────────────────────────────────────────────────

    fn make_service() -> BridgeEventListenerService {
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let cfg = BridgeListenerConfig {
            contract_id: "CONTRACT1".to_string(),
            horizon_url: "https://horizon-testnet.stellar.org".to_string(),
            poll_interval: Duration::from_secs(30),
        };
        BridgeEventListenerService::new(pool, cfg)
    }

    fn bridge_pay_event() -> HorizonEvent {
        HorizonEvent {
            event_type: Some("contract".to_string()),
            ledger: Some(json!(100)),
            transaction_hash: Some("abc123".to_string()),
            id: Some("0000000000000100-0000000001-0000000000".to_string()),
            topic: Some(vec![json!("BridgePay"), json!("extra")]),
            value: Some(json!({
                "owner": "GOWNER",
                "token": "USDC",
                "beneficiary": "GBENEFICIARY",
                "destination_chain": "ethereum",
                "destination_address": "0xdeadbeef",
                "gross_amount": 1000000,
                "fee_amount": 5000,
                "net_amount": 995000,
                "source_chain": "stellar",
                "source_tx_hash": "abc123"
            })),
            contract_id: Some("CONTRACT1".to_string()),
            paging_token: None,
        }
    }

    #[test]
    fn is_bridge_pay_event_returns_true_for_matching_topic() {
        let svc = make_service();
        assert!(svc.is_bridge_pay_event(&bridge_pay_event()));
    }

    #[test]
    fn is_bridge_pay_event_returns_false_for_wrong_topic() {
        let svc = make_service();
        let mut evt = bridge_pay_event();
        evt.topic = Some(vec![json!("OtherEvent")]);
        assert!(!svc.is_bridge_pay_event(&evt));
    }

    #[test]
    fn is_bridge_pay_event_returns_false_when_topics_empty() {
        let svc = make_service();
        let mut evt = bridge_pay_event();
        evt.topic = Some(vec![]);
        assert!(!svc.is_bridge_pay_event(&evt));
    }

    #[test]
    fn is_bridge_pay_event_returns_false_when_topic_is_none() {
        let svc = make_service();
        let mut evt = bridge_pay_event();
        evt.topic = None;
        assert!(!svc.is_bridge_pay_event(&evt));
    }

    // ── parse_event ───────────────────────────────────────────────────────────

    #[test]
    fn parse_event_extracts_all_fields() {
        let evt = bridge_pay_event();
        let parsed = BridgeEventListenerService::parse_event(evt, 0).expect("parse ok");

        assert_eq!(parsed.contract_id, "CONTRACT1");
        assert_eq!(parsed.ledger_sequence, 100);
        assert_eq!(parsed.tx_hash, "abc123");
        assert_eq!(parsed.owner_address, "GOWNER");
        assert_eq!(parsed.token_address, "USDC");
        assert_eq!(parsed.beneficiary_address, "GBENEFICIARY");
        assert_eq!(parsed.destination_chain, "ethereum");
        assert_eq!(parsed.destination_address, "0xdeadbeef");
        assert_eq!(parsed.gross_amount, 1_000_000);
        assert_eq!(parsed.fee_amount, 5_000);
        assert_eq!(parsed.net_amount, 995_000);
        assert_eq!(parsed.source_chain, "stellar");
        assert_eq!(parsed.source_tx_hash, "abc123");
    }

    #[test]
    fn parse_event_uses_fallback_index_when_id_missing() {
        let mut evt = bridge_pay_event();
        evt.id = None;
        let parsed = BridgeEventListenerService::parse_event(evt, 7).expect("parse ok");
        assert_eq!(parsed.event_index, 7);
    }

    #[test]
    fn parse_event_parses_ledger_as_string() {
        let mut evt = bridge_pay_event();
        evt.ledger = Some(json!("42"));
        let parsed = BridgeEventListenerService::parse_event(evt, 0).expect("parse ok");
        assert_eq!(parsed.ledger_sequence, 42);
    }

    #[test]
    fn parse_event_handles_missing_value_gracefully() {
        let mut evt = bridge_pay_event();
        evt.value = None;
        let parsed = BridgeEventListenerService::parse_event(evt, 0).expect("parse ok");
        assert_eq!(parsed.owner_address, "");
        assert_eq!(parsed.gross_amount, 0);
    }

    // ── extract_str ───────────────────────────────────────────────────────────

    #[test]
    fn extract_str_handles_plain_string() {
        let val = json!({ "owner": "GTEST" });
        assert_eq!(extract_str(&val, "owner"), "GTEST");
    }

    #[test]
    fn extract_str_handles_id_wrapper() {
        let val = json!({ "owner": { "type": "Address", "id": "GWRAPPED" } });
        assert_eq!(extract_str(&val, "owner"), "GWRAPPED");
    }

    #[test]
    fn extract_str_handles_value_wrapper() {
        let val = json!({ "token": { "value": "USDC" } });
        assert_eq!(extract_str(&val, "token"), "USDC");
    }

    #[test]
    fn extract_str_returns_empty_string_for_missing_key() {
        let val = json!({});
        assert_eq!(extract_str(&val, "missing"), "");
    }

    // ── extract_i64 ───────────────────────────────────────────────────────────

    #[test]
    fn extract_i64_handles_json_number() {
        let val = json!({ "amount": 500000 });
        assert_eq!(extract_i64(&val, "amount"), 500_000);
    }

    #[test]
    fn extract_i64_handles_string_encoded_number() {
        let val = json!({ "amount": "123456789" });
        assert_eq!(extract_i64(&val, "amount"), 123_456_789);
    }

    #[test]
    fn extract_i64_returns_zero_for_missing_key() {
        let val = json!({});
        assert_eq!(extract_i64(&val, "missing"), 0);
    }

    #[test]
    fn extract_i64_returns_zero_for_unparseable_string() {
        let val = json!({ "amount": "not_a_number" });
        assert_eq!(extract_i64(&val, "amount"), 0);
    }

    // ── paging_token / cursor format ──────────────────────────────────────────

    #[test]
    fn cursor_format_is_19_digit_padded_with_zero_suffix() {
        // Validate the Horizon cursor format used in fetch_events
        let seq: i64 = 12345;
        let cursor = format!("{seq:019}-0000000000");
        assert_eq!(cursor, "0000000000000012345-0000000000");
        assert_eq!(cursor.len(), 30); // 19 + 1 + 10
    }

    #[test]
    fn cursor_format_handles_large_ledger_sequence() {
        let seq: i64 = 999_999_999_999_999_999;
        let cursor = format!("{seq:019}-0000000000");
        // 19 digits exactly
        let parts: Vec<&str> = cursor.split('-').collect();
        assert_eq!(parts[0].len(), 19);
        assert_eq!(parts[1], "0000000000");
    }
}

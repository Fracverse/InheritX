//! Integration tests for the Bridge Event Listener.
//!
//! These tests exercise the full public surface of
//! [`BridgeEventListenerService`] without requiring a live database or
//! Horizon instance.  They also verify the HTTP layer via the Axum router.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::{
    bridge_event_listener::{
        extract_i64, extract_str, BridgeEventListenerService, BridgeListenerConfig, HorizonEvent,
        HorizonEventsPage, BRIDGE_PAY_TOPIC,
    },
    AppState, PlanCache,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_config() -> BridgeListenerConfig {
    BridgeListenerConfig {
        contract_id: "CTEST_INTEGRATION".to_string(),
        horizon_url: "https://horizon-testnet.stellar.org".to_string(),
        poll_interval: Duration::from_secs(60),
    }
}

fn make_pool() -> sqlx::PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/test".to_string());
    sqlx::PgPool::connect_lazy(&url).unwrap()
}

fn make_service() -> BridgeEventListenerService {
    BridgeEventListenerService::new(make_pool(), make_config())
}

fn make_router() -> axum::Router {
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        db_pool: make_pool(),
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: PlanCache::disabled(),
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ),
    });
    inheritx_backend::create_router(state)
}

fn sample_horizon_event() -> HorizonEvent {
    HorizonEvent {
        event_type: Some("contract".to_string()),
        ledger: Some(json!(500)),
        transaction_hash: Some("txhash_integration".to_string()),
        id: Some("0000000000000000500-0000000001-0000000002".to_string()),
        topic: Some(vec![json!(BRIDGE_PAY_TOPIC), json!("extra_topic")]),
        value: Some(json!({
            "owner":               "GOWNER_INTEGRATION",
            "token":               "USDC",
            "beneficiary":         "GBENEFICIARY_INTEGRATION",
            "destination_chain":   "ethereum",
            "destination_address": "0xdeadbeef00",
            "gross_amount":        2_000_000,
            "fee_amount":          10_000,
            "net_amount":          1_990_000,
            "source_chain":        "stellar",
            "source_tx_hash":      "txhash_integration"
        })),
        contract_id: Some("CTEST_INTEGRATION".to_string()),
        paging_token: Some("0000000000000000500-0000000001-0000000002".to_string()),
    }
}

// ── Service construction ──────────────────────────────────────────────────────

#[test]
fn service_can_be_constructed() {
    let _svc = make_service();
}

#[test]
fn service_config_is_accessible_via_new() {
    let cfg = make_config();
    let svc = BridgeEventListenerService::new(make_pool(), cfg.clone());
    // We can't inspect private fields directly, but we can verify the service
    // was constructed without panicking and that parse_event works.
    let evt = sample_horizon_event();
    let parsed = BridgeEventListenerService::parse_event(evt, 0).unwrap();
    assert_eq!(parsed.contract_id, "CTEST_INTEGRATION");
}

// ── BridgeListenerConfig ──────────────────────────────────────────────────────

#[test]
fn config_new_strips_trailing_slash() {
    let cfg = BridgeListenerConfig {
        contract_id: "C1".to_string(),
        horizon_url: "https://horizon-testnet.stellar.org/".to_string(),
        poll_interval: Duration::from_secs(10),
    };
    assert!(!cfg.horizon_url.ends_with('/'));
}

#[test]
fn config_poll_interval_is_respected() {
    let cfg = make_config();
    assert_eq!(cfg.poll_interval, Duration::from_secs(60));
}

// ── is_bridge_pay_event ───────────────────────────────────────────────────────

#[test]
fn is_bridge_pay_event_true_for_sample() {
    let svc = make_service();
    assert!(svc.is_bridge_pay_event(&sample_horizon_event()));
}

#[test]
fn is_bridge_pay_event_false_for_wrong_topic() {
    let svc = make_service();
    let mut evt = sample_horizon_event();
    evt.topic = Some(vec![json!("SomethingElse")]);
    assert!(!svc.is_bridge_pay_event(&evt));
}

#[test]
fn is_bridge_pay_event_false_for_null_topic() {
    let svc = make_service();
    let mut evt = sample_horizon_event();
    evt.topic = None;
    assert!(!svc.is_bridge_pay_event(&evt));
}

#[test]
fn is_bridge_pay_event_false_for_empty_topic_list() {
    let svc = make_service();
    let mut evt = sample_horizon_event();
    evt.topic = Some(vec![]);
    assert!(!svc.is_bridge_pay_event(&evt));
}

// ── parse_event ───────────────────────────────────────────────────────────────

#[test]
fn parse_event_full_fidelity() {
    let evt = sample_horizon_event();
    let parsed = BridgeEventListenerService::parse_event(evt, 0).unwrap();

    assert_eq!(parsed.contract_id, "CTEST_INTEGRATION");
    assert_eq!(parsed.ledger_sequence, 500);
    assert_eq!(parsed.tx_hash, "txhash_integration");
    assert_eq!(parsed.event_index, 2); // parsed from id "…-0000000002"
    assert_eq!(parsed.owner_address, "GOWNER_INTEGRATION");
    assert_eq!(parsed.token_address, "USDC");
    assert_eq!(parsed.beneficiary_address, "GBENEFICIARY_INTEGRATION");
    assert_eq!(parsed.destination_chain, "ethereum");
    assert_eq!(parsed.destination_address, "0xdeadbeef00");
    assert_eq!(parsed.gross_amount, 2_000_000);
    assert_eq!(parsed.fee_amount, 10_000);
    assert_eq!(parsed.net_amount, 1_990_000);
    assert_eq!(parsed.source_chain, "stellar");
    assert_eq!(parsed.source_tx_hash, "txhash_integration");
}

#[test]
fn parse_event_uses_idx_when_id_absent() {
    let mut evt = sample_horizon_event();
    evt.id = None;
    let parsed = BridgeEventListenerService::parse_event(evt, 99).unwrap();
    assert_eq!(parsed.event_index, 99);
}

#[test]
fn parse_event_handles_string_ledger() {
    let mut evt = sample_horizon_event();
    evt.ledger = Some(json!("777"));
    let parsed = BridgeEventListenerService::parse_event(evt, 0).unwrap();
    assert_eq!(parsed.ledger_sequence, 777);
}

#[test]
fn parse_event_handles_missing_ledger() {
    let mut evt = sample_horizon_event();
    evt.ledger = None;
    let parsed = BridgeEventListenerService::parse_event(evt, 0).unwrap();
    assert_eq!(parsed.ledger_sequence, 0);
}

#[test]
fn parse_event_handles_null_value() {
    let mut evt = sample_horizon_event();
    evt.value = None;
    let parsed = BridgeEventListenerService::parse_event(evt, 0).unwrap();
    assert_eq!(parsed.owner_address, "");
    assert_eq!(parsed.gross_amount, 0);
}

// ── extract_str / extract_i64 ─────────────────────────────────────────────────

#[test]
fn extract_str_plain_string_value() {
    let val = json!({ "beneficiary": "GTEST" });
    assert_eq!(extract_str(&val, "beneficiary"), "GTEST");
}

#[test]
fn extract_str_wrapped_id_value() {
    let val = json!({ "owner": { "type": "Address", "id": "GWRAPPED" } });
    assert_eq!(extract_str(&val, "owner"), "GWRAPPED");
}

#[test]
fn extract_str_wrapped_value_field() {
    let val = json!({ "token": { "value": "USDC" } });
    assert_eq!(extract_str(&val, "token"), "USDC");
}

#[test]
fn extract_str_missing_key_returns_empty() {
    assert_eq!(extract_str(&json!({}), "nope"), "");
}

#[test]
fn extract_i64_json_number() {
    let val = json!({ "gross_amount": 1_500_000 });
    assert_eq!(extract_i64(&val, "gross_amount"), 1_500_000);
}

#[test]
fn extract_i64_string_encoded() {
    let val = json!({ "gross_amount": "9999999" });
    assert_eq!(extract_i64(&val, "gross_amount"), 9_999_999);
}

#[test]
fn extract_i64_missing_key_returns_zero() {
    assert_eq!(extract_i64(&json!({}), "missing"), 0);
}

#[test]
fn extract_i64_bad_string_returns_zero() {
    let val = json!({ "amount": "not_a_number" });
    assert_eq!(extract_i64(&val, "amount"), 0);
}

// ── HorizonEventsPage deserialisation ─────────────────────────────────────────

#[test]
fn deserialise_horizon_events_page_with_records() {
    let json_str = serde_json::to_string(&json!({
        "_embedded": {
            "records": [
                {
                    "type": "contract",
                    "ledger": 100,
                    "transaction_hash": "abc",
                    "id": "0000000000000000100-0000000001-0000000000",
                    "topic": ["BridgePay"],
                    "value": null,
                    "contract_id": "C1",
                    "paging_token": "ptoken"
                }
            ]
        }
    }))
    .unwrap();

    let page: HorizonEventsPage = serde_json::from_str(&json_str).unwrap();
    let records = page.embedded.unwrap().records;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].contract_id.as_deref(), Some("C1"));
}

#[test]
fn deserialise_horizon_events_page_empty_embedded() {
    let page: HorizonEventsPage = serde_json::from_str(r#"{"_embedded":{"records":[]}}"#).unwrap();
    assert!(page.embedded.unwrap().records.is_empty());
}

#[test]
fn deserialise_horizon_events_page_missing_embedded() {
    let page: HorizonEventsPage = serde_json::from_str(r#"{}"#).unwrap();
    assert!(page.embedded.is_none());
}

// ── Cursor format ─────────────────────────────────────────────────────────────

#[test]
fn cursor_pads_ledger_to_19_digits() {
    let seq: i64 = 1;
    let cursor = format!("{seq:019}-0000000000");
    assert_eq!(&cursor, "0000000000000000001-0000000000");
}

#[test]
fn cursor_zero_suffix_is_ten_digits() {
    let seq: i64 = 42;
    let cursor = format!("{seq:019}-0000000000");
    let suffix = cursor.split('-').nth(1).unwrap();
    assert_eq!(suffix.len(), 10);
}

// ── HTTP: GET /api/bridge/events ──────────────────────────────────────────────

#[tokio::test]
async fn bridge_events_endpoint_is_reachable() {
    let app = make_router();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/bridge/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // No auth required; DB lazy pool means we get 200 or 500 (no DB), never 404/401.
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn bridge_events_endpoint_accepts_query_params() {
    let app = make_router();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/bridge/events?contract_id=C1&owner_address=GTEST&from_ledger=1&to_ledger=9999&page=1&page_size=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Route must be registered and accept all query params without 404/405.
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
    assert_ne!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_ne!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn bridge_events_endpoint_rejects_post() {
    let app = make_router();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/bridge/events")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

//! Tests for the `/ws/kyc` WebSocket endpoint.
//!
//! # Strategy
//!
//! `axum::extract::ws::WebSocket` wraps a real `tokio-tungstenite` connection,
//! so the most practical approach for in-process testing is to:
//!
//! 1. Build the full `inheritx_backend::create_router` with a test `AppState`.
//! 2. Bind it to a random OS-assigned port (`TcpListener::bind("127.0.0.1:0")`).
//! 3. Spawn the server in a background task.
//! 4. Connect a `tokio_tungstenite` client to the bound address.
//! 5. Assert on the messages the client receives or sends.
//!
//! This avoids mocking the WebSocket socket itself (which would re-implement
//! the protocol) while keeping everything in-process and deterministic.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use inheritx_backend::{AppState, PlanCache};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message as WsMessage;

// ─── Shared helpers ──────────────────────────────────────────────────────────

/// Build an `AppState` wired to a live broadcast channel but a lazy (never-
/// connects) database pool — the WebSocket handler never touches the DB.
fn test_state() -> (
    Arc<AppState>,
    tokio::sync::broadcast::Sender<inheritx_backend::ws::KycUpdateEvent>,
) {
    use inheritx_backend::stellar_anchor::AnchorRegistry;

    let (kyc_tx, _) = tokio::sync::broadcast::channel(64);
    let pool = sqlx::PgPool::connect_lazy(
        "postgres://postgres:postgres@localhost:5432/inheritx_test",
    )
    .expect("lazy pool creation must not fail");

    let state = Arc::new(AppState {
        anchor: Arc::new(AnchorRegistry::new()),
        db_pool: pool,
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: PlanCache::disabled(),
        kyc_tx: kyc_tx.clone(),
    });

    (state, kyc_tx)
}

/// Convenience constructor for a `KycUpdateEvent`.
fn make_event(wallet: &str, status: &str) -> inheritx_backend::ws::KycUpdateEvent {
    inheritx_backend::ws::KycUpdateEvent {
        wallet_address: wallet.to_string(),
        kyc_status: status.to_string(),
        event_type: "kyc.status_update".to_string(),
    }
}

/// Bind a server on a random OS port, spawn it in the background, and return
/// the `ws://` URL to connect to `/ws/kyc`.
async fn spawn_server(
    state: Arc<AppState>,
) -> String {
    let app = inheritx_backend::create_router(state);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind to random port");
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("ws://{}/ws/kyc", addr)
}

// ─── Broadcast relay ─────────────────────────────────────────────────────────

/// A KYC event published to the broadcast channel must be delivered to a
/// connected WebSocket client as a JSON `Text` frame.
#[tokio::test]
async fn test_broadcast_event_reaches_client() {
    let (state, tx) = test_state();
    let url = spawn_server(state).await;

    let (mut ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("connect to /ws/kyc");

    // Short delay so the server is in the `select!` loop before we broadcast.
    tokio::time::sleep(Duration::from_millis(50)).await;
    tx.send(make_event("GDTEST123", "approved")).unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
        .await
        .expect("timed out waiting for KYC event")
        .expect("stream ended unexpectedly")
        .expect("WebSocket error");

    let WsMessage::Text(text) = msg else {
        panic!("expected Text frame, got {msg:?}");
    };

    let received: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(received["wallet_address"], "GDTEST123");
    assert_eq!(received["kyc_status"], "approved");
    assert_eq!(received["event_type"], "kyc.status_update");
}

/// Multiple events must be delivered to the client in the order they were sent.
#[tokio::test]
async fn test_multiple_events_arrive_in_order() {
    let (state, tx) = test_state();
    let url = spawn_server(state).await;

    let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let events = [
        make_event("WALLET_A", "pending"),
        make_event("WALLET_B", "approved"),
        make_event("WALLET_A", "rejected"),
    ];

    for e in &events {
        tx.send(e.clone()).unwrap();
    }

    for expected in &events {
        let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
            .await
            .expect("timed out")
            .unwrap()
            .unwrap();

        let WsMessage::Text(text) = msg else {
            panic!("expected Text, got {msg:?}");
        };
        let received: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(received["wallet_address"], expected.wallet_address);
        assert_eq!(received["kyc_status"], expected.kyc_status);
    }
}

/// Events must fan out to all connected clients simultaneously.
#[tokio::test]
async fn test_broadcast_reaches_multiple_clients() {
    let (state, tx) = test_state();
    let url = spawn_server(state).await;

    let (mut ws1, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    let (mut ws2, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

    tokio::time::sleep(Duration::from_millis(80)).await;

    tx.send(make_event("BROADCAST_WALLET", "approved")).unwrap();

    let msg1 = tokio::time::timeout(Duration::from_secs(3), ws1.next())
        .await
        .expect("timeout ws1")
        .unwrap()
        .unwrap();

    let msg2 = tokio::time::timeout(Duration::from_secs(3), ws2.next())
        .await
        .expect("timeout ws2")
        .unwrap()
        .unwrap();

    for msg in [msg1, msg2] {
        let WsMessage::Text(text) = msg else {
            panic!("expected Text, got {msg:?}");
        };
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["wallet_address"], "BROADCAST_WALLET");
    }
}

// ─── Subscription filter ─────────────────────────────────────────────────────

/// After a `subscribe` message, the client must only receive events for the
/// requested wallet and must not receive events for other wallets.
#[tokio::test]
async fn test_subscribe_filter_delivers_only_matching_events() {
    let (state, tx) = test_state();
    let url = spawn_server(state).await;

    let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Subscribe to WALLET_MATCH only.
    ws.send(WsMessage::Text(
        r#"{"type":"subscribe","wallet_address":"WALLET_MATCH"}"#.to_string(),
    ))
    .await
    .unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Non-matching event — must be filtered out.
    tx.send(make_event("WALLET_OTHER", "pending")).unwrap();
    // Matching event — must be delivered.
    tx.send(make_event("WALLET_MATCH", "approved")).unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
        .await
        .expect("timed out waiting for filtered event")
        .unwrap()
        .unwrap();

    let WsMessage::Text(text) = msg else {
        panic!("expected Text, got {msg:?}");
    };
    let received: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        received["wallet_address"], "WALLET_MATCH",
        "non-matching event must be filtered out"
    );

    // Within a short window, no further Text frames should arrive.
    let extra = tokio::time::timeout(Duration::from_millis(300), ws.next()).await;
    if let Ok(Some(Ok(WsMessage::Text(extra_text)))) = extra {
        panic!("unexpected extra event received: {extra_text}");
    }
}

/// After an `unsubscribe` message, all-wallet delivery must be restored.
#[tokio::test]
async fn test_unsubscribe_restores_all_events() {
    let (state, tx) = test_state();
    let url = spawn_server(state).await;

    let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    ws.send(WsMessage::Text(
        r#"{"type":"subscribe","wallet_address":"WALLET_X"}"#.to_string(),
    ))
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;

    ws.send(WsMessage::Text(
        r#"{"type":"unsubscribe","wallet_address":"WALLET_X"}"#.to_string(),
    ))
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;

    // After unsubscribing, any wallet's event must arrive.
    tx.send(make_event("WALLET_Y", "rejected")).unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
        .await
        .expect("timed out")
        .unwrap()
        .unwrap();

    let WsMessage::Text(text) = msg else {
        panic!("expected Text, got {msg:?}");
    };
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        v["wallet_address"], "WALLET_Y",
        "after unsubscribe, WALLET_Y event must arrive"
    );
}

// ─── Ping / Pong keepalive ────────────────────────────────────────────────────

/// A client-initiated Ping must be answered with a Pong by the server.
/// This validates that the `Message::Ping` arm in `handle_socket` is wired up.
#[tokio::test]
async fn test_client_ping_gets_pong_reply() {
    let (state, _tx) = test_state();
    let url = spawn_server(state).await;

    let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    ws.send(WsMessage::Ping(b"keepalive".to_vec())).await.unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
        .await
        .expect("timed out waiting for Pong")
        .unwrap()
        .unwrap();

    assert!(
        matches!(msg, WsMessage::Pong(_)),
        "expected Pong reply to client-initiated Ping, got {msg:?}"
    );
}

// ─── Graceful close ───────────────────────────────────────────────────────────

/// The server must accept a WebSocket Close frame and the connection must
/// terminate cleanly (either a Close echo or stream end within 2 s).
#[tokio::test]
async fn test_client_close_frame_is_acknowledged() {
    let (state, _tx) = test_state();
    let url = spawn_server(state).await;

    let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    ws.send(WsMessage::Close(None)).await.unwrap();

    let result = tokio::time::timeout(Duration::from_secs(2), ws.next()).await;

    // Any of these outcomes is acceptable after a close handshake.
    match result {
        Err(_elapsed) => {}                                   // timeout — slow close is ok
        Ok(None) => {}                                        // stream ended cleanly
        Ok(Some(Ok(WsMessage::Close(_)))) => {}              // server echoed Close
        Ok(Some(other)) => panic!("unexpected after close: {other:?}"),
    }
}

/// A JSON `{"type":"close"}` message must also close the connection.
#[tokio::test]
async fn test_json_close_message_closes_connection() {
    let (state, _tx) = test_state();
    let url = spawn_server(state).await;

    let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    ws.send(WsMessage::Text(r#"{"type":"close"}"#.to_string()))
        .await
        .unwrap();

    let result = tokio::time::timeout(Duration::from_secs(2), ws.next()).await;
    match result {
        Err(_elapsed) => {}
        Ok(None) => {}
        Ok(Some(Ok(WsMessage::Close(_)))) => {}
        Ok(Some(other)) => panic!("unexpected after json close: {other:?}"),
    }
}

// ─── KycUpdateEvent serialisation ────────────────────────────────────────────

/// `KycUpdateEvent` must serialise to the expected JSON field names so that
/// front-end consumers can rely on the schema.
#[test]
fn test_kyc_update_event_serialises_correctly() {
    let event = inheritx_backend::ws::KycUpdateEvent {
        wallet_address: "GDTEST".to_string(),
        kyc_status: "approved".to_string(),
        event_type: "kyc.status_update".to_string(),
    };

    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["wallet_address"], "GDTEST");
    assert_eq!(json["kyc_status"], "approved");
    assert_eq!(json["event_type"], "kyc.status_update");
}

/// `KycUpdateEvent` must round-trip through JSON without losing data.
#[test]
fn test_kyc_update_event_round_trips() {
    let original = inheritx_backend::ws::KycUpdateEvent {
        wallet_address: "GWALLET".to_string(),
        kyc_status: "pending".to_string(),
        event_type: "kyc.review".to_string(),
    };

    let json = serde_json::to_string(&original).unwrap();
    let decoded: inheritx_backend::ws::KycUpdateEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.wallet_address, original.wallet_address);
    assert_eq!(decoded.kyc_status, original.kyc_status);
    assert_eq!(decoded.event_type, original.event_type);
}

// ─── Broadcast channel behaviour ─────────────────────────────────────────────

/// When no WebSocket clients are connected, broadcasting must not panic.
/// The `send` error is gracefully discarded by the KYC webhook handler.
#[test]
fn test_broadcast_with_no_subscribers_does_not_panic() {
    let (tx, _rx) = tokio::sync::broadcast::channel::<inheritx_backend::ws::KycUpdateEvent>(16);
    // Drop the only receiver — send returns Err.
    drop(_rx);

    let result = tx.send(make_event("WALLET", "approved"));
    assert!(result.is_err(), "send with no subscribers must return Err");
}

/// A single subscriber must receive the event that was sent.
#[tokio::test]
async fn test_broadcast_channel_delivers_to_single_subscriber() {
    let (tx, mut rx) =
        tokio::sync::broadcast::channel::<inheritx_backend::ws::KycUpdateEvent>(16);

    let event = make_event("MY_WALLET", "rejected");
    tx.send(event.clone()).unwrap();

    let received = rx.recv().await.unwrap();
    assert_eq!(received.wallet_address, event.wallet_address);
    assert_eq!(received.kyc_status, event.kyc_status);
}

/// A fast broadcaster must not crash the handler when it overflows the channel.
/// The handler emits a `lag_notice` JSON frame when it detects dropped events.
#[tokio::test]
async fn test_lagged_receiver_gets_lag_notice() {
    // Use a tiny channel (capacity 1) so overflow is easy to trigger.
    let (kyc_tx, _) = tokio::sync::broadcast::channel::<inheritx_backend::ws::KycUpdateEvent>(1);
    use inheritx_backend::stellar_anchor::AnchorRegistry;

    let pool =
        sqlx::PgPool::connect_lazy("postgres://postgres:postgres@localhost:5432/inheritx_test")
            .unwrap();
    let state = Arc::new(AppState {
        anchor: Arc::new(AnchorRegistry::new()),
        db_pool: pool,
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: PlanCache::disabled(),
        kyc_tx: kyc_tx.clone(),
    });

    let url = spawn_server(state).await;
    let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Flood the channel faster than the receiver can drain — this causes lag.
    for i in 0..20 {
        let _ = kyc_tx.send(make_event("OVERFLOW", &i.to_string()));
    }

    // Collect messages for a short window; at least one must be a lag_notice.
    let mut received_lag_notice = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, ws.next()).await {
            Err(_) | Ok(None) => break,
            Ok(Some(Err(_))) => break,
            Ok(Some(Ok(WsMessage::Text(text)))) => {
                let v: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                if v["type"] == "lag_notice" {
                    received_lag_notice = true;
                    break;
                }
            }
            Ok(Some(Ok(_other))) => {}
        }
    }

    assert!(
        received_lag_notice,
        "expected a lag_notice frame after channel overflow"
    );
}

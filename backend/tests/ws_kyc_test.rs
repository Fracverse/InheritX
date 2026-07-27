//! Integration tests for the `/ws/kyc` WebSocket endpoint.
//!
//! These tests spin up the Axum router in-process using `tower::ServiceExt`
//! and verify end-to-end WebSocket behaviour without requiring a real database
//! or a running server.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use futures_util::{SinkExt, StreamExt};
use inheritx_backend::{ws::KycUpdateEvent, AppState};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message as TungsteniteMessage};
use tower::ServiceExt;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a minimal [`AppState`] with a real broadcast sender so tests can
/// inject KYC events without a live PostgreSQL instance.
fn make_state() -> (Arc<AppState>, broadcast::Sender<KycUpdateEvent>) {
    let (kyc_tx, _) = broadcast::channel(64);
    let pool =
        sqlx::PgPool::connect_lazy("postgres://postgres:postgres@localhost:5432/inheritx_test")
            .unwrap();

    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        db_pool: pool,
        kyc_webhook_secret: None,
        apy_config: inheritx_backend::yield_calculator::ApyConfig::default(),
        plan_cache: inheritx_backend::PlanCache::disabled(),
        kyc_tx: kyc_tx.clone(),
    });
    (state, kyc_tx)
}

// ── HTTP upgrade tests ────────────────────────────────────────────────────────

/// A GET to `/ws/kyc` with a valid `Upgrade: websocket` handshake headers
/// should return HTTP 101 Switching Protocols.
#[tokio::test]
async fn test_ws_upgrade_returns_101() {
    let (state, _tx) = make_state();
    let app = inheritx_backend::create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/ws/kyc")
                .header("Host", "localhost")
                .header("Upgrade", "websocket")
                .header("Connection", "Upgrade")
                .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
                .header("Sec-WebSocket-Version", "13")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
}

/// A plain GET to `/ws/kyc` without upgrade headers should not return 200 as
/// if it were a normal REST endpoint.
#[tokio::test]
async fn test_ws_non_upgrade_request_rejected() {
    let (state, _tx) = make_state();
    let app = inheritx_backend::create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/ws/kyc")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Axum rejects non-upgrade requests to WebSocket routes with 400 or 426.
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UPGRADE_REQUIRED,
        "Expected 400 or 426, got {}",
        response.status()
    );
}

// ── Live WebSocket tests ──────────────────────────────────────────────────────
//
// These tests bind a real TCP listener, start the Axum server in a background
// task, connect with a real WebSocket client, and exchange frames.

/// Bind a random port and return (addr, server_join_handle).
async fn spawn_server(
    state: Arc<AppState>,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = inheritx_backend::create_router(state);
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, handle)
}

/// Connect a tungstenite WebSocket client to the given address and path.
async fn connect_ws(
    addr: std::net::SocketAddr,
    path: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let url = format!("ws://{}{}", addr, path);
    let (ws, _response) = connect_async(url).await.expect("WebSocket connect failed");
    ws
}

/// A KYC event published to the broadcast channel should be delivered to all
/// connected clients as a JSON text frame.
#[tokio::test]
async fn test_ws_receives_kyc_event() {
    let (state, tx) = make_state();
    let (addr, _srv) = spawn_server(state).await;

    let mut ws = connect_ws(addr, "/ws/kyc").await;

    // Give the server a moment to register the subscriber before publishing.
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let event = KycUpdateEvent {
        wallet_address: "GDTEST_RECEIVES".to_string(),
        kyc_status: "approved".to_string(),
        event_type: "kyc.status_update".to_string(),
    };
    tx.send(event.clone()).ok();

    let msg = tokio::time::timeout(tokio::time::Duration::from_secs(2), ws.next())
        .await
        .expect("Timeout waiting for KYC event")
        .expect("Stream ended")
        .expect("WebSocket error");

    if let TungsteniteMessage::Text(text) = msg {
        let received: KycUpdateEvent = serde_json::from_str(&text).unwrap();
        assert_eq!(received.wallet_address, "GDTEST_RECEIVES");
        assert_eq!(received.kyc_status, "approved");
        assert_eq!(received.event_type, "kyc.status_update");
    } else {
        panic!("Expected Text frame, got {:?}", msg);
    }
}

/// Multiple events should all be delivered in order.
#[tokio::test]
async fn test_ws_receives_multiple_events_in_order() {
    let (state, tx) = make_state();
    let (addr, _srv) = spawn_server(state).await;

    let mut ws = connect_ws(addr, "/ws/kyc").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let statuses = ["pending", "submitted", "approved"];
    for status in &statuses {
        tx.send(KycUpdateEvent {
            wallet_address: "GDTEST_ORDER".to_string(),
            kyc_status: status.to_string(),
            event_type: "kyc.status_update".to_string(),
        })
        .ok();
    }

    for expected_status in &statuses {
        let msg = tokio::time::timeout(tokio::time::Duration::from_secs(2), ws.next())
            .await
            .expect("Timeout")
            .expect("Stream ended")
            .unwrap();

        if let TungsteniteMessage::Text(text) = msg {
            let received: KycUpdateEvent = serde_json::from_str(&text).unwrap();
            assert_eq!(&received.kyc_status, expected_status);
        } else {
            panic!("Expected Text frame, got {:?}", msg);
        }
    }
}

/// When `wallet_address` filter is set, only events matching that wallet are
/// forwarded; events for other wallets are silently dropped.
#[tokio::test]
async fn test_ws_wallet_address_filter() {
    let (state, tx) = make_state();
    let (addr, _srv) = spawn_server(state).await;

    // Subscribe filtered to wallet "GDFILTERED"
    let mut ws = connect_ws(addr, "/ws/kyc?wallet_address=GDFILTERED").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Send event for a DIFFERENT wallet first — should be dropped
    tx.send(KycUpdateEvent {
        wallet_address: "GDOTHER".to_string(),
        kyc_status: "rejected".to_string(),
        event_type: "kyc.status_update".to_string(),
    })
    .ok();

    // Then send the event for the correct wallet
    tx.send(KycUpdateEvent {
        wallet_address: "GDFILTERED".to_string(),
        kyc_status: "approved".to_string(),
        event_type: "kyc.status_update".to_string(),
    })
    .ok();

    // The first message received should be for GDFILTERED, not GDOTHER.
    let msg = tokio::time::timeout(tokio::time::Duration::from_secs(2), ws.next())
        .await
        .expect("Timeout")
        .expect("Stream ended")
        .unwrap();

    if let TungsteniteMessage::Text(text) = msg {
        let received: KycUpdateEvent = serde_json::from_str(&text).unwrap();
        assert_eq!(received.wallet_address, "GDFILTERED");
        assert_eq!(received.kyc_status, "approved");
    } else {
        panic!("Expected Text frame, got {:?}", msg);
    }
}

/// When `wallet_address` filter is set and NO events match, the client should
/// receive nothing within a short window (no spurious messages).
#[tokio::test]
async fn test_ws_wallet_filter_no_match_delivers_nothing() {
    let (state, tx) = make_state();
    let (addr, _srv) = spawn_server(state).await;

    let mut ws = connect_ws(addr, "/ws/kyc?wallet_address=GDMINE").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Send only events for a different wallet
    for _ in 0..3 {
        tx.send(KycUpdateEvent {
            wallet_address: "GDNOTMINE".to_string(),
            kyc_status: "pending".to_string(),
            event_type: "kyc.status_update".to_string(),
        })
        .ok();
    }

    // Expect no Text frame within 300 ms
    let result = tokio::time::timeout(tokio::time::Duration::from_millis(300), ws.next()).await;
    match result {
        Err(_) => {} // timeout — correct, no message received
        Ok(Some(Ok(TungsteniteMessage::Text(_)))) => {
            panic!("Received unexpected text frame for filtered-out wallet");
        }
        _ => {} // Ping / Close / etc. are fine
    }
}

/// The server should respond to a client-initiated Ping with a Pong frame.
#[tokio::test]
async fn test_ws_responds_to_client_ping() {
    let (state, _tx) = make_state();
    let (addr, _srv) = spawn_server(state).await;

    let mut ws = connect_ws(addr, "/ws/kyc").await;

    ws.send(TungsteniteMessage::Ping(b"hello".to_vec().into()))
        .await
        .unwrap();

    let msg = tokio::time::timeout(tokio::time::Duration::from_secs(2), ws.next())
        .await
        .expect("Timeout waiting for Pong")
        .expect("Stream ended")
        .unwrap();

    assert!(
        matches!(msg, TungsteniteMessage::Pong(_)),
        "Expected Pong frame, got {:?}",
        msg
    );
}

/// The server should complete the closing handshake when the client sends a
/// Close frame: the client should receive a Close frame back.
#[tokio::test]
async fn test_ws_graceful_close_handshake() {
    let (state, _tx) = make_state();
    let (addr, _srv) = spawn_server(state).await;

    let mut ws = connect_ws(addr, "/ws/kyc").await;

    ws.send(TungsteniteMessage::Close(None)).await.unwrap();

    // The server should echo a Close frame to complete the handshake.
    let msg = tokio::time::timeout(tokio::time::Duration::from_secs(2), ws.next())
        .await
        .expect("Timeout waiting for Close echo")
        .expect("Stream ended")
        .unwrap();

    assert!(
        matches!(msg, TungsteniteMessage::Close(_)),
        "Expected Close frame, got {:?}",
        msg
    );
}

/// Two simultaneous clients should each receive the same broadcast event.
#[tokio::test]
async fn test_ws_multiple_clients_all_receive_event() {
    let (state, tx) = make_state();
    let (addr, _srv) = spawn_server(state).await;

    let mut ws1 = connect_ws(addr, "/ws/kyc").await;
    let mut ws2 = connect_ws(addr, "/ws/kyc").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let event = KycUpdateEvent {
        wallet_address: "GDBROADCAST".to_string(),
        kyc_status: "approved".to_string(),
        event_type: "kyc.status_update".to_string(),
    };
    tx.send(event).ok();

    for ws in [&mut ws1, &mut ws2] {
        let msg = tokio::time::timeout(tokio::time::Duration::from_secs(2), ws.next())
            .await
            .expect("Timeout")
            .expect("Stream ended")
            .unwrap();

        if let TungsteniteMessage::Text(text) = msg {
            let received: KycUpdateEvent = serde_json::from_str(&text).unwrap();
            assert_eq!(received.wallet_address, "GDBROADCAST");
        } else {
            panic!("Expected Text frame, got {:?}", msg);
        }
    }
}

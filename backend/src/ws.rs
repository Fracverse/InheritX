//! WebSocket handler for real-time KYC status updates.
//!
//! Endpoint: `GET /ws/kyc`
//!
//! Clients connect and receive [`KycUpdateEvent`] messages as JSON whenever a
//! KYC webhook is processed.  An optional query parameter `wallet_address` can
//! be supplied to receive events for only that specific wallet:
//!
//! ```
//! ws://host/ws/kyc                        # all events
//! ws://host/ws/kyc?wallet_address=GABCDE  # filtered to one wallet
//! ```
//!
//! ## Protocol
//!
//! | Direction        | Frame type | Description                         |
//! |------------------|-----------|-------------------------------------|
//! | Server → Client  | Text       | JSON-encoded [`KycUpdateEvent`]      |
//! | Server → Client  | Ping       | Heartbeat every `PING_INTERVAL_SECS` |
//! | Client → Server  | Pong       | Required heartbeat response          |
//! | Client → Server  | Text/Binary| Acknowledged but otherwise ignored  |
//! | Client → Server  | Close      | Triggers graceful shutdown           |
//!
//! If the server sends a Ping and does not receive a Pong within
//! `PONG_TIMEOUT_SECS` seconds, the connection is closed.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, timeout, Duration, Instant};
use tracing::{debug, info, warn};

use crate::api::AppState;

// ── Timing constants ──────────────────────────────────────────────────────────

/// How often the server sends a Ping frame to keep the connection alive.
const PING_INTERVAL_SECS: u64 = 30;

/// How long the server waits for a Pong response before disconnecting.
const PONG_TIMEOUT_SECS: u64 = 10;

// ── Public types ──────────────────────────────────────────────────────────────

/// An event broadcast to all connected WebSocket clients (or a filtered subset)
/// whenever a KYC status update is processed by the webhook handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycUpdateEvent {
    pub wallet_address: String,
    pub kyc_status: String,
    pub event_type: String,
}

// ── Query params ──────────────────────────────────────────────────────────────

/// Optional query parameters accepted by [`ws_handler`].
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// When set, only KYC events for this wallet address are forwarded.
    pub wallet_address: Option<String>,
}

// ── Axum handler ─────────────────────────────────────────────────────────────

/// Upgrades an HTTP connection to a WebSocket and starts [`handle_socket`].
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let wallet_filter = query.wallet_address;
    ws.on_upgrade(move |socket| handle_socket(socket, state, wallet_filter))
}

// ── Core socket loop ──────────────────────────────────────────────────────────

/// Manages a single WebSocket client for the lifetime of the connection.
///
/// Responsibilities:
/// * Subscribe to the `kyc_tx` broadcast channel and forward matching events.
/// * Send periodic Ping frames and enforce Pong replies within the timeout.
/// * Handle Close frames from the client for a graceful shutdown.
/// * Log lag warnings if the broadcast channel overflows.
async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<AppState>,
    wallet_filter: Option<String>,
) {
    info!(
        wallet_filter = ?wallet_filter,
        "WebSocket client connected for KYC updates"
    );

    let mut rx = state.kyc_tx.subscribe();

    // Heartbeat state
    let mut ping_ticker = interval(Duration::from_secs(PING_INTERVAL_SECS));
    ping_ticker.tick().await; // consume the immediate first tick
    let mut awaiting_pong = false;
    let mut last_ping_at: Option<Instant> = None;

    loop {
        tokio::select! {
            // ── Outbound: KYC events from the broadcast channel ───────────────
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        // Apply optional wallet-address filter
                        if let Some(ref filter) = wallet_filter {
                            if &event.wallet_address != filter {
                                debug!(
                                    wallet_address = %event.wallet_address,
                                    filter = %filter,
                                    "WebSocket: skipping event (wallet filter)"
                                );
                                continue;
                            }
                        }

                        let msg = match serde_json::to_string(&event) {
                            Ok(s) => s,
                            Err(e) => {
                                warn!(error = %e, "Failed to serialize KYC event");
                                continue;
                            }
                        };

                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            info!("WebSocket client disconnected while sending KYC event");
                            break;
                        }
                    }

                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(
                            missed = n,
                            "WebSocket receiver lagged; {} KYC events were skipped", n
                        );
                        // Keep going — the client stays connected but missed some events.
                    }

                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!("KYC broadcast channel closed; terminating WebSocket");
                        break;
                    }
                }
            }

            // ── Heartbeat: send Ping at regular intervals ─────────────────────
            _ = ping_ticker.tick() => {
                if awaiting_pong {
                    // Previous ping was never answered — the client is gone.
                    warn!("WebSocket heartbeat timeout (no Pong received); closing connection");
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }

                let payload = chrono::Utc::now().timestamp_millis().to_string();
                if socket
                    .send(Message::Ping(payload.into_bytes().into()))
                    .await
                    .is_err()
                {
                    info!("WebSocket client disconnected during Ping");
                    break;
                }

                awaiting_pong = true;
                last_ping_at = Some(Instant::now());
                debug!("WebSocket heartbeat Ping sent");
            }

            // ── Inbound: messages from the client ─────────────────────────────
            msg = timeout(
                // Use a generous timeout so the receive future doesn't block
                // the other arms forever when the client is idle.
                Duration::from_secs(PING_INTERVAL_SECS + PONG_TIMEOUT_SECS),
                socket.recv()
            ) => {
                match msg {
                    // Pong response — reset heartbeat state
                    Ok(Some(Ok(Message::Pong(_)))) => {
                        let rtt = last_ping_at.map(|t| t.elapsed().as_millis()).unwrap_or(0);
                        debug!(rtt_ms = rtt, "WebSocket heartbeat Pong received");
                        awaiting_pong = false;
                    }

                    // Client initiated a clean close
                    Ok(Some(Ok(Message::Close(frame)))) => {
                        info!(
                            reason = ?frame.as_ref().map(|f| f.reason.as_ref() as &str),
                            "WebSocket client sent Close frame; closing gracefully"
                        );
                        // Echo the Close to complete the WebSocket closing handshake
                        let _ = socket.send(Message::Close(frame)).await;
                        break;
                    }

                    // Text / Binary messages — acknowledge but do not act on them
                    Ok(Some(Ok(Message::Text(text)))) => {
                        debug!(text = %text, "WebSocket received text from client (ignored)");
                    }
                    Ok(Some(Ok(Message::Binary(_)))) => {
                        debug!("WebSocket received binary from client (ignored)");
                    }

                    // Ping from client — send Pong back
                    Ok(Some(Ok(Message::Ping(data)))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            info!("WebSocket client disconnected during Pong reply");
                            break;
                        }
                    }

                    // Socket read error or EOF
                    Ok(Some(Err(e))) => {
                        debug!(error = %e, "WebSocket read error; closing connection");
                        break;
                    }

                    // Stream exhausted (clean close from the transport layer)
                    Ok(None) => {
                        info!("WebSocket client stream closed");
                        break;
                    }

                    // Receive-future timed out — just continue the loop so the
                    // other arms (ping ticker, broadcast channel) can fire.
                    Err(_timeout) => {}
                }
            }
        }
    }

    info!(
        wallet_filter = ?wallet_filter,
        "WebSocket connection closed"
    );
}

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info, warn};

use crate::api::AppState;

/// How often the server sends a WebSocket Ping frame to keep the connection alive.
const PING_INTERVAL: Duration = Duration::from_secs(30);

/// Maximum wait for a Pong reply before considering the connection stale.
const PONG_TIMEOUT: Duration = Duration::from_secs(10);

/// An event broadcast to all `/ws/kyc` subscribers when a KYC status changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycUpdateEvent {
    pub wallet_address: String,
    pub kyc_status: String,
    pub event_type: String,
}

/// Optional subscription filter a client may send after connecting.
///
/// ```json
/// { "type": "subscribe", "wallet_address": "GDTEST123" }
/// ```
///
/// If a client sends no filter, it receives updates for *all* wallet addresses.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to updates for a specific wallet address only.
    Subscribe { wallet_address: String },
    /// Unsubscribe from a specific wallet address (receive all again).
    Unsubscribe { wallet_address: String },
    /// Explicit close request from the client.
    Close,
}

/// Upgrade HTTP → WebSocket and hand off to `handle_socket`.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Per-connection handler.
///
/// Lifecycle:
/// 1. Subscribe to the application-wide broadcast channel.
/// 2. Enter a `select!` loop that concurrently:
///    - Relays incoming broadcast events to the client (optionally filtered).
///    - Reads client-sent frames (subscribe/unsubscribe/close/pong).
///    - Fires a server-initiated Ping every `PING_INTERVAL`.
/// 3. Close gracefully when the client disconnects or the channel closes.
async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket client connected on /ws/kyc");

    let mut rx = state.kyc_tx.subscribe();
    let mut ping_ticker = interval(PING_INTERVAL);
    // The first tick fires immediately; skip it so we don't ping on connect.
    ping_ticker.tick().await;

    // Optional wallet-address filter set by the client.
    let mut wallet_filter: Option<String> = None;

    // Track whether we are waiting for a Pong reply.
    let mut awaiting_pong = false;
    let mut pong_deadline: Option<tokio::time::Instant> = None;

    loop {
        tokio::select! {
            // ── Server-initiated keepalive ping ───────────────────────────────
            _ = ping_ticker.tick() => {
                if awaiting_pong {
                    // Previous ping was not answered in time — connection stale.
                    warn!("WebSocket client did not respond to Ping; closing stale connection");
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }

                debug!("Sending WebSocket Ping to client");
                if socket.send(Message::Ping(b"ping".to_vec().into())).await.is_err() {
                    info!("WebSocket client disconnected (send failed)");
                    break;
                }

                awaiting_pong = true;
                pong_deadline = Some(tokio::time::Instant::now() + PONG_TIMEOUT);
            }

            // ── Pong timeout check ────────────────────────────────────────────
            _ = async {
                match pong_deadline {
                    Some(deadline) => tokio::time::sleep_until(deadline).await,
                    None => std::future::pending::<()>().await,
                }
            } => {
                if awaiting_pong {
                    warn!("WebSocket Pong timeout; closing stale connection");
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }
                pong_deadline = None;
            }

            // ── Incoming frame from client ────────────────────────────────────
            maybe_msg = socket.recv() => {
                match maybe_msg {
                    None => {
                        // Stream closed — client disconnected cleanly.
                        info!("WebSocket client disconnected");
                        break;
                    }
                    Some(Err(e)) => {
                        warn!(error = %e, "WebSocket receive error; closing connection");
                        break;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        debug!("Received WebSocket Pong from client");
                        awaiting_pong = false;
                        pong_deadline = None;
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        // Client-initiated ping — reply with pong.
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            info!("WebSocket client disconnected (pong send failed)");
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket client requested close");
                        let _ = socket.send(Message::Close(None)).await;
                        break;
                    }
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::Subscribe { wallet_address }) => {
                                info!(
                                    wallet_address = %wallet_address,
                                    "WebSocket client subscribed to KYC updates"
                                );
                                wallet_filter = Some(wallet_address);
                            }
                            Ok(ClientMessage::Unsubscribe { wallet_address: _ }) => {
                                info!("WebSocket client unsubscribed from wallet filter");
                                wallet_filter = None;
                            }
                            Ok(ClientMessage::Close) => {
                                info!("WebSocket client requested close via message");
                                let _ = socket.send(Message::Close(None)).await;
                                break;
                            }
                            Err(e) => {
                                warn!(error = %e, raw = %text, "Unknown WebSocket message from client; ignoring");
                            }
                        }
                    }
                    Some(Ok(Message::Binary(_))) => {
                        // Binary frames are not part of this protocol; ignore silently.
                    }
                }
            }

            // ── Broadcast KYC event from server ──────────────────────────────
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        // Apply wallet-address filter if the client set one.
                        if let Some(ref filter) = wallet_filter {
                            if &event.wallet_address != filter {
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
                            info!("WebSocket client disconnected (send failed)");
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(
                            missed = n,
                            "WebSocket receiver lagged; {} KYC events were dropped", n
                        );
                        // Notify client that it missed some events.
                        let notice = serde_json::json!({
                            "type": "lag_notice",
                            "missed_events": n
                        });
                        if let Ok(s) = serde_json::to_string(&notice) {
                            if socket.send(Message::Text(s.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        // Sender was dropped — server shutting down.
                        info!("KYC broadcast channel closed; disconnecting WebSocket client");
                        let _ = socket.send(Message::Close(None)).await;
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket session ended for /ws/kyc");
}

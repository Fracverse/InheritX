use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::api::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycUpdateEvent {
    pub wallet_address: String,
    pub kyc_status: String,
    pub event_type: String,
}

/// Loan-lifecycle (and other plan) status updates pushed to WebSocket clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStatusEvent {
    pub event_type: String,
    pub plan_id: Uuid,
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeze_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recall_progress: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_loaned: Option<u64>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket client connected");
    let mut kyc_rx = state.kyc_tx.subscribe();
    let mut status_rx = state.status_tx.subscribe();

    loop {
        tokio::select! {
            result = kyc_rx.recv() => {
                match result {
                    Ok(event) => {
                        if send_json(&mut socket, &event).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket KYC receiver lagged by {} messages", n);
                        continue;
                    }
                    Err(_) => break,
                }
            }
            result = status_rx.recv() => {
                match result {
                    Ok(event) => {
                        if send_json(&mut socket, &event).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket plan-status receiver lagged by {} messages", n);
                        continue;
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket client sent close frame");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        match socket.send(Message::Pong(data)).await {
                            Ok(_) => {}
                            Err(_) => break,
                        }
                    }
                    Some(Err(e)) => {
                        warn!(error = %e, "WebSocket error");
                        break;
                    }
                    None => {
                        info!("WebSocket client disconnected");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("WebSocket client disconnected");
}

async fn send_json<T: Serialize>(socket: &mut WebSocket, event: &T) -> Result<(), ()> {
    let msg = match serde_json::to_string(event) {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "Failed to serialize WebSocket event");
            return Ok(());
        }
    };
    socket.send(Message::Text(msg.into())).await.map_err(|_| ())
}

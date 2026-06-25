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

use crate::api::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycUpdateEvent {
    pub wallet_address: String,
    pub kyc_status: String,
    pub event_type: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket client connected for KYC updates");
    let mut rx = state.kyc_tx.subscribe();

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        let msg = match serde_json::to_string(&event) {
                            Ok(s) => s,
                            Err(e) => {
                                warn!(error = %e, "Failed to serialize KYC event");
                                continue;
                            }
                        };
                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            info!("WebSocket client disconnected");
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket receiver lagged by {} messages", n);
                    }
                    Err(_) => break,
                }
            }
        }
    }
}

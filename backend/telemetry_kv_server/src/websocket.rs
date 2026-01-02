use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade, Message},
        State,
    },
    response::IntoResponse,
};
use tokio::sync::{Mutex, broadcast};
use std::sync::Arc;
use crate::storage::TelemetryStorage;
use crate::types::TelemetryPacket;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Mutex<TelemetryStorage>>,
    pub broadcast_tx: broadcast::Sender<TelemetryPacket>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    println!("✓ Client connected to WebSocket");
    
    // Send historical data (current flight)
    {
        let storage = state.storage.lock().await;
        if let Some(flight_id) = storage.get_current_flight_id() {
            let packets = storage.get_flight_data(&flight_id);
            println!("  Sending {} historical packets from current flight", packets.len());
            
            for packet in packets {
                let json = serde_json::to_string(&packet).unwrap();
                if socket.send(Message::Text(json)).await.is_err() {
                    println!("✗ Client disconnected during historical send");
                    return;
                }
            }
        }
    }
    
    // Stream real-time data
    let mut rx = state.broadcast_tx.subscribe();
    while let Ok(packet) = rx.recv().await {
        let json = serde_json::to_string(&packet).unwrap();
        if socket.send(Message::Text(json)).await.is_err() {
            println!("✗ Client disconnected");
            break;
        }
    }
}

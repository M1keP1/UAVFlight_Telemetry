use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::sync::broadcast;
use crate::telemetry::TelemetryPacket;

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<TelemetryPacket>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/ws/binary", get(websocket_binary_handler))
        .with_state(state)
}

/// Binary WebSocket endpoint - simulates ESP32 → KV store
/// This is what the ESP32 LoRa will send
async fn websocket_binary_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_binary_socket(socket, state))
}

async fn handle_binary_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    
    println!("[Simulator] Binary client connected");
    
    while let Ok(packet) = rx.recv().await {
        let bytes = packet.to_bytes();
        
        if socket.send(Message::Binary(bytes)).await.is_err() {
            println!("[Simulator] Binary client disconnected");
            break;
        }
    }
}

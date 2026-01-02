mod types;
mod storage;
mod binary_client;
mod websocket;
mod api;

use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use axum::{
    routing::get,
    Router,
    response::Html,
};
use tower_http::cors::CorsLayer;

use storage::TelemetryStorage;
use websocket::AppState;

async fn serve_control_panel() -> Html<&'static str> {
    Html(include_str!("../../control_panel.html"))
}

#[tokio::main]
async fn main() {
    println!("[Server] Telemetry KV Server\n");
    
    // Initialize storage
    let storage = Arc::new(Mutex::new(
        TelemetryStorage::new("telemetry_data")
            .expect("Failed to initialize storage")
    ));
    
    // Create broadcast channel for real-time streaming
    let (broadcast_tx, _) = broadcast::channel(1000);
    
    // Start binary client task
    let storage_clone = storage.clone();
    let tx_clone = broadcast_tx.clone();
    tokio::spawn(async move {
        binary_client::run_binary_client(storage_clone, tx_clone).await;
    });
    
    // Create app state
    let state = AppState {
        storage,
        broadcast_tx,
    };
    
    // Build router with all routes
    let app = Router::new()
        .route("/", get(serve_control_panel))
        .route("/health", get(|| async { "OK" }))
        .route("/ws/stream", get(websocket::websocket_handler))
        .route("/api/flights", get(api::list_flights))
        .route("/api/flights/:id/data", get(api::get_flight_data))
        .route("/api/flights/:id", 
            get(api::get_flight)
                .delete(api::delete_flight))
        .with_state(state)
        .layer(CorsLayer::permissive());
    
    
    println!("[Server] Control Panel: http://0.0.0.0:9090");
    println!("[Server] WebSocket: ws://0.0.0.0:9090/ws/stream");
    println!("[Server] REST API:  http://0.0.0.0:9090/api");
    println!("\nEndpoints:");
    println!("  GET    /api/flights          - List all flights");
    println!("  GET    /api/flights/:id      - Get flight details");
    println!("  GET    /api/flights/:id/data - Get flight telemetry");
    println!("  DELETE /api/flights/:id      - Delete flight");
    println!("\nWaiting for telemetry data...\n");
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9090")
        .await
        .expect("Failed to bind to port 9090");
    
    axum::serve(listener, app.into_make_service())
        .await
        .expect("Server error");
}

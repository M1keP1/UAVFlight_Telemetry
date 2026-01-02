use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::StreamExt;
use tokio::sync::{Mutex, broadcast};
use std::sync::Arc;
use crate::storage::TelemetryStorage;
use crate::types::TelemetryPacket;

pub async fn run_binary_client(
    storage: Arc<Mutex<TelemetryStorage>>,
    broadcast_tx: broadcast::Sender<TelemetryPacket>,
) {
    let url = std::env::var("SIMULATOR_WS_URL")
        .unwrap_or_else(|_| "ws://localhost:8080/ws/binary".to_string());
    
    loop {
        println!("[Server] Connecting to telemetry sim at {}...", url);
        
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                println!("[Server] Connected to telemetry sim");
                let (_, mut read) = ws_stream.split();
                
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Binary(bytes)) => {
                            if let Ok(packet) = TelemetryPacket::from_bytes(&bytes) {
                                // Store packet
                                if let Err(e) = storage.lock().await.save_packet(&packet) {
                                    eprintln!("Error saving packet: {}", e);
                                }
                                
                                // Broadcast to WebSocket clients
                                let _ = broadcast_tx.send(packet);
                            }
                        }
                        Ok(Message::Close(_)) => {
                            println!("[Server] Telemetry sim closed connection");
                            break;
                        }
                        Err(e) => {
                            eprintln!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to telemetry sim: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
        
        println!("Reconnecting in 5 seconds...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

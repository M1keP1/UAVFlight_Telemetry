mod telemetry;
mod trajectory;
mod generator;
mod server;

use generator::Generator;
use server::{create_router, AppState};
use tokio::time::{interval, Duration};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    println!("🚀 XtraChallenge Telemetry Simulator\n");
    
    // Create broadcast channel
    let (tx, _rx) = broadcast::channel(100);
    
    // Start generator task
    let gen_tx = tx.clone();
    tokio::spawn(async move {
        let mut gen = Generator::new();
        let mut ticker = interval(Duration::from_millis(500));
        
        println!("📡 Generator started (2 Hz)\n");
        
        loop {
            ticker.tick().await;
            
            let packet = gen.generate_packet();
            
            // Print to console
            println!(
                "#{:04} | GPS: {:.6},{:.6} | Alt: {:6.1}m | Batt: {:4.2}V ({:5.1}W) | RSSI: {:4}dBm",
                packet.packet_sequence,
                packet.latitude,
                packet.longitude,
                packet.altitude_baro,
                packet.battery_voltage,
                packet.battery_power,
                packet.rssi
            );
            
            // Broadcast to WebSocket clients
            gen_tx.send(packet).ok();
        }
    });
    
    // Start WebSocket server
    let state = AppState { tx };
    let app = create_router(state);
    
    let addr = "0.0.0.0:8080";
    println!("🌐 WebSocket endpoints:");
    println!("   Binary (ESP32 → KV):  ws://{}/ws/binary", addr);
    println!("   JSON (KV → Frontend): ws://{}/ws\n", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

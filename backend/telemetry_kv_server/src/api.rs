use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use crate::websocket::AppState;
use crate::types::{FlightMetadata, TelemetryPacket};

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryPacketWithPhase {
    #[serde(flatten)]
    pub packet: TelemetryPacket,
    pub flight_phase: String,
}

pub async fn list_flights(
    State(state): State<AppState>,
) -> Json<Vec<FlightMetadata>> {
    let storage = state.storage.lock().await;
    Json(storage.list_flights())
}

pub async fn get_flight(
    Path(flight_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<FlightMetadata>, StatusCode> {
    let storage = state.storage.lock().await;
    storage.get_flight(&flight_id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn get_flight_data(
    Path(flight_id): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<TelemetryPacketWithPhase>> {
    let storage = state.storage.lock().await;
    let packets = storage.get_flight_data(&flight_id);
    
    // Add flight_phase to each packet
    let packets_with_phase: Vec<TelemetryPacketWithPhase> = packets
        .into_iter()
        .map(|packet| TelemetryPacketWithPhase {
            flight_phase: packet.get_flight_phase().to_string(),
            packet,
        })
        .collect();
    
    Json(packets_with_phase)
}

pub async fn delete_flight(
    Path(flight_id): Path<String>,
    State(state): State<AppState>,
) -> StatusCode {
    let mut storage = state.storage.lock().await;
    match storage.delete_flight(&flight_id) {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

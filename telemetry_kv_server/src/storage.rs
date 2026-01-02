use kiwi_store::{Store, Key, Value, BorrowedEntry, StoreError};
use crate::types::{TelemetryPacket, FlightMetadata};
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
enum FlightState {
    OnGround,
    InFlight,
    Landing,
}

pub struct TelemetryStorage {
    store: Store,
    current_flight_id: Option<String>,
    flight_state: FlightState,
    landing_check_start: Option<u64>,
    last_position: Option<(f64, f64)>,
    last_packet_time: Option<u64>,
    total_distance_km: f64,
    last_phase: Option<String>,
}

impl TelemetryStorage {
    // Flight detection thresholds
    const ALTITUDE_THRESHOLD: f32 = 5.0;  // Only detect flights when airborne
    const SPEED_THRESHOLD: f32 = 2.0;     // Minimum airspeed
    const GPS_STABLE_THRESHOLD: f64 = 0.0001;
    const LANDING_CONFIRM_MS: u64 = 5000;
    const TIMEOUT_MS: u64 = 60000;
    
    pub fn new(path: &str) -> Result<Self> {
        Ok(Self {
            store: Store::with_path(path)?,
            current_flight_id: None,
            flight_state: FlightState::OnGround,
            landing_check_start: None,
            last_position: None,
            last_packet_time: None,
            total_distance_km: 0.0,
            last_phase: None,
        })
    }
    
    pub fn save_packet(&mut self, packet: &TelemetryPacket) -> Result<()> {
        // Check for timeout (catastrophic stop)
        if let Some(last_time) = self.last_packet_time {
            let gap = packet.timestamp.saturating_sub(last_time);
            if gap > Self::TIMEOUT_MS && self.current_flight_id.is_some() {
                println!("⚠️  Stream timeout detected ({:.1}s gap) - ending flight", 
                         gap as f64 / 1000.0);
                self.end_current_flight_catastrophic()?;
            }
        }
        
        let new_state = self.detect_flight_state(packet);
        
        // Calculate distance if in flight
        if self.current_flight_id.is_some() {
            if let Some((last_lat, last_lon)) = self.last_position {
                let distance = Self::haversine_distance(
                    last_lat, last_lon,
                    packet.latitude, packet.longitude
                );
                self.total_distance_km += distance;
            }
        }
        
        // State transitions
        match (self.flight_state, new_state) {
            (FlightState::OnGround, FlightState::InFlight) => {
                self.start_new_flight(packet)?;
            }
            (FlightState::Landing, FlightState::OnGround) => {
                self.end_current_flight(packet, true)?;
            }
            _ => {}
        }
        
        self.flight_state = new_state;
        
        // Store packet if in flight
        if let Some(flight_id) = &self.current_flight_id {
            let key = format!("telem:{}:{}", flight_id, packet.timestamp);
            let value = serde_json::to_string(packet)?;
            self.store.put(Key::String(key), Value::String(value));
            
            self.update_flight_metadata(packet)?;
        }
        
        self.last_position = Some((packet.latitude, packet.longitude));
        self.last_packet_time = Some(packet.timestamp);
        Ok(())
    }
    
    fn detect_flight_state(&mut self, packet: &TelemetryPacket) -> FlightState {
        let is_on_ground = 
            packet.altitude_gps <= Self::ALTITUDE_THRESHOLD &&
            packet.ground_speed <= Self::SPEED_THRESHOLD &&
            self.is_gps_stable(packet);
        
        match self.flight_state {
            FlightState::OnGround => {
                if !is_on_ground {
                    FlightState::InFlight
                } else {
                    FlightState::OnGround
                }
            }
            
            FlightState::InFlight => {
                if is_on_ground {
                    self.landing_check_start = Some(packet.timestamp);
                    FlightState::Landing
                } else {
                    FlightState::InFlight
                }
            }
            
            FlightState::Landing => {
                if !is_on_ground {
                    self.landing_check_start = None;
                    FlightState::InFlight
                } else {
                    let stable_duration = packet.timestamp.saturating_sub(
                        self.landing_check_start.unwrap_or(packet.timestamp)
                    );
                    
                    if stable_duration >= Self::LANDING_CONFIRM_MS {
                        FlightState::OnGround
                    } else {
                        FlightState::Landing
                    }
                }
            }
        }
    }
    
    fn is_gps_stable(&self, packet: &TelemetryPacket) -> bool {
        if let Some((last_lat, last_lon)) = self.last_position {
            let lat_diff = (packet.latitude - last_lat).abs();
            let lon_diff = (packet.longitude - last_lon).abs();
            
            lat_diff < Self::GPS_STABLE_THRESHOLD && 
            lon_diff < Self::GPS_STABLE_THRESHOLD
        } else {
            true
        }
    }
    
    fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const R: f64 = 6371.0;
        
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();
        
        let a = (delta_lat / 2.0).sin().powi(2) +
                lat1_rad.cos() * lat2_rad.cos() *
                (delta_lon / 2.0).sin().powi(2);
        
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        R * c
    }
    
    fn start_new_flight(&mut self, packet: &TelemetryPacket) -> Result<()> {
        let flight_id = format!("flight_{:03}", self.get_next_flight_number());
        
        println!("[Flight] {} started at altitude {:.1}m", 
                 flight_id, packet.altitude_gps);
        
        let metadata = FlightMetadata {
            flight_id: flight_id.clone(),
            start_time: packet.timestamp,
            end_time: packet.timestamp,
            duration_secs: 0,
            packet_count: 0,
            distance_km: 0.0,
            first_lat: packet.latitude,
            first_lon: packet.longitude,
            last_lat: packet.latitude,
            last_lon: packet.longitude,
            max_altitude: packet.altitude_gps,
            min_battery: packet.battery_voltage,
            ended_normally: true,
            current_status: packet.get_flight_phase().to_string(),
        };
        
        let key = format!("flight:{}", flight_id);
        let value = serde_json::to_string(&metadata)?;
        self.store.put(Key::String(key), Value::String(value));
        
        self.current_flight_id = Some(flight_id);
        self.total_distance_km = 0.0;
        Ok(())
    }
    
    fn update_flight_metadata(&mut self, packet: &TelemetryPacket) -> Result<()> {
        if let Some(flight_id) = &self.current_flight_id {
            let key = format!("flight:{}", flight_id);
            if let Ok(BorrowedEntry::Text(json)) = self.store.get(&Key::String(key.clone())) {
                if let Ok(mut metadata) = serde_json::from_str::<FlightMetadata>(json) {
                    metadata.end_time = packet.timestamp;
                    metadata.duration_secs = (packet.timestamp - metadata.start_time) / 1000;
                    metadata.packet_count += 1;
                    metadata.distance_km = self.total_distance_km;
                    metadata.last_lat = packet.latitude;
                    metadata.last_lon = packet.longitude;
                    metadata.max_altitude = metadata.max_altitude.max(packet.altitude_gps);
                    metadata.min_battery = metadata.min_battery.min(packet.battery_voltage);
                    
                    let current_phase = packet.get_flight_phase().to_string();
                    metadata.current_status = current_phase.clone();
                    
                    // Log phase transitions
                    if self.last_phase.as_ref() != Some(&current_phase) {
                        println!("[Flight] {} -> {}", flight_id, current_phase);
                        self.last_phase = Some(current_phase);
                    }
                    
                    let value = serde_json::to_string(&metadata)?;
                    self.store.put(Key::String(key), Value::String(value));
                }
            }
        }
        Ok(())
    }
    
    fn end_current_flight(&mut self, packet: &TelemetryPacket, normal: bool) -> Result<()> {
        if let Some(flight_id) = &self.current_flight_id {
            println!("[Flight] {} ended {}", flight_id, 
                     if normal { "normally" } else { "abnormally" });
            
            // Update metadata one last time and set status to "Landed" if normal
            if normal {
                let key = format!("flight:{}", flight_id);
                if let Ok(BorrowedEntry::Text(json)) = self.store.get(&Key::String(key.clone())) {
                    if let Ok(mut metadata) = serde_json::from_str::<FlightMetadata>(json) {
                        metadata.current_status = "Landed".to_string();
                        let value = serde_json::to_string(&metadata)?;
                        self.store.put(Key::String(key), Value::String(value));
                    }
                }
            } else {
                self.update_flight_metadata(packet)?;
            }
            
            self.current_flight_id = None;
            self.landing_check_start = None;
            self.total_distance_km = 0.0;
            self.last_phase = None;
        }
        Ok(())
    }
    
    fn end_current_flight_catastrophic(&mut self) -> Result<()> {
        if let Some(flight_id) = &self.current_flight_id {
            println!("[Flight] {} ended catastrophically (stream lost)", flight_id);
            
            let key = format!("flight:{}", flight_id);
            if let Ok(BorrowedEntry::Text(json)) = self.store.get(&Key::String(key.clone())) {
                if let Ok(mut metadata) = serde_json::from_str::<FlightMetadata>(json) {
                    metadata.ended_normally = false;
                    metadata.distance_km = self.total_distance_km;
                    let value = serde_json::to_string(&metadata)?;
                    self.store.put(Key::String(key), Value::String(value));
                }
            }
            
            self.current_flight_id = None;
            self.landing_check_start = None;
            self.total_distance_km = 0.0;
            self.flight_state = FlightState::OnGround;
            self.last_phase = None;
        }
        Ok(())
    }
    
    fn get_next_flight_number(&self) -> usize {
        let mut max_num = 0;
        for key in self.store.keys() {
            if let Key::String(k) = key {
                if k.starts_with("flight:flight_") {
                    if let Some(num_str) = k.strip_prefix("flight:flight_") {
                        if let Ok(num) = num_str.parse::<usize>() {
                            max_num = max_num.max(num);
                        }
                    }
                }
            }
        }
        max_num + 1
    }
    
    pub fn list_flights(&self) -> Vec<FlightMetadata> {
        let mut flights: Vec<FlightMetadata> = Vec::new();
        
        for key in self.store.keys() {
            if let Key::String(k) = key {
                if k.starts_with("flight:") {
                    if let Ok(BorrowedEntry::Text(json)) = self.store.get(key) {
                        if let Ok(flight) = serde_json::from_str(json) {
                            flights.push(flight);
                        }
                    }
                }
            }
        }
        
        flights.sort_by_key(|f| f.start_time);
        flights
    }
    
    pub fn get_flight(&self, flight_id: &str) -> Option<FlightMetadata> {
        let key = format!("flight:{}", flight_id);
        if let Ok(BorrowedEntry::Text(json)) = self.store.get(&Key::String(key)) {
            serde_json::from_str(json).ok()
        } else {
            None
        }
    }
    
    pub fn get_flight_data(&self, flight_id: &str) -> Vec<TelemetryPacket> {
        let prefix = format!("telem:{}:", flight_id);
        let mut packets: Vec<TelemetryPacket> = Vec::new();
        
        for key in self.store.keys() {
            if let Key::String(k) = key {
                if k.starts_with(&prefix) {
                    if let Ok(BorrowedEntry::Text(json)) = self.store.get(key) {
                        if let Ok(packet) = serde_json::from_str(json) {
                            packets.push(packet);
                        }
                    }
                }
            }
        }
        
        packets.sort_by_key(|p| p.timestamp);
        packets
    }
    
    pub fn delete_flight(&mut self, flight_id: &str) -> Result<()> {
        let meta_key = format!("flight:{}", flight_id);
        self.store.delete(&Key::String(meta_key))?;
        
        let prefix = format!("telem:{}:", flight_id);
        let keys_to_delete: Vec<Key> = self.store.keys()
            .filter(|k| {
                if let Key::String(s) = k {
                    s.starts_with(&prefix)
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        
        for key in keys_to_delete {
            self.store.delete(&key)?;
        }
        
        self.store.compact()?;
        Ok(())
    }
    
    pub fn get_current_flight_id(&self) -> Option<String> {
        self.current_flight_id.clone()
    }
}

use serde::{Deserialize, Serialize};

/// Telemetry packet matching ESP32 LoRa hardware format
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct TelemetryPacket {
    // GPS
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_gps: f32,
    pub ground_speed: f32,
    pub heading: f32,
    pub num_satellites: u8,
    pub gps_fix_type: u8,
    
    // Barometer
    pub altitude_baro: f32,
    pub vertical_speed: f32,
    pub temperature: f32,
    
    // IMU
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    
    // Power
    pub battery_voltage: f32,
    pub battery_current: f32,
    pub battery_power: f32,
    pub battery_mah_used: f32,
    
    // Communication
    pub rssi: i16,
    pub snr: f32,
    
    // System
    pub timestamp: u64,
    pub packet_sequence: u32,
    pub system_status: u8,
}

impl TelemetryPacket {
    /// Deserialize from binary format (little-endian)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 113 {
            return Err("Insufficient bytes for telemetry packet");
        }
        
        let mut offset = 0;
        
        macro_rules! read_f64 {
            () => {{
                let val = f64::from_le_bytes(bytes[offset..offset+8].try_into().unwrap());
                offset += 8;
                val
            }};
        }
        
        macro_rules! read_f32 {
            () => {{
                let val = f32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap());
                offset += 4;
                val
            }};
        }
        
        macro_rules! read_u8 {
            () => {{
                let val = bytes[offset];
                offset += 1;
                val
            }};
        }
        
        macro_rules! read_i16 {
            () => {{
                let val = i16::from_le_bytes(bytes[offset..offset+2].try_into().unwrap());
                offset += 2;
                val
            }};
        }
        
        macro_rules! read_u64 {
            () => {{
                let val = u64::from_le_bytes(bytes[offset..offset+8].try_into().unwrap());
                offset += 8;
                val
            }};
        }
        
        macro_rules! read_u32 {
            () => {{
                let val = u32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap());
                offset += 4;
                val
            }};
        }
        
        Ok(TelemetryPacket {
            latitude: read_f64!(),
            longitude: read_f64!(),
            altitude_gps: read_f32!(),
            ground_speed: read_f32!(),
            heading: read_f32!(),
            num_satellites: read_u8!(),
            gps_fix_type: read_u8!(),
            
            altitude_baro: read_f32!(),
            vertical_speed: read_f32!(),
            temperature: read_f32!(),
            
            roll: read_f32!(),
            pitch: read_f32!(),
            yaw: read_f32!(),
            gyro_x: read_f32!(),
            gyro_y: read_f32!(),
            gyro_z: read_f32!(),
            accel_x: read_f32!(),
            accel_y: read_f32!(),
            accel_z: read_f32!(),
            
            battery_voltage: read_f32!(),
            battery_current: read_f32!(),
            battery_power: read_f32!(),
            battery_mah_used: read_f32!(),
            
            rssi: read_i16!(),
            snr: read_f32!(),
            
            timestamp: read_u64!(),
            packet_sequence: read_u32!(),
            system_status: read_u8!(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightMetadata {
    pub flight_id: String,
    pub start_time: u64,
    pub end_time: u64,
    pub duration_secs: u64,
    pub packet_count: usize,
    pub distance_km: f64,
    pub first_lat: f64,
    pub first_lon: f64,
    pub last_lat: f64,
    pub last_lon: f64,
    pub max_altitude: f32,
    pub min_battery: f32,
    pub ended_normally: bool,
    pub current_status: String,
}

impl TelemetryPacket {
    /// Determine flight phase from telemetry data
    pub fn get_flight_phase(&self) -> &'static str {
        const GROUND_ALTITUDE: f32 = 2.0;      // Consider on ground if below 2m
        const CRUISE_ALTITUDE: f32 = 140.0;     // Cruise altitude threshold
        const TAKEOFF_SPEED: f32 = 3.0;         // Speed threshold for takeoff roll
        const CLIMB_RATE: f32 = 0.8;            // Minimum climb rate for ascent
        const DESCENT_RATE: f32 = -0.8;         // Descent rate threshold
        
        let is_on_ground = self.altitude_baro < GROUND_ALTITUDE;
        let is_moving = self.ground_speed >= TAKEOFF_SPEED;
        let is_climbing = self.vertical_speed > CLIMB_RATE;
        let is_descending = self.vertical_speed < DESCENT_RATE;
        
        // On Ground: stationary or slow movement
        if is_on_ground && !is_moving {
            return "On Ground";
        }
        
        // Taking Off: on ground, accelerating
        if is_on_ground && is_moving {
            return "Taking Off";
        }
        
        // Landing: low altitude, descending
        if self.altitude_baro < 20.0 && is_descending {
            return "Landing";
        }
        
        // Ascent: airborne and climbing
        if !is_on_ground && is_climbing && self.altitude_baro < CRUISE_ALTITUDE {
            return "Ascent";
        }
        
        // Cruise: at or near cruise altitude, level flight
        if self.altitude_baro >= CRUISE_ALTITUDE && !is_climbing && !is_descending {
            return "Cruise";
        }
        
        // Descent: descending from altitude
        if is_descending && self.altitude_baro > 20.0 {
            return "Descent";
        }
        
        // Still climbing to cruise
        if is_climbing {
            return "Ascent";
        }
        
        // Default for airborne but not clearly in a phase
        if !is_on_ground {
            return "Cruise";
        }
        
        "On Ground"
    }
}


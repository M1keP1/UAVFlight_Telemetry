use serde::{Deserialize, Serialize};

/// Telemetry packet matching ESP32 LoRa hardware format
/// 
/// Architecture:
/// ESP32 (binary) → KV Store Backend → WebSocket (JSON) → Frontend
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
    /// Serialize to binary format (little-endian) matching ESP32 output
    /// This is what the ESP32 LoRa will transmit
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(120);
        
        // GPS (8 + 8 + 4 + 4 + 4 + 1 + 1 = 30 bytes)
        bytes.extend_from_slice(&self.latitude.to_le_bytes());
        bytes.extend_from_slice(&self.longitude.to_le_bytes());
        bytes.extend_from_slice(&self.altitude_gps.to_le_bytes());
        bytes.extend_from_slice(&self.ground_speed.to_le_bytes());
        bytes.extend_from_slice(&self.heading.to_le_bytes());
        bytes.push(self.num_satellites);
        bytes.push(self.gps_fix_type);
        
        // Barometer (4 + 4 + 4 = 12 bytes)
        bytes.extend_from_slice(&self.altitude_baro.to_le_bytes());
        bytes.extend_from_slice(&self.vertical_speed.to_le_bytes());
        bytes.extend_from_slice(&self.temperature.to_le_bytes());
        
        // IMU (4 * 9 = 36 bytes)
        bytes.extend_from_slice(&self.roll.to_le_bytes());
        bytes.extend_from_slice(&self.pitch.to_le_bytes());
        bytes.extend_from_slice(&self.yaw.to_le_bytes());
        bytes.extend_from_slice(&self.gyro_x.to_le_bytes());
        bytes.extend_from_slice(&self.gyro_y.to_le_bytes());
        bytes.extend_from_slice(&self.gyro_z.to_le_bytes());
        bytes.extend_from_slice(&self.accel_x.to_le_bytes());
        bytes.extend_from_slice(&self.accel_y.to_le_bytes());
        bytes.extend_from_slice(&self.accel_z.to_le_bytes());
        
        // Power (4 * 4 = 16 bytes)
        bytes.extend_from_slice(&self.battery_voltage.to_le_bytes());
        bytes.extend_from_slice(&self.battery_current.to_le_bytes());
        bytes.extend_from_slice(&self.battery_power.to_le_bytes());
        bytes.extend_from_slice(&self.battery_mah_used.to_le_bytes());
        
        // Communication (2 + 4 = 6 bytes)
        bytes.extend_from_slice(&self.rssi.to_le_bytes());
        bytes.extend_from_slice(&self.snr.to_le_bytes());
        
        // System (8 + 4 + 1 = 13 bytes)
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.packet_sequence.to_le_bytes());
        bytes.push(self.system_status);
        
        bytes
    }
    
    /// Deserialize from binary format (little-endian)
    /// Used for testing and receiving from ESP32
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 113 {
            return Err("Insufficient bytes for telemetry packet");
        }
        
        let mut offset = 0;
        
        // Helper macro to read values
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
            // GPS
            latitude: read_f64!(),
            longitude: read_f64!(),
            altitude_gps: read_f32!(),
            ground_speed: read_f32!(),
            heading: read_f32!(),
            num_satellites: read_u8!(),
            gps_fix_type: read_u8!(),
            
            // Barometer
            altitude_baro: read_f32!(),
            vertical_speed: read_f32!(),
            temperature: read_f32!(),
            
            // IMU
            roll: read_f32!(),
            pitch: read_f32!(),
            yaw: read_f32!(),
            gyro_x: read_f32!(),
            gyro_y: read_f32!(),
            gyro_z: read_f32!(),
            accel_x: read_f32!(),
            accel_y: read_f32!(),
            accel_z: read_f32!(),
            
            // Power
            battery_voltage: read_f32!(),
            battery_current: read_f32!(),
            battery_power: read_f32!(),
            battery_mah_used: read_f32!(),
            
            // Communication
            rssi: read_i16!(),
            snr: read_f32!(),
            
            // System
            timestamp: read_u64!(),
            packet_sequence: read_u32!(),
            system_status: read_u8!(),
        })
    }
}

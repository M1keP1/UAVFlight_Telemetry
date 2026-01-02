use crate::telemetry::TelemetryPacket;
use crate::trajectory::{get_flight_state_at_time, FlightPhase};
use rand::Rng;
use std::time::Instant;

pub struct Generator {
    start_time: Instant,
    packet_seq: u32,
    battery_start: f32,
    prev_heading: f32,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            packet_seq: 0,
            battery_start: 16.8,
            prev_heading: 90.0,
        }
    }
    
    pub fn generate_packet(&mut self) -> TelemetryPacket {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let mut rng = rand::thread_rng();
        
        // Get flight state from trajectory
        let state = get_flight_state_at_time(elapsed);
        
        // Add GPS noise
        let lat = state.lat + rng.gen_range(-0.000005..0.000005);
        let lon = state.lon + rng.gen_range(-0.000005..0.000005);
        
        // Calculate heading change for roll
        let heading_change = state.heading - self.prev_heading;
        let heading_change = if heading_change > 180.0 {
            heading_change - 360.0
        } else if heading_change < -180.0 {
            heading_change + 360.0
        } else {
            heading_change
        };
        
        // Calculate roll angle (bank angle during turns)
        // Standard rate turn: ~15 degrees of bank per 3 deg/sec turn rate
        let turn_rate = heading_change * 2.0; // We sample at 2 Hz
        let roll = (turn_rate * 5.0).clamp(-30.0, 30.0); // Max 30 degrees bank
        
        // Calculate pitch angle based on vertical speed and ground speed
        let pitch = if state.ground_speed > 1.0 {
            (state.vertical_speed / state.ground_speed).atan().to_degrees().clamp(-15.0, 15.0)
        } else {
            0.0
        };
        
        self.prev_heading = state.heading;
        
        // Battery drain varies by phase
        let battery_drain_rate = match state.phase {
            FlightPhase::Rest => 0.0,
            FlightPhase::Taxi => 0.3,
            FlightPhase::Takeoff => 1.5,  // High power during climb
            FlightPhase::Cruise => 0.8,
            FlightPhase::Landing => 0.6,
        };
        let battery_voltage = self.battery_start - (elapsed / 600.0) * battery_drain_rate;
        
        // Current varies by phase
        let battery_current = match state.phase {
            FlightPhase::Rest => 0.5 + rng.gen_range(-0.1..0.1),
            FlightPhase::Taxi => 5.0 + rng.gen_range(-1.0..1.0),
            FlightPhase::Takeoff => 15.0 + rng.gen_range(-2.0..2.0),
            FlightPhase::Cruise => 10.0 + rng.gen_range(-1.5..1.5),
            FlightPhase::Landing => 8.0 + rng.gen_range(-1.0..1.0),
        };
        
        // RSSI decreases with altitude
        let rssi = -50 - ((state.alt / 10.0) as i16);
        
        // Gyro rates (angular velocities in deg/s)
        let gyro_x = roll * 0.5 + rng.gen_range(-2.0..2.0); // Roll rate
        let gyro_y = pitch * 0.5 + rng.gen_range(-2.0..2.0); // Pitch rate
        let gyro_z = turn_rate + rng.gen_range(-1.0..1.0); // Yaw rate
        
        // Accelerations
        let accel_x = roll.to_radians().sin() * 9.81 + rng.gen_range(-0.3..0.3); // Lateral
        let accel_y = pitch.to_radians().sin() * 9.81 + rng.gen_range(-0.3..0.3); // Longitudinal
        let accel_z = roll.to_radians().cos() * pitch.to_radians().cos() * 9.81 + rng.gen_range(-0.2..0.2); // Vertical
        
        let packet = TelemetryPacket {
            // GPS
            latitude: lat,
            longitude: lon,
            altitude_gps: state.alt + rng.gen_range(-1.0..1.0),
            ground_speed: state.ground_speed + rng.gen_range(-0.5..0.5),
            heading: state.heading,
            num_satellites: if state.phase == FlightPhase::Rest { 8 } else { 10 },
            gps_fix_type: 3,
            
            // Barometer
            altitude_baro: state.alt + rng.gen_range(-0.3..0.3),
            vertical_speed: state.vertical_speed + rng.gen_range(-0.2..0.2),
            temperature: 20.0 - (state.alt / 100.0) + rng.gen_range(-0.5..0.5), // Temperature decreases with altitude
            
            // IMU - Now realistic!
            roll,
            pitch,
            yaw: state.heading,
            gyro_x,
            gyro_y,
            gyro_z,
            accel_x,
            accel_y,
            accel_z,
            
            // Power
            battery_voltage,
            battery_current,
            battery_power: battery_voltage * battery_current,
            battery_mah_used: battery_current * elapsed / 3.6,
            
            // Communication
            rssi,
            snr: 8.0 + rng.gen_range(-2.0..2.0),
            
            // System
            timestamp: self.start_time.elapsed().as_millis() as u64,
            packet_sequence: self.packet_seq,
            system_status: match state.phase {
                FlightPhase::Rest => 0x01,      // Idle
                FlightPhase::Taxi => 0x02,      // Taxiing
                FlightPhase::Takeoff => 0x04,   // Takeoff
                FlightPhase::Cruise => 0x08,    // Cruise
                FlightPhase::Landing => 0x10,   // Landing
            },
        };
        
        self.packet_seq += 1;
        packet
    }
}

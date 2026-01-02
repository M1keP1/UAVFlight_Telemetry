use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlightPhase {
    Rest,
    Taxi,
    Takeoff,
    Cruise,
    Landing,
}

#[derive(Debug, Clone, Copy)]
pub struct FlightState {
    pub lat: f64,
    pub lon: f64,
    pub alt: f32,
    pub heading: f32,
    pub vertical_speed: f32,
    pub ground_speed: f32,
    pub phase: FlightPhase,
}

// Airport location (Darmstadt area)
const AIRPORT_LAT: f64 = 49.8728;
const AIRPORT_LON: f64 = 8.6512;
const CRUISE_ALTITUDE: f32 = 150.0;

// Flight timing (in seconds)
const REST_DURATION: f32 = 45.0;
const TAXI_DURATION: f32 = 20.0;
const TAKEOFF_DURATION: f32 = 25.0;
const CRUISE_DURATION: f32 = 120.0;
const LANDING_DURATION: f32 = 30.0;

const TOTAL_FLIGHT_CYCLE: f32 = REST_DURATION + TAXI_DURATION + TAKEOFF_DURATION + CRUISE_DURATION + LANDING_DURATION;

/// Calculate heading between two GPS coordinates (in degrees)
fn calculate_heading(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f32 {
    let lat1_rad = lat1 * PI / 180.0;
    let lat2_rad = lat2 * PI / 180.0;
    let dlon = (lon2 - lon1) * PI / 180.0;
    
    let y = dlon.sin() * lat2_rad.cos();
    let x = lat1_rad.cos() * lat2_rad.sin() - lat1_rad.sin() * lat2_rad.cos() * dlon.cos();
    
    let heading = y.atan2(x) * 180.0 / PI;
    ((heading + 360.0) % 360.0) as f32
}

/// Calculate distance between two GPS coordinates (in meters)
fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f32 {
    let r = 6371000.0; // Earth radius in meters
    let lat1_rad = lat1 * PI / 180.0;
    let lat2_rad = lat2 * PI / 180.0;
    let dlat = (lat2 - lat1) * PI / 180.0;
    let dlon = (lon2 - lon1) * PI / 180.0;
    
    let a = (dlat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    (r * c) as f32
}

/// Move from one point to another at given speed and time
fn interpolate_position(lat1: f64, lon1: f64, lat2: f64, lon2: f64, progress: f32) -> (f64, f64) {
    let lat = lat1 + (lat2 - lat1) * progress as f64;
    let lon = lon1 + (lon2 - lon1) * progress as f64;
    (lat, lon)
}

pub fn get_flight_state_at_time(t: f32) -> FlightState {
    // Loop the flight cycle
    let t = t % TOTAL_FLIGHT_CYCLE;
    
    let mut time_offset = 0.0;
    
    // REST PHASE
    if t < time_offset + REST_DURATION {
        return FlightState {
            lat: AIRPORT_LAT,
            lon: AIRPORT_LON,
            alt: 0.0,
            heading: 90.0, // Facing east
            vertical_speed: 0.0,
            ground_speed: 0.0,
            phase: FlightPhase::Rest,
        };
    }
    time_offset += REST_DURATION;
    
    // TAXI PHASE
    if t < time_offset + TAXI_DURATION {
        let phase_time = t - time_offset;
        let progress = phase_time / TAXI_DURATION;
        
        // Taxi to runway (move east)
        let taxi_distance = 0.002; // ~200m in degrees
        let end_lat = AIRPORT_LAT;
        let end_lon = AIRPORT_LON + taxi_distance;
        
        let (lat, lon) = interpolate_position(AIRPORT_LAT, AIRPORT_LON, end_lat, end_lon, progress);
        let heading = calculate_heading(AIRPORT_LAT, AIRPORT_LON, end_lat, end_lon);
        
        return FlightState {
            lat,
            lon,
            alt: 0.0,
            heading,
            vertical_speed: 0.0,
            ground_speed: 5.0 + progress * 5.0, // Accelerate from 5 to 10 m/s
            phase: FlightPhase::Taxi,
        };
    }
    time_offset += TAXI_DURATION;
    
    // TAKEOFF PHASE
    if t < time_offset + TAKEOFF_DURATION {
        let phase_time = t - time_offset;
        let progress = phase_time / TAKEOFF_DURATION;
        
        let start_lat = AIRPORT_LAT;
        let start_lon = AIRPORT_LON + 0.002;
        let end_lat = AIRPORT_LAT + 0.003;
        let end_lon = AIRPORT_LON + 0.005;
        
        let (lat, lon) = interpolate_position(start_lat, start_lon, end_lat, end_lon, progress);
        let heading = calculate_heading(start_lat, start_lon, end_lat, end_lon);
        
        // Smooth altitude climb
        let alt = CRUISE_ALTITUDE * (progress * progress); // Quadratic for smooth acceleration
        let vertical_speed = (CRUISE_ALTITUDE / TAKEOFF_DURATION) * (2.0 * progress); // Derivative
        
        return FlightState {
            lat,
            lon,
            alt,
            heading,
            vertical_speed,
            ground_speed: 10.0 + progress * 15.0, // Accelerate from 10 to 25 m/s
            phase: FlightPhase::Takeoff,
        };
    }
    time_offset += TAKEOFF_DURATION;
    
    // CRUISE PHASE - Fly a rectangular pattern
    if t < time_offset + CRUISE_DURATION {
        let phase_time = t - time_offset;
        let leg_duration = CRUISE_DURATION / 4.0;
        let leg = (phase_time / leg_duration).floor() as i32;
        let leg_progress = (phase_time % leg_duration) / leg_duration;
        
        let waypoints = [
            (AIRPORT_LAT + 0.003, AIRPORT_LON + 0.005), // Start (after takeoff)
            (AIRPORT_LAT + 0.008, AIRPORT_LON + 0.005), // North
            (AIRPORT_LAT + 0.008, AIRPORT_LON + 0.000), // West
            (AIRPORT_LAT + 0.003, AIRPORT_LON + 0.000), // South
            (AIRPORT_LAT + 0.003, AIRPORT_LON + 0.005), // Back to start
        ];
        
        let start_wp = waypoints[leg as usize];
        let end_wp = waypoints[(leg + 1) as usize];
        
        let (lat, lon) = interpolate_position(start_wp.0, start_wp.1, end_wp.0, end_wp.1, leg_progress);
        let heading = calculate_heading(start_wp.0, start_wp.1, end_wp.0, end_wp.1);
        
        return FlightState {
            lat,
            lon,
            alt: CRUISE_ALTITUDE,
            heading,
            vertical_speed: 0.0,
            ground_speed: 25.0,
            phase: FlightPhase::Cruise,
        };
    }
    time_offset += CRUISE_DURATION;
    
    // LANDING PHASE
    if t < time_offset + LANDING_DURATION {
        let phase_time = t - time_offset;
        let progress = phase_time / LANDING_DURATION;
        
        let start_lat = AIRPORT_LAT + 0.003;
        let start_lon = AIRPORT_LON + 0.005;
        
        let (lat, lon) = interpolate_position(start_lat, start_lon, AIRPORT_LAT, AIRPORT_LON, progress);
        let heading = calculate_heading(start_lat, start_lon, AIRPORT_LAT, AIRPORT_LON);
        
        // Smooth descent
        let alt = CRUISE_ALTITUDE * (1.0 - progress * progress);
        let vertical_speed = -(CRUISE_ALTITUDE / LANDING_DURATION) * (2.0 * progress);
        
        return FlightState {
            lat,
            lon,
            alt,
            heading,
            vertical_speed,
            ground_speed: 25.0 - progress * 15.0, // Decelerate from 25 to 10 m/s
            phase: FlightPhase::Landing,
        };
    }
    
    // Fallback (should not reach here)
    FlightState {
        lat: AIRPORT_LAT,
        lon: AIRPORT_LON,
        alt: 0.0,
        heading: 90.0,
        vertical_speed: 0.0,
        ground_speed: 0.0,
        phase: FlightPhase::Rest,
    }
}

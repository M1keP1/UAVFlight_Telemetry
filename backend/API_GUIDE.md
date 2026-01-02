# Telemetry System API Documentation

## System Overview

Real-time telemetry data pipeline with flight tracking, phase detection, and persistent storage.

**Architecture:**
```
Simulator (Port 8080) → Binary WebSocket → KV Server (Port 9090) → REST API / WebSocket → Frontend
```

---

## Simulator Endpoints

### Base URL
```
http://localhost:8080
```

### WebSocket - Binary Telemetry
**Endpoint:** `ws://localhost:8080/ws/binary`

**Purpose:** Simulates ESP32 LoRa output - binary telemetry packets

**Connection:**
```javascript
const ws = new WebSocket('ws://localhost:8080/ws/binary');

ws.onopen = () => {
    console.log('Connected to simulator');
};

ws.onmessage = (event) => {
    // event.data is ArrayBuffer (113 bytes)
    const packet = parseBinaryPacket(event.data);
};
```

**Binary Packet Format (113 bytes, little-endian):**
```
Offset | Size | Field              | Type
-------|------|--------------------|---------
0      | 8    | latitude           | f64
8      | 8    | longitude          | f64
16     | 4    | altitude_gps       | f32
20     | 4    | ground_speed       | f32
24     | 4    | heading            | f32
28     | 1    | num_satellites     | u8
29     | 1    | gps_fix_type       | u8
30     | 4    | altitude_baro      | f32
34     | 4    | vertical_speed     | f32
38     | 4    | temperature        | f32
42     | 4    | roll               | f32
46     | 4    | pitch              | f32
50     | 4    | yaw                | f32
54     | 4    | gyro_x             | f32
58     | 4    | gyro_y             | f32
62     | 4    | gyro_z             | f32
66     | 4    | accel_x            | f32
70     | 4    | accel_y            | f32
74     | 4    | accel_z            | f32
78     | 4    | battery_voltage    | f32
82     | 4    | battery_current    | f32
86     | 4    | battery_power      | f32
90     | 4    | battery_mah_used   | f32
94     | 2    | rssi               | i16
96     | 4    | snr                | f32
100    | 8    | timestamp          | u64
108    | 4    | packet_sequence    | u32
112    | 1    | system_status      | u8
```

---

## KV Server Endpoints

### Base URL
```
http://localhost:9090
```

### REST API

#### 1. List All Flights
**Endpoint:** `GET /api/flights`

**Response:**
```json
[
  {
    "flight_id": "flight_001",
    "start_time": 65002,
    "end_time": 245504,
    "duration_secs": 180,
    "packet_count": 361,
    "distance_km": 2.867,
    "first_lat": 49.8728,
    "first_lon": 8.6512,
    "last_lat": 49.8728,
    "last_lon": 8.6512,
    "max_altitude": 150.99,
    "min_battery": 16.52,
    "ended_normally": true,
    "current_status": "Cruise"
  }
]
```

**Example:**
```bash
curl http://localhost:9090/api/flights
```

```javascript
const flights = await fetch('http://localhost:9090/api/flights')
    .then(r => r.json());
```

---

#### 2. Get Flight Details
**Endpoint:** `GET /api/flights/:id`

**Parameters:**
- `id` - Flight ID (e.g., "flight_001")

**Response:**
```json
{
  "flight_id": "flight_001",
  "start_time": 65002,
  "end_time": 245504,
  "duration_secs": 180,
  "packet_count": 361,
  "distance_km": 2.867,
  "first_lat": 49.8728,
  "first_lon": 8.6512,
  "last_lat": 49.8728,
  "last_lon": 8.6512,
  "max_altitude": 150.99,
  "min_battery": 16.52,
  "ended_normally": true,
  "current_status": "Cruise"
}
```

**Example:**
```bash
curl http://localhost:9090/api/flights/flight_001
```

```javascript
const flight = await fetch('http://localhost:9090/api/flights/flight_001')
    .then(r => r.json());
```

**Error Response (404):**
```
Flight not found
```

---

#### 3. Get Flight Telemetry Data
**Endpoint:** `GET /api/flights/:id/data`

**Parameters:**
- `id` - Flight ID (e.g., "flight_001")

**Response:** Array of telemetry packets with flight phase
```json
[
  {
    "latitude": 49.8728,
    "longitude": 8.6512,
    "altitude_gps": 0.8,
    "ground_speed": 5.2,
    "heading": 90.0,
    "num_satellites": 10,
    "gps_fix_type": 3,
    "altitude_baro": 0.5,
    "vertical_speed": 0.1,
    "temperature": 20.0,
    "roll": 0.0,
    "pitch": 0.0,
    "yaw": 90.0,
    "gyro_x": 0.5,
    "gyro_y": 0.2,
    "gyro_z": 0.1,
    "accel_x": 0.1,
    "accel_y": 0.0,
    "accel_z": 9.8,
    "battery_voltage": 16.8,
    "battery_current": 10.5,
    "battery_power": 176.4,
    "battery_mah_used": 0.5,
    "rssi": -50,
    "snr": 8.0,
    "timestamp": 65002,
    "packet_sequence": 130,
    "system_status": 2,
    "flight_phase": "Taking Off"
  }
]
```

**Example:**
```bash
curl http://localhost:9090/api/flights/flight_001/data
```

```javascript
const telemetry = await fetch('http://localhost:9090/api/flights/flight_001/data')
    .then(r => r.json());

// Filter by phase
const takeoffData = telemetry.filter(p => p.flight_phase === 'Taking Off');
const cruiseData = telemetry.filter(p => p.flight_phase === 'Cruise');
```

---

#### 4. Delete Flight
**Endpoint:** `DELETE /api/flights/:id`

**Parameters:**
- `id` - Flight ID (e.g., "flight_001")

**Response:** `204 No Content` (success) or `500 Internal Server Error`

**Example:**
```bash
curl -X DELETE http://localhost:9090/api/flights/flight_001
```

```javascript
await fetch('http://localhost:9090/api/flights/flight_001', {
    method: 'DELETE'
});
```

---

#### 5. Health Check
**Endpoint:** `GET /health`

**Response:** `"OK"`

**Example:**
```bash
curl http://localhost:9090/health
```

---

### WebSocket - Real-Time Telemetry Stream

**Endpoint:** `ws://localhost:9090/ws/stream`

**Purpose:** Real-time JSON telemetry stream for live monitoring

**Connection:**
```javascript
const ws = new WebSocket('ws://localhost:9090/ws/stream');

ws.onopen = () => {
    console.log('Connected to telemetry stream');
};

ws.onmessage = (event) => {
    const packet = JSON.parse(event.data);
    console.log(`Alt: ${packet.altitude_baro}m, Phase: ${packet.flight_phase}`);
    updateUI(packet);
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};

ws.onclose = () => {
    console.log('Disconnected from telemetry stream');
};
```

**Message Format:** Same as telemetry packet (JSON)

**Update Rate:** 2 Hz (every 500ms)

---

## Flight Phase Detection

### Algorithm

The backend analyzes telemetry data to determine flight phase:

```rust
fn get_flight_phase(packet: &TelemetryPacket) -> &str {
    const GROUND_ALTITUDE: f32 = 2.0;      // Below 2m = on ground
    const CRUISE_ALTITUDE: f32 = 140.0;    // Cruise at 140m+
    const TAKEOFF_SPEED: f32 = 3.0;        // Takeoff roll speed
    const CLIMB_RATE: f32 = 0.8;           // Climbing threshold
    const DESCENT_RATE: f32 = -0.8;        // Descending threshold
    
    let is_on_ground = altitude_baro < GROUND_ALTITUDE;
    let is_moving = ground_speed >= TAKEOFF_SPEED;
    let is_climbing = vertical_speed > CLIMB_RATE;
    let is_descending = vertical_speed < DESCENT_RATE;
    
    // Decision tree (in order of priority)
    if is_on_ground && !is_moving → "On Ground"
    if is_on_ground && is_moving → "Taking Off"
    if altitude < 20m && is_descending → "Landing"
    if is_climbing && altitude < 140m → "Ascent"
    if altitude >= 140m && level flight → "Cruise"
    if is_descending && altitude > 20m → "Descent"
    if is_climbing → "Ascent"
    if airborne → "Cruise"
    else → "On Ground"
}
```

### Flight Phases

| Phase | Criteria | Description |
|-------|----------|-------------|
| **On Ground** | Alt < 2m, Speed < 3 m/s | Stationary or slow taxi |
| **Taking Off** | Alt < 2m, Speed ≥ 3 m/s | Takeoff roll, acceleration |
| **Ascent** | Climbing, Alt < 140m | Climbing to cruise altitude |
| **Cruise** | Alt ≥ 140m, Level flight | Stable cruise flight |
| **Descent** | Descending, Alt > 20m | Controlled descent |
| **Landing** | Alt < 20m, Descending | Final approach, touchdown |

### Phase Transitions Example

```
On Ground (rest/taxi)
    ↓
Taking Off (ground acceleration)
    ↓
Ascent (climbing to 140m)
    ↓
Cruise (level at 150m)
    ↓
Descent (descending from altitude)
    ↓
Landing (final approach)
    ↓
On Ground (landed)
```

---

## Flight Detection Logic

### When Does a Flight Start?

**Criteria:**
- Altitude > 5m AND Speed > 2 m/s
- Aircraft must be airborne (not just taxiing)
- "Taking Off" phase is included in flight

**Example:**
```
Time 0s:   On Ground (not a flight)
Time 45s:  Taking Off (speed 5 m/s, alt 0.5m) → Flight starts!
Time 50s:  Ascent (alt 10m)
Time 70s:  Cruise (alt 150m)
```

### When Does a Flight End?

**Criteria:**
- Altitude < 5m AND Speed < 2 m/s
- GPS stable for 5 seconds
- Aircraft has landed and stopped

**Timeout:**
- If no data for 60 seconds → flight ends catastrophically

---

## Data Processing Pipeline

### 1. Simulator → Server
```
Simulator generates packet (2 Hz)
    ↓
Encode to binary (113 bytes)
    ↓
Send via WebSocket (ws://simulator:8080/ws/binary)
    ↓
Server receives binary packet
```

### 2. Server Processing
```
Receive binary packet
    ↓
Decode to TelemetryPacket struct
    ↓
Determine flight phase (get_flight_phase)
    ↓
Check flight state (on ground / in flight / landing)
    ↓
If flight detected:
    - Store packet in KV store (key: "telem:flight_id:timestamp")
    - Update flight metadata
    - Log phase transitions
    ↓
Broadcast to WebSocket clients (JSON)
```

### 3. Storage Schema

**Flight Metadata:**
```
Key: "flight:flight_001"
Value: JSON FlightMetadata
```

**Telemetry Packets:**
```
Key: "telem:flight_001:65002"
Value: JSON TelemetryPacket
```

### 4. API Response
```
Client requests /api/flights/flight_001/data
    ↓
Server queries KV store for all keys matching "telem:flight_001:*"
    ↓
Deserialize packets, sort by timestamp
    ↓
Add flight_phase to each packet
    ↓
Return JSON array
```

---

## Integration Examples

### React/Vue Frontend

```javascript
// Fetch all flights
const flights = await fetch('http://localhost:9090/api/flights')
    .then(r => r.json());

// Get specific flight data
const flightData = await fetch(`http://localhost:9090/api/flights/${flightId}/data`)
    .then(r => r.json());

// Real-time updates
const ws = new WebSocket('ws://localhost:9090/ws/stream');
ws.onmessage = (event) => {
    const packet = JSON.parse(event.data);
    updateMap(packet.latitude, packet.longitude);
    updateAltitude(packet.altitude_baro);
    updatePhase(packet.flight_phase);
};
```

### Python Analysis

```python
import requests
import pandas as pd

# Get flight data
response = requests.get('http://localhost:9090/api/flights/flight_001/data')
data = response.json()

# Convert to DataFrame
df = pd.DataFrame(data)

# Analyze by phase
phase_stats = df.groupby('flight_phase').agg({
    'altitude_baro': ['mean', 'max'],
    'vertical_speed': 'mean',
    'ground_speed': 'mean'
})

# Plot altitude profile
import matplotlib.pyplot as plt
plt.plot(df['timestamp'], df['altitude_baro'])
plt.xlabel('Time (ms)')
plt.ylabel('Altitude (m)')
plt.show()
```

---

## Performance Characteristics

- **Telemetry Rate:** 2 Hz (500ms intervals)
- **Packet Size:** 113 bytes (binary), ~500 bytes (JSON)
- **Storage:** ~180 KB per 10-minute flight
- **Latency:** <10ms (simulator → server → client)
- **Concurrent Clients:** Unlimited WebSocket connections

---

## Error Handling

### Common Errors

**404 Not Found:**
```json
Flight not found
```

**500 Internal Server Error:**
```json
Storage error or server issue
```

**WebSocket Disconnection:**
- Auto-reconnect with exponential backoff
- Server logs: `[Server] Telemetry sim closed connection`

---

## CORS

**Policy:** Permissive (all origins allowed)

Safe for development, **disable for production**.

---

## Control Panel

### Overview

A web-based control panel for monitoring system status and testing API endpoints.

**File:** `control_panel.html`

**Access:** Open directly in browser (no server required)
```bash
open control_panel.html
# or
file:///path/to/control_panel.html
```

### Features

#### 1. Connection Status Indicators

Real-time status lights showing service health:

| Indicator | Port | Status Check |
|-----------|------|--------------|
| **Simulator** | 8080 | WebSocket connection test |
| **Server** | 9090 | HTTP health endpoint |
| **WebSocket Stream** | 9090 | Active WebSocket connection |

**Status Colors:**
- 🟢 **Green** - Connected/Running
- 🔴 **Red** - Disconnected/Not running

**Auto-refresh:** Every 5 seconds

#### 2. API Testing Interface

Interactive endpoint testing with editable URLs and live responses.

**Available Endpoints:**

1. **GET /api/flights**
   - Lists all flights
   - Default URL: `http://localhost:9090/api/flights`

2. **GET /api/flights/:id**
   - Get specific flight details
   - Default URL: `http://localhost:9090/api/flights/flight_001`
   - Edit flight ID in URL field

3. **GET /api/flights/:id/data**
   - Get flight telemetry data
   - Default URL: `http://localhost:9090/api/flights/flight_001/data`
   - Returns array with `flight_phase` field

4. **DELETE /api/flights/:id**
   - Delete a flight
   - Default URL: `http://localhost:9090/api/flights/flight_001`
   - Returns 204 on success

**Features:**
- ✅ Editable URL fields
- ✅ One-click send buttons
- ✅ Auto-formatted JSON responses
- ✅ HTTP status codes displayed
- ✅ Color-coded success/error indicators

### Usage Examples

#### Testing Different Flights
```
1. Click "Send" on GET /api/flights to see all flights
2. Copy a flight_id from the response
3. Edit the URL in "Get Flight Details" endpoint
4. Change flight_001 to your flight_id
5. Click "Send" to get that flight's details
```

#### Viewing Telemetry Data
```
1. Edit URL: http://localhost:9090/api/flights/flight_002/data
2. Click "Send"
3. Response shows all telemetry packets with flight_phase field
4. Use browser's JSON formatter or copy to external tool for analysis
```

#### Deleting Flights
```
1. Edit URL with flight ID to delete
2. Click "Send" on DELETE endpoint
3. Response shows "Deleted successfully" on success
4. Refresh GET /api/flights to verify deletion
```

### Screenshots

**Status Panel:**
```
🟢 Simulator (8080)
🟢 Server (9090)
🟢 WebSocket Stream
```

**API Response Example:**
```json
{
  "flight_id": "flight_001",
  "current_status": "Landed",
  "duration_secs": 180,
  "max_altitude": 150.99
}
```

**Status Code Display:**
```
200 OK          (Green - Success)
404 Not Found   (Red - Error)
```

### Troubleshooting

**All lights red:**
- Services not running
- Run `docker-compose up -d`

**Simulator light red:**
- Simulator not streaming
- Check `docker-compose logs simulator`

**Server light red:**
- Server not responding
- Check `docker-compose logs server`

**WebSocket light red:**
- Connection failed
- Will auto-reconnect every 5 seconds

**API request errors:**
- Check URL is correct
- Verify service is running (green light)
- Check browser console for CORS errors

---

## Docker Deployment

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f server

# Stop services
docker-compose down

# Remove data
docker-compose down -v
```

**Ports:**
- Simulator: 8080
- Server: 9090

**Data Persistence:**
- Volume: `telemetry-data`
- Location: `/app/data` in container

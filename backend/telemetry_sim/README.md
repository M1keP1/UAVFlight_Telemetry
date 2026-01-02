# 🚀 XtraChallenge Telemetry Simulator

A high-performance Rust-based telemetry simulator that generates realistic flight data matching ESP32 LoRa hardware output. Supports both binary (ESP32 format) and JSON (frontend format) streaming.

## 🏗️ Architecture

```
ESP32 LoRa (binary) → KV Store Backend → WebSocket (JSON) → Frontend
         ↑                                        ↑
    Simulated by                            Simulated by
    /ws/binary                                  /ws
```

The simulator provides **two WebSocket endpoints**:
- **`/ws/binary`**: Binary telemetry (113 bytes) - simulates ESP32 LoRa output
- **`/ws`**: JSON telemetry (~450 bytes) - simulates KV store → Frontend

## ✨ Features

- **Binary Protocol**: Matches ESP32 LoRa hardware format (little-endian, 113 bytes)
- **JSON API**: Frontend-friendly JSON streaming
- **Realistic Flight Simulation**: Follows predefined flight path with smooth interpolation
- **Comprehensive Telemetry**: GPS, barometer, IMU, power, and communication data
- **Low Resource Usage**: Efficient Rust implementation
- **Docker Ready**: Easy deployment with Docker and Docker Compose

## 📊 Telemetry Data

Each packet includes 29 fields:

| Category | Fields |
|----------|--------|
| **GPS** | latitude, longitude, altitude_gps, ground_speed, heading, num_satellites, gps_fix_type |
| **Barometer** | altitude_baro, vertical_speed, temperature |
| **IMU** | roll, pitch, yaw, gyro_x, gyro_y, gyro_z, accel_x, accel_y, accel_z |
| **Power** | battery_voltage, battery_current, battery_power, battery_mah_used |
| **Communication** | rssi, snr |
| **System** | timestamp, packet_sequence, system_status |

## 🐳 Quick Start with Docker

### Using Docker Compose (Recommended)

```bash
# Start the simulator
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the simulator
docker-compose down
```

### Using Helper Scripts

```bash
# Build the image
./build.sh

# Run the container
./run.sh

# View logs
docker logs -f xtra-telemetry
```

## 💻 Local Development

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))

### Build and Run

```bash
# Build in release mode
cargo build --release

# Run the simulator
cargo run --release
```

You should see:
```
🚀 XtraChallenge Telemetry Simulator

📡 Generator started (2 Hz)

🌐 WebSocket endpoints:
   Binary (ESP32 → KV):  ws://0.0.0.0:8080/ws/binary
   JSON (KV → Frontend): ws://0.0.0.0:8080/ws

#0000 | GPS: 49.872800,8.651200 | Alt:    0.0m | Batt: 16.80V ( 168.0W) | RSSI:  -50dBm
...
```

## 🔌 Connecting to WebSocket

### JSON Endpoint (Frontend)

**JavaScript:**
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
    const telemetry = JSON.parse(event.data);
    console.log(`Packet #${telemetry.packet_sequence}`);
    console.log(`GPS: ${telemetry.latitude}, ${telemetry.longitude}`);
    console.log(`Battery: ${telemetry.battery_voltage}V`);
};
```

**Python:**
```python
import asyncio
import websockets
import json

async def receive_json():
    uri = "ws://localhost:8080/ws"
    async with websockets.connect(uri) as websocket:
        async for message in websocket:
            data = json.loads(message)
            print(f"Packet #{data['packet_sequence']}: "
                  f"GPS {data['latitude']:.6f},{data['longitude']:.6f}")

asyncio.run(receive_json())
```

### Binary Endpoint (ESP32 Simulation)

**Python:**
```python
import asyncio
import websockets
import struct

async def receive_binary():
    uri = "ws://localhost:8080/ws/binary"
    async with websockets.connect(uri) as websocket:
        async for message in websocket:
            # Parse binary packet (113 bytes, little-endian)
            offset = 0
            
            # GPS (30 bytes)
            latitude = struct.unpack_from('<d', message, offset)[0]; offset += 8
            longitude = struct.unpack_from('<d', message, offset)[0]; offset += 8
            altitude_gps = struct.unpack_from('<f', message, offset)[0]; offset += 4
            ground_speed = struct.unpack_from('<f', message, offset)[0]; offset += 4
            heading = struct.unpack_from('<f', message, offset)[0]; offset += 4
            num_satellites = message[offset]; offset += 1
            gps_fix_type = message[offset]; offset += 1
            
            # ... parse remaining fields
            
            print(f"GPS: {latitude:.6f}, {longitude:.6f}, Alt: {altitude_gps:.1f}m")

asyncio.run(receive_binary())
```

**Using websocat:**
```bash
# View binary data as hex
websocat -b ws://localhost:8080/ws/binary | xxd

# View JSON data
websocat ws://localhost:8080/ws
```

## 📡 Binary Protocol Format

**Total Size**: 113 bytes (little-endian)

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 8 | f64 | latitude |
| 8 | 8 | f64 | longitude |
| 16 | 4 | f32 | altitude_gps |
| 20 | 4 | f32 | ground_speed |
| 24 | 4 | f32 | heading |
| 28 | 1 | u8 | num_satellites |
| 29 | 1 | u8 | gps_fix_type |
| 30 | 4 | f32 | altitude_baro |
| 34 | 4 | f32 | vertical_speed |
| 38 | 4 | f32 | temperature |
| 42 | 4 | f32 | roll |
| 46 | 4 | f32 | pitch |
| 50 | 4 | f32 | yaw |
| 54 | 4 | f32 | gyro_x |
| 58 | 4 | f32 | gyro_y |
| 62 | 4 | f32 | gyro_z |
| 66 | 4 | f32 | accel_x |
| 70 | 4 | f32 | accel_y |
| 74 | 4 | f32 | accel_z |
| 78 | 4 | f32 | battery_voltage |
| 82 | 4 | f32 | battery_current |
| 86 | 4 | f32 | battery_power |
| 90 | 4 | f32 | battery_mah_used |
| 94 | 2 | i16 | rssi |
| 96 | 4 | f32 | snr |
| 100 | 8 | u64 | timestamp |
| 108 | 4 | u32 | packet_sequence |
| 112 | 1 | u8 | system_status |

**Note**: All multi-byte values use **little-endian** byte order to match ESP32 architecture.

## 🛠️ Project Structure

```
telemetry_sim/
├── src/
│   ├── main.rs          # Application entry point
│   ├── telemetry.rs     # Data structures + binary serialization
│   ├── trajectory.rs    # Flight path logic
│   ├── generator.rs     # Telemetry generation
│   └── server.rs        # Dual WebSocket endpoints
├── Dockerfile           # Multi-stage Docker build
├── docker-compose.yml   # Docker Compose config
├── build.sh            # Build helper script
└── run.sh              # Run helper script
```

## 🔧 Configuration

### Modify Flight Path

Edit `src/trajectory.rs`:
```rust
pub const FLIGHT_PATH: &[Waypoint] = &[
    Waypoint { lat: 49.8728, lon: 8.6512, alt: 0.0,   time: 0.0 },
    Waypoint { lat: 49.8730, lon: 8.6515, alt: 50.0,  time: 10.0 },
    // Add more waypoints...
];
```

### Change Update Rate

Edit `src/main.rs`:
```rust
let mut ticker = interval(Duration::from_millis(500)); // 2 Hz
```

### Change Port

Edit `src/main.rs`:
```rust
let addr = "0.0.0.0:8080"; // Change port here
```

## 📝 Example JSON Output

```json
{
  "latitude": 49.872850,
  "longitude": 8.651235,
  "altitude_gps": 150.2,
  "ground_speed": 15.3,
  "heading": 45.0,
  "num_satellites": 9,
  "gps_fix_type": 3,
  "altitude_baro": 150.1,
  "vertical_speed": 0.5,
  "temperature": 20.1,
  "roll": 5.2,
  "pitch": 2.1,
  "yaw": 45.0,
  "gyro_x": 2.6,
  "gyro_y": 1.05,
  "gyro_z": 0.3,
  "accel_x": 0.1,
  "accel_y": -0.2,
  "accel_z": 1.05,
  "battery_voltage": 15.8,
  "battery_current": 11.2,
  "battery_power": 176.96,
  "battery_mah_used": 280.5,
  "rssi": -65,
  "snr": 9.5,
  "timestamp": 30254,
  "packet_sequence": 60,
  "system_status": 31
}
```

## 🎯 Use Cases

- **ESP32 Development**: Test your KV store backend before ESP32 hardware is ready
- **Frontend Development**: Build UI with realistic telemetry data
- **Integration Testing**: Validate entire data pipeline
- **Performance Testing**: Stress test with continuous data streams
- **Protocol Validation**: Verify binary serialization/deserialization

## 📝 License

MIT License - feel free to use for your projects!

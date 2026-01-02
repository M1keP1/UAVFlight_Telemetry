# Telemetry Backend System

Complete backend system for real-time flight telemetry data with simulator, KV storage, and REST API.

## Quick Start

### Prerequisites
- Docker
- Docker Compose

### Start the System

```bash
docker-compose up -d
```

That's it! The system is now running.

### Access Points

- **Control Panel**: http://localhost:9090
- **REST API**: http://localhost:9090/api
- **WebSocket Stream**: ws://localhost:9090/ws/stream

### Stop the System

```bash
docker-compose down
```

### Remove All Data

```bash
docker-compose down -v
```

---

## System Components

### 1. Telemetry Simulator (Port 8080)
- Generates realistic flight telemetry data
- Simulates complete flight cycles: Rest → Taxi → Takeoff → Ascent → Cruise → Descent → Landing
- Updates at 2 Hz (500ms intervals)
- Binary WebSocket endpoint: `ws://localhost:8080/ws/binary`

### 2. Telemetry KV Server (Port 9090)
- Receives telemetry from simulator
- Stores flight data in persistent KV store
- Detects and tracks flights automatically
- Provides REST API and WebSocket streaming
- Hosts control panel web interface

### 3. Control Panel (http://localhost:9090)
- Connection status indicators
- API testing interface
- Real-time monitoring

---

## API Endpoints

### List All Flights
```bash
GET http://localhost:9090/api/flights
```

### Get Flight Details
```bash
GET http://localhost:9090/api/flights/flight_001
```

### Get Flight Telemetry Data
```bash
GET http://localhost:9090/api/flights/flight_001/data
```
Returns array with `flight_phase` field for each packet.

### Delete Flight
```bash
DELETE http://localhost:9090/api/flights/flight_001
```

### WebSocket Stream
```javascript
const ws = new WebSocket('ws://localhost:9090/ws/stream');
ws.onmessage = (event) => {
    const packet = JSON.parse(event.data);
    console.log(packet);
};
```

---

## Flight Phases

The system automatically detects and tracks flight phases:

- **On Ground** - Stationary or slow movement
- **Taking Off** - Ground acceleration before rotation
- **Ascent** - Climbing to cruise altitude
- **Cruise** - Level flight at 140m+
- **Descent** - Controlled descent
- **Landing** - Final approach and touchdown
- **Landed** - Flight ended status

---

## Data Format

### Flight Metadata
```json
{
  "flight_id": "flight_001",
  "start_time": 65002,
  "end_time": 245504,
  "duration_secs": 180,
  "packet_count": 361,
  "distance_km": 2.867,
  "max_altitude": 150.99,
  "current_status": "Landed"
}
```

### Telemetry Packet
```json
{
  "latitude": 49.8728,
  "longitude": 8.6512,
  "altitude_baro": 150.0,
  "ground_speed": 25.0,
  "heading": 90.0,
  "roll": -15.3,
  "pitch": 8.2,
  "vertical_speed": 2.5,
  "flight_phase": "Cruise",
  ...
}
```

---

## Logs

### View Server Logs
```bash
docker-compose logs -f server
```

### View Simulator Logs
```bash
docker-compose logs -f simulator
```

### View All Logs
```bash
docker-compose logs -f
```

---

## Troubleshooting

### Services won't start
```bash
# Check if ports are in use
lsof -i :8080
lsof -i :9090

# Restart services
docker-compose restart
```

### Data not appearing
```bash
# Check simulator is running
docker-compose ps

# Check logs for errors
docker-compose logs server
docker-compose logs simulator
```

### Reset everything
```bash
docker-compose down -v
docker-compose up -d
```

---

## Development

### Rebuild After Code Changes
```bash
docker-compose up --build -d
```

### Access Container Shell
```bash
docker exec -it telemetry-kv-server sh
docker exec -it telemetry-simulator sh
```

---

## Architecture

```
┌─────────────────┐         ┌──────────────────┐
│   Simulator     │ Binary  │   KV Server      │
│   (Port 8080)   │────────▶│   (Port 9090)    │
│                 │  WS     │                  │
│ - Flight data   │         │ - Storage        │
│ - 2 Hz updates  │         │ - Flight detect  │
│ - Realistic     │         │ - REST API       │
│   physics       │         │ - WebSocket      │
└─────────────────┘         │ - Control Panel  │
                            └──────────────────┘
                                     │
                                     ▼
                            ┌──────────────────┐
                            │   Frontend       │
                            │   Application    │
                            └──────────────────┘
```

---

## Performance

- **Telemetry Rate**: 2 Hz (500ms)
- **Packet Size**: 113 bytes (binary), ~500 bytes (JSON)
- **Storage**: ~180 KB per 10-minute flight
- **Latency**: <10ms end-to-end
- **Concurrent Clients**: Unlimited WebSocket connections

---

## Data Persistence

Flight data is stored in a Docker volume:
- Volume name: `telemetry-data`
- Data persists across container restarts
- Delete with: `docker-compose down -v`

---

## Support

For detailed API documentation, see `API_GUIDE.md`

For issues or questions, check the logs first:
```bash
docker-compose logs
```

# Telemetry Backend

Standalone backend system for flight telemetry data.

## Quick Start

```bash
docker-compose up -d
```

Access the control panel at: **http://localhost:9090**

## Documentation

- **README.md** - Quick start guide and system overview
- **API_GUIDE.md** - Complete API documentation with examples

## Ports

- **9090** - KV Server (API + Control Panel)
- **8080** - Simulator (Internal)

## Stop

```bash
docker-compose down
```

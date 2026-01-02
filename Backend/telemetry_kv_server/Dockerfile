# Build stage
FROM rust:1.83 AS builder

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

# Copy telemetry_kv_server
COPY telemetry_kv_server/Cargo.toml ./telemetry_kv_server/
COPY telemetry_kv_server/src ./telemetry_kv_server/src

# Copy control panel
COPY control_panel.html ./

WORKDIR /build/telemetry_kv_server
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /build/telemetry_kv_server/target/release/telemetry-kv-server .

# Create directory for data persistence
RUN mkdir -p /app/data

# Expose REST API and WebSocket port
EXPOSE 9090

# Run the binary
CMD ["./telemetry-kv-server"]

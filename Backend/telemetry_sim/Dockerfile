# Multi-stage build for minimal image size
FROM rust:1.83 AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./
COPY Cargo.lock* ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/xtra-telemetry .

# Expose WebSocket port
EXPOSE 8080

# Run the binary
CMD ["./xtra-telemetry"]

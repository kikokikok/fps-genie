# Multi-stage build for CS2 Demo Analysis Tools
FROM rust:1.80-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libfontconfig1-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source code
COPY . .

# Build for release - skip tests and examples for faster builds
RUN cargo build --release --workspace --bins

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 cs2user

WORKDIR /app

# Copy binaries from builder stage
COPY --from=builder /app/target/release/cs2-analytics /usr/local/bin/
COPY --from=builder /app/target/release/cs2-data-pipeline /usr/local/bin/
COPY --from=builder /app/target/release/cs2-demo-analyzer /usr/local/bin/
COPY --from=builder /app/target/release/cs2-ml /usr/local/bin/
COPY --from=builder /app/target/release/csgoproto /usr/local/bin/

# Create necessary directories
RUN mkdir -p /app/demos /app/temp /app/models \
    && chown -R cs2user:cs2user /app

USER cs2user

# Default command
CMD ["cs2-data-pipeline"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD cs2-data-pipeline --help || exit 1

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 cs2user

WORKDIR /app

# Copy binaries from builder stage
COPY --from=builder /app/target/release/cs2-analytics /usr/local/bin/
COPY --from=builder /app/target/release/cs2-data-pipeline /usr/local/bin/
COPY --from=builder /app/target/release/cs2-demo-analyzer /usr/local/bin/
COPY --from=builder /app/target/release/cs2-ml /usr/local/bin/
COPY --from=builder /app/target/release/csgoproto /usr/local/bin/

# Create necessary directories
RUN mkdir -p /app/demos /app/temp /app/models \
    && chown -R cs2user:cs2user /app

USER cs2user

# Default command
CMD ["cs2-data-pipeline"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD cs2-data-pipeline --help || exit 1

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 cs2user

WORKDIR /app

# Copy binaries from builder stage
COPY --from=builder /app/target/release/cs2-analytics /usr/local/bin/
COPY --from=builder /app/target/release/cs2-data-pipeline /usr/local/bin/
COPY --from=builder /app/target/release/cs2-demo-analyzer /usr/local/bin/
COPY --from=builder /app/target/release/cs2-ml /usr/local/bin/
COPY --from=builder /app/target/release/csgoproto /usr/local/bin/

# Create necessary directories
RUN mkdir -p /app/demos /app/temp /app/models \
    && chown -R cs2user:cs2user /app

USER cs2user

# Default command
CMD ["cs2-data-pipeline"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD cs2-data-pipeline --help || exit 1

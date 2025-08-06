# Multi-stage build for CS2 Demo Analysis Tools
FROM rust:1.75 as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY cs2-analytics/Cargo.toml cs2-analytics/
COPY cs2-client/Cargo.toml cs2-client/
COPY cs2-common/Cargo.toml cs2-common/
COPY cs2-data-pipeline/Cargo.toml cs2-data-pipeline/
COPY cs2-demo-analyzer/Cargo.toml cs2-demo-analyzer/
COPY cs2-demo-parser/Cargo.toml cs2-demo-parser/
COPY cs2-integration-tests/Cargo.toml cs2-integration-tests/
COPY cs2-ml/Cargo.toml cs2-ml/
COPY csgoproto/Cargo.toml csgoproto/

# Copy source code
COPY . .

# Build for release
RUN cargo build --release --workspace

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

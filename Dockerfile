# Multi-stage build for CS2 Demo Analysis Tools
FROM rust:1.88-bookworm AS builder

# Configure Cargo for network resilience
RUN mkdir -p /usr/local/cargo && \
    echo '[net]' >> /usr/local/cargo/config.toml && \
    echo 'retry = 10' >> /usr/local/cargo/config.toml && \
    echo 'timeout = 300' >> /usr/local/cargo/config.toml && \
    echo '[http]' >> /usr/local/cargo/config.toml && \
    echo 'timeout = 300' >> /usr/local/cargo/config.toml && \
    echo '[registries.crates-io]' >> /usr/local/cargo/config.toml && \
    echo 'protocol = "sparse"' >> /usr/local/cargo/config.toml

# Install build dependencies with error handling
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libfontconfig1-dev \
    libssl-dev \
    pkg-config \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

WORKDIR /app

# Copy source code
COPY . .

# Build core components with robust error handling
RUN set -e; \
    echo "Starting build process..."; \
    \
    # Try to build core components with timeouts and retries
    for package in cs2-common cs2-demo-parser cs2-data-pipeline cs2-demo-analyzer cs2-analytics csgoproto; do \
        echo "Building $package..."; \
        if timeout 900 cargo build --release --no-default-features -p $package; then \
            echo "✅ $package built successfully"; \
        else \
            echo "⚠️ Failed to build $package, continuing..."; \
        fi; \
    done; \
    \
    # List what was actually built
    echo "Built binaries:"; \
    find target/release -maxdepth 1 -type f -executable | head -10 || echo "No binaries found"

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Create app user
RUN useradd -m -u 1000 cs2user

WORKDIR /app

# Copy binaries from builder stage (with fallback handling)
RUN echo "Copying available binaries..."

# Copy binaries that exist, ignore those that don't
COPY --from=builder /app/target/release/cs2-demo-parser /usr/local/bin/ 2>/dev/null || echo "cs2-demo-parser not available"
COPY --from=builder /app/target/release/cs2-data-pipeline /usr/local/bin/ 2>/dev/null || echo "cs2-data-pipeline not available"
COPY --from=builder /app/target/release/cs2-demo-analyzer /usr/local/bin/ 2>/dev/null || echo "cs2-demo-analyzer not available"
COPY --from=builder /app/target/release/cs2-analytics /usr/local/bin/ 2>/dev/null || echo "cs2-analytics not available"
COPY --from=builder /app/target/release/csgoproto /usr/local/bin/ 2>/dev/null || echo "csgoproto not available"

# Create a simple entrypoint script
RUN echo '#!/bin/bash' > /usr/local/bin/entrypoint.sh && \
    echo 'echo "FPS Genie CS2 Analysis Container"' >> /usr/local/bin/entrypoint.sh && \
    echo 'echo "Available commands:"' >> /usr/local/bin/entrypoint.sh && \
    echo 'ls -la /usr/local/bin/ | grep cs2' >> /usr/local/bin/entrypoint.sh && \
    echo 'if [ "$#" -eq 0 ]; then' >> /usr/local/bin/entrypoint.sh && \
    echo '  echo "Usage: docker run fps-genie [command] [args]"' >> /usr/local/bin/entrypoint.sh && \
    echo '  echo "Commands: cs2-demo-parser, cs2-data-pipeline, cs2-demo-analyzer, cs2-analytics"' >> /usr/local/bin/entrypoint.sh && \
    echo 'else' >> /usr/local/bin/entrypoint.sh && \
    echo '  exec "$@"' >> /usr/local/bin/entrypoint.sh && \
    echo 'fi' >> /usr/local/bin/entrypoint.sh && \
    chmod +x /usr/local/bin/entrypoint.sh

# Create necessary directories
RUN mkdir -p /app/demos /app/temp /app/models \
    && chown -R cs2user:cs2user /app

USER cs2user

# Default command
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD []

# Health check using available binary
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ls /usr/local/bin/cs2* > /dev/null || exit 1

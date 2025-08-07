# Multi-stage build for CS2 Demo Analysis Tools
# Use our custom base image with all dependencies pre-installed
FROM ghcr.io/kikokikok/fps-genie-ci-base:latest as builder

WORKDIR /app

# Copy manifests first for better layer caching
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

# Create dummy source files to cache dependencies
RUN mkdir -p cs2-analytics/src cs2-client/src cs2-common/src cs2-data-pipeline/src \
    cs2-demo-analyzer/src cs2-demo-parser/src cs2-integration-tests/src \
    cs2-ml/src csgoproto/src && \
    echo "fn main() {}" > cs2-analytics/src/main.rs && \
    echo "fn main() {}" > cs2-data-pipeline/src/main.rs && \
    echo "fn main() {}" > cs2-demo-analyzer/src/main.rs && \
    echo "fn main() {}" > cs2-ml/src/main.rs && \
    echo "fn main() {}" > csgoproto/src/main.rs && \
    echo "// dummy" > cs2-client/src/lib.rs && \
    echo "// dummy" > cs2-common/src/lib.rs && \
    echo "// dummy" > cs2-demo-parser/src/lib.rs && \
    echo "// dummy" > cs2-integration-tests/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release --workspace

# Copy actual source code
COPY . .

# Touch source files to force rebuild of app code only
RUN find . -name "*.rs" -exec touch {} +

# Build for release with all dependencies cached
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

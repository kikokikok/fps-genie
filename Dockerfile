# =============================================================================
# Base image with all dependencies - shared across all environments
# =============================================================================
FROM rust:1.88-bookworm as base

# Install all system dependencies in one layer
RUN apt-get update && apt-get install -y \
    # Build essentials
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    libprotobuf-dev \
    git \
    # Font libraries for GUI components
    libfontconfig1-dev \
    libfreetype6-dev \
    # Python for ML pipeline
    python3 \
    python3-pip \
    python3-dev \
    python3-venv \
    # Database clients
    postgresql-client \
    # Utilities
    curl \
    wget \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Set ARM64 compatibility environment variables to avoid FP16 issues
ENV RUSTFLAGS="-C target-cpu=generic"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C target-cpu=generic"

WORKDIR /workspace

# =============================================================================
# Development environment (for devcontainer)
# =============================================================================
FROM base as development

# Install development tools
RUN cargo install \
    cargo-watch \
    cargo-expand \
    cargo-audit \
    sqlx-cli \
    && pip3 install --break-system-packages \
    jupyter \
    pandas \
    numpy \
    matplotlib \
    seaborn \
    scikit-learn \
    notebook

# Expose development ports
EXPOSE 8080 8888 3000 5432

CMD ["bash"]

# =============================================================================
# Test stage for CI/CD
# =============================================================================
FROM base as test

# Copy project files
COPY . .

# Build and test the project with generic ARM64 target
RUN cargo build --workspace --tests
RUN cargo test --workspace

CMD ["bash"]

# =============================================================================
# Builder stage for production
# =============================================================================
FROM base as builder

# Copy dependency manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY cs2-*/Cargo.toml ./
COPY csgoproto/Cargo.toml csgoproto/

# Create dummy source files to build dependencies
RUN mkdir -p cs2-analytics/src cs2-client/src cs2-common/src \
    cs2-data-pipeline/src cs2-demo-analyzer/src cs2-demo-parser/src \
    cs2-integration-tests/src cs2-ml/src csgoproto/src && \
    echo "fn main() {}" > cs2-analytics/src/main.rs && \
    echo "fn main() {}" > cs2-data-pipeline/src/main.rs && \
    echo "fn main() {}" > cs2-demo-analyzer/src/main.rs && \
    echo "fn main() {}" > cs2-ml/src/main.rs && \
    echo "fn main() {}" > csgoproto/src/main.rs && \
    echo "// dummy" > cs2-client/src/lib.rs && \
    echo "// dummy" > cs2-common/src/lib.rs && \
    echo "// dummy" > cs2-demo-parser/src/lib.rs && \
    echo "// dummy" > cs2-integration-tests/src/lib.rs

# Build dependencies with generic ARM64 target (this layer will be cached)
RUN cargo build --release

# Remove dummy files and copy real source
RUN rm -rf cs2-*/src csgoproto/src
COPY . .

# Build the actual application
RUN cargo build --release --workspace

# =============================================================================
# Production runtime
# =============================================================================
FROM debian:bookworm-slim as production

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libfontconfig1 \
    libfreetype6 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 appuser

# Copy binaries
COPY --from=builder /workspace/target/release/cs2-* /usr/local/bin/
COPY --from=builder /workspace/target/release/csgoproto /usr/local/bin/

# Set ownership and switch to non-root user
RUN chown -R appuser:appuser /usr/local/bin
USER appuser

EXPOSE 8080

# Default to data pipeline service
CMD ["cs2-data-pipeline"]

# Multi-stage Docker base image for FPS Genie CI/CD
FROM rust:1.88-bookworm as rust-base

# Set environment variables for non-interactive installation
ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_BACKTRACE=1
ENV CARGO_TERM_COLOR=always

# Install system dependencies in a single layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Build essentials
    build-essential \
    clang \
    gobjc \
    # Protocol buffers
    protobuf-compiler \
    # Graphics and fonts
    libfontconfig1-dev \
    # SSL and networking
    libssl-dev \
    pkg-config \
    # Database client
    postgresql-client \
    # Utilities
    curl \
    git \
    ca-certificates \
    # Cleanup
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Verify installations
RUN protoc --version && \
    clang --version && \
    rustc --version && \
    cargo --version

# Set up Rust environment optimizations
ENV CARGO_NET_RETRY=3
ENV CARGO_NET_TIMEOUT=30
ENV CARGO_HTTP_MULTIPLEXING=false

# Create a non-root user for security
RUN useradd -m -u 1000 runner && \
    chown -R runner:runner /usr/local/cargo

# CI-specific stage with additional tools
FROM rust-base as ci-base

# Pre-install common Rust CI tools to save time in CI
RUN cargo install --locked \
    cargo-audit \
    cargo-llvm-cov \
    cargo-deny \
    && rm -rf /usr/local/cargo/registry

# Set up caching directories
RUN mkdir -p /usr/local/cargo/registry && \
    chown -R runner:runner /usr/local/cargo

# Development stage with additional tools for local development
FROM ci-base as dev-base

# Add development tools
RUN cargo install --locked \
    cargo-watch \
    cargo-expand \
    cargo-edit \
    && rm -rf /usr/local/cargo/registry

# Add additional development dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Debugging tools
    gdb \
    lldb \
    # Performance tools
    valgrind \
    # Editor support
    ctags \
    # Cleanup
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Set default user
USER runner
WORKDIR /workspace

# Default command
CMD ["cargo", "--version"]

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD cargo --version || exit 1

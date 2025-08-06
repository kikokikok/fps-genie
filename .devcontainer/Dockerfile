FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# Install additional system dependencies including Redis CLI
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libprotobuf-dev \
    pkg-config \
    libssl-dev \
    build-essential \
    gobjc \
    libobjc-9-dev \
    postgresql-client \
    redis-tools \
    curl \
    jq \
    python3 \
    python3-pip \
    python3-venv \
    htop \
    vim \
    && rm -rf /var/lib/apt/lists/*

# Install cargo tools for development
RUN cargo install cargo-watch cargo-audit cargo-llvm-cov

# Install Python dependencies for ML experimentation
RUN pip3 install --no-cache-dir \
    torch \
    numpy \
    pandas \
    matplotlib \
    seaborn \
    jupyter \
    psycopg2-binary \
    redis \
    requests

# Create workspace directory
WORKDIR /workspace

# Set up Rust environment
ENV CARGO_TARGET_DIR=/workspace/target
ENV RUST_LOG=debug

# CS2 Demo Analysis & AI Training System - Developer Instructions

Always reference these instructions first and fallback to search or additional commands only when you encounter unexpected information that does not match the info here.

## Project Overview

This is a high-performance CS2 demo analysis and AI training system built in Rust. The system processes professional CS2 match demos to provide AI-driven coaching insights, pro player comparisons, and behavioral analysis. Architecture includes multi-tier databases (PostgreSQL/TimescaleDB, Qdrant vector storage, Redis, MinIO) and advanced ML capabilities.

## Critical Build Requirements & Dependencies

**NEVER CANCEL** any build or test command. Builds can take 2-5 minutes, initial setups can take 5-10 minutes.

### Essential System Dependencies
Install ALL of these before building - failure to install ANY will cause build failures:
```bash
# Core build tools (REQUIRED)
sudo apt-get update
sudo apt-get install -y build-essential clang gobjc protobuf-compiler libfontconfig1-dev

# Or on macOS:
brew install cmake protobuf pkg-config
```

### Rust Workspace Structure
```
fps-genie/
├── cs2-demo-parser/       # Core demo parsing (builds in 10s)
├── cs2-ml/                # ML models (2+ min build, has platform issues)
├── cs2-data-pipeline/     # Batch processing (1.5 min build)
├── cs2-common/            # Shared utilities (fast build)
├── cs2-demo-analyzer/     # CLI analysis tool (1+ min build)
├── cs2-client/            # Client library
├── cs2-analytics/         # Advanced analytics
├── cs2-integration-tests/ # E2E testing with TestContainers
└── csgoproto/            # Protocol buffer definitions
```

## Building and Testing (NEVER CANCEL)

### Core Build Commands - Validated Working
```bash
# Check workspace (NEVER CANCEL: 1-3 minutes)
cargo check --workspace

# Build core components individually (RECOMMENDED approach)
cargo check -p cs2-demo-parser        # 10 seconds
cargo check -p cs2-common             # 5 seconds  
cargo check -p cs2-data-pipeline      # 90 seconds

# Run unit tests (NEVER CANCEL: 1-2 minutes total)
cargo test --lib -p cs2-demo-parser -p cs2-common --quiet

# Full workspace build (NEVER CANCEL: 3-5 minutes)
cargo build --workspace
```

### Critical Platform-Specific Issues
**Linux Users**: The `cs2-ml` crate defaults to Metal (macOS-only). If build fails with objc_exception errors:
1. Edit `cs2-ml/Cargo.toml` 
2. Change `default = ["metal"]` to `default = ["cpu-only"]`
3. Clean and rebuild: `cargo clean && cargo build`

**Always use CPU-only features on Linux**: `cargo build --workspace --no-default-features`

## Database Infrastructure Setup (NEVER CANCEL)

### Complete Infrastructure Setup
```bash
# Setup all databases (NEVER CANCEL: 5-10 minutes for first run)
chmod +x setup_databases.sh
./setup_databases.sh

# Manual database start (if needed)
docker compose up -d

# Check database status
docker compose ps
```

**TIMING EXPECTATIONS**: 
- Initial Docker image downloads: 5-10 minutes
- Database startup: 2-3 minutes
- Connection verification: 30 seconds

### Connection Details (After Setup)
```bash
export DATABASE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
export TIMESCALE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
export QDRANT_URL="http://localhost:6333"
export REDIS_URL="redis://localhost:6379"
```

## Working With Demo Files

### Demo Analysis Commands (Working)
```bash
# Core demo parsing (works reliably)
cargo run -p cs2-demo-parser --bin parser -- path/to/demo.dem

# Data pipeline operations
cd cs2-data-pipeline
cargo run -- init                    # Initialize pipeline
cargo run -- discover --recursive    # Find demo files  
cargo run -- process                # Process demos (NEVER CANCEL: 5+ min for large demos)
cargo run -- stats                  # Show processing stats
```

### Demo File Locations
- Test files: `./test_data/*.dem`
- Place demo files in: `./demos/` directory
- Expected processing: 700MB+/second on high-end hardware

## Testing Strategy (NEVER CANCEL)

### Unit Tests (Fast)
```bash
# Core unit tests (45 seconds)
cargo test --lib --workspace --quiet

# Individual crate tests
cargo test -p cs2-demo-parser        # 15 seconds, 334 tests
cargo test -p cs2-common             # 5 seconds, 11 tests
```

### Integration Tests (NEVER CANCEL: 5-15 minutes)
```bash
# Database integration tests (requires running databases)
cargo test --package cs2-integration-tests --features integration-tests

# E2E testing with TestContainers (NEVER CANCEL: 10+ minutes)
cargo test --workspace --test '*' --features integration-tests
```

### Performance Benchmarks (NEVER CANCEL: 10+ minutes)
```bash
cargo bench --workspace
```

**Expected Performance**:
- Demo parsing: 700MB+/second 
- Database ingestion: 10K+ snapshots/second
- Memory usage: 2-8GB for large demos

## Common Development Tasks

### Code Quality (Always Run Before Committing)
```bash
# Format code
cargo fmt --all

# Lint code (required for CI)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Security audit
cargo audit
```

### CI Pipeline Requirements
The GitHub Actions CI will fail if you don't run these locally first:
1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings` 
3. All tests must pass
4. Security audit must pass

## ML Training and AI Components

### ML Training Commands (NEVER CANCEL: 15+ minutes)
```bash
cd cs2-ml

# Prepare training data (5+ minutes for large datasets)
cargo run -- prepare "demos/*.dem" ./data

# Train models (NEVER CANCEL: 15-60 minutes depending on data size)
cargo run -- train ./data/*.parquet ./policy.ot --epochs 1000

# Serve trained model
cargo run -- serve ./policy.ot --port 8123
```

**ML Features Available**:
- Pro player behavior cloning
- Crosshair placement analysis  
- Tactical decision analysis
- Performance comparison algorithms

## Troubleshooting Common Issues

### Build Failures
1. **protobuf errors**: Install `protobuf-compiler`
2. **objc_exception on Linux**: Switch cs2-ml to cpu-only features
3. **fontconfig errors**: Install `libfontconfig1-dev`
4. **Build hanging**: NEVER CANCEL - builds can take 5+ minutes

### Database Issues  
1. **Connection refused**: Run `./setup_databases.sh` and wait 5+ minutes
2. **TimescaleDB extension errors**: Database needs 2-3 minutes to fully initialize
3. **Docker issues**: Check `docker compose ps` and `docker compose logs`

### Performance Issues
1. **Slow demo parsing**: Expected for large files (50MB+ demos take 1+ minutes)
2. **High memory usage**: Normal for large demos (2-8GB RAM usage)
3. **Database ingestion slow**: Use batch processing in cs2-data-pipeline

## Project Architecture Context

### Data Flow
1. **Demo Discovery**: Scan directories for .dem files
2. **Demo Parsing**: Extract player snapshots using cs2-demo-parser  
3. **Database Storage**: TimescaleDB for time-series, Qdrant for vectors
4. **ML Analysis**: Generate behavioral embeddings and coaching insights
5. **API Serving**: REST APIs for coaching recommendations

### Database Schema (Optimized for Scale)
- **PostgreSQL**: Match metadata, player info, tournament data
- **TimescaleDB**: Millions of player snapshots per match (time-series)
- **Qdrant**: High-dimensional behavioral vectors for similarity search
- **Redis**: Caching and job queues for processing pipeline

### Key Data Models
- `PlayerSnapshot`: Complete player state at specific tick (~100 fields)
- `BehavioralVector`: AI-generated behavior embeddings  
- `KeyMoment`: Identified significant gameplay situations (clutches, aces)
- `Match`: Match metadata and processing status

## Expected Development Workflow

### For New Features
1. **Start databases**: `./setup_databases.sh` (wait 5+ minutes)
2. **Build workspace**: `cargo check --workspace` (2-3 minutes) 
3. **Run tests**: `cargo test --lib --workspace` (1-2 minutes)
4. **Develop changes** with frequent `cargo check -p <crate-name>`
5. **Test changes**: Run relevant test suites
6. **Format and lint**: `cargo fmt --all && cargo clippy --workspace`
7. **Integration test**: `cargo test --workspace` (NEVER CANCEL: 5+ minutes)

### Performance Targets (From PDF Specifications)
- **Processing**: 700MB+/s demo parsing on high-end PC
- **Storage**: 5TB initial TimescaleDB, 2TB vector embeddings, 20TB+ archives  
- **Scale**: Process 50+ professional matches daily
- **Throughput**: 10K+ player snapshots/second database ingestion

**NEVER CANCEL** any long-running operations. The system is designed for high-volume professional esports data processing and requires patience for large operations.

## File Locations Reference

### Configuration Files
- `Cargo.toml` - Workspace configuration
- `docker-compose.yml` - Database infrastructure  
- `setup_databases.sh` - Automated database setup
- `.github/workflows/ci.yml` - CI pipeline (comprehensive)

### Test Data
- `test_data/vitality-vs-spirit-m1-dust2.dem` - Professional match demo
- `test_data/test_demo.dem` - Test demo file
- `sql/init.sql` - Database initialization schema

### Generated Outputs
- `target/` - Rust build artifacts (gitignored)
- `demos/` - Demo file processing directory
- `training_data/` - ML training datasets
- `models/` - Trained AI models

Always ensure databases are running before attempting ML training or data pipeline operations.
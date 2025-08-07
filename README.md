# 🎯 FPS Genie - CS2 Demo Analysis & AI Training System

[![CI Status](https://github.com/kikokikok/fps-genie/workflows/CI/badge.svg)](https://github.com/kikokikok/fps-genie/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)

> **Revolutionary AI-powered CS2 demo analysis system that transforms gameplay data into actionable coaching insights, pro player comparisons, and realistic AI training scenarios.**

## 🚀 What is FPS Genie?

FPS Genie is a high-performance, Rust-based system that analyzes Counter-Strike 2 demo files to provide:

- **🤖 AI-Driven Coaching**: Real-time crosshair placement correction, tactical decision analysis, and personalized feedback
- **🏆 Pro Player Comparison**: Compare your gameplay against professional players with detailed skill gap analysis
- **🎮 AI Behavior Cloning**: Train against AI that mimics professional player behaviors in realistic scenarios
- **📊 Advanced Analytics**: Deep behavioral analysis, key moment extraction, and performance tracking
- **🔄 Multi-Game Support**: Extensible architecture designed for Valorant, Apex Legends, and other FPS games

## ✨ Key Features

### 🎯 **Smart Demo Analysis**
- **Lightning-fast parsing**: 700MB+/second demo processing on high-end hardware
- **Comprehensive state tracking**: 100+ player attributes per game tick
- **Key moment detection**: Automatic identification of clutches, aces, and tactical plays
- **Behavioral vector analysis**: Advanced ML-powered pattern recognition

### 🧠 **AI Coaching Engine**
- **Crosshair placement analysis**: Real-time feedback on aim positioning and pre-aiming
- **Movement optimization**: Guidance on positioning, rotations, and map control
- **Utility usage insights**: Grenades, smokes, and tactical equipment effectiveness
- **Decision-making analysis**: Evaluate engagement choices and tactical decisions

### 🏅 **Pro Player Insights**
- **Playstyle similarity scoring**: Find which pro player you most resemble
- **Skill gap identification**: Pinpoint specific areas for improvement
- **Role recommendation**: Discover optimal in-game roles based on your strengths
- **Benchmarking**: Compare statistics against professional tournaments

### 🎮 **Ephemeral Training Servers**
- **AI behavior cloning**: Face off against bots that play like specific pro players
- **Scenario recreation**: Practice crucial match moments and clutch situations
- **Adaptive difficulty**: AI opponents that scale to your skill level
- **Custom training**: Generate specific scenarios targeting your weaknesses

## 🏗️ Architecture Overview

```mermaid
graph TB
    A[CS2 Demo Files] --> B[Rust Analysis Engine]
    B --> C[Pro Moment Extraction]
    B --> D[Player Performance Analysis]
    C --> E[BehavioralVector Storage]
    D --> F[AI Coaching Engine]
    E --> G[Pro Scenario Rebuilder]
    F --> H[Real-time Overlay]
    G --> I[Ephemeral Training Servers]
    H --> J[User Experience]
    I --> J
    
    subgraph "Multi-Tier Database Architecture"
        K[PostgreSQL<br/>Match Metadata]
        L[TimescaleDB<br/>Time-Series Data]
        M[Qdrant<br/>Vector Storage]
        N[Redis<br/>Caching & Queues]
        O[MinIO<br/>Object Storage]
    end
    
    B --> K
    D --> L
    E --> M
    F --> N
    A --> O
```

### 🧩 **Component Structure**

The system is built as a multi-crate Rust workspace:

| Crate | Purpose | Build Time | Key Features |
|-------|---------|------------|--------------|
| `cs2-demo-parser` | Core demo parsing engine | ~10s | High-performance binary parsing, 700MB+/s throughput |
| `cs2-ml` | Machine learning models | ~2 min | Behavior cloning, tactical analysis, PyTorch integration |
| `cs2-data-pipeline` | Batch processing system | ~90s | Demo discovery, processing orchestration, data ingestion |
| `cs2-common` | Shared utilities | ~5s | Common data structures, utilities, error handling |
| `cs2-demo-analyzer` | CLI analysis tool | ~60s | Command-line interface, batch analysis capabilities |
| `cs2-client` | Client library | ~30s | API client, data access layer |
| `cs2-analytics` | Advanced analytics | ~45s | Statistical analysis, reporting, visualizations |
| `cs2-integration-tests` | End-to-end testing | ~5-15 min | TestContainers, full system validation |
| `csgoproto` | Protocol definitions | ~10s | CS2 protobuf definitions, message parsing |

## 🚀 Quick Start

### Prerequisites

- **Rust 1.70+** with Cargo
- **Docker & Docker Compose** for database infrastructure
- **System dependencies**:
  ```bash
  # Ubuntu/Debian
  sudo apt-get update
  sudo apt-get install -y build-essential clang protobuf-compiler libfontconfig1-dev
  
  # macOS
  brew install cmake protobuf pkg-config
  ```

### 🔧 **1. Setup Infrastructure**

```bash
# Clone the repository
git clone https://github.com/kikokikok/fps-genie.git
cd fps-genie

# Setup multi-tier database infrastructure (5-10 minutes)
./setup_databases.sh

# Verify services are running
docker compose ps
```

### ⚡ **2. Build the System**

```bash
# Build core components (2-3 minutes total)
cargo check --workspace

# Build individual components for faster iteration
cargo check -p cs2-demo-parser    # 10 seconds
cargo check -p cs2-common         # 5 seconds
cargo check -p cs2-data-pipeline  # 90 seconds
```

### 📁 **3. Analyze Your First Demo**

```bash
# Place your demo files in the demos/ directory
mkdir -p demos/
cp your_demo.dem demos/

# Initialize the data pipeline
cd cs2-data-pipeline
cargo run -- init

# Discover and process demos
cargo run -- discover --recursive
cargo run -- process

# View processing statistics
cargo run -- stats
```

### 🎯 **4. Get AI Coaching Insights**

```bash
# Run crosshair placement analysis
cargo run -p cs2-demo-analyzer -- analyze demos/your_demo.dem --coaching

# Compare against pro players
cargo run -p cs2-demo-analyzer -- compare demos/your_demo.dem --pro-player s1mple

# Generate training scenarios
cargo run -p cs2-demo-analyzer -- extract-scenarios demos/your_demo.dem
```

## 💾 Database Architecture

FPS Genie uses a sophisticated multi-tier database architecture designed for scale:

### 🗄️ **PostgreSQL + TimescaleDB**
- **Purpose**: Relational data and time-series player snapshots
- **Scale**: 5TB+ initial capacity, millions of snapshots per match
- **Performance**: 10K+ player snapshots/second ingestion rate

### 🔍 **Qdrant Vector Database**
- **Purpose**: High-dimensional behavioral embeddings and similarity search
- **Scale**: 2TB+ vector storage capacity
- **Features**: Fast similarity search for pro player comparisons

### ⚡ **Redis**
- **Purpose**: Caching, job queues, and session management
- **Features**: Processing pipeline coordination, real-time data caching

### 📦 **MinIO Object Storage**
- **Purpose**: Raw demo files, exports, and large binary objects
- **Scale**: 20TB+ storage capacity for demo archives

### 🔧 **Connection Setup**

After running `./setup_databases.sh`, set these environment variables:

```bash
export DATABASE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
export QDRANT_URL="http://localhost:6333"
export REDIS_URL="redis://localhost:6379"
export MINIO_URL="http://localhost:9000"
```

## 🎮 Usage Examples

### 📊 **Basic Demo Analysis**

```bash
# Parse a demo file and extract player snapshots
cargo run -p cs2-demo-parser --bin parser -- path/to/demo.dem

# Analyze crosshair placement patterns
cargo run -p cs2-demo-analyzer -- crosshair-analysis demos/match.dem

# Extract key moments (clutches, aces, multi-kills)
cargo run -p cs2-demo-analyzer -- key-moments demos/match.dem --output moments.json
```

### 🏆 **Pro Player Comparison**

```bash
# Compare your gameplay to a specific pro player
cargo run -p cs2-demo-analyzer -- compare \
  --demo demos/your_match.dem \
  --pro s1mple \
  --metrics aim,positioning,utility

# Find your playstyle similarity to all pros
cargo run -p cs2-demo-analyzer -- similarity-search demos/your_match.dem

# Generate skill gap report
cargo run -p cs2-demo-analyzer -- skill-gap \
  --demo demos/your_match.dem \
  --output skill_report.html
```

### 🤖 **AI Training & Behavior Cloning**

```bash
# Train AI models on professional gameplay
cd cs2-ml
cargo run -- prepare "demos/pro_matches/*.dem" ./training_data
cargo run -- train ./training_data/*.parquet ./models/behavior_model.ot --epochs 1000

# Serve trained model for ephemeral training servers
cargo run -- serve ./models/behavior_model.ot --port 8123

# Generate training scenarios
cargo run -p cs2-demo-analyzer -- generate-scenarios \
  --demo demos/clutch_situations.dem \
  --output training_scenarios/
```

### 📈 **Advanced Analytics**

```bash
# Generate comprehensive match report
cargo run -p cs2-analytics -- match-report \
  --demo demos/tournament_match.dem \
  --output reports/match_analysis.html

# Analyze team coordination patterns
cargo run -p cs2-analytics -- team-analysis \
  --demos "demos/team_matches/*.dem" \
  --team "Team Vitality"

# Track performance trends over time
cargo run -p cs2-analytics -- performance-trends \
  --player "ZywOo" \
  --timerange "2024-01-01:2024-12-31"
```

## 🔬 Development Guide

### 🧪 **Testing Strategy**

The project uses a tiered testing approach optimized for different development phases:

```bash
# Tier 1: Fast unit tests (1-2 minutes)
cargo test --lib --workspace --quiet

# Tier 2: Individual crate tests
cargo test -p cs2-demo-parser        # 15 seconds, 334 tests
cargo test -p cs2-common             # 5 seconds, 11 tests
cargo test -p cs2-data-pipeline      # 45 seconds, integration tests

# Tier 3: Full integration tests (5-15 minutes)
cargo test --workspace --features integration-tests

# Performance benchmarks (10+ minutes)
cargo bench --workspace
```

### 🛠️ **Code Quality & CI**

```bash
# Format code (required for CI)
cargo fmt --all

# Lint with Clippy (required for CI)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Security audit
cargo audit

# Check for dependency issues
cargo deny check
```

### 🏗️ **Build Optimization Tips**

1. **Platform-Specific Builds**:
   ```bash
   # Linux: Use CPU-only features for cs2-ml to avoid Metal issues
   cargo build --workspace --no-default-features
   
   # macOS: Default Metal acceleration works
   cargo build --workspace
   ```

2. **Incremental Development**:
   ```bash
   # Check individual crates during development
   cargo check -p cs2-demo-parser  # Fast iteration
   cargo check -p cs2-ml          # Slower, build when needed
   ```

3. **Memory Management**:
   - Expect 2-8GB RAM usage for large demo processing
   - Use `--release` mode for performance testing
   - Monitor with `cargo flamegraph` for profiling

### 🔧 **Common Issues & Solutions**

| Issue | Platform | Solution |
|-------|----------|----------|
| `protobuf-compiler not found` | All | Install protobuf: `apt-get install protobuf-compiler` or `brew install protobuf` |
| `objc_exception` errors | Linux | Edit `cs2-ml/Cargo.toml`: change `default = ["metal"]` to `default = ["cpu-only"]` |
| Slow builds | All | Use `cargo check` for iteration, `cargo build` only when needed |
| Database connection errors | All | Run `./setup_databases.sh` and wait 5+ minutes for full initialization |
| High memory usage | All | Expected for large demos (50MB+ files), not an error |

## 📈 Performance Expectations

### ⚡ **Processing Performance**
- **Demo parsing**: 700MB+/second on high-end hardware (Ryzen 5900x + NVMe SSD)
- **Database ingestion**: 10K+ player snapshots/second
- **Memory usage**: 2-8GB for large professional match demos
- **ML training**: 15-60 minutes depending on dataset size

### 📊 **Scale Targets**
- **Daily processing**: 50+ professional matches
- **Storage capacity**: 5TB TimescaleDB, 2TB vector embeddings, 20TB archives
- **Concurrent analysis**: Multiple demo processing pipelines
- **API throughput**: Sub-100ms response times for coaching insights

### ⏱️ **Build Times**
| Component | Clean Build | Incremental | Notes |
|-----------|-------------|-------------|-------|
| `cs2-demo-parser` | 10s | 2s | Core parsing engine |
| `cs2-ml` | 2+ min | 30s | PyTorch compilation heavy |
| `cs2-data-pipeline` | 90s | 15s | Database integration |
| Full workspace | 3-5 min | 45s | Complete system |

> **⚠️ Important**: Never cancel long-running builds or tests. The system is designed for high-volume professional esports data processing and requires patience for large operations.

## 🤝 Contributing

We welcome contributions from the community! Here's how to get involved:

### 🔀 **Contribution Workflow**

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Setup** development environment: `./setup_databases.sh`
4. **Make** your changes with appropriate tests
5. **Test** thoroughly: `cargo test --workspace`
6. **Format** and lint: `cargo fmt --all && cargo clippy --workspace`
7. **Commit** with clear messages
8. **Push** and create a Pull Request

### 🧪 **Development Environment**

```bash
# Setup complete development environment
git clone https://github.com/kikokikok/fps-genie.git
cd fps-genie

# Install system dependencies
sudo apt-get install -y build-essential clang protobuf-compiler libfontconfig1-dev

# Setup databases (wait 5-10 minutes for full initialization)
./setup_databases.sh

# Setup enhanced development tools (optional)
./setup_mcp_servers.sh

# Build and test
cargo check --workspace
cargo test --lib --workspace
```

### 📋 **Areas for Contribution**

- **🎯 New Game Support**: Implement Valorant or Apex Legends demo parsing
- **🤖 ML Models**: Improve behavior cloning algorithms or coaching logic
- **📊 Analytics**: Add new performance metrics or visualization features
- **🔧 Performance**: Optimize parsing speed or memory usage
- **📚 Documentation**: Improve guides, examples, or API documentation
- **🧪 Testing**: Add test coverage or integration tests
- **🎮 UI/UX**: Build web interfaces or real-time overlays

### 🏷️ **Issue Labels**

- `good-first-issue`: Perfect for newcomers
- `performance`: Speed or memory optimization
- `ml-enhancement`: Machine learning improvements
- `new-game`: Multi-game support
- `documentation`: Docs and guides
- `bug`: Something isn't working
- `enhancement`: New feature requests

## 📄 License & Acknowledgments

### 📜 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### 🙏 **Acknowledgments**

- **CS2 Community**: For demo files and gameplay insights
- **LaihoE/demoparser**: Inspiration for high-performance demo parsing
- **Professional Players**: Whose gameplay powers our AI training
- **Rust Community**: For excellent crates and tooling
- **TimescaleDB**: For time-series database capabilities
- **Qdrant**: For vector similarity search
- **Contributors**: Everyone who helps improve FPS Genie

### 🔗 **Related Projects**

- [demoparser](https://github.com/LaihoE/demoparser): High-performance CS2 demo parser
- [awpy](https://github.com/pnxenopoulos/awpy): CS2 spatial analysis library
- [TimescaleDB](https://github.com/timescale/timescaledb): Time-series SQL database
- [Qdrant](https://github.com/qdrant/qdrant): Vector similarity search engine

---

<div align="center">

**🎯 Transform your gameplay with AI-powered insights!**

[**📖 Documentation**](docs/) • [**🚀 Quick Start**](#-quick-start) • [**💬 Community**](https://github.com/kikokikok/fps-genie/discussions) • [**🐛 Report Bug**](https://github.com/kikokikok/fps-genie/issues)

</div>
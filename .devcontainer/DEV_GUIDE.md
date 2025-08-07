# CS2 Demo Analysis - Development Setup Guide

This directory contains everything needed to set up a complete local development environment for the CS2 Demo Analysis & AI Training System.

## 🚀 Quick Start

### Option 1: DevContainer (Recommended)
```bash
# Open in VS Code
code .
# Then: Cmd+Shift+P -> "Dev Containers: Reopen in Container"
```

### Option 2: Docker Compose Only
```bash
# Start infrastructure
docker-compose -f docker-compose.dev.yml up -d infrastructure

# Start development environment
docker-compose -f docker-compose.dev.yml up devcontainer
```

## 📋 What Gets Set Up

### 🗄️ Database Layer
- **TimescaleDB**: Complete schema with time-series optimization
  - Player snapshots (high-frequency tick data)
  - Match metadata and tournament structure
  - Behavioral vectors for ML training
  - Key moments detection (clutches, aces, etc.)
- **Redis**: Caching and session management
- **Qdrant**: Vector database for similarity search

### 🔬 Analytics & ML
- **Jupyter Lab**: Interactive data analysis environment
- **Python ML Stack**: pandas, numpy, scikit-learn, matplotlib
- **Behavioral Analysis**: Pre-configured notebooks for player analysis
- **Real-time Processing**: Stream analysis capabilities

### 🛠️ Development Tools
- **Rust Toolchain**: Latest with cargo extensions
- **IDE Integration**: VS Code with Rust, Python, Docker extensions
- **Database Tools**: PostgreSQL client, Redis CLI
- **Monitoring**: Grafana dashboards for system metrics

## 🎯 Available Services

After setup completion:

| Service | URL | Credentials |
|---------|-----|-------------|
| TimescaleDB | `localhost:5432` | `cs2_user` / `cs2_password` |
| Redis | `localhost:6379` | No auth |
| Qdrant | `localhost:6333` | No auth |
| Grafana | `localhost:3001` | `admin` / `admin` |
| Jupyter Lab | `localhost:8888` | Token: `cs2analysis` |

## 📊 Database Schema

The setup creates a comprehensive schema designed for CS2 demo analysis:

### Core Tables
- `tournaments` - Tournament metadata
- `teams` - Team information and rankings  
- `players` - Player profiles and roles
- `matches` - Match metadata and processing status

### Time-Series Data (TimescaleDB)
- `player_snapshots` - High-frequency player state data
- `game_events` - All game events (kills, bomb events, etc.)
- `rounds` - Round-by-round match progression

### ML & Analytics
- `key_moments` - Extracted significant moments (clutches, aces)
- `behavioral_vectors` - ML embeddings for player behavior
- Pre-built views for common analytics queries

## 🎮 Usage Examples

### Demo Processing
```bash
# Analyze a demo file
cargo run --bin cs2-demo-analyzer -- analyze test_data/demo.dem

# Batch process multiple demos
cargo run --bin cs2-data-pipeline -- process --demo-dir test_data

# Stream real-time analysis
cargo run --bin cs2-data-pipeline -- stream --demo-path demo.dem
```

### ML Training
```bash
# Train player behavior model
cargo run --bin cs2-ml -- train --dataset behavioral_vectors

# Generate player insights
cargo run --bin cs2-analytics -- analyze-player --steam-id 76561198034202275

# Find similar players
cargo run --bin cs2-ml -- find-similar-players --target-player s1mple
```

### Interactive Analysis
```bash
# Start Jupyter Lab
./start_jupyter.sh

# Or manually
cd notebooks && jupyter lab --allow-root
```

## 🔧 Development Workflow

### 1. Code Development
```bash
# Watch mode for continuous compilation
cargo watch -x "test --workspace"

# Specific component development  
cd cs2-demo-parser && cargo watch -x "test --lib"

# Auto-format on changes
cargo watch -s "cargo fmt && cargo clippy"
```

### 2. Testing
```bash
# Unit tests
cargo test --workspace

# Integration tests with real database
cargo test --workspace --features integration-tests

# Performance benchmarks
cargo bench --workspace
```

### 3. Database Development
```bash
# Connect to database
psql postgresql://cs2_user:cs2_password@localhost:5432/cs2_analytics

# View processing status
SELECT processing_status, COUNT(*) FROM matches GROUP BY processing_status;

# Check player performance
SELECT * FROM player_stats WHERE nickname = 's1mple';
```

## 📁 Directory Structure

```
.devcontainer/
├── devcontainer.json         # VS Code devcontainer config
├── docker-compose.dev.yml    # Development infrastructure
├── setup.sh                  # Main setup script
├── setup_jupyter.sh          # Jupyter environment setup
├── README.md                 # This file
├── Dockerfile               # Development container image
├── qdrant-config.yaml       # Qdrant configuration
├── grafana/                 # Grafana dashboards & datasources
└── test-data/
    └── sample_data.sql      # Sample data for testing
```

## 🚨 Troubleshooting

### Service Issues
```bash
# Check service health
docker-compose -f docker-compose.dev.yml ps

# View logs
docker-compose -f docker-compose.dev.yml logs timescaledb
docker-compose -f docker-compose.dev.yml logs qdrant

# Restart services
docker-compose -f docker-compose.dev.yml restart
```

### Database Issues
```bash
# Test database connection
pg_isready -h localhost -p 5432 -U cs2_user

# Check TimescaleDB extensions
psql -U cs2_user -d cs2_analytics -c "SELECT * FROM pg_extension;"

# Verify schema
psql -U cs2_user -d cs2_analytics -c "\dt"
```

### Build Issues
```bash
# Clean rebuild
cargo clean && cargo build --workspace

# Check dependencies
cargo tree --duplicates

# Update dependencies
cargo update
```

## 🎯 Performance Targets

The development environment is configured to handle:
- **Demo Processing**: 500MB/s sustained throughput
- **Database Ingestion**: 100k player snapshots/second  
- **ML Training**: Real-time behavioral model training
- **Vector Search**: <50ms similarity queries
- **Analytics**: Interactive dashboard performance

## 📚 Next Steps

1. **Run Sample Analysis**: Start with the provided demo files
2. **Explore Notebooks**: Open Jupyter Lab and try the analysis examples
3. **Review Database**: Check the pre-loaded sample data
4. **Build Features**: Add new analysis capabilities
5. **Train Models**: Experiment with behavioral prediction models

## 🤝 Contributing

The development environment includes:
- Pre-commit hooks for code quality
- Comprehensive test suite with real data
- Performance benchmarking capabilities
- Documentation generation tools

See the main project README for contribution guidelines.
# CS2 Demo Analysis & AI Training System - Setup Guide

## Project Overview
This is a comprehensive CS2 demo analysis and AI training system built in Rust, designed to process professional CS2 matches and provide AI-driven coaching insights. The system implements a three-tier database architecture for scalable data processing and analysis.

## Architecture Components

### Core Rust Crates
- **cs2-demo-parser**: High-performance CS2 demo file parser
- **cs2-ml**: Machine learning models for behavioral analysis and coaching
- **cs2-data-pipeline**: Batch processing pipeline for massive demo ingestion
- **cs2-common**: Shared data structures and utilities
- **cs2-client**: Client library for connecting to training servers
- **cs2-demo-analyzer**: CLI tool for demo analysis
- **csgoproto**: Protocol buffer definitions for CS2

### Database Tiers
1. **PostgreSQL**: Match metadata, player info, tournament data
2. **TimescaleDB**: Time-series player snapshots (millions of ticks per match)
3. **Qdrant**: Vector database for behavioral embeddings and similarity search

### Key Data Models
- **BehavioralVector**: Player state at specific game ticks
- **PlayerSnapshot**: Comprehensive player state (health, position, aim, weapons)
- **Match**: Match metadata and processing status
- **KeyMoment**: Identified significant gameplay situations (clutches, aces)
- **BehavioralEmbedding**: High-dimensional vectors for AI similarity search

## Current Status
- ✅ Core demo parser working (fixed file path issues)
- ✅ ML pipeline implemented with PyTorch integration
- ✅ Database models and pipeline architecture designed
- ✅ Multi-tier database schema created
- ✅ TestContainers integration framework completed
- ✅ E2E testing suite implemented
- ✅ Advanced analytics training pipeline created
- ✅ Performance benchmarking framework
- ✅ Complete local infrastructure setup
- ✅ Database managers for all three tiers (PostgreSQL, TimescaleDB, Qdrant)
- ✅ Batch processing pipeline for massive demo ingestion
- ✅ Visualization and analytics engines
- 🚧 Production deployment configurations
- 🚧 Real demo data collection and processing
- 🚧 Advanced AI model training with professional data

## Development Environment Setup

### Prerequisites
- Rust 1.70+
- Docker & Docker Compose
- PostgreSQL client tools
- Python 3.8+ (for some ML components)

### Quick Start
```bash
# 1. Clone and build
git clone <repo> && cd fps-genie
cargo build

# 2. Setup databases
chmod +x setup_databases.sh
./setup_databases.sh

# 3. Initialize pipeline
cd cs2-data-pipeline
cargo run -- init

# 4. Run tests
cargo test --workspace
```

## Testing Strategy

### Unit Tests
- Demo parser functionality
- ML model training/inference
- Database operations
- Data transformation pipelines

### Integration Tests
- End-to-end demo processing
- Database integration
- ML pipeline integration
- API endpoints

### Performance Tests
- Large demo file processing
- Concurrent pipeline execution
- Database query performance
- Memory usage under load

## Data Pipeline Flow

1. **Demo Discovery**: Scan directories for .dem files
2. **Registration**: Register demos in PostgreSQL with metadata
3. **Parsing**: Extract player snapshots using cs2-demo-parser
4. **Batch Processing**: Insert snapshots into TimescaleDB in batches
5. **Analysis**: Identify key moments and generate behavioral vectors
6. **ML Processing**: Create embeddings and store in Qdrant
7. **Serving**: Provide APIs for coaching insights and pro comparisons

## Key Features (from PDF)

### Pro Player Comparison
- Skill gap analysis against professional players
- Playstyle similarity scoring
- Role recommendation based on gameplay patterns

### AI Coaching
- Real-time crosshair placement correction
- Tactical decision analysis
- Movement and positioning feedback

### Ephemeral Training Servers
- AI behavior cloning from pro demos
- Scenario recreation for specific situations
- Adaptive difficulty scaling

### Multi-Game Support (Planned)
- CS2 (current focus)
- Valorant integration
- Apex Legends support
- Unified analytics across games

## Performance Targets (from PDF)

### Processing
- 700MB+/s demo parsing on high-end PC
- 10K+ player snapshots/second database ingestion
- 4-8 concurrent demo processing jobs

### Storage
- 5TB initial TimescaleDB capacity
- 2TB vector embeddings in Qdrant
- 20TB+ object storage for demo archives

### Scale
- Handle entire professional CS2 scene
- Process 50+ matches daily
- Support 36TB+ annual data growth

## Environment Variables

```bash
# Database connections
export DATABASE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
export TIMESCALE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
export QDRANT_URL="http://localhost:6334"

# Pipeline configuration
export DEMO_DIR="./demos"
export MAX_CONCURRENT_JOBS="4"
export BATCH_SIZE="1000"

# ML configuration
export MODEL_PATH="./models"
export TRAINING_DATA_PATH="./training_data"
```

## Common Commands

```bash
# Pipeline operations
cargo run --bin cs2-data-pipeline -- init
cargo run --bin cs2-data-pipeline -- discover --recursive
cargo run --bin cs2-data-pipeline -- process
cargo run --bin cs2-data-pipeline -- stats

# ML training
cargo run --bin cs2-ml -- train --dataset ./training_data
cargo run --bin cs2-ml -- serve --port 8080

# Demo analysis
cargo run --bin cs2-demo-analyzer -- analyze ./demos/match.dem
cargo run --bin cs2-demo-analyzer -- visualize ./demos/match.dem

# Testing
cargo test --workspace
cargo test --package cs2-demo-parser
cargo test --package cs2-ml --features=integration
```

## Troubleshooting

### Common Issues
1. **Demo parsing fails**: Check file paths in e2e tests (fixed)
2. **Database connection**: Verify Docker containers are running
3. **Memory issues**: Reduce batch size or concurrent jobs
4. **PyTorch errors**: Ensure compatible torch-sys version

### Performance Tuning
1. **Batch size**: Adjust based on available RAM
2. **Concurrency**: Match to CPU cores (4-8 typical)
3. **Database**: Tune TimescaleDB chunk intervals
4. **Vector DB**: Optimize Qdrant collection settings

## File Structure Context

```
fps-genie/
├── cs2-demo-parser/          # Core demo parsing (working)
├── cs2-ml/                   # ML models and training (working)
├── cs2-data-pipeline/        # Batch processing pipeline (new)
├── cs2-common/               # Shared data structures
├── cs2-client/               # Client library
├── cs2-demo-analyzer/        # CLI analysis tool
├── csgoproto/                # Protocol buffer definitions
├── test_data/                # Sample demo files
├── setup_databases.sh        # Infrastructure setup
└── docker-compose.yml        # Generated by setup script
```

## Next Steps

1. **Local E2E Testing**: Set up complete local testing environment
2. **TestContainers**: Implement Docker-based integration tests
3. **Advanced Analytics**: Build ML training pipelines
4. **Performance Benchmarking**: Establish baseline metrics
5. **Production Deployment**: Kubernetes deployment configs

## Contact/Sessions
- Current session focus: Infrastructure setup and E2E testing
- Use this context for future sessions with other agents
- Key working components: demo parser, ML pipeline, database schemas
- Main challenges: TestContainers integration, advanced analytics training

## Infrastructure Components (Newly Added)

### TestContainers Integration (`cs2-integration-tests/`)
- **Automated test infrastructure**: Docker-based testing with PostgreSQL, TimescaleDB, Qdrant, and Redis
- **E2E test suites**: Complete pipeline testing from demo discovery to database storage
- **Performance benchmarking**: Criterion-based benchmarks for database operations and ML processing
- **Test data factories**: Realistic professional player scenario generation

### Advanced Analytics Pipeline (`cs2-analytics/`)
- **Behavior cloning training**: ML models for professional player action prediction
- **Crosshair placement optimization**: Real-time aim correction algorithms
- **Tactical decision analysis**: Strategic gameplay pattern recognition
- **Visualization engine**: Heatmaps, trajectory plots, performance radar charts

### Database Infrastructure
- **Complete schema design**: Optimized for high-volume professional demo data
- **Batch processing**: 10K+ snapshots/second ingestion capability
- **Vector similarity search**: Behavioral pattern matching using Qdrant
- **Time-series optimization**: TimescaleDB partitioning for massive datasets

### Setup and Automation
- **complete_setup.sh**: End-to-end infrastructure deployment script
- **Docker Compose**: Multi-service database orchestration
- **Automated testing**: CI/CD ready test suites with performance validation

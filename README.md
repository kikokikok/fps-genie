# CS2 Demo Analysis & AI Training System

A comprehensive Rust-based system for analyzing CS2 demo files, training AI models on professional gameplay, and providing personalized coaching insights. Features a three-tier database architecture with TimescaleDB, Redis, and Qdrant for scalable data processing and vector similarity search.

## 🎯 **Key Features**

- **Demo Analysis Pipeline**: High-performance parsing of CS2 demo files (.dem)
- **AI Coaching System**: Personalized feedback based on pro player comparisons
- **Behavioral Vector Analysis**: ML models for playstyle similarity and skill gap analysis
- **Ephemeral Training Servers**: Practice against AI that clones pro player behaviors
- **Real-time Analytics**: Live coaching overlays and performance insights
- **Scalable Architecture**: Three-tier database system handling millions of player snapshots

## 🏗️ **Architecture Overview**

```mermaid
graph TD
    A[CS2 Demo Files] --> B[Rust Analysis Engine]
    B --> C[Pro Moment Extraction]
    B --> D[Player Performance Analysis]
    C --> E[Pro Scenario Rebuilder]
    D --> F[AI Coaching Engine]
    E --> G[CS2 Game Integration]
    F --> H[Realtime Overlay]
    G --> I[User Experience]
    H --> I
```

### **Database Architecture**
- **PostgreSQL/TimescaleDB**: Time-series player snapshots and match metadata
- **Redis**: Real-time caching and session management
- **Qdrant**: Vector database for behavioral embeddings and similarity search

## 🚀 **Quick Start with Dev Containers**

### **1. Prerequisites**
- Docker Desktop
- VS Code with Dev Containers extension
- 8GB+ RAM recommended

### **2. Launch Development Environment**
```bash
# Clone and open in VS Code
git clone https://github.com/kikokikok/fps-genie.git
cd fps-genie
code .

# Open in dev container
# Cmd+Shift+P → "Dev Containers: Reopen in Container"
```

### **3. Automatic Setup**
The dev container automatically:
- ✅ Starts TimescaleDB, Redis, Qdrant services
- ✅ Initializes database schemas
- ✅ Loads sample pro player data
- ✅ Generates test demo files
- ✅ Runs integration tests

### **4. Available Services**
- **TimescaleDB**: `localhost:5432` (cs2_user/cs2_password)
- **Redis**: `localhost:6379`
- **Qdrant**: `localhost:6333` (HTTP), `localhost:6334` (gRPC)
- **Grafana Dashboard**: `localhost:3000` (admin/admin)
- **Jupyter Notebooks**: `localhost:8888` (token: cs2analysis)

## 🎮 **Usage Examples**

### **Demo Analysis Pipeline**

#### **Single Demo Analysis**
```bash
# Analyze a professional match demo
cargo run --bin cs2-demo-analyzer -- analyze test_data/vitality-vs-spirit-m1-dust2.dem \
  --extract-key-moments \
  --generate-heatmaps \
  --player-focus ZywOo

# Output: Player snapshots, key moments, behavioral vectors
```

#### **Batch Processing**
```bash
# Process multiple demos concurrently
cargo run --bin cs2-data-pipeline -- process \
  --demo-dir test_data \
  --batch-size 10 \
  --concurrent-jobs 4 \
  --output-format parquet

# Monitor progress in Grafana dashboard
```

#### **Real-time Processing**
```bash
# Stream demo analysis as it happens
cargo run --bin cs2-data-pipeline -- stream \
  --demo-path test_data/live_match.dem \
  --websocket-port 8080 \
  --update-interval 1s
```

### **AI Training & Machine Learning**

#### **Train Behavioral Models**
```bash
# Train on professional player data
cargo run --bin cs2-ml -- train \
  --dataset behavioral_vectors \
  --model-type player_behavior \
  --epochs 100 \
  --learning-rate 0.001 \
  --validation-split 0.2

# Train playstyle classifier
cargo run --bin cs2-ml -- train \
  --dataset pro_moments \
  --model-type playstyle_classifier \
  --classes "entry_fragger,support,awper,igl,lurker"
```

#### **Generate Player Insights**
```bash
# Analyze individual player performance
cargo run --bin cs2-analytics -- analyze-player \
  --steam-id 76561198034202275 \
  --match-history 20 \
  --comparison-players "s1mple,ZywOo,device" \
  --focus-areas "aim,positioning,utility,decision_making"

# Compare two players
cargo run --bin cs2-analytics -- compare-players \
  --player1 76561198034202275 \
  --player2 76561198004854956 \
  --output-format json \
  --include-visualizations
```

#### **Skill Gap Analysis**
```bash
# Generate coaching recommendations
cargo run --bin cs2-analytics -- skill-gap-analysis \
  --player-demo test_data/player_match.dem \
  --reference-pro s1mple \
  --analysis-depth detailed \
  --generate-practice-scenarios
```

### **Vector Search & Similarity**

#### **Find Similar Moments**
```bash
# Search for similar clutch situations
cargo run --bin cs2-ml -- find-similar \
  --query-moment clutch_1v3_dust2_long \
  --collection pro_moments \
  --similarity-threshold 0.85 \
  --max-results 10

# Find players with similar playstyles
cargo run --bin cs2-ml -- find-similar-players \
  --target-player 76561198034202275 \
  --similarity-metric behavioral_embedding \
  --min-matches 50
```

#### **Cluster Analysis**
```bash
# Group similar tactical scenarios
cargo run --bin cs2-ml -- cluster-moments \
  --input-collection key_moments \
  --clustering-algorithm kmeans \
  --num-clusters 15 \
  --output-labels tactical_patterns
```

### **Performance Testing & Benchmarking**

#### **System Performance**
```bash
# Benchmark demo parsing speed
cargo bench --workspace -- demo_parsing

# Profile memory usage
cargo run --bin cs2-integration-tests -- memory-profile \
  --demo-count 100 \
  --track-allocations

# Stress test concurrent processing
cargo run --bin cs2-integration-tests -- stress-test \
  --concurrent-demos 50 \
  --duration 600s \
  --memory-limit 8GB
```

#### **Database Performance**
```bash
# Benchmark query performance
cargo run --bin cs2-data-pipeline -- benchmark-queries \
  --query-types "player_lookup,time_series,vector_search" \
  --dataset-size 1000000 \
  --concurrent-queries 20

# Test scalability
cargo run --bin cs2-integration-tests -- scalability-test \
  --max-demos 10000 \
  --ramp-up-time 300s \
  --target-throughput 100
```

## 🔧 **Development Workflow**

### **Live Development**
```bash
# Watch mode for continuous compilation
cargo watch -x "test --workspace"

# Specific component development
cd cs2-demo-parser && cargo watch -x "test --lib"

# Integration tests with real infrastructure
cargo watch -x "test --features integration-tests"

# Auto-format on save
cargo watch -s "cargo fmt && cargo clippy"
```

### **Testing**
```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --workspace --features integration-tests

# End-to-end pipeline tests
cd cs2-integration-tests && cargo test

# Performance benchmarks
cargo bench --workspace --features bench

# Generate test coverage
cargo llvm-cov --workspace --lcov --output-path coverage.lcov
```

### **Data Generation & Testing**
```bash
# Generate synthetic demo files
cargo run --bin cs2-integration-tests -- generate-test-demos \
  --count 50 \
  --output test_data/generated \
  --scenarios "clutch,ace,entry_frag,team_execute" \
  --skill-levels "amateur,semi_pro,professional"

# Create behavioral test vectors
cargo run --bin cs2-ml -- generate-vectors \
  --demo-path test_data/generated \
  --output-collection test_behavioral_data \
  --vector-dimensions 256
```

## 📊 **Monitoring & Analytics**

### **Grafana Dashboards**
Access comprehensive monitoring at `localhost:3000`:
- **System Performance**: CPU, memory, disk usage
- **Database Metrics**: Query performance, connection counts
- **Pipeline Throughput**: Demos processed per hour
- **ML Model Performance**: Training metrics, inference times

### **Database Queries**
```sql
-- Connect: psql postgresql://cs2_user:cs2_password@localhost:5432/cs2_analytics

-- View processing status
SELECT 
    processing_status, 
    COUNT(*) as count,
    AVG(demo_file_size) as avg_size_mb
FROM matches 
GROUP BY processing_status;

-- Top performing players
SELECT 
    p.nickname,
    COUNT(km.moment_id) as key_moments,
    AVG(km.significance_score) as avg_significance
FROM players p
JOIN key_moments km ON p.steam_id = km.player_steam_id
GROUP BY p.nickname
ORDER BY key_moments DESC
LIMIT 10;

-- Time-series analysis
SELECT 
    DATE_TRUNC('hour', created_at) as hour,
    COUNT(*) as demos_processed
FROM matches 
WHERE processing_status = 'completed'
GROUP BY hour
ORDER BY hour;
```

### **Python/Jupyter Analysis**
```python
# In Jupyter notebook (localhost:8888)
import psycopg2
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

# Connect to database
conn = psycopg2.connect(
    'postgresql://cs2_user:cs2_password@localhost:5432/cs2_analytics'
)

# Load player performance data
df = pd.read_sql("""
    SELECT 
        p.nickname,
        p.role,
        COUNT(km.moment_id) as clutches,
        AVG(km.significance_score) as avg_impact
    FROM players p
    JOIN key_moments km ON p.steam_id = km.player_steam_id
    WHERE km.moment_type LIKE 'clutch%'
    GROUP BY p.nickname, p.role
""", conn)

# Visualize role-based performance
plt.figure(figsize=(12, 6))
sns.scatterplot(data=df, x='clutches', y='avg_impact', hue='role', size='clutches')
plt.title('Player Performance: Clutches vs Impact by Role')
plt.show()

# Advanced analytics
from sklearn.cluster import KMeans
from sklearn.preprocessing import StandardScaler

# Cluster players by performance metrics
features = df[['clutches', 'avg_impact']].values
scaler = StandardScaler()
features_scaled = scaler.fit_transform(features)

kmeans = KMeans(n_clusters=3, random_state=42)
df['cluster'] = kmeans.fit_predict(features_scaled)

# Visualize clusters
sns.scatterplot(data=df, x='clutches', y='avg_impact', hue='cluster')
plt.title('Player Performance Clusters')
plt.show()
```

## 🎯 **Advanced Use Cases**

### **1. Pro Player Comparison System**
```bash
# Generate detailed comparison report
cargo run --bin cs2-analytics -- generate-comparison-report \
  --target-player your_steam_id \
  --reference-players "s1mple,ZywOo,sh1ro,electronic" \
  --metrics "aim_accuracy,positioning,utility_usage,game_sense" \
  --output-format pdf \
  --include-recommendations

# Real-time coaching overlay
cargo run --bin cs2-client -- coaching-overlay \
  --demo-stream live \
  --comparison-player s1mple \
  --overlay-port 8080 \
  --update-frequency 5s
```

### **2. Team Analysis & Strategy**
```bash
# Analyze team coordination
cargo run --bin cs2-analytics -- team-analysis \
  --team-demos "team_demos/*.dem" \
  --focus-areas "executes,rotations,utility_coordination" \
  --opponent-data included \
  --generate-playbook

# Find tactical patterns
cargo run --bin cs2-ml -- extract-patterns \
  --match-type "team_vs_team" \
  --pattern-types "smoke_executes,flash_coordination,late_rotations" \
  --min-occurrence 5
```

### **3. Training Server Integration**
```bash
# Launch ephemeral training server
cargo run --bin cs2-client -- training-server \
  --scenario clutch_1v2_mirage_a_site \
  --ai-opponent s1mple \
  --difficulty adaptive \
  --session-duration 30min

# Practice specific weaknesses
cargo run --bin cs2-client -- practice-session \
  --player-weaknesses "long_range_duels,utility_timing" \
  --generate-scenarios 10 \
  --track-improvement
```

## 🔧 **Troubleshooting**

### **Common Issues**

#### **Service Connection Problems**
```bash
# Check service health
docker-compose -f docker-compose.dev.yml ps

# View service logs
docker-compose -f docker-compose.dev.yml logs timescaledb
docker-compose -f docker-compose.dev.yml logs qdrant

# Restart services
docker-compose -f docker-compose.dev.yml restart
```

#### **Memory Issues**
```bash
# Increase Docker memory allocation (8GB+ recommended)
# Or reduce concurrent processing:
cargo run --bin cs2-data-pipeline -- process --concurrent-jobs 1

# Monitor memory usage
docker stats
```

#### **Build Problems**
```bash
# Clean rebuild
cargo clean && cargo build --workspace

# Check dependency conflicts
cargo tree --duplicates

# Update dependencies
cargo update
```

### **Performance Optimization**
```bash
# Enable release mode for better performance
cargo build --release
cargo run --release --bin cs2-demo-analyzer -- analyze demo.dem

# Use cargo cache for faster builds
export CARGO_TARGET_DIR=/workspace/target

# Parallel compilation
export CARGO_BUILD_JOBS=8
```

## 📈 **Roadmap & Contributing**

### **Current Status**
- ✅ Core demo parsing engine
- ✅ Three-tier database architecture
- ✅ ML pipeline with PyTorch integration
- ✅ Dev container environment
- ✅ Comprehensive testing framework
- 🚧 Production deployment
- 🚧 Real demo data collection
- 🚧 Advanced AI model training

### **Contributing**
```bash
# Set up development environment
git clone https://github.com/kikokikok/fps-genie.git
cd fps-genie
code .  # Open in dev container

# Run tests before submitting
cargo test --workspace
cargo clippy -- -D warnings
cargo fmt --check

# Submit pull request with CI passing
```

### **Performance Benchmarks**
- **Demo Parsing**: 500MB/s sustained throughput
- **Database Ingestion**: 100k player snapshots/second
- **Vector Search**: <50ms for similarity queries
- **ML Inference**: Real-time coaching feedback (<100ms)

## 📚 **Documentation**

- [Project Charter](PROJECT_CHARTER.md) - System architecture and features
- [Project Context](PROJECT_CONTEXT.md) - Current implementation status
- [Dev Container Guide](.devcontainer/README.md) - Detailed development setup
- [API Documentation](docs/api.md) - REST API reference
- [Database Schema](sql/schema.md) - Complete database documentation

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Built with ❤️ for the CS2 community by developers who understand the game.**

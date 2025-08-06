# CS2 Demo Analysis Dev Container Setup

## Overview
This dev container provides a complete local development environment for the CS2 Demo Analysis & AI Training System with fake infrastructure for testing. It includes all necessary services and tools to develop, test, and debug the entire system locally.

## 🏗️ Architecture
The dev container setup includes:

### Core Services
- **TimescaleDB**: Time-series database for player snapshots and tick data
- **Redis**: Caching layer for real-time data and session management  
- **Qdrant**: Vector database for behavioral embeddings and similarity search
- **Grafana**: Analytics dashboard for monitoring and visualization

### Development Tools
- **Rust toolchain** with cargo extensions (watch, audit, llvm-cov, criterion)
- **Python environment** for ML experimentation with PyTorch, pandas, numpy
- **Jupyter Notebooks** for data analysis and model prototyping
- **PostgreSQL client tools** for database management

## 🚀 Quick Start

### 1. Open in Dev Container
```bash
# Using VS Code
code .
# Then: Cmd+Shift+P -> "Dev Containers: Reopen in Container"

# Or using Dev Containers CLI
devcontainer up --workspace-folder .
```

### 2. Verify Setup
The setup script automatically:
- Initializes all database schemas
- Creates Qdrant vector collections
- Generates test demo data
- Runs integration tests

### 3. Available Services
After startup, you'll have access to:
- **TimescaleDB**: `localhost:5432` (cs2_user/cs2_password)
- **Redis**: `localhost:6379`  
- **Qdrant**: `localhost:6333` (HTTP), `localhost:6334` (gRPC)
- **Grafana**: `localhost:3000` (admin/admin)
- **Jupyter**: `localhost:8888` (token: cs2analysis)

## 🎮 Complete Usage Guide

### Demo Analysis Workflow

#### Single Demo Analysis
```bash
# Analyze a professional match with key moment extraction
cargo run --bin cs2-demo-analyzer -- analyze test_data/vitality-vs-spirit-m1-dust2.dem \
  --extract-key-moments \
  --generate-heatmaps \
  --player-focus ZywOo

# Quick analysis with basic stats
cargo run --bin cs2-demo-analyzer -- analyze test_data/test_demo.dem --format json
```

#### Batch Processing
```bash
# Process multiple demos concurrently
cargo run --bin cs2-data-pipeline -- process \
  --demo-dir test_data \
  --batch-size 10 \
  --concurrent-jobs 4 \
  --output-format parquet

# Monitor batch processing progress
cargo run --bin cs2-data-pipeline -- status --show-progress
```

#### Real-time Stream Processing
```bash
# Stream demo analysis as it happens
cargo run --bin cs2-data-pipeline -- stream \
  --demo-path test_data/live_match.dem \
  --websocket-port 8080 \
  --update-interval 1s
```

### AI Training & Machine Learning

#### Train Behavioral Models
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

# Train with custom parameters
cargo run --bin cs2-ml -- train \
  --config configs/advanced_training.toml \
  --gpu-acceleration \
  --distributed
```

#### Generate Player Insights
```bash
# Comprehensive player analysis
cargo run --bin cs2-analytics -- analyze-player \
  --steam-id 76561198034202275 \
  --match-history 20 \
  --comparison-players "s1mple,ZywOo,device" \
  --focus-areas "aim,positioning,utility,decision_making"

# Compare two specific players
cargo run --bin cs2-analytics -- compare-players \
  --player1 76561198034202275 \
  --player2 76561198004854956 \
  --output-format json \
  --include-visualizations

# Generate coaching recommendations
cargo run --bin cs2-analytics -- skill-gap-analysis \
  --player-demo test_data/player_match.dem \
  --reference-pro s1mple \
  --analysis-depth detailed \
  --generate-practice-scenarios
```

### Vector Search & Similarity Analysis

#### Find Similar Game Moments
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

# Search by tactical pattern
cargo run --bin cs2-ml -- search-patterns \
  --pattern-type "smoke_execute_mirage_a" \
  --team-context included \
  --effectiveness-threshold 0.7
```

#### Cluster Analysis
```bash
# Group similar tactical scenarios
cargo run --bin cs2-ml -- cluster-moments \
  --input-collection key_moments \
  --clustering-algorithm kmeans \
  --num-clusters 15 \
  --output-labels tactical_patterns

# Cluster player behaviors
cargo run --bin cs2-ml -- cluster-players \
  --feature-set "aim,movement,utility,positioning" \
  --min-cluster-size 5 \
  --export-results clusters.json
```

## 🔧 Development Workflow

### Live Development with Auto-Reload
```bash
# Watch mode for continuous compilation
cargo watch -x "test --workspace"

# Specific component development
cd cs2-demo-parser && cargo watch -x "test --lib"

# Integration tests with real infrastructure
cargo watch -x "test --features integration-tests"

# Auto-format and lint on changes
cargo watch -s "cargo fmt && cargo clippy -- -D warnings"

# Watch with custom command
cargo watch -x "run --bin cs2-demo-analyzer -- analyze test_data/test_demo.dem"
```

### Comprehensive Testing

#### Unit and Integration Tests
```bash
# Run all tests
cargo test --workspace

# Integration tests with database
cargo test --workspace --features integration-tests

# End-to-end pipeline tests
cd cs2-integration-tests && cargo test

# Test specific modules
cargo test --package cs2-demo-parser --lib parsing_tests
cargo test --package cs2-ml --lib model_training_tests
```

#### Performance Benchmarks
```bash
# Run all benchmarks
cargo bench --workspace --features bench

# Specific benchmarks
cargo bench --package cs2-demo-parser -- demo_parsing
cargo bench --package cs2-ml -- vector_similarity

# Generate benchmark report
cargo bench --workspace -- --output-format html --output-dir bench_results
```

#### Coverage Analysis
```bash
# Generate test coverage report
cargo llvm-cov --workspace --lcov --output-path coverage.lcov

# Coverage with integration tests
cargo llvm-cov --workspace --features integration-tests --html
```

### Data Generation & Testing Scenarios

#### Generate Synthetic Test Data
```bash
# Create synthetic demo files with various scenarios
cargo run --bin cs2-integration-tests -- generate-test-demos \
  --count 50 \
  --output test_data/generated \
  --scenarios "clutch,ace,entry_frag,team_execute" \
  --skill-levels "amateur,semi_pro,professional"

# Generate specific tactical scenarios
cargo run --bin cs2-integration-tests -- generate-scenarios \
  --scenario-types "1v1_duels,utility_executes,retake_situations" \
  --maps "de_dust2,de_mirage,de_inferno" \
  --count-per-scenario 10
```

#### Create Test Vectors
```bash
# Generate behavioral test vectors
cargo run --bin cs2-ml -- generate-vectors \
  --demo-path test_data/generated \
  --output-collection test_behavioral_data \
  --vector-dimensions 256

# Create training datasets
cargo run --bin cs2-ml -- create-dataset \
  --source-demos test_data/pro_matches \
  --output-format parquet \
  --split-ratio "0.7,0.2,0.1"  # train,val,test
```

## 🎯 Advanced Use Cases

### 1. Pro Player Comparison System
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

### 2. Team Analysis & Strategy
```bash
# Analyze team coordination patterns
cargo run --bin cs2-analytics -- team-analysis \
  --team-demos "team_demos/*.dem" \
  --focus-areas "executes,rotations,utility_coordination" \
  --opponent-data included \
  --generate-playbook

# Extract tactical patterns
cargo run --bin cs2-ml -- extract-patterns \
  --match-type "team_vs_team" \
  --pattern-types "smoke_executes,flash_coordination,late_rotations" \
  --min-occurrence 5 \
  --export-to-training-data
```

### 3. Training Server Integration
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

# Custom training scenarios
cargo run --bin cs2-client -- custom-scenario \
  --map de_dust2 \
  --situation "1v3_retake_b_site" \
  --opponent-skill professional \
  --iterations 20
```

## 📊 Monitoring & Analytics

### Grafana Dashboard Usage
Access comprehensive monitoring at `localhost:3000`:

#### System Performance Metrics
- **CPU Usage**: Monitor processing load during demo analysis
- **Memory Usage**: Track memory consumption during ML training
- **Disk I/O**: Database write performance and storage usage
- **Network**: Redis and Qdrant communication patterns

#### Application-Specific Dashboards
- **Demo Processing**: Throughput, error rates, processing times
- **ML Pipeline**: Training progress, model accuracy, inference times
- **Database Performance**: Query execution times, connection pools
- **Vector Search**: Similarity query performance, index statistics

### Database Analysis

#### Direct Database Queries
```sql
-- Connect via: psql postgresql://cs2_user:cs2_password@localhost:5432/cs2_analytics

-- Processing status overview
SELECT 
    processing_status, 
    COUNT(*) as count,
    ROUND(AVG(demo_file_size)/1024/1024, 2) as avg_size_mb,
    MIN(created_at) as oldest,
    MAX(created_at) as newest
FROM matches 
GROUP BY processing_status
ORDER BY count DESC;

-- Top performing players by key moments
SELECT 
    p.nickname,
    p.role,
    COUNT(km.moment_id) as total_key_moments,
    COUNT(CASE WHEN km.moment_type LIKE 'clutch%' THEN 1 END) as clutches,
    COUNT(CASE WHEN km.moment_type = 'ace' THEN 1 END) as aces,
    ROUND(AVG(km.significance_score), 3) as avg_significance
FROM players p
JOIN key_moments km ON p.steam_id = km.player_steam_id
GROUP BY p.nickname, p.role
HAVING COUNT(km.moment_id) > 5
ORDER BY total_key_moments DESC, avg_significance DESC
LIMIT 20;

-- Time-series analysis of player performance
SELECT 
    DATE_TRUNC('day', m.match_date) as match_day,
    COUNT(DISTINCT m.match_id) as matches_played,
    COUNT(km.moment_id) as key_moments,
    ROUND(AVG(km.significance_score), 3) as avg_impact
FROM matches m
JOIN key_moments km ON m.match_id = km.match_id
WHERE m.match_date >= NOW() - INTERVAL '30 days'
GROUP BY match_day
ORDER BY match_day;

-- Map performance analysis
SELECT 
    map_name,
    COUNT(*) as total_matches,
    ROUND(AVG(team1_score + team2_score), 1) as avg_total_rounds,
    COUNT(CASE WHEN ABS(team1_score - team2_score) <= 2 THEN 1 END) as close_matches
FROM matches
WHERE processing_status = 'completed'
GROUP BY map_name
ORDER BY total_matches DESC;
```

#### Redis Analytics
```bash
# Connect to Redis
redis-cli -h localhost -p 6379

# Check cache performance
INFO stats
INFO memory

# View active keys
KEYS pattern:*
SCAN 0 MATCH player:* COUNT 100

# Monitor real-time operations
MONITOR
```

#### Qdrant Vector Database
```bash
# Check collection status
curl http://localhost:6333/collections

# Get collection info
curl http://localhost:6333/collections/behavioral_vectors

# Search vectors
curl -X POST http://localhost:6333/collections/behavioral_vectors/points/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.8, 0.3, 0.9, 0.2, 0.7, 0.4, 0.6],
    "limit": 5,
    "with_payload": true
  }'

# Collection statistics
curl http://localhost:6333/collections/behavioral_vectors/cluster
```

### Python/Jupyter Data Analysis
```python
# In Jupyter notebook (localhost:8888)
import psycopg2
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from sklearn.cluster import KMeans
from sklearn.preprocessing import StandardScaler
from sklearn.decomposition import PCA

# Database connection
conn = psycopg2.connect(
    'postgresql://cs2_user:cs2_password@localhost:5432/cs2_analytics'
)

# Load comprehensive player data
player_stats = pd.read_sql("""
    SELECT 
        p.nickname,
        p.role,
        p.team_id,
        t.name as team_name,
        COUNT(DISTINCT m.match_id) as matches_played,
        COUNT(km.moment_id) as key_moments,
        COUNT(CASE WHEN km.moment_type LIKE 'clutch%' THEN 1 END) as clutches,
        COUNT(CASE WHEN km.moment_type = 'ace' THEN 1 END) as aces,
        COUNT(CASE WHEN km.moment_type = 'entry_frag' THEN 1 END) as entry_frags,
        ROUND(AVG(km.significance_score), 3) as avg_significance
    FROM players p
    LEFT JOIN teams t ON p.team_id = t.id
    LEFT JOIN key_moments km ON p.steam_id = km.player_steam_id
    LEFT JOIN matches m ON km.match_id = m.match_id
    GROUP BY p.nickname, p.role, p.team_id, t.name
    HAVING COUNT(DISTINCT m.match_id) > 0
""", conn)

# Advanced visualizations
fig, axes = plt.subplots(2, 2, figsize=(16, 12))

# 1. Role-based performance scatter
sns.scatterplot(data=player_stats, x='key_moments', y='avg_significance', 
                hue='role', size='matches_played', ax=axes[0,0])
axes[0,0].set_title('Player Performance by Role')

# 2. Team comparison
team_performance = player_stats.groupby('team_name').agg({
    'key_moments': 'sum',
    'avg_significance': 'mean',
    'clutches': 'sum',
    'matches_played': 'sum'
}).reset_index()

sns.barplot(data=team_performance, x='team_name', y='key_moments', ax=axes[0,1])
axes[0,1].set_title('Total Key Moments by Team')
axes[0,1].tick_params(axis='x', rotation=45)

# 3. Player clustering analysis
features = ['key_moments', 'clutches', 'aces', 'entry_frags', 'avg_significance']
X = player_stats[features].fillna(0)
scaler = StandardScaler()
X_scaled = scaler.fit_transform(X)

# Apply PCA for visualization
pca = PCA(n_components=2)
X_pca = pca.fit_transform(X_scaled)

# K-means clustering
kmeans = KMeans(n_clusters=4, random_state=42)
player_stats['cluster'] = kmeans.fit_predict(X_scaled)

scatter = axes[1,0].scatter(X_pca[:, 0], X_pca[:, 1], c=player_stats['cluster'], 
                           cmap='viridis', alpha=0.7)
axes[1,0].set_title('Player Performance Clusters (PCA)')
axes[1,0].set_xlabel(f'PC1 ({pca.explained_variance_ratio_[0]:.2%} variance)')
axes[1,0].set_ylabel(f'PC2 ({pca.explained_variance_ratio_[1]:.2%} variance)')

# 4. Performance correlation heatmap
correlation_matrix = player_stats[features].corr()
sns.heatmap(correlation_matrix, annot=True, cmap='coolwarm', center=0, ax=axes[1,1])
axes[1,1].set_title('Performance Metrics Correlation')

plt.tight_layout()
plt.show()

# Detailed cluster analysis
print("Cluster Analysis:")
for cluster in sorted(player_stats['cluster'].unique()):
    cluster_data = player_stats[player_stats['cluster'] == cluster]
    print(f"\nCluster {cluster} ({len(cluster_data)} players):")
    print(f"  Avg Key Moments: {cluster_data['key_moments'].mean():.1f}")
    print(f"  Avg Significance: {cluster_data['avg_significance'].mean():.3f}")
    print(f"  Top Players: {', '.join(cluster_data.nlargest(3, 'key_moments')['nickname'].tolist())}")

# Time series analysis
match_timeline = pd.read_sql("""
    SELECT 
        DATE_TRUNC('week', match_date) as week,
        COUNT(*) as matches,
        COUNT(CASE WHEN processing_status = 'completed' THEN 1 END) as processed,
        ROUND(AVG(demo_file_size)/1024/1024, 1) as avg_size_mb
    FROM matches
    WHERE match_date >= NOW() - INTERVAL '12 weeks'
    GROUP BY week
    ORDER BY week
""", conn)

plt.figure(figsize=(12, 6))
plt.subplot(1, 2, 1)
plt.plot(match_timeline['week'], match_timeline['matches'], marker='o', label='Total Matches')
plt.plot(match_timeline['week'], match_timeline['processed'], marker='s', label='Processed')
plt.title('Match Processing Over Time')
plt.legend()
plt.xticks(rotation=45)

plt.subplot(1, 2, 2)
plt.plot(match_timeline['week'], match_timeline['avg_size_mb'], marker='o', color='green')
plt.title('Average Demo File Size Trend')
plt.ylabel('Size (MB)')
plt.xticks(rotation=45)

plt.tight_layout()
plt.show()
```

## 🎯 Performance Testing & Benchmarking

### System Performance Tests
```bash
# Comprehensive system benchmark
cargo run --bin cs2-integration-tests -- full-system-benchmark \
  --demo-count 100 \
  --concurrent-jobs 8 \
  --duration 600s \
  --memory-limit 8GB \
  --report-format detailed

# Demo parsing performance
cargo bench --package cs2-demo-parser -- parsing_speed \
  --bench-args="--sample-size 50"

# ML inference benchmarks
cargo bench --package cs2-ml -- inference_performance \
  --bench-args="--measurement-time 60"

# Database performance stress test
cargo run --bin cs2-data-pipeline -- stress-test-db \
  --insert-rate 10000 \
  --query-rate 1000 \
  --duration 300s \
  --connection-pool-size 20
```

### Memory and Resource Profiling
```bash
# Memory usage profiling
cargo run --bin cs2-integration-tests -- memory-profile \
  --demo-count 50 \
  --track-allocations \
  --heap-profiling \
  --output-format flame-graph

# CPU profiling
cargo run --bin cs2-demo-analyzer -- analyze test_data/vitality-vs-spirit-m1-dust2.dem \
  --profile-cpu \
  --profile-output cpu_profile.pb

# I/O performance analysis
cargo run --bin cs2-data-pipeline -- io-benchmark \
  --test-sequential-read \
  --test-random-read \
  --test-write-throughput \
  --file-sizes "1MB,10MB,100MB"
```

### Scalability Testing
```bash
# Horizontal scaling simulation
cargo run --bin cs2-integration-tests -- scaling-test \
  --max-demos 10000 \
  --ramp-up-time 300s \
  --target-throughput 100 \
  --measure-latency \
  --measure-memory

# Database scalability
cargo run --bin cs2-data-pipeline -- db-scaling-test \
  --max-connections 100 \
  --query-complexity high \
  --data-volume 1000000 \
  --measure-degradation
```

## 🚨 Troubleshooting & Debugging

### Service Health Monitoring
```bash
# Check all services status
docker-compose -f docker-compose.dev.yml ps

# Detailed service health
docker-compose -f docker-compose.dev.yml exec timescaledb pg_isready -U cs2_user
docker-compose -f docker-compose.dev.yml exec redis redis-cli ping
curl -f http://localhost:6333/health

# Service resource usage
docker stats cs2-timescaledb cs2-redis cs2-qdrant

# Service logs
docker-compose -f docker-compose.dev.yml logs --tail=100 timescaledb
docker-compose -f docker-compose.dev.yml logs --tail=100 qdrant
docker-compose -f docker-compose.dev.yml logs --follow redis
```

### Common Issues & Solutions

#### Memory Issues
```bash
# Increase Docker memory (8GB+ recommended)
# Check Docker Desktop settings

# Monitor memory usage
free -h
docker stats

# Reduce memory usage in processing
cargo run --bin cs2-data-pipeline -- process \
  --concurrent-jobs 1 \
  --batch-size 5 \
  --memory-limit 2GB
```

#### Database Connection Issues
```bash
# Check PostgreSQL connections
psql postgresql://cs2_user:cs2_password@localhost:5432/cs2_analytics \
  -c "SELECT count(*) FROM pg_stat_activity;"

# Reset database if needed
docker-compose -f docker-compose.dev.yml restart timescaledb

# Check database locks
psql postgresql://cs2_user:cs2_password@localhost:5432/cs2_analytics \
  -c "SELECT * FROM pg_locks WHERE NOT granted;"
```

#### Build and Compilation Issues
```bash
# Clean rebuild everything
cargo clean && cargo build --workspace

# Check for dependency conflicts
cargo tree --duplicates

# Update all dependencies
cargo update

# Specific package rebuild
cargo build --package cs2-demo-parser

# Check for circular dependencies
cargo tree --format "{p} {f}" | grep -E "(build|dev)"
```

#### Performance Debugging
```bash
# Enable detailed logging
export RUST_LOG=debug
cargo run --bin cs2-demo-analyzer -- analyze test_data/test_demo.dem

# Profile specific operations
cargo run --release --bin cs2-demo-analyzer -- analyze \
  test_data/vitality-vs-spirit-m1-dust2.dem \
  --profile-memory \
  --profile-cpu

# Network debugging
ss -tulpn | grep -E "(5432|6379|6333)"
netstat -an | grep -E "(5432|6379|6333)"
```

### Environment Reset & Recovery
```bash
# Complete environment reset
docker-compose -f docker-compose.dev.yml down -v
docker system prune -f
docker volume prune -f

# Rebuild dev container
devcontainer rebuild

# Reset just databases
docker-compose -f docker-compose.dev.yml down timescaledb redis qdrant
docker-compose -f docker-compose.dev.yml up -d timescaledb redis qdrant

# Reinitialize data
bash .devcontainer/setup.sh
```

## 🎓 Learning & Development Tips

### Code Organization
- **cs2-demo-parser**: Core parsing logic, start here for demo format understanding
- **cs2-ml**: Machine learning models and training pipelines
- **cs2-data-pipeline**: Batch processing and ETL operations
- **cs2-analytics**: High-level analysis and insights generation
- **cs2-integration-tests**: End-to-end testing and system validation

### Best Practices
1. **Always run tests before committing**: `cargo test --workspace`
2. **Use cargo watch for rapid development**: `cargo watch -x test`
3. **Profile performance-critical code**: Use criterion benchmarks
4. **Monitor resource usage**: Keep an eye on Grafana dashboards
5. **Use integration tests**: Test with real database connections

### Debugging Strategies
1. **Start with unit tests**: Isolate the problem to specific components
2. **Use structured logging**: `RUST_LOG=debug` for detailed output
3. **Database query analysis**: Use EXPLAIN ANALYZE in PostgreSQL
4. **Vector search debugging**: Check Qdrant collection statistics
5. **Memory profiling**: Use valgrind or cargo-profiling for memory leaks

This dev container environment provides everything you need to develop, test, and debug the CS2 Demo Analysis system locally. The fake infrastructure closely mirrors production while being completely self-contained and reproducible.

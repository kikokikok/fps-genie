# CS2 Demo Analysis - Infrastructure Coordination Guide

## Overview

This document explains how the various Docker and infrastructure components work together in the CS2 Demo Analysis system, addressing the coordination between `docker-compose.yml`, root `Dockerfile`, CI pipelines, and integration testing.

## Infrastructure Components Architecture

### 1. Development Infrastructure (`docker-compose.yml`)

**Purpose**: Full-stack development and testing environment
**Location**: Root of repository
**Services**:
- **PostgreSQL + TimescaleDB** (port 5432): Time-series player data
- **Qdrant** (ports 6333/6334): Vector database for behavioral embeddings  
- **Redis** (port 6379): Caching and job queues
- **MinIO** (ports 9000/9001): Object storage for demo files

**Usage**:
```bash
# Start full development stack
docker compose up -d

# Use with development
export DATABASE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
export QDRANT_URL="http://localhost:6333"
export REDIS_URL="redis://localhost:6379"
export MINIO_ENDPOINT="http://localhost:9000"
```

### 2. Application Image (`Dockerfile` at root)

**Purpose**: Production-ready CS2 analysis application
**Base**: `rust:1.88-bookworm`
**Contains**:
- All compiled CS2 binaries (5 total)
- Optimized runtime environment
- Health checks
- Non-root user security

**Build**:
```bash
docker build -t fps-genie:latest .
```

**Usage**:
```bash
# Run specific tools
docker run --rm fps-genie:latest cs2-data-pipeline --help
docker run --rm fps-genie:latest cs2-demo-analyzer process demo.dem
```

### 3. CI Base Images (`.docker/base.Dockerfile`)

**Purpose**: Optimized CI/CD build environments
**Stages**:
- **rust-base**: Core Rust + system dependencies
- **ci-base**: + CI tools (audit, coverage)
- **dev-base**: + Development tools

**Usage in CI**:
```yaml
container:
  image: ghcr.io/kikokikok/fps-genie-ci-base:latest
```

## CI/CD Pipeline Coordination

### Multi-Tier Testing Strategy

#### Tier 1: Fast Checks (2-3 minutes)
- **Trigger**: Every push/PR
- **Infrastructure**: None (local compilation only)
- **Tests**: Format, clippy, basic unit tests
- **Purpose**: Immediate developer feedback

#### Tier 2: Infrastructure Integration (8-12 minutes)
- **Trigger**: PRs to main, push to main/master
- **Infrastructure**: GitHub Actions service containers
- **Services**: postgres, redis, qdrant (no MinIO for speed)
- **Tests**: Database operations, integration tests
- **Purpose**: Validate infrastructure connectivity

#### Tier 3: ML Pipeline Testing (15-25 minutes)
- **Trigger**: ML code changes, weekly schedule
- **Infrastructure**: Service containers + Python environment
- **Tests**: ML model training, Jupyter notebooks, behavioral analysis
- **Purpose**: Validate AI/ML components

#### Tier 4: Full Integration (30-45 minutes)  
- **Trigger**: Monthly, manual dispatch
- **Infrastructure**: Complete stack (all services)
- **Tests**: End-to-end workflows, cross-platform builds, performance
- **Purpose**: Complete system validation

## Infrastructure Usage by Component

### Integration Tests (`cs2-integration-tests/`)

**Two Approaches**:

1. **TestContainers** (for local development):
```rust
let infra = TestInfrastructure::new().await?;
// Automatically starts all required containers
```

2. **Service Containers** (for CI):
```yaml
services:
  postgres:
    image: timescale/timescaledb:latest-pg15
    # Configuration
```

**Feature Flag**: `integration-tests`
```bash
# Local with TestContainers
cargo test --features integration-tests

# CI with service containers  
# (feature enabled automatically)
```

### Data Pipeline (`cs2-data-pipeline/`)

**Infrastructure Dependencies**:
- PostgreSQL/TimescaleDB: Player snapshots, match metadata
- Qdrant: Behavioral vectors and similarity search
- Redis: Job queues and caching
- MinIO: Demo file storage

**Environment Variables**:
```bash
DATABASE_URL=postgresql://user:pass@host:5432/db
TIMESCALE_URL=postgresql://user:pass@host:5432/db  
QDRANT_URL=http://host:6333
REDIS_URL=redis://host:6379
MINIO_ENDPOINT=http://host:9000
```

### ML Pipeline (`cs2-ml/`)

**Infrastructure Dependencies**:
- PostgreSQL: Training data source
- Qdrant: Vector storage and retrieval
- (Optional) Jupyter: Analysis notebooks

**Platform Compatibility**:
- **Linux**: CPU-only features (default in CI)
- **macOS**: Metal GPU acceleration available
- **Windows**: CPU-only features

## Coordination Strategies

### Local Development Workflow

1. **Start Infrastructure**:
```bash
./setup_databases.sh  # Starts docker-compose stack
# Wait 2-3 minutes for initialization
```

2. **Develop with Hot Reload**:
```bash
export DATABASE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
cd cs2-data-pipeline
cargo watch -x run
```

3. **Run Integration Tests**:
```bash
cargo test --features integration-tests
```

### CI/CD Coordination

#### Service Container Configuration
```yaml
# .github/workflows/ci.yml - Tier 2
services:
  postgres:
    image: timescale/timescaledb:latest-pg15
    env:
      POSTGRES_PASSWORD: test_password
      POSTGRES_USER: test_user  
      POSTGRES_DB: cs2_test
    options: >-
      --health-cmd pg_isready
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5
    ports:
      - 5432:5432
```

#### Container Build Strategy
```yaml
# Build application image
- name: Build Docker image
  uses: docker/build-push-action@v5
  with:
    context: .
    file: ./Dockerfile  # Root Dockerfile
    
# Test with infrastructure  
- name: Test with services
  run: |
    docker run --rm --network host \
      -e DATABASE_URL="postgresql://test_user:test_password@localhost:5432/cs2_test" \
      fps-genie:test cs2-data-pipeline init
```

### Production Deployment

#### Using Docker Compose for Production
```bash
# Production docker-compose override
version: '3.8'
services:
  app:
    image: ghcr.io/kikokikok/fps-genie:latest
    depends_on:
      - postgres
      - redis
      - qdrant
      - minio
    environment:
      - DATABASE_URL=postgresql://cs2_user:cs2_password@postgres:5432/cs2_analysis
      - QDRANT_URL=http://qdrant:6333
      - REDIS_URL=redis://redis:6379
    command: cs2-data-pipeline run
```

#### Kubernetes Deployment
```yaml
# Use the compiled application image
spec:
  containers:
  - name: cs2-data-pipeline
    image: ghcr.io/kikokikok/fps-genie:latest
    command: ["cs2-data-pipeline", "run"]
    env:
    - name: DATABASE_URL
      value: "postgresql://user:pass@postgres-service:5432/cs2_analysis"
```

## Asset Management Strategy

### Image Registry Usage

#### Base Images
- **Registry**: `ghcr.io/kikokikok/fps-genie-ci-base`
- **Purpose**: CI optimization, pre-installed tools
- **Update Trigger**: `.docker/base.Dockerfile` changes

#### Application Images  
- **Registry**: `ghcr.io/kikokikok/fps-genie`
- **Purpose**: Production deployment
- **Update Trigger**: Push to main/master

#### Tags Strategy
```bash
# Development
ghcr.io/kikokikok/fps-genie:main-sha123abc

# Release
ghcr.io/kikokikok/fps-genie:latest
ghcr.io/kikokikok/fps-genie:v1.0.0
```

### Artifact Coordination

#### Build Artifacts (Cross-Platform)
```yaml
# Upload per-platform binaries
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: cs2-tools-${{ matrix.target }}
    path: |
      target/${{ matrix.target }}/release/cs2-analytics*
      target/${{ matrix.target }}/release/cs2-data-pipeline*
      # ... other binaries
```

#### Test Artifacts
- **Performance Benchmarks**: `target/criterion/` (30-day retention)
- **ML Model Outputs**: `models/trained/` (90-day retention)  
- **Integration Test Reports**: `test-results/` (30-day retention)

## Troubleshooting Common Coordination Issues

### Issue: CI Tests Fail with Connection Refused
**Cause**: Service containers not ready
**Solution**: Add proper health checks and wait logic
```yaml
- name: Wait for services
  run: |
    until pg_isready -h localhost -p 5432 -U test_user; do sleep 1; done
```

### Issue: Local Development Database Connection Issues
**Cause**: docker-compose not running or incorrect URLs
**Solution**: 
```bash
# Check services
docker compose ps

# Restart if needed
docker compose down && docker compose up -d

# Wait for initialization
sleep 30
```

### Issue: TestContainers vs Service Containers Conflicts
**Cause**: Port conflicts between approaches
**Solution**: Use feature flags to separate environments
```rust
#[cfg(feature = "integration-tests")]
// Use TestContainers for local development

#[cfg(not(feature = "integration-tests"))]  
// Use environment variables (CI service containers)
```

### Issue: Docker Build Context Too Large
**Cause**: Including unnecessary files in build context
**Solution**: Optimize `.dockerignore`
```
target/
.git/
*.dem
test_data/
docs/
node_modules/
```

## Summary

This infrastructure coordination provides:

1. **Separation of Concerns**: Development (docker-compose) vs Production (Dockerfile) vs CI (base images)
2. **Flexible Testing**: Multiple tiers based on change scope and frequency
3. **Efficient Builds**: Cached base images, optimized contexts
4. **Cross-Platform Support**: Consistent behavior across development and deployment
5. **Scalable Architecture**: Ready for Kubernetes, cloud deployment

The key insight is that each Docker asset serves a specific purpose in the development lifecycle, with clear coordination points for maximum efficiency and maintainability.
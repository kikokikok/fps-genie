# CI Pipeline Analysis & Docker Base Image Improvements

## Current State Analysis

### Issues Identified
1. **Redundant Dependency Installation**: Every CI job repeats the same system dependency installation
2. **Network Reliability**: Multiple jobs depend on apt-get which can timeout or fail
3. **Build Time Overhead**: 30-60 seconds per job just for dependency installation
4. **Inconsistency**: Different timing of package updates across jobs may lead to version drift
5. **Matrix Build Complexity**: Platform-specific dependency management is scattered

### Current CI Jobs Analysis
- **check**: Installs full dependency stack for simple cargo check
- **test**: Installs dependencies + database setup
- **fmt**: Installs dependencies just for rustfmt check
- **clippy**: Installs dependencies for linting
- **build**: Matrix across 3 platforms with different dependency approaches
- **benchmark**: Duplicate dependency installation + database setup
- **coverage**: Full dependency stack for coverage generation

## Proposed Docker Base Image Approach

### 1. Custom Base Images Strategy

#### Create Specialized Base Images:

**Development Base Image** (`fps-genie-dev-base`):
```dockerfile
FROM rust:1.75-bookworm as base
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    gobjc \
    protobuf-compiler \
    libfontconfig1-dev \
    libssl-dev \
    pkg-config \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*
```

**CI Base Image** (`fps-genie-ci-base`):
```dockerfile
FROM fps-genie-dev-base
# Add CI-specific tools
RUN cargo install cargo-audit cargo-llvm-cov
```

### 2. Benefits Analysis

#### Performance Improvements:
- **Time Savings**: ~45-60 seconds per job (7 jobs = 5-7 minutes total)
- **Reliability**: Eliminate apt-get network dependency failures
- **Consistency**: Same environment across all jobs
- **Caching**: Docker layer caching reduces rebuild times

#### Cost Benefits:
- **Reduced CI Minutes**: Faster jobs = lower GitHub Actions costs
- **Parallel Efficiency**: Jobs can start immediately without dependency setup
- **Network Bandwidth**: Reduced package downloads

### 3. Implementation Strategy

#### Phase 1: Base Image Creation
1. Create multi-stage base images in separate repository or registry
2. Automate base image updates with Dependabot
3. Tag images with Rust version for consistency

#### Phase 2: CI Migration
1. Replace `ubuntu-latest` with custom base images
2. Remove redundant dependency installation steps
3. Optimize caching strategies

#### Phase 3: Advanced Optimizations
1. Implement build cache mounting
2. Add development containers for local consistency
3. Create platform-specific optimized images

## Specific Recommendations

### 1. Immediate Improvements (Low Effort, High Impact)

#### A. Create Base Image Dockerfile
```dockerfile
# .docker/base.Dockerfile
FROM rust:1.75-bookworm as rust-base

# Install system dependencies once
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    gobjc \
    protobuf-compiler \
    libfontconfig1-dev \
    libssl-dev \
    pkg-config \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Pre-install common Rust tools
RUN cargo install cargo-audit cargo-llvm-cov

FROM rust-base as ci-base
# Add any CI-specific tools here
```

#### B. Update CI Jobs
Replace system dependency installation with container usage:

```yaml
jobs:
  check:
    runs-on: ubuntu-latest
    container: 
      image: ghcr.io/kikokikok/fps-genie-ci-base:latest
    steps:
      - uses: actions/checkout@v4
      # Remove dependency installation step
      - run: cargo check --workspace
```

### 2. Medium-term Improvements

#### A. Multi-platform Base Images
```yaml
# Build base images for each platform
strategy:
  matrix:
    include:
      - platform: linux/amd64
        runner: ubuntu-latest
      - platform: linux/arm64
        runner: ubuntu-latest
```

#### B. Development Container
```json
// .devcontainer/devcontainer.json
{
  "name": "FPS Genie Development",
  "image": "ghcr.io/kikokikok/fps-genie-dev-base:latest",
  "features": {
    "ghcr.io/devcontainers/features/docker-in-docker:2": {}
  }
}
```

### 3. Advanced Optimizations

#### A. Build Cache Optimization
```yaml
- name: Set up Docker Buildx
  uses: docker/setup-buildx-action@v3
  with:
    driver-opts: |
      network=host

- name: Build with cache
  uses: docker/build-push-action@v5
  with:
    context: .
    cache-from: type=gha
    cache-to: type=gha,mode=max
    target: builder
```

#### B. Registry Optimization
```yaml
# Use GitHub Container Registry for fast pulls
- name: Login to GitHub Container Registry
  uses: docker/login-action@v3
  with:
    registry: ghcr.io
    username: ${{ github.actor }}
    password: ${{ secrets.GITHUB_TOKEN }}
```

## Implementation Roadmap

### Week 1: Foundation
- [ ] Create base Dockerfile with all dependencies
- [ ] Set up GitHub Container Registry
- [ ] Build and test base images

### Week 2: CI Migration
- [ ] Update check and fmt jobs to use base image
- [ ] Migrate test and clippy jobs
- [ ] Update build matrix jobs

### Week 3: Optimization
- [ ] Implement advanced caching
- [ ] Add development container support
- [ ] Performance testing and tuning

### Week 4: Documentation & Monitoring
- [ ] Update developer documentation
- [ ] Add image update automation
- [ ] Implement monitoring for build times

## Expected Outcomes

### Performance Metrics:
- **CI Time Reduction**: 15-25% faster overall pipeline
- **Reliability Increase**: 90%+ reduction in dependency-related failures
- **Developer Experience**: Consistent local/CI environments

### Maintenance Benefits:
- **Centralized Dependency Management**: Single location for system deps
- **Version Control**: Tagged, versioned base images
- **Security**: Regular base image updates with vulnerability scanning

## Migration Checklist

- [ ] Review current CI failures related to dependency installation
- [ ] Create base image Dockerfile
- [ ] Test base image locally with cargo build
- [ ] Set up container registry and automation
- [ ] Migrate one CI job as proof of concept
- [ ] Gradually migrate remaining jobs
- [ ] Update documentation and developer guides
- [ ] Monitor performance improvements

## Risk Mitigation

### Potential Issues:
1. **Image Size**: Base images might be large
   - *Solution*: Multi-stage builds and layer optimization
2. **Update Lag**: Base images might become stale
   - *Solution*: Automated updates with Dependabot
3. **Platform Compatibility**: Issues with different architectures
   - *Solution*: Multi-platform image builds

### Rollback Plan:
- Keep current CI configuration in separate branch
- Feature flag approach for gradual migration
- Quick revert capability if issues arise
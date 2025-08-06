# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2025-08-06

### 🚀 Major Infrastructure Improvements

#### Added
- **Complete Docker-based CI/CD pipeline** with proven Rust 1.88 + ARM64 compatibility
- **Multi-stage Dockerfile** with optimized caching and dependency management
- **GitHub Container Registry integration** for automated image publishing
- **Multi-platform builds** supporting both AMD64 and ARM64 architectures
- **Comprehensive security scanning** with Trivy, CodeQL, and cargo audit
- **Performance benchmarking pipeline** with automated regression detection
- **Professional release automation** with binary extraction and Docker image publishing

#### Changed
- **BREAKING**: Upgraded from Rust 1.75 to Rust 1.88 for ARM64 FP16 compatibility
- **Unified Docker setup**: Single docker-compose.yml replacing separate dev/prod configurations
- **Enhanced CI workflows**: All jobs now use proven Docker environments instead of native runners
- **Optimized dependency installation**: Added missing fontconfig and freetype libraries for GUI components
- **Improved caching strategy**: GitHub Actions cache integration for faster builds

#### Removed
- Legacy setup scripts (`setup.sh`, `complete_setup.sh`, `env_setup.sh`)
- Redundant Docker configurations (`docker-compose.dev.yml`, `.devcontainer/Dockerfile.api`)
- Obsolete test dockerfile with incorrect dependencies
- Unused build outputs and configuration files

### 🔧 Technical Details

#### Docker Improvements
- **Base Image**: `rust:1.88-bookworm` with all required system dependencies
- **Multi-stage builds**: `base` → `development` → `test` → `builder` → `production`
- **ARM64 Compatibility**: Added `RUSTFLAGS="-C target-cpu=generic"` to avoid FP16 instruction issues
- **Complete dependency stack**: 
  - protobuf-compiler + libprotobuf-dev
  - libfontconfig1-dev + libfreetype6-dev (for font rendering)
  - pkg-config + libssl-dev
  - Python ML stack + development tools

#### CI/CD Pipeline Features
- **Parallel execution**: Check, test, clippy, and audit run simultaneously
- **Service integration**: PostgreSQL + Redis + Qdrant for realistic testing
- **Artifact management**: Automatic binary extraction and release publishing
- **Security integration**: Results appear in GitHub Security tab
- **Performance monitoring**: Weekly automated benchmark runs

#### Dependency Management
- **Fixed ARM64 build issues**: Resolved `gemm-f16` FP16 instruction compatibility
- **Streamlined installations**: Single apt-get command with all required packages
- **Version consistency**: Locked Rust version prevents future compatibility breaks

### 📦 Release Artifacts

Starting with this release, automated builds will provide:
- **Docker Images**: `ghcr.io/fps-genie:latest` and versioned tags
- **Binary Archives**: `fps-genie-linux-amd64.tar.gz` with all executables
- **Multi-platform support**: Both AMD64 and ARM64 builds
- **Professional documentation**: Auto-generated release notes with usage examples

### 🧪 Testing Improvements

- **Container-based testing**: All tests run in consistent Docker environments
- **Database integration**: Automated TimescaleDB + Redis setup for integration tests
- **Performance regression detection**: Automated weekly benchmark comparisons
- **Security scanning**: Comprehensive vulnerability and license compliance checks

### 🔄 Migration Notes

For developers:
- Use `docker-compose up` for the complete development environment
- Previous `docker-compose -f docker-compose.dev.yml up` commands now use the main file
- All development tools (cargo-watch, jupyter, etc.) are pre-installed in containers
- Build artifacts are cached in named volumes for faster incremental builds

For deployment:
- Production images are now available at `ghcr.io/fps-genie`
- Kubernetes deployment can reference these pre-built images
- No need to build images in production environments

---

## Previous Releases

*This changelog was introduced with the Docker and CI improvements. Previous release history is available in git commit messages.*

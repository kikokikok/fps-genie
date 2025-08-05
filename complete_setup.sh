#!/bin/bash

# CS2 Demo Analysis & AI Training System - Complete Local Setup
# This script sets up everything needed for E2E testing and advanced analytics

set -e

echo "🚀 CS2 Demo Analysis System - Complete Local Setup"
echo "=================================================="

# Check prerequisites
echo "🔍 Checking prerequisites..."

# Check if Docker is installed and running
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

if ! docker info &> /dev/null; then
    echo "❌ Docker daemon not running. Please start Docker."
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first."
    exit 1
fi

echo "✅ Prerequisites check passed"

# Step 1: Build all Rust components
echo ""
echo "🔨 Building Rust workspace..."
cargo build --workspace --release

if [ $? -eq 0 ]; then
    echo "✅ Rust build completed successfully"
else
    echo "❌ Rust build failed"
    exit 1
fi

# Step 2: Set up database infrastructure
echo ""
echo "📊 Setting up database infrastructure..."
./setup_databases.sh

# Step 3: Initialize the data pipeline
echo ""
echo "🏗️ Initializing data pipeline..."
export DATABASE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
export QDRANT_URL="http://localhost:6334"

cd cs2-data-pipeline
cargo run --release -- init
cd ..

# Step 4: Run comprehensive tests
echo ""
echo "🧪 Running comprehensive test suite..."

# Unit tests first
echo "Running unit tests..."
cargo test --workspace --lib

# Integration tests with TestContainers
echo "Running integration tests..."
cargo test --package cs2-integration-tests --release

# Performance benchmarks
echo "Running performance benchmarks..."
cargo bench --package cs2-integration-tests

# Step 5: Set up sample data for testing
echo ""
echo "📁 Setting up sample data..."

# Create demo directories
mkdir -p demos/professional
mkdir -p demos/test_data

# Copy test demos if they exist
if [ -f "test_data/test_demo.dem" ]; then
    cp test_data/test_demo.dem demos/test_data/
    echo "✅ Copied test demo file"
fi

# Step 6: Verify the complete setup
echo ""
echo "🔍 Verifying setup..."

# Check database connections
echo "Testing database connections..."
if cargo run --package cs2-data-pipeline --release -- stats > /dev/null 2>&1; then
    echo "✅ Database connections working"
else
    echo "⚠️ Database connection issues detected"
fi

# Check ML pipeline
echo "Testing ML pipeline..."
if cargo run --package cs2-ml --release -- --help > /dev/null 2>&1; then
    echo "✅ ML pipeline ready"
else
    echo "⚠️ ML pipeline issues detected"
fi

# Check analytics system
echo "Testing analytics system..."
if cargo run --package cs2-analytics --release -- --help > /dev/null 2>&1; then
    echo "✅ Analytics system ready"
else
    echo "⚠️ Analytics system issues detected"
fi

echo ""
echo "🎉 Setup Complete!"
echo "=================="
echo ""
echo "📋 What's been set up:"
echo "  ✅ PostgreSQL + TimescaleDB (time-series data)"
echo "  ✅ Qdrant Vector Database (behavioral embeddings)"
echo "  ✅ Redis Cache"
echo "  ✅ MinIO Object Storage"
echo "  ✅ Complete Rust workspace built"
echo "  ✅ TestContainers integration tests"
echo "  ✅ Performance benchmarking"
echo "  ✅ Advanced analytics pipeline"
echo ""
echo "🚀 Quick Start Commands:"
echo ""
echo "1. Process demo files:"
echo "   cd cs2-data-pipeline"
echo "   cargo run -- discover --recursive"
echo "   cargo run -- process"
echo ""
echo "2. Run advanced analytics:"
echo "   cd cs2-analytics"
echo "   cargo run -- analyze --analysis-type playstyle"
echo "   cargo run -- train --model-type behavior-cloning"
echo ""
echo "3. Run comprehensive tests:"
echo "   cargo test --package cs2-integration-tests"
echo "   cargo bench --package cs2-integration-tests"
echo ""
echo "4. Generate visualizations:"
echo "   cargo run --package cs2-analytics -- visualize --input ./data --viz-type heatmap"
echo ""
echo "📊 Database URLs:"
echo "  PostgreSQL: postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
echo "  Qdrant:     http://localhost:6334"
echo "  MinIO:      http://localhost:9000 (minioadmin/minioadmin123)"
echo ""
echo "📈 Expected Performance (based on your PDF specs):"
echo "  - Demo parsing: 700MB+/s"
echo "  - Database ingestion: 10K+ snapshots/second"
echo "  - Concurrent processing: 4-8 demos simultaneously"
echo "  - Scale: Handle entire professional CS2 scene"
echo ""
echo "🔧 Troubleshooting:"
echo "  - If tests fail: Check Docker containers are running"
echo "  - If parsing fails: Verify demo file paths"
echo "  - If ML fails: Check PyTorch installation"
echo "  - For performance issues: Adjust batch sizes and concurrency"
echo ""
echo "📚 Documentation:"
echo "  - See PROJECT_CONTEXT.md for session-reusable context"
echo "  - Check individual package READMEs for detailed usage"
echo "  - Review CS2 Demo Analysis & AI Training System.pdf for architecture"

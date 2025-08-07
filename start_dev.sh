#!/bin/bash
# Quick start script for CS2 Demo Analysis development environment

set -e

echo "🚀 CS2 Demo Analysis - Quick Start"
echo "=================================="

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo "🔍 Checking prerequisites..."

if ! command_exists docker; then
    echo "❌ Docker not found. Please install Docker first."
    echo "   Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

if ! command_exists docker-compose; then
    echo "❌ Docker Compose not found. Please install Docker Compose first."
    echo "   Visit: https://docs.docker.com/compose/install/"
    exit 1
fi

if ! docker info >/dev/null 2>&1; then
    echo "❌ Docker daemon not running. Please start Docker."
    exit 1
fi

echo "✅ Prerequisites check passed"

# Check if we're in the right directory
if [ ! -f "docker-compose.dev.yml" ]; then
    echo "❌ docker-compose.dev.yml not found. Are you in the project root?"
    exit 1
fi

echo ""
echo "📋 Available startup options:"
echo "1. DevContainer (VS Code) - Recommended for development"
echo "2. Infrastructure Only - Just databases and services"
echo "3. Full Environment - Everything including dev container"
echo "4. Clean Start - Remove existing containers and start fresh"
echo ""

read -p "Choose option [1-4]: " choice

case $choice in
    1)
        echo ""
        echo "🔧 DevContainer Setup"
        echo "===================="
        echo ""
        echo "To use the DevContainer:"
        echo "1. Open VS Code in this directory: code ."
        echo "2. Install 'Dev Containers' extension if not already installed"
        echo "3. Press Cmd+Shift+P (or Ctrl+Shift+P)"
        echo "4. Select 'Dev Containers: Reopen in Container'"
        echo "5. Wait for automatic setup to complete"
        echo ""
        echo "This will give you a complete development environment with:"
        echo "  - Rust toolchain with all extensions"
        echo "  - Database with sample data"
        echo "  - Jupyter notebooks for analysis"
        echo "  - All services pre-configured"
        ;;
        
    2)
        echo ""
        echo "🏗️ Starting infrastructure services..."
        docker-compose -f docker-compose.dev.yml up -d timescaledb redis qdrant grafana
        echo ""
        echo "✅ Infrastructure started! Services available at:"
        echo "  📊 TimescaleDB: localhost:5432 (cs2_user/cs2_password)"
        echo "  🔗 Redis: localhost:6379"
        echo "  🎯 Qdrant: localhost:6333"
        echo "  📈 Grafana: localhost:3001 (admin/admin)"
        echo ""
        echo "Connect to database:"
        echo "  psql postgresql://cs2_user:cs2_password@localhost:5432/cs2_analytics"
        ;;
        
    3)
        echo ""
        echo "🌟 Starting full development environment..."
        docker-compose -f docker-compose.dev.yml up -d
        echo ""
        echo "✅ Full environment started! All services available:"
        echo "  📊 TimescaleDB: localhost:5432 (cs2_user/cs2_password)"
        echo "  🔗 Redis: localhost:6379"
        echo "  🎯 Qdrant: localhost:6333"
        echo "  📈 Grafana: localhost:3001 (admin/admin)"
        echo "  🔬 Jupyter: localhost:8888 (token: cs2analysis)"
        echo "  🐳 Dev Container: Running"
        echo ""
        echo "Access dev container:"
        echo "  docker exec -it cs2-dev bash"
        ;;
        
    4)
        echo ""
        echo "🧹 Cleaning up existing containers..."
        docker-compose -f docker-compose.dev.yml down -v
        docker-compose -f docker-compose.yml down -v 2>/dev/null || true
        echo "✅ Cleanup complete"
        echo ""
        echo "🚀 Starting fresh environment..."
        docker-compose -f docker-compose.dev.yml up -d
        echo ""
        echo "✅ Fresh environment started!"
        ;;
        
    *)
        echo "❌ Invalid option. Exiting."
        exit 1
        ;;
esac

echo ""
echo "🎯 Next Steps:"
echo ""
echo "📖 Documentation:"
echo "  - Development Guide: .devcontainer/DEV_GUIDE.md"
echo "  - DevContainer Setup: .devcontainer/README.md"
echo "  - Project Overview: README.md"
echo ""
echo "🛠️ Quick Commands:"
echo "  - Build project: cargo build --workspace"
echo "  - Run tests: cargo test --workspace"
echo "  - Start Jupyter: ./start_jupyter.sh"
echo "  - Analyze demo: cargo run --bin cs2-demo-analyzer -- analyze test_data/demo.dem"
echo ""
echo "🔧 Service Management:"
echo "  - View logs: docker-compose -f docker-compose.dev.yml logs [service]"
echo "  - Stop services: docker-compose -f docker-compose.dev.yml down"
echo "  - Restart service: docker-compose -f docker-compose.dev.yml restart [service]"
echo ""
echo "🎮 Happy CS2 analysis development!"
#!/bin/bash
set -e

echo "🚀 Setting up CS2 Demo Analysis Dev Environment..."

# Wait for services to be healthy
echo "⏳ Waiting for infrastructure services to be ready..."
until docker-compose -f docker-compose.dev.yml exec timescaledb pg_isready -U cs2_user -d cs2_analytics; do
  echo "Waiting for TimescaleDB..."
  sleep 2
done

until docker-compose -f docker-compose.dev.yml exec redis redis-cli ping; do
  echo "Waiting for Redis..."
  sleep 2
done

until curl -f http://qdrant:6333/health; do
  echo "Waiting for Qdrant..."
  sleep 2
done

echo "✅ Infrastructure services are ready!"

# Initialize database schemas
echo "🗄️ Initializing database schemas..."
cd /workspace
cargo run --bin cs2-data-pipeline -- init-db

# Create Qdrant collections
echo "🔍 Setting up Qdrant vector collections..."
cargo run --bin cs2-ml -- init-vectors

# Generate test data
echo "🎮 Generating test demo data..."
mkdir -p test_data/generated
cargo run --bin cs2-integration-tests -- generate-test-demos --count 5 --output test_data/generated

# Run initial tests to verify setup
echo "🧪 Running integration tests..."
cargo test --workspace --features integration-tests

echo "🎯 Dev environment setup complete!"
echo ""
echo "Available services:"
echo "  - TimescaleDB: localhost:5432 (user: cs2_user, password: cs2_password)"
echo "  - Redis: localhost:6379"
echo "  - Qdrant: localhost:6333 (HTTP), localhost:6334 (gRPC)"
echo "  - Analytics Dashboard: localhost:3000 (admin/admin)"
echo "  - Jupyter Notebooks: localhost:8888 (token: cs2analysis)"
echo ""
echo "Quick start commands:"
echo "  cargo run --bin cs2-demo-analyzer -- analyze test_data/test_demo.dem"
echo "  cargo run --bin cs2-data-pipeline -- process --demo-dir test_data"
echo "  cargo run --bin cs2-ml -- train --dataset behavioral_vectors"
echo ""

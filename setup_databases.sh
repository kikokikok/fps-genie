#!/bin/bash

# CS2 Demo Analysis & AI Training System - Database Setup Script
# This script sets up the complete database infrastructure as described in the PDF

set -e

echo "🚀 Setting up CS2 Demo Analysis Database Infrastructure..."

# Default configuration
POSTGRES_DB=${POSTGRES_DB:-"cs2_analysis"}
POSTGRES_USER=${POSTGRES_USER:-"cs2_user"}
POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-"cs2_password"}
QDRANT_PORT=${QDRANT_PORT:-6334}

# Create docker-compose.yml for the complete infrastructure
cat > docker-compose.yml << EOF
version: '3.8'

services:
  # PostgreSQL + TimescaleDB for relational and time-series data
  postgres:
    image: timescale/timescaledb:latest-pg15
    environment:
      POSTGRES_DB: ${POSTGRES_DB}
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./sql/init.sql:/docker-entrypoint-initdb.d/init.sql
    command: postgres -c shared_preload_libraries=timescaledb
    restart: unless-stopped

  # Qdrant Vector Database for behavioral embeddings
  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "${QDRANT_PORT}:6333"
      - "6334:6334"
    volumes:
      - qdrant_data:/qdrant/storage
    environment:
      QDRANT__SERVICE__HTTP_PORT: 6333
      QDRANT__SERVICE__GRPC_PORT: 6334
    restart: unless-stopped

  # Redis for caching and job queues
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    restart: unless-stopped

  # MinIO for object storage (demo files, exports)
  minio:
    image: minio/minio:latest
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - minio_data:/data
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin123
    command: server /data --console-address ":9001"
    restart: unless-stopped

volumes:
  postgres_data:
  qdrant_data:
  redis_data:
  minio_data:
EOF

# Create SQL initialization script
mkdir -p sql
cat > sql/init.sql << EOF
-- CS2 Demo Analysis Database Initialization
-- Creates optimized schemas for high-volume professional demo data

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create optimized indexes for professional demo queries
-- These indexes support the high-volume queries described in the PDF

-- Index for finding similar professional player behaviors
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_snapshots_behavior_analysis
ON player_snapshots (steamid, weapon_id, health, armor, is_alive)
WHERE is_alive = true;

-- Index for trajectory analysis (movement patterns)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_snapshots_trajectory
ON player_snapshots (match_id, steamid, tick)
INCLUDE (pos_x, pos_y, pos_z, vel_x, vel_y, vel_z);

-- Index for aim analysis
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_snapshots_aim
ON player_snapshots (match_id, steamid, tick)
INCLUDE (yaw, pitch, weapon_id, is_scoped);

-- Partitioning for massive data volumes
-- Partition player_snapshots by month for optimal query performance
-- This supports the projected 100GB+ of time-series data mentioned in the PDF
SELECT add_dimension('player_snapshots', 'steamid', number_partitions => 16);

-- Create materialized views for common aggregations
CREATE MATERIALIZED VIEW IF NOT EXISTS daily_player_stats AS
SELECT
    DATE(timestamp) as date,
    steamid,
    COUNT(*) as total_ticks,
    AVG(health) as avg_health,
    AVG(armor) as avg_armor,
    COUNT(*) FILTER (WHERE is_alive) as alive_ticks,
    AVG(SQRT(vel_x^2 + vel_y^2 + vel_z^2)) as avg_velocity
FROM player_snapshots
GROUP BY DATE(timestamp), steamid;

CREATE UNIQUE INDEX ON daily_player_stats (date, steamid);

-- Refresh policy for materialized views
SELECT add_continuous_aggregate_policy('daily_player_stats',
    start_offset => INTERVAL '1 month',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 hour');

EOF

echo "📦 Starting database infrastructure..."
docker-compose up -d

echo "⏳ Waiting for databases to be ready..."
sleep 10

# Test connections
echo "🔍 Testing database connections..."

# Test PostgreSQL/TimescaleDB
if docker-compose exec -T postgres psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -c "SELECT version();" > /dev/null 2>&1; then
    echo "✅ PostgreSQL/TimescaleDB: Connected"
else
    echo "❌ PostgreSQL/TimescaleDB: Connection failed"
    exit 1
fi

# Test Qdrant
if curl -s http://localhost:${QDRANT_PORT}/health > /dev/null; then
    echo "✅ Qdrant Vector DB: Connected"
else
    echo "❌ Qdrant Vector DB: Connection failed"
    exit 1
fi

# Test Redis
if docker-compose exec -T redis redis-cli ping | grep -q PONG; then
    echo "✅ Redis: Connected"
else
    echo "❌ Redis: Connection failed"
    exit 1
fi

# Test MinIO
if curl -s http://localhost:9000/minio/health/live > /dev/null; then
    echo "✅ MinIO Object Storage: Connected"
else
    echo "❌ MinIO Object Storage: Connection failed"
    exit 1
fi

echo ""
echo "🎉 Database infrastructure setup complete!"
echo ""
echo "📊 Connection Details:"
echo "  PostgreSQL/TimescaleDB: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:5432/${POSTGRES_DB}"
echo "  Qdrant Vector DB:       http://localhost:${QDRANT_PORT}"
echo "  Redis Cache:            redis://localhost:6379"
echo "  MinIO Object Storage:   http://localhost:9000 (admin/admin123)"
echo ""
echo "🚀 Next steps:"
echo "  1. Export DATABASE_URL=\"postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:5432/${POSTGRES_DB}\""
echo "  2. Export QDRANT_URL=\"http://localhost:${QDRANT_PORT}\""
echo "  3. Run: cd cs2-data-pipeline && cargo run -- init"
echo "  4. Place demo files in ./demos/ directory"
echo "  5. Run: cargo run -- run"
echo ""
echo "📈 Expected Performance (from PDF specifications):"
echo "  - Process 700MB+/s demo data on high-end PC"
echo "  - Support 5TB initial TimescaleDB storage"
echo "  - Handle 2TB vector embeddings in Qdrant"
echo "  - Scale to 20TB+ object storage for demo archives"

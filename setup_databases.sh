#!/usr/bin/env bash
# CS2 Demo Analysis & AI Training System - Database Setup Script
# Sets up Postgres/TimescaleDB, Qdrant, Redis, and Garage (S3-compatible) with v2 layout init.

set -euo pipefail

echo "🚀 Setting up CS2 Demo Analysis Database Infrastructure..."

# Defaults (override with env)
POSTGRES_DB=${POSTGRES_DB:-"cs2_analysis"}
POSTGRES_USER=${POSTGRES_USER:-"cs2_user"}
POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-"cs2_password"}

QDRANT_HTTP_PORT=${QDRANT_HTTP_PORT:-6333}
QDRANT_GRPC_PORT=${QDRANT_GRPC_PORT:-6334}

# Garage init defaults (per docs)
GARAGE_BUCKET=${GARAGE_BUCKET:-"fps-genie"}
GARAGE_KEY_NAME=${GARAGE_KEY_NAME:-"fps-genie-key"}
GARAGE_ENV_OUT=${GARAGE_ENV_OUT:-"garage/s3-credentials.env"}
GARAGE_ZONE=${GARAGE_ZONE:-"dc1"}
GARAGE_CAPACITY=${GARAGE_CAPACITY:-"1G"}

echo "📦 Starting services with Docker Compose..."
docker compose up -d

echo "⏳ Waiting for services to be ready..."

# Wait for Postgres/TimescaleDB
echo "🔍 Checking PostgreSQL/TimescaleDB..."
for i in $(seq 1 60); do
  if docker compose exec -T postgres psql -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -c "SELECT 1;" >/dev/null 2>&1; then
    echo "✅ PostgreSQL/TimescaleDB: Connected"
    break
  fi
  sleep 1
  if [ "$i" -eq 60 ]; then
    echo "❌ PostgreSQL/TimescaleDB: Connection failed"
    exit 1
  fi
done

# Wait for Qdrant HTTP
echo "🔍 Checking Qdrant (HTTP ${QDRANT_HTTP_PORT})..."
for i in $(seq 1 60); do
  if curl http://localhost:6333/telemetry -H "api-key: 1234" >/dev/null 2>&1; then
    echo "✅ Qdrant Vector DB (HTTP): Connected"
    break
  fi
  sleep 1
  if [ "$i" -eq 60 ]; then
    echo "❌ Qdrant Vector DB (HTTP): Connection failed"
    exit 1
  fi
done

# Wait for Redis
echo "🔍 Checking Redis..."
for i in $(seq 1 60); do
  if docker compose exec -T redis redis-cli ping | grep -q PONG; then
    echo "✅ Redis: Connected"
    break
  fi
  sleep 1
  if [ "$i" -eq 60 ]; then
    echo "❌ Redis: Connection failed"
    exit 1
  fi
done

# Wait for Garage health
echo "🔍 Checking Garage..."
for i in $(seq 1 60); do
  STATUS="$(docker inspect -f '{{.State.Health.Status}}' garage 2>/dev/null || echo 'starting')"
  if [ "$STATUS" = "healthy" ]; then
    echo "✅ Garage: Healthy"
    break
  fi
  sleep 1
  if [ "$i" -eq 60 ]; then
    echo "❌ Garage: Not healthy"
    exit 1
  fi
done

# Helper to run Garage CLI inside the running container (no shell inside image)
gexec() {
  docker compose exec -T garage /garage -c /etc/garage/config.toml "$@"
}

echo "🧩 Initializing Garage v2 layout (single node, per docs)..."

# 1) Get node id and strip any @host:port suffix
NODE_ID_RAW="$(gexec node id 2>/dev/null | head -n1 | tr -d '\r' | tr -d '[:space:]')"
if [ -z "${NODE_ID_RAW}" ]; then
  echo "❌ Unable to retrieve Garage node id"
  exit 1
fi
NODE_ID="${NODE_ID_RAW%@*}"
echo "ℹ️  Garage node id: ${NODE_ID_RAW} (using ${NODE_ID})"

# 2) Assign role to node with zone and capacity (idempotent)
echo "➡️  Assigning layout: zone=${GARAGE_ZONE}, capacity=${GARAGE_CAPACITY}, node=${NODE_ID}"
gexec layout assign -z "${GARAGE_ZONE}" -c "${GARAGE_CAPACITY}" "${NODE_ID}" >/dev/null 2>&1 || true

# 3) Compute next version from layout history and apply
LAST_VER="$(gexec layout history 2>/dev/null | grep -Eo 'Version[[:space:]]+[0-9]+' | awk '{print $2}' | sort -n | tail -n1 || true)"
if [ -z "${LAST_VER:-}" ]; then LAST_VER=0; fi
NEXT_VER=$((LAST_VER + 1))

echo "➡️  Applying layout with --version ${NEXT_VER}..."
gexec layout apply --version "${NEXT_VER}" >/dev/null 2>&1 || true

# Show resulting layout for visibility
gexec layout show || true

# 4) Create bucket if needed (robust idempotency)
if gexec bucket info "${GARAGE_BUCKET}" >/dev/null 2>&1; then
  echo "ℹ️  Bucket '${GARAGE_BUCKET}' already exists"
else
  echo "➡️  Creating bucket: ${GARAGE_BUCKET}"
  if ! OUT="$(gexec bucket create "${GARAGE_BUCKET}" 2>&1)"; then
    if echo "${OUT}" | grep -q "BucketAlreadyExists"; then
      echo "ℹ️  Bucket '${GARAGE_BUCKET}' already exists"
    else
      echo "❌ Failed to create bucket '${GARAGE_BUCKET}':"
      echo "${OUT}"
      exit 1
    fi
  fi
fi

# 5) Create or read API key (v2 docs: key create <name>)
NEW_KEY_CREATED=0
KEY_ID=""
SECRET=""

if gexec key info "${GARAGE_KEY_NAME}" >/dev/null 2>&1; then
  echo "ℹ️  Key '${GARAGE_KEY_NAME}' already exists"
  # Parse Key ID (secret is redacted for existing keys)
  KEY_ID="$(gexec key info "${GARAGE_KEY_NAME}" | awk -F': ' '/Key ID:/ {print $2; exit}' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' | tr -d '\r')"
else
  echo "➡️  Creating access key: ${GARAGE_KEY_NAME}"
  OUT="$(gexec key create "${GARAGE_KEY_NAME}")"
  KEY_ID="$(echo "${OUT}"   | awk -F': ' '/Key ID:/    {print $2; exit}' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' | tr -d '\r')"
  SECRET="$(echo "${OUT}"   | awk -F': ' '/Secret key:/ {print $2; exit}' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' | tr -d '\r')"
  if [ -z "${KEY_ID:-}" ] || [ -z "${SECRET:-}" ]; then
    echo "❌ Failed to parse Garage access key output:"
    echo "${OUT}"
    exit 1
  fi
  NEW_KEY_CREATED=1
fi

# 6) Allow key to access the bucket (idempotent; using key name as per docs)
echo "➡️  Ensuring key has permissions on bucket..."
if ! gexec bucket info "${GARAGE_BUCKET}" | grep -q "${GARAGE_KEY_NAME}"; then
  gexec bucket allow --read --write --owner "${GARAGE_BUCKET}" --key "${GARAGE_KEY_NAME}" >/dev/null 2>&1 || true
fi

# 7) Write credentials to host file ONLY if it doesn't already exist
if [ -f "${GARAGE_ENV_OUT}" ]; then
  echo "🔐 Credentials file already exists at ${GARAGE_ENV_OUT}; not overwriting."
else
cat > "${GARAGE_ENV_OUT}" <<EOF
# Generated by setup_databases.sh
S3_ENDPOINT=http://localhost:3900
AWS_ENDPOINT_URL=http://localhost:3900
AWS_ACCESS_KEY_ID=${KEY_ID}
AWS_SECRET_ACCESS_KEY=${SECRET}
AWS_REGION=garage
S3_BUCKET=${GARAGE_BUCKET}
AWS_S3_FORCE_PATH_STYLE=true
EOF
  echo "✅ Garage credentials written to ${GARAGE_ENV_OUT}"
fi

echo ""
echo "🎉 Infrastructure setup complete!"
echo ""
echo "📊 Connection Details:"
echo "  PostgreSQL/TimescaleDB: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:5432/${POSTGRES_DB}"
echo "  Qdrant Vector DB:       HTTP http://localhost:${QDRANT_HTTP_PORT} | gRPC http://localhost:${QDRANT_GRPC_PORT}"
echo "  Redis Cache:            redis://localhost:6379"
echo "  Garage Object Storage:  S3 http://localhost:3900 (credentials in ${GARAGE_ENV_OUT})"
echo ""
echo "🚀 Next steps:"
echo "  1. export DATABASE_URL=\"postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:5432/${POSTGRES_DB}\""
echo "  2. export QDRANT_URL=\"http://localhost:${QDRANT_GRPC_PORT}\""
echo "  3. source ${GARAGE_ENV_OUT}    # to export S3 variables for your app"
echo "  4. Run: cd cs2-data-pipeline && cargo run -- init"
echo "  5. Place demo files in ./demos/ directory"
echo "  6. Run: cargo run -- run"
echo ""
echo "📈 Expected Performance (from PDF specifications):"
echo "  - Process 700MB+/s demo data on high-end PC"
echo "  - Support 5TB initial TimescaleDB storage"
echo "  - Handle 2TB vector embeddings in Qdrant"
echo "  - Scale to 20TB+ object storage for demo archives"
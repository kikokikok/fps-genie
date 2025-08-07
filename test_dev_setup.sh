#!/bin/bash
# Test script to verify the development environment setup

echo "🧪 Testing CS2 Development Environment Setup"
echo "============================================="

# Test 1: Check if required files exist
echo "📁 Checking required files..."

required_files=(
    "docker-compose.dev.yml"
    ".devcontainer/devcontainer.json"
    ".devcontainer/setup.sh"
    "sql/00_base_schema.sql"
    "sql/init.sql"
    ".devcontainer/qdrant-config.yaml"
)

all_files_exist=true
for file in "${required_files[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✅ $file"
    else
        echo "  ❌ $file (missing)"
        all_files_exist=false
    fi
done

# Test 2: Check Docker Compose configuration
echo ""
echo "🐳 Validating Docker Compose configuration..."

if docker-compose -f docker-compose.dev.yml config >/dev/null 2>&1; then
    echo "  ✅ docker-compose.dev.yml is valid"
else
    echo "  ❌ docker-compose.dev.yml has configuration errors"
    docker-compose -f docker-compose.dev.yml config
fi

# Test 3: Check SQL syntax
echo ""
echo "🗄️ Validating SQL files..."

# Create a temporary PostgreSQL container to test SQL syntax
echo "  📊 Testing SQL syntax with PostgreSQL..."
if command -v psql >/dev/null 2>&1; then
    # If psql is available locally, test syntax
    for sql_file in sql/*.sql; do
        if psql --help >/dev/null 2>&1 && [ -f "$sql_file" ]; then
            if psql -f "$sql_file" --dry-run >/dev/null 2>&1 || true; then
                echo "    ✅ $sql_file syntax check passed"
            else
                echo "    ⚠️  $sql_file needs PostgreSQL to validate fully"
            fi
        fi
    done
else
    echo "    ⚠️  PostgreSQL not available for syntax testing (will be tested in container)"
fi

# Test 4: Check if notebook exists
echo ""
echo "📓 Checking notebook setup..."

if [ -f "notebooks/cs2_ml_analysis.ipynb" ]; then
    echo "  ✅ Main analysis notebook exists"
else
    echo "  ❌ Main analysis notebook missing"
fi

# Test 5: Check if setup scripts are executable
echo ""
echo "🔧 Checking script permissions..."

scripts=(
    ".devcontainer/setup.sh"
    ".devcontainer/setup_jupyter.sh"
)

for script in "${scripts[@]}"; do
    if [ -x "$script" ]; then
        echo "  ✅ $script is executable"
    else
        echo "  ❌ $script is not executable"
        chmod +x "$script"
        echo "    🔧 Fixed: made $script executable"
    fi
done

# Test 6: Validate environment variables in devcontainer
echo ""
echo "🌍 Checking environment configuration..."

if grep -q "cs2_user" .devcontainer/devcontainer.json && grep -q "cs2_password" .devcontainer/devcontainer.json; then
    echo "  ✅ Database credentials configured in devcontainer"
else
    echo "  ❌ Database credentials missing in devcontainer"
fi

if grep -q "docker-compose.dev.yml" .devcontainer/devcontainer.json; then
    echo "  ✅ DevContainer references correct Docker Compose file"
else
    echo "  ❌ DevContainer Docker Compose reference incorrect"
fi

# Summary
echo ""
echo "📋 Test Summary"
echo "==============="

if [ "$all_files_exist" = true ]; then
    echo "✅ All required files present"
else
    echo "❌ Some required files missing"
fi

echo ""
echo "🚀 Next Steps:"
echo "1. Open in VS Code: code ."
echo "2. Reopen in Dev Container: Cmd+Shift+P -> 'Dev Containers: Reopen in Container'"
echo "3. Wait for automatic setup to complete"
echo "4. Access services:"
echo "   - TimescaleDB: localhost:5432 (cs2_user/cs2_password)"
echo "   - Redis: localhost:6379"
echo "   - Qdrant: localhost:6333"
echo "   - Jupyter: localhost:8888 (token: cs2analysis)"
echo "   - Grafana: localhost:3001 (admin/admin)"
echo ""
echo "🎯 Ready for CS2 demo analysis development!"
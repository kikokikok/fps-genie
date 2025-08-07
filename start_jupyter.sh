#!/bin/bash
# Start Jupyter Lab for CS2 analysis

cd "$(dirname "$0")"

# Create notebooks directory if it doesn't exist
mkdir -p notebooks

cd notebooks

# Set environment variables for database connections
export DATABASE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analytics"
export REDIS_URL="redis://localhost:6379"
export QDRANT_URL="http://localhost:6333"

echo "🚀 Starting Jupyter Lab for CS2 Demo Analysis"
echo "=============================================="
echo ""
echo "🌐 Jupyter Lab will be available at: http://localhost:8888"
echo "🔑 Token: cs2analysis"
echo ""
echo "🔗 Database connections configured:"
echo "  📊 TimescaleDB: $DATABASE_URL"
echo "  🔗 Redis: $REDIS_URL"
echo "  🎯 Qdrant: $QDRANT_URL"
echo ""
echo "📚 Available notebooks:"
echo "  - cs2_ml_analysis.ipynb - Main ML analysis workflow"
echo "  - data_analysis/ - Data exploration notebooks"
echo "  - ml_experiments/ - Machine learning experiments"
echo "  - player_analysis/ - Player performance analysis"
echo ""
echo "Press Ctrl+C to stop Jupyter Lab"
echo ""

# Check if jupyter is available
if ! command -v jupyter >/dev/null 2>&1; then
    echo "❌ Jupyter not found. Installing..."
    pip3 install --user jupyter jupyterlab pandas numpy matplotlib seaborn scikit-learn plotly psycopg2-binary
fi

# Start Jupyter Lab
jupyter lab \
    --ip=0.0.0.0 \
    --port=8888 \
    --no-browser \
    --allow-root \
    --NotebookApp.token='cs2analysis' \
    --NotebookApp.password='' \
    --notebook-dir=.
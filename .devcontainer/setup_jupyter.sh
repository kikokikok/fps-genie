#!/bin/bash
# Jupyter Notebook Setup for CS2 Demo Analysis

echo "🔬 Setting up Jupyter environment for CS2 analysis..."

# Create notebooks directory structure
mkdir -p /workspace/notebooks/{data_analysis,ml_experiments,player_analysis,demo_exploration}

# Set up Python environment with CS2 analysis dependencies
pip3 install --user --break-system-packages \
    jupyter \
    jupyterlab \
    pandas \
    numpy \
    matplotlib \
    seaborn \
    scikit-learn \
    plotly \
    psycopg2-binary \
    redis \
    requests \
    ipywidgets \
    notebook

# Create jupyter config if it doesn't exist
mkdir -p ~/.jupyter
if [ ! -f ~/.jupyter/jupyter_notebook_config.py ]; then
    cat > ~/.jupyter/jupyter_notebook_config.py << 'EOF'
c.NotebookApp.token = 'cs2analysis'
c.NotebookApp.password = ''
c.NotebookApp.open_browser = False
c.NotebookApp.ip = '0.0.0.0'
c.NotebookApp.port = 8888
c.NotebookApp.allow_root = True
c.NotebookApp.notebook_dir = '/workspace/notebooks'
EOF
fi

# Create a startup script for Jupyter
cat > /workspace/start_jupyter.sh << 'EOF'
#!/bin/bash
cd /workspace/notebooks
export DATABASE_URL="postgresql://cs2_user:cs2_password@timescaledb:5432/cs2_analytics"
export REDIS_URL="redis://redis:6379"
export QDRANT_URL="http://qdrant:6333"

echo "🚀 Starting Jupyter Lab..."
echo "📊 Available at: http://localhost:8888"
echo "🔑 Token: cs2analysis"
echo ""
echo "Database connections:"
echo "  📈 TimescaleDB: $DATABASE_URL"
echo "  🔗 Redis: $REDIS_URL"
echo "  🎯 Qdrant: $QDRANT_URL"

jupyter lab --allow-root
EOF

chmod +x /workspace/start_jupyter.sh

echo "✅ Jupyter environment setup complete!"
echo "📚 Start with: ./start_jupyter.sh"
echo "🌐 Access at: http://localhost:8888 (token: cs2analysis)"
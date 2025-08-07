#!/bin/bash
# Post-create script for FPS Genie development container

set -e

echo "🚀 Setting up FPS Genie development environment..."

# Ensure proper ownership
sudo chown -R $(whoami):$(whoami) /workspace

# Install additional development tools if needed
echo "📦 Installing additional development tools..."

# Setup git if not configured
if [ -z "$(git config --global user.name)" ]; then
    echo "⚙️  Setting up git configuration..."
    git config --global user.name "${GITHUB_USER:-Developer}"
    git config --global user.email "${GITHUB_EMAIL:-developer@localhost}"
    git config --global init.defaultBranch main
fi

# Setup Rust environment
echo "🦀 Setting up Rust environment..."

# Add common aliases
cat >> ~/.bashrc << 'EOF'

# FPS Genie Development Aliases
alias cb='cargo build'
alias cr='cargo run'
alias ct='cargo test'
alias cc='cargo check'
alias cf='cargo fmt'
alias ccl='cargo clippy'

# Database aliases
alias db-setup='./setup_databases.sh'
alias db-status='docker compose ps'
alias db-logs='docker compose logs'

# Quick navigation
alias ll='ls -alF'
alias la='ls -A'
alias l='ls -CF'

EOF

# Setup cargo config for faster builds
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[build]
jobs = 4

[net]
retry = 3
timeout = 30

[registries.crates-io]
protocol = "sparse"

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
EOF

# Pre-compile common dependencies if workspace is empty
if [ ! -d "target" ]; then
    echo "🔧 Pre-compiling dependencies for faster development..."
    cargo fetch --quiet || true
fi

# Setup database environment variables
cat >> ~/.bashrc << 'EOF'

# Database Environment Variables
export DATABASE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
export TIMESCALE_URL="postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
export QDRANT_URL="http://localhost:6333"
export REDIS_URL="redis://localhost:6379"
EOF

# Create helpful directories
mkdir -p demos test_data models

echo "✅ Development environment setup complete!"
echo ""
echo "🎯 Quick start commands:"
echo "  db-setup          # Start database infrastructure"
echo "  cargo check       # Quick compile check"
echo "  cargo test        # Run test suite"
echo "  cargo run -p cs2-demo-analyzer -- --help"
echo ""
echo "📊 Database URLs:"
echo "  PostgreSQL: postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis"
echo "  Qdrant:     http://localhost:6333"
echo "  Redis:      redis://localhost:6379"
echo "  MinIO:      http://localhost:9001 (admin/minioadmin123)"
echo ""
echo "Happy coding! 🚀"
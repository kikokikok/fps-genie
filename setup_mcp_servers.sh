#!/bin/bash

# FPS Genie MCP Servers Setup Script
# This script sets up the Model Context Protocol servers for enhanced Copilot functionality

set -e

echo "🚀 Setting up FPS Genie MCP Servers..."
echo ""

# Check Node.js version
echo "📋 Checking Node.js version..."
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 18+ to use MCP servers."
    echo "   Visit: https://nodejs.org/"
    exit 1
fi

NODE_VERSION=$(node -v | sed 's/v//')
MAJOR_VERSION=$(echo $NODE_VERSION | cut -d. -f1)

if [ "$MAJOR_VERSION" -lt 18 ]; then
    echo "❌ Node.js version $NODE_VERSION is too old. MCP servers require Node.js 18+."
    echo "   Please update Node.js: https://nodejs.org/"
    exit 1
fi

echo "✅ Node.js version $NODE_VERSION is compatible"
echo ""

# Install MCP server dependencies
echo "📦 Installing MCP server dependencies..."
cd .mcp-servers

if [ ! -f "package.json" ]; then
    echo "❌ package.json not found in .mcp-servers directory"
    exit 1
fi

npm install

if [ $? -eq 0 ]; then
    echo "✅ MCP server dependencies installed successfully"
else
    echo "❌ Failed to install MCP server dependencies"
    exit 1
fi

cd ..
echo ""

# Make MCP servers executable
echo "🔧 Setting up MCP server permissions..."
chmod +x .mcp-servers/*.js
echo "✅ MCP servers are now executable"
echo ""

# Verify MCP configuration exists
echo "⚙️  Checking MCP configuration..."
if [ ! -f ".mcp/settings.json" ]; then
    echo "❌ MCP settings file not found at .mcp/settings.json"
    exit 1
fi

echo "✅ MCP configuration found"
echo ""

# Check if databases are running (optional)
echo "🗄️  Checking database services (optional)..."
if command -v docker &> /dev/null && command -v docker-compose &> /dev/null; then
    if docker-compose ps | grep -q "Up"; then
        echo "✅ Database services are running"
    else
        echo "⚠️  Database services not detected. Run './setup_databases.sh' if you plan to use database MCP tools."
    fi
else
    echo "⚠️  Docker not available. Database MCP tools will not work without running databases."
fi
echo ""

# Test MCP servers (basic connectivity)
echo "🧪 Testing MCP servers..."

# Test basic server startup (quick test)
echo "Testing database inspector..."
timeout 5s node .mcp-servers/database-inspector.js --test 2>/dev/null || echo "⚠️  Database inspector test failed (expected if databases not running)"

echo "Testing demo analyzer..."
timeout 5s node .mcp-servers/demo-analyzer.js --test 2>/dev/null || echo "⚠️  Demo analyzer test failed"

echo "Testing cargo helper..."
timeout 5s node .mcp-servers/cargo-helper.js --test 2>/dev/null || echo "⚠️  Cargo helper test failed"

echo ""

# Display setup summary
echo "🎉 MCP Server Setup Complete!"
echo ""
echo "📊 Setup Summary:"
echo "   ✅ Node.js $NODE_VERSION (compatible)"
echo "   ✅ MCP server dependencies installed"
echo "   ✅ Server permissions configured"
echo "   ✅ MCP configuration verified"
echo ""

echo "🔗 Available MCP Servers:"
echo "   1. database-inspector.js - Database operations and health checks"
echo "   2. demo-analyzer.js      - CS2 demo file analysis and processing"  
echo "   3. cargo-helper.js       - Rust/Cargo development tools"
echo ""

echo "📚 Usage Instructions:"
echo "   • MCP servers integrate automatically with GitHub Copilot"
echo "   • Use @copilot commands in VS Code for enhanced assistance"
echo "   • See .mcp-servers/README.md for detailed usage examples"
echo ""

echo "🚀 Quick Start:"
echo "   1. Open project in VS Code with Copilot enabled"
echo "   2. Try: '@copilot /explain' for project overview"
echo "   3. Try: '@copilot /build-check' for workspace validation"
echo "   4. Use database tools after running './setup_databases.sh'"
echo ""

echo "💡 Tip: The enhanced Copilot instructions are in .github/copilot-instructions.md"
echo "    They provide comprehensive project knowledge for better assistance."
echo ""

echo "🎯 Ready to enhance your development experience with intelligent Copilot assistance!"
# 🤖 GitHub Copilot Enhanced Configuration

FPS Genie includes comprehensive GitHub Copilot enhancements to provide intelligent development assistance for CS2 demo analysis and AI training workflows.

## 🚀 Enhanced Features

### 1. **Custom Instructions** (`.github/copilot-instructions.md`)
Comprehensive project knowledge for Copilot including:
- ✅ Detailed project architecture and Rust workspace structure
- ✅ Build requirements and timing expectations (2-5 minutes for builds)
- ✅ Platform-specific guidance (Linux Metal issues, macOS acceleration)
- ✅ Database setup and multi-tier architecture
- ✅ ML training workflows and performance targets
- ✅ Troubleshooting guides for common issues

### 2. **Interactive Chat Configuration** (`.github/copilot-chat.json`)
Project-specific chat responses and shortcuts:
- ✅ Quick command shortcuts (`@workspace /explain`, `/debug-demo`, `/optimize-ml`)
- ✅ Specialized agent contexts (demo-parser, ml-engineer, database-architect)
- ✅ Contextual help for common development tasks
- ✅ Performance guidance and build optimization tips

### 3. **Workspace Configuration** (`.github/copilot-workspace.json`)
Detailed workspace structure understanding:
- ✅ Complete crate dependency mapping and build times
- ✅ Development workflow guidance and performance targets
- ✅ Database architecture and connection details
- ✅ Platform-specific configurations and troubleshooting

### 4. **Model Context Protocol (MCP) Servers**
Custom MCP servers providing specialized tools:

#### 🗄️ **Database Inspector** (`.mcp-servers/database-inspector.js`)
- **Purpose**: Multi-tier database operations and health monitoring
- **Capabilities**:
  - Inspect CS2 demo processing status
  - Query TimescaleDB player snapshots with time-series analysis
  - Search Qdrant behavioral vectors for similarity patterns
  - Monitor Redis processing queue status
  - Analyze match performance data
  - Check health of all database services

#### 🎮 **Demo Analyzer** (`.mcp-servers/demo-analyzer.js`)
- **Purpose**: CS2 demo file analysis and processing
- **Capabilities**:
  - List and validate CS2 demo files
  - Analyze demos using cs2-demo-parser
  - Extract metadata without full parsing
  - Run complete processing pipelines
  - Extract key moments (aces, clutches, multi-kills)
  - Compare performance between demos

#### ⚙️ **Cargo Helper** (`.mcp-servers/cargo-helper.js`)
- **Purpose**: Rust/Cargo development tools and workspace management
- **Capabilities**:
  - Analyze workspace structure and build times
  - Build specific crates with platform optimizations
  - Run tests with appropriate configurations
  - Check dependency health and suggest optimizations
  - Apply platform-specific fixes (Linux Metal issues)
  - Monitor build performance and suggest improvements

### 5. **Enhanced Development Environment** (`.devcontainer/`)
Optimized devcontainer with Copilot integration:
- ✅ Pre-configured Rust development tools
- ✅ Enhanced Copilot settings and suggestions
- ✅ Database port forwarding for all services
- ✅ Performance optimizations for large builds
- ✅ Automatic environment setup scripts

## 🔧 Setup Instructions

### Quick Setup
```bash
# 1. Setup MCP servers (one-time setup)
./setup_mcp_servers.sh

# 2. Setup databases (required for database MCP tools)
./setup_databases.sh

# 3. Open in VS Code with Copilot enabled
code .
```

### Manual Setup
```bash
# Install MCP server dependencies
cd .mcp-servers
npm install
chmod +x *.js
cd ..

# Verify configuration files exist
ls -la .github/copilot*.json
ls -la .mcp/settings.json
```

## 🎯 Usage Examples

### Project Overview and Navigation
```
@workspace /explain
# Provides comprehensive project architecture overview
# Shows crate dependencies and build order
# Explains database tiers and data flow
```

### Development Workflow
```
@workspace /new-feature
# Guides through feature development workflow
# Suggests appropriate crate for new features
# Provides development best practices

@workspace /build-check
# Runs cargo workspace health checks
# Provides platform-specific build guidance
# Suggests optimization strategies
```

### Debugging and Troubleshooting
```
@workspace /debug-demo
# Helps debug CS2 demo parsing issues
# Provides file validation and performance checks
# Suggests memory and build optimizations

@workspace /database-help
# Explains multi-tier database architecture
# Provides connection strings and setup guidance
# Helps with query optimization and troubleshooting
```

### ML Development
```
@workspace /optimize-ml
# Guides ML model training and optimization
# Explains platform-specific configurations
# Provides performance tuning suggestions
```

### Quick Commands (via MCP servers)
```
# Cargo operations
cargo_workspace_info()          # Get workspace structure
cargo_build_specific()          # Build with optimizations
cargo_test_runner()            # Run tests with config

# Database operations  
inspect_demo_processing_status() # Check demo processing
query_player_snapshots()        # Time-series data queries
get_database_health()           # Health check all services

# Demo analysis
list_demo_files()              # Find available demos
analyze_demo_file()            # Parse and analyze
extract_key_moments()          # Find clutches, aces
```

## 🚀 Key Benefits

### For Rust Development
- **Build Optimization**: Intelligent suggestions for faster builds (cargo check vs build)
- **Platform Awareness**: Automatic detection and fixes for Linux/macOS issues
- **Dependency Management**: Smart analysis of workspace dependencies
- **Test Strategy**: Appropriate test configurations for unit vs integration tests

### For CS2 Demo Analysis
- **File Validation**: Smart validation before expensive parsing operations
- **Performance Guidance**: Memory and processing expectations for large demos
- **Key Moment Detection**: AI-assisted identification of significant gameplay events
- **Batch Processing**: Intelligent pipeline orchestration for multiple demos

### For Database Operations
- **Multi-Tier Awareness**: Understanding of PostgreSQL, TimescaleDB, Qdrant, Redis roles
- **Query Optimization**: Time-series and vector search optimizations
- **Health Monitoring**: Proactive database health and performance monitoring
- **Schema Understanding**: Deep knowledge of player snapshots and behavioral vectors

### For ML Training
- **Platform Optimization**: Metal vs CPU-only configurations
- **Data Pipeline**: Intelligent data preparation and training workflows
- **Model Management**: Training, serving, and optimization guidance
- **Performance Tuning**: Memory, batch size, and concurrency optimizations

## 📊 Performance Context

### Build Time Awareness
Copilot understands expected build times and won't suggest canceling:
- `cs2-demo-parser`: 10 seconds ✅
- `cs2-common`: 5 seconds ✅
- `cs2-data-pipeline`: 90 seconds ⚠️
- `cs2-ml`: 2+ minutes ⚠️ (platform dependent)

### Processing Expectations
- **Demo Parsing**: 700MB+/second target
- **Database Ingestion**: 10K+ snapshots/second
- **Memory Usage**: 2-8GB for large demos (normal)
- **Integration Tests**: 5-15 minutes (never cancel)

### Scale Targets
- **Storage**: 5TB+ TimescaleDB, 2TB+ vector embeddings
- **Processing**: 50+ professional matches daily
- **Throughput**: Real-time coaching feedback

## 🔍 Troubleshooting

### MCP Server Issues
```bash
# Check Node.js version (requires 18+)
node --version

# Reinstall dependencies
cd .mcp-servers && npm install

# Test server connectivity
./setup_mcp_servers.sh
```

### Copilot Not Using Enhanced Features
1. **Verify files exist**:
   ```bash
   ls -la .github/copilot*.json .mcp/settings.json
   ```

2. **Check VS Code Copilot settings**:
   - Ensure Copilot and Copilot Chat extensions are enabled
   - Restart VS Code after configuration changes

3. **Database MCP requires running services**:
   ```bash
   ./setup_databases.sh
   docker compose ps  # Verify all services are Up
   ```

### Common Issues
- **Build timeouts**: Increase timeout expectations, builds take 2-5 minutes
- **Linux Metal errors**: MCP will suggest `--no-default-features` for cs2-ml
- **Database connections**: MCP will verify service health before operations
- **Demo parsing memory**: MCP monitors and suggests appropriate resource limits

## 🎖️ Advanced Features

### Context-Aware Suggestions
- **Crate-specific guidance**: Different advice for parser vs ML vs pipeline crates
- **Platform intelligence**: Automatic Linux/macOS configuration differences
- **Performance awareness**: Build and runtime performance considerations
- **Architecture understanding**: Multi-tier database and ML pipeline context

### Intelligent Workflows
- **Feature development**: Guided workflows for adding new analysis capabilities
- **Debugging assistance**: Step-by-step troubleshooting for complex issues
- **Performance optimization**: Data-driven suggestions for bottlenecks
- **Testing strategy**: Appropriate test selection for different development phases

### Project-Specific Knowledge
- **Domain expertise**: Understanding of CS2 demos, player behavior, esports analysis
- **Technical depth**: Rust performance, database optimization, ML training
- **Operational awareness**: CI/CD, deployment, monitoring considerations
- **Ecosystem integration**: Docker, TestContainers, cloud deployment

The enhanced Copilot configuration transforms development experience with intelligent, context-aware assistance that understands the specific needs and constraints of professional esports data analysis systems.
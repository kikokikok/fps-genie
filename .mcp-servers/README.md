# FPS Genie Model Context Protocol (MCP) Servers

This directory contains custom MCP servers that provide specialized tools for the FPS Genie CS2 demo analysis system. These servers enhance GitHub Copilot's understanding of the project and provide intelligent assistance for development tasks.

## 🚀 MCP Servers Overview

### 1. Database Inspector (`database-inspector.js`)
Provides specialized database operations for the multi-tier database architecture:

**Capabilities:**
- Inspect CS2 demo processing status
- Query TimescaleDB player snapshots with time-series analysis
- Search Qdrant behavioral vectors for similarity patterns
- Monitor Redis processing queue status
- Analyze match performance data
- Check health of all database services (PostgreSQL, Qdrant, Redis)

**Example Usage:**
```javascript
// Check demo processing status
await inspect_demo_processing_status({ demo_path: "test_data/match.dem" })

// Query player snapshots for a match
await query_player_snapshots({ 
  match_id: "12345", 
  time_range: "1 hour",
  limit: 100 
})

// Search for similar behavioral patterns
await search_behavioral_vectors({ 
  vector: [0.1, 0.2, ...], 
  limit: 10 
})
```

### 2. Demo Analyzer (`demo-analyzer.js`)
Provides CS2 demo file analysis and processing tools:

**Capabilities:**
- List and validate CS2 demo files
- Analyze demo files using cs2-demo-parser
- Extract demo metadata without full parsing
- Run complete demo processing pipelines
- Extract key moments (aces, clutches, multi-kills)
- Compare performance between multiple demos

**Example Usage:**
```javascript
// List available demo files
await list_demo_files({ directory: "test_data", recursive: true })

// Analyze a specific demo
await analyze_demo_file({ 
  demo_path: "test_data/match.dem",
  output_format: "summary" 
})

// Extract key moments
await extract_key_moments({ 
  demo_path: "test_data/match.dem",
  moment_types: ["ace", "clutch"] 
})
```

### 3. Cargo Helper (`cargo-helper.js`)
Provides Rust/Cargo development tools and workspace management:

**Capabilities:**
- Analyze Cargo workspace structure and build times
- Build specific crates with optimized commands
- Run tests with appropriate configurations
- Check dependency health and suggest optimizations
- Apply platform-specific fixes (Linux Metal issues, etc.)
- Monitor build performance and suggest improvements
- Analyze feature flags and suggest optimal combinations

**Example Usage:**
```javascript
// Get workspace information
await cargo_workspace_info({})

// Build specific crate with platform fixes
await cargo_build_specific({ 
  crate_name: "cs2-ml",
  build_type: "check",
  no_default_features: true  // Linux fix
})

// Run optimized tests
await cargo_test_runner({ 
  test_type: "integration",
  parallel: false 
})
```

## 🔧 Setup Instructions

### 1. Install Dependencies
```bash
cd .mcp-servers
npm install
```

### 2. Configure MCP Settings
The MCP configuration is located in `.mcp/settings.json`. Update environment variables as needed:

```json
{
  "mcpServers": {
    "database-inspector": {
      "env": {
        "DATABASE_URL": "postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis",
        "QDRANT_URL": "http://localhost:6333",
        "REDIS_URL": "redis://localhost:6379"
      }
    }
  }
}
```

### 3. Start Database Services
Before using database-related MCP tools:
```bash
./setup_databases.sh
```

### 4. Test MCP Servers
```bash
# Test individual servers
npm run test-database-inspector
npm run test-demo-analyzer  
npm run test-cargo-helper
```

## 🎯 Integration with GitHub Copilot

### Enhanced Copilot Configuration
The MCP servers work alongside enhanced Copilot configurations:

1. **Custom Instructions** (`.github/copilot-instructions.md`)
   - Comprehensive project knowledge
   - Build requirements and timing expectations
   - Platform-specific guidance
   - Troubleshooting workflows

2. **Chat Configuration** (`.github/copilot-chat.json`)
   - Project-specific chat responses
   - Quick command shortcuts
   - Specialized agents for different areas
   - Contextual help responses

3. **Workspace Configuration** (`.github/copilot-workspace.json`)
   - Detailed workspace structure
   - Performance targets and build times
   - Development workflow guidance
   - Platform-specific notes

### Development Environment
The enhanced `.devcontainer/devcontainer.json` includes:
- Optimized Copilot settings
- Rust development tools
- Database port forwarding
- Performance optimizations

## 📊 Performance Considerations

### Database MCP Server
- **PostgreSQL queries**: Optimized for TimescaleDB time-series data
- **Qdrant searches**: Efficient vector similarity operations
- **Redis operations**: Fast queue status checks
- **Connection pooling**: Reuses database connections

### Demo Analyzer MCP Server
- **File validation**: Quick checks before expensive parsing
- **Streaming output**: Handles large demo file outputs
- **Timeout management**: Appropriate timeouts for different operations
- **Memory management**: Monitors resource usage during parsing

### Cargo Helper MCP Server
- **Build optimization**: Suggests fastest build strategies
- **Parallel execution**: Manages concurrent operations
- **Platform detection**: Automatically applies platform-specific fixes
- **Incremental builds**: Leverages Cargo's incremental compilation

## 🔍 Troubleshooting

### Common Issues

**MCP Server Connection Issues:**
```bash
# Check Node.js version (requires 18+)
node --version

# Verify dependencies
cd .mcp-servers && npm install

# Test server startup
node database-inspector.js --test
```

**Database Connection Failures:**
```bash
# Ensure databases are running
docker compose ps

# Check connection strings in .mcp/settings.json
# Wait for full database initialization (2-3 minutes)
```

**Permission Issues:**
```bash
# Make MCP servers executable
chmod +x .mcp-servers/*.js

# Check file permissions
ls -la .mcp-servers/
```

### Performance Issues

**Slow Database Queries:**
- Check TimescaleDB chunk intervals
- Optimize query time ranges
- Use appropriate indexes

**Demo Parsing Timeouts:**
- Increase timeout for large demos (50MB+ files)
- Monitor memory usage (2-8GB normal for large demos)
- Use streaming processing for very large files

**Build Performance:**
- Use `cargo check` instead of `cargo build` for iteration
- Enable incremental compilation
- Consider using `sccache` for dependency caching

## 🚀 Usage Examples

### Typical Development Workflow

1. **Start Development Session:**
```bash
# @copilot /setup-development
# This triggers the cargo helper to check workspace health
```

2. **Analyze Demo Files:**
```bash
# @copilot /analyze-demo test_data/match.dem
# Uses demo analyzer MCP to provide comprehensive analysis
```

3. **Check Database Status:**
```bash
# @copilot /check-databases
# Uses database inspector to verify all services are healthy
```

4. **Build and Test:**
```bash
# @copilot /build-crate cs2-demo-parser
# Uses cargo helper with optimized build commands
```

### Advanced Operations

**Performance Analysis:**
- Use database inspector to query player snapshot data
- Analyze behavioral vectors for pattern recognition
- Monitor processing queue performance

**ML Model Development:**
- Extract training data using demo analyzer
- Query behavioral embeddings with database inspector
- Optimize build configurations with cargo helper

**Integration Testing:**
- Validate demo files before processing
- Check database health before test runs
- Monitor resource usage during tests

## 📚 Additional Resources

- **Project Documentation**: See `PROJECT_CONTEXT.md` for architecture overview
- **Build Instructions**: See `.github/copilot-instructions.md` for detailed build guidance
- **Database Schema**: See `sql/init.sql` for database structure
- **CI/CD**: See `.github/workflows/` for automated testing and deployment

The MCP servers provide intelligent, context-aware assistance that understands the specific needs and constraints of the FPS Genie CS2 demo analysis system.
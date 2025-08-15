# CS2 Demo Parser Next - Agentic Mesh Architecture

A next-generation CS2 demo parser built using Winnow parser combinators and an agentic mesh development approach.

## Overview

This parser represents a modern, high-performance approach to CS2 demo file parsing, designed with:

- **Winnow Parser Combinators**: For composable, fast parsing
- **Agentic Mesh Architecture**: Collaborative development with specialized agents
- **Performance-First Design**: Targeting 700MB+/s parsing speeds
- **Modular Structure**: Clean separation of concerns

## Agentic Mesh Architecture

The development follows an agentic mesh approach with specialized roles:

| Agent | Responsibility | Status |
|-------|---------------|--------|
| **Parser Agent** | Core Winnow parser implementation | ✅ Phase 1 Complete |
| **Test Agent** | Testing infrastructure and fixtures | ✅ Basic framework |
| **Documentation Agent** | Technical documentation | ✅ Initial docs |
| **Performance Agent** | Benchmarking and optimization | 🔄 In progress |
| **Integration Agent** | System integration and examples | 📋 Planned |
| **Review Agent** | Code quality and best practices | 📋 Planned |

## Performance Targets

- **Parsing Speed**: 700+ MB/s on high-end hardware
- **Memory Usage**: <100 MB for typical demos
- **Initialization**: <500ms startup time

## Quick Start

```bash
# Build the parser
cargo build --release -p cs2-demo-parser-next

# Parse a demo file
cargo run --release -p cs2-demo-parser-next --bin parser-next -- demo.dem

# Run tests
cargo test -p cs2-demo-parser-next

# Run benchmarks
cargo bench -p cs2-demo-parser-next
```

## Usage Example

```rust
use cs2_demo_parser_next::{DemoParser, DemoHeader};

let demo_data = std::fs::read("demo.dem")?;
let mut parser = DemoParser::new();

// Parse header
let header = parser.parse_header(&demo_data)?;
println!("Map: {}, Server: {}", header.map_name, header.server_name);

// Parse frames
while let Some(frame) = parser.parse_frame(&demo_data)? {
    println!("Frame: {:?} at tick {}", frame.command, frame.tick);
}

// Check performance
let metrics = parser.metrics();
println!("Parsed at {:.2} MB/s", metrics.parsing_speed_mbs);
```

## Architecture

```
├── src/
│   ├── parser.rs          # Core Winnow-based parsers (Parser Agent)
│   ├── entities.rs        # Entity management system
│   ├── events.rs          # Game event processing
│   ├── net_messages.rs    # Network message parsing
│   ├── game_state.rs      # Game state tracking
│   ├── common.rs          # Shared utilities and errors
│   └── bin/parser.rs      # CLI binary
├── tests/                 # Test fixtures and integration tests (Test Agent)
├── benches/              # Performance benchmarks (Performance Agent)
└── examples/             # Usage examples (Integration Agent)
```

## Development Phases

### ✅ Phase 1: Foundation (Current)
- [x] Project scaffolding with Cargo.toml
- [x] Winnow-based header parser
- [x] Basic command packet parser
- [x] Test infrastructure framework
- [x] Performance metrics system
- [x] CLI binary interface

### 🔄 Phase 2: Core Functionality (Next)
- [ ] Network message parsers with Winnow
- [ ] Entity update system implementation
- [ ] Game event parsing framework
- [ ] Comprehensive test fixtures with real demos
- [ ] Performance benchmarking suite

### 📋 Phase 3: Advanced Features (Future)
- [ ] Player state tracking system
- [ ] Game state management
- [ ] Event system for coaching insights
- [ ] Memory optimization and zero-copy strategies
- [ ] Example applications

### 📋 Phase 4: Integration & Release (Future)  
- [ ] Integration with existing fps-genie ecosystem
- [ ] Complete documentation and user guide
- [ ] Performance validation against targets
- [ ] Production release preparation

## Contributing

This project follows the agentic mesh development model. Contributors can take on specific agent roles:

1. **Fork** and **clone** the repository
2. **Choose an agent role** (Parser, Test, Documentation, Performance, Integration)
3. **Work on agent-specific tasks** from the project phases
4. **Submit PRs** with agent role indicated in commit messages
5. **Coordinate** with other agents through issues and discussions

## Performance Notes

- Built with Winnow for composable, high-performance parsing
- Uses zero-copy strategies where possible
- Designed for streaming large demo files
- Optimized for batch processing scenarios

## License

MIT License - see LICENSE file for details
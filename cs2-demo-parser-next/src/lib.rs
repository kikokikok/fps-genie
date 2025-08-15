//! # CS2 Demo Parser Next - Agentic Mesh Architecture
//!
//! This is a next-generation CS2 demo parser built using the Winnow parser combinator library
//! and an agentic mesh development approach. The system is designed for high-performance
//! parsing of CS2 demo files with a focus on modularity and extensibility.
//!
//! ## Architecture Overview
//!
//! The parser is organized around specialized "agents" (modules) that handle different
//! aspects of demo parsing:
//!
//! - **Parser Agent**: Core Winnow-based parsers for headers, commands, and packets
//! - **Test Agent**: Comprehensive testing infrastructure with fixtures
//! - **Documentation Agent**: Technical documentation and examples
//! - **Performance Agent**: Benchmarking and optimization
//! - **Integration Agent**: System integration and examples
//!
//! ## Usage
//!
//! ```rust
//! use cs2_demo_parser_next::{DemoParser, DemoHeader};
//! 
//! let demo_data = std::fs::read("demo.dem")?;
//! let parser = DemoParser::new();
//! let header = parser.parse_header(&demo_data)?;
//! println!("Demo: {} on {}", header.server_name, header.map_name);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod parser;
pub mod entities;
pub mod events;
pub mod net_messages;
pub mod game_state;
pub mod common;

// Re-export main types for convenience
pub use parser::{DemoParser, DemoHeader, DemoCommand, DemoFrame};
pub use common::{Error, Result};

#[cfg(feature = "tracing")]
pub use tracing;

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parser performance targets
pub mod performance {
    /// Target parsing speed in MB/s
    pub const TARGET_PARSING_SPEED_MBS: u64 = 700;
    
    /// Maximum memory usage target in MB
    pub const MAX_MEMORY_USAGE_MB: usize = 100;
    
    /// Initialization time target in milliseconds  
    pub const INIT_TIME_TARGET_MS: u64 = 500;
}

/// Agentic mesh configuration and coordination
pub mod mesh {
    //! Coordination utilities for the agentic mesh development approach
    
    /// Agent responsibilities and capabilities
    #[derive(Debug, Clone, PartialEq)]
    pub enum AgentRole {
        Parser,
        Test, 
        Documentation,
        Performance,
        Integration,
        Review,
        Orchestrator,
    }
    
    /// Task priority levels for agent coordination
    #[derive(Debug, Clone, PartialEq, PartialOrd)]
    pub enum TaskPriority {
        Critical,
        High,
        Medium,
        Low,
    }
}
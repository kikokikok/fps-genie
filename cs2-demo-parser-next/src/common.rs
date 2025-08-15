//! Common utilities and types for the CS2 demo parser
//!
//! This module provides shared functionality used across the agentic mesh,
//! including error types, result types, and common data structures.

use std::fmt;
use thiserror::Error;

/// Result type used throughout the parser
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error types for demo parsing
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parsing error: {message} at position {position}")]
    Parse {
        message: String,
        position: usize,
    },
    
    #[error("Invalid demo format: {0}")]
    InvalidFormat(String),
    
    #[error("Unsupported demo version: {version}")]
    UnsupportedVersion { version: String },
    
    #[error("Malformed packet: {packet_type} - {details}")]
    MalformedPacket {
        packet_type: String,
        details: String,
    },
    
    #[error("Missing required data: {0}")]
    MissingData(String),
    
    #[error("Buffer underrun: expected {expected} bytes, got {actual}")]
    BufferUnderrun { expected: usize, actual: usize },
    
    #[error("Protocol buffer error: {0}")]
    Protobuf(#[from] prost::DecodeError),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Agent coordination error: {agent} - {message}")]
    AgentCoordination {
        agent: String,
        message: String,
    },
}

/// Demo parsing configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Enable verbose logging
    pub verbose: bool,
    
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    
    /// Enable performance tracking
    pub track_performance: bool,
    
    /// Parse only specific packet types
    pub packet_filter: Option<Vec<String>>,
    
    /// Enable experimental features
    pub experimental: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            max_memory_mb: 100,
            track_performance: true,
            packet_filter: None,
            experimental: false,
        }
    }
}

/// Performance metrics for parser operations
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Total parsing time in milliseconds
    pub parse_time_ms: u64,
    
    /// Number of bytes processed
    pub bytes_processed: usize,
    
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    
    /// Number of packets parsed
    pub packets_parsed: usize,
    
    /// Parsing speed in MB/s
    pub parsing_speed_mbs: f64,
}

impl PerformanceMetrics {
    /// Calculate parsing speed
    pub fn calculate_speed(&mut self) {
        if self.parse_time_ms > 0 {
            let seconds = self.parse_time_ms as f64 / 1000.0;
            let megabytes = self.bytes_processed as f64 / (1024.0 * 1024.0);
            self.parsing_speed_mbs = megabytes / seconds;
        }
    }
}

impl fmt::Display for PerformanceMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Performance: {:.2} MB/s, {} packets, {:.2} MB processed in {} ms",
            self.parsing_speed_mbs,
            self.packets_parsed,
            self.bytes_processed as f64 / (1024.0 * 1024.0),
            self.parse_time_ms
        )
    }
}

/// Utility functions for byte manipulation
pub mod bytes {
    /// Read a little-endian u32 from bytes
    pub fn read_u32_le(bytes: &[u8]) -> Option<u32> {
        if bytes.len() >= 4 {
            Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            None
        }
    }
    
    /// Read a little-endian u16 from bytes
    pub fn read_u16_le(bytes: &[u8]) -> Option<u16> {
        if bytes.len() >= 2 {
            Some(u16::from_le_bytes([bytes[0], bytes[1]]))
        } else {
            None
        }
    }
    
    /// Check if we have enough bytes for a read operation
    pub fn check_bounds(data: &[u8], position: usize, required: usize) -> crate::Result<()> {
        if position + required > data.len() {
            Err(crate::Error::BufferUnderrun {
                expected: required,
                actual: data.len() - position,
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_metrics_speed_calculation() {
        let mut metrics = PerformanceMetrics {
            parse_time_ms: 1000, // 1 second
            bytes_processed: 1024 * 1024, // 1 MB
            ..Default::default()
        };
        
        metrics.calculate_speed();
        assert_eq!(metrics.parsing_speed_mbs, 1.0);
    }
    
    #[test] 
    fn test_bytes_utilities() {
        let data = vec![0x12, 0x34, 0x56, 0x78];
        assert_eq!(bytes::read_u32_le(&data), Some(0x78563412));
        assert_eq!(bytes::read_u16_le(&data), Some(0x3412));
    }
    
    #[test]
    fn test_bounds_checking() {
        let data = vec![1, 2, 3, 4];
        assert!(bytes::check_bounds(&data, 0, 4).is_ok());
        assert!(bytes::check_bounds(&data, 0, 5).is_err());
        assert!(bytes::check_bounds(&data, 2, 3).is_err());
    }
}
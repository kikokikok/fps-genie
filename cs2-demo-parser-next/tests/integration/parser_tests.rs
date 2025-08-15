//! Integration tests for the CS2 Demo Parser Next
//!
//! Test Agent responsibility - comprehensive testing of parser functionality

use cs2_demo_parser_next::{DemoParser, DemoCommand};

#[test]
fn test_parser_creation() {
    let parser = DemoParser::new();
    assert_eq!(parser.position(), 0);
    assert!(parser.header().is_none());
}

#[test]
fn test_demo_command_parsing() {
    use cs2_demo_parser_next::DemoCommand;
    
    // Test valid commands
    assert_eq!(DemoCommand::try_from(1).unwrap(), DemoCommand::SignOn);
    assert_eq!(DemoCommand::try_from(2).unwrap(), DemoCommand::Packet);
    assert_eq!(DemoCommand::try_from(7).unwrap(), DemoCommand::Stop);
    
    // Test invalid command
    assert!(DemoCommand::try_from(255).is_err());
}

#[test]
fn test_header_parsing_with_invalid_data() {
    let mut parser = DemoParser::new();
    
    // Test with insufficient data
    let invalid_data = vec![1, 2, 3, 4];
    let result = parser.parse_header(&invalid_data);
    assert!(result.is_err());
}

#[test]
fn test_header_parsing_with_wrong_signature() {
    let mut parser = DemoParser::new();
    
    // Test with wrong signature  
    let mut wrong_data = vec![0; 100];
    wrong_data[..8].copy_from_slice(b"WRONGSIG");
    let result = parser.parse_header(&wrong_data);
    assert!(result.is_err());
}

#[test]
fn test_performance_metrics() {
    use cs2_demo_parser_next::common::PerformanceMetrics;
    
    let mut metrics = PerformanceMetrics {
        parse_time_ms: 1000, // 1 second
        bytes_processed: 1024 * 1024, // 1 MB
        ..Default::default()
    };
    
    metrics.calculate_speed();
    assert_eq!(metrics.parsing_speed_mbs, 1.0);
}

// TODO: Add tests with actual demo file fixtures once we have test data
// This would be part of the Test Agent's comprehensive test suite

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    // Placeholder for integration tests that would use real demo files
    // These would be implemented as part of Phase 2 of the agentic mesh plan
    
    #[test]
    #[ignore] // Ignore until we have test fixtures
    fn test_parse_real_demo_file() {
        // This test would use actual demo files from test_data directory
        // let demo_data = std::fs::read("../test_data/test_demo.dem").unwrap();
        // let mut parser = DemoParser::new();
        // let header = parser.parse_header(&demo_data).unwrap();
        // assert!(!header.map_name.is_empty());
    }
}
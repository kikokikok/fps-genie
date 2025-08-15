//! CS2 Demo Parser Next - CLI Binary
//!
//! Command-line interface for the next-generation CS2 demo parser
//! built with the agentic mesh architecture.

use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use cs2_demo_parser_next::{DemoParser, performance};

fn main() -> Result<()> {
    // Simple CLI for now - just parse header of first argument
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <demo_file.dem>", args[0]);
        eprintln!();
        eprintln!("CS2 Demo Parser Next - Agentic Mesh Architecture");
        eprintln!("Performance targets:");
        eprintln!("  - Parsing speed: {} MB/s", performance::TARGET_PARSING_SPEED_MBS);
        eprintln!("  - Memory usage: <{} MB", performance::MAX_MEMORY_USAGE_MB);
        eprintln!("  - Init time: <{} ms", performance::INIT_TIME_TARGET_MS);
        std::process::exit(1);
    }
    
    let demo_path = PathBuf::from(&args[1]);
    println!("Parsing demo file: {}", demo_path.display());
    
    // Read demo file
    let demo_data = fs::read(&demo_path)
        .with_context(|| format!("Failed to read demo file: {}", demo_path.display()))?;
    
    println!("Demo file size: {:.2} MB", demo_data.len() as f64 / (1024.0 * 1024.0));
    
    // Parse header
    let mut parser = DemoParser::new();
    let header = parser.parse_header(&demo_data)
        .context("Failed to parse demo header")?;
    
    // Display header information
    println!("\n=== Demo Header ===");
    println!("File format: {}", header.demo_file_stamp);
    println!("Network protocol: {}", header.network_protocol);
    println!("Server: {}", header.server_name);
    println!("Client: {}", header.client_name);
    println!("Map: {}", header.map_name);
    println!("Game directory: {}", header.game_directory);
    println!("Playback time: {:.2} seconds", header.playback_time);
    println!("Playback ticks: {}", header.playback_ticks);
    println!("Playback frames: {}", header.playback_frames);
    println!("Signon length: {} bytes", header.signon_length);
    
    // Parse a few frames to demonstrate functionality
    println!("\n=== Parsing Frames ===");
    let mut frame_count = 0;
    while frame_count < 5 {
        match parser.parse_frame(&demo_data)? {
            Some(frame) => {
                println!("Frame {}: {:?} at tick {} ({} bytes)", 
                    frame_count + 1, frame.command, frame.tick, frame.data.len());
                frame_count += 1;
            }
            None => {
                println!("End of demo reached");
                break;
            }
        }
    }
    
    // Display performance metrics
    let metrics = parser.metrics();
    println!("\n=== Performance Metrics ===");
    println!("{}", metrics);
    
    // Verify against targets
    println!("\n=== Performance Validation ===");
    if metrics.parsing_speed_mbs >= performance::TARGET_PARSING_SPEED_MBS as f64 {
        println!("✓ Parsing speed target met: {:.2} MB/s >= {} MB/s", 
            metrics.parsing_speed_mbs, performance::TARGET_PARSING_SPEED_MBS);
    } else {
        println!("⚠ Parsing speed below target: {:.2} MB/s < {} MB/s", 
            metrics.parsing_speed_mbs, performance::TARGET_PARSING_SPEED_MBS);
    }
    
    if metrics.parse_time_ms <= performance::INIT_TIME_TARGET_MS {
        println!("✓ Init time target met: {} ms <= {} ms", 
            metrics.parse_time_ms, performance::INIT_TIME_TARGET_MS);
    } else {
        println!("⚠ Init time above target: {} ms > {} ms", 
            metrics.parse_time_ms, performance::INIT_TIME_TARGET_MS);
    }
    
    println!("\nParsing completed successfully!");
    Ok(())
}
//! Performance benchmarks for CS2 Demo Parser Next
//!
//! Performance Agent responsibility - benchmarking and optimization tracking

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use cs2_demo_parser_next::{DemoParser, performance};

/// Benchmark header parsing performance
fn benchmark_header_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("header_parsing");
    
    // Create a realistic demo header for benchmarking
    let demo_header = create_test_header_data();
    
    group.bench_function("parse_header", |b| {
        b.iter(|| {
            let mut parser = DemoParser::new();
            parser.parse_header(black_box(&demo_header)).unwrap()
        })
    });
    
    group.finish();
}

/// Benchmark frame parsing performance  
fn benchmark_frame_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_parsing");
    
    // Create test frame data of different sizes
    for size in [1024, 4096, 16384, 65536].iter() {
        let frame_data = create_test_frame_data(*size);
        
        group.bench_with_input(
            BenchmarkId::new("parse_frame", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut parser = DemoParser::new();
                    // Skip header for frame parsing test
                    parser.parse_frame(black_box(&frame_data))
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark complete demo parsing performance
fn benchmark_complete_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_parsing");
    group.measurement_time(std::time::Duration::from_secs(10));
    
    // Create test demo data of different sizes
    for mb_size in [1, 5, 10].iter() {
        let demo_data = create_test_demo_data(*mb_size * 1024 * 1024);
        
        group.bench_with_input(
            BenchmarkId::new("complete_demo", format!("{}MB", mb_size)),
            &demo_data,
            |b, data| {
                b.iter(|| {
                    let mut parser = DemoParser::new();
                    let _header = parser.parse_header(black_box(data)).unwrap();
                    
                    // Parse several frames
                    for _ in 0..10 {
                        if parser.parse_frame(black_box(data)).unwrap().is_none() {
                            break;
                        }
                    }
                    
                    parser.metrics().parsing_speed_mbs
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    let large_demo = create_test_demo_data(10 * 1024 * 1024); // 10MB
    
    group.bench_function("memory_efficiency", |b| {
        b.iter(|| {
            let mut parser = DemoParser::new();
            let _header = parser.parse_header(black_box(&large_demo)).unwrap();
            
            // Simulate parsing workflow
            let mut frame_count = 0;
            while frame_count < 100 {
                match parser.parse_frame(black_box(&large_demo)).unwrap() {
                    Some(_) => frame_count += 1,
                    None => break,
                }
            }
            
            parser.metrics().bytes_processed
        })
    });
    
    group.finish();
}

/// Create test header data that matches expected CS2 demo format
fn create_test_header_data() -> Vec<u8> {
    let mut data = Vec::new();
    
    // HL2DEMO signature
    data.extend_from_slice(b"HL2DEMO\0");
    
    // Demo protocol version
    data.extend_from_slice(&1u32.to_le_bytes());
    
    // Network protocol version  
    data.extend_from_slice(&13000u32.to_le_bytes());
    
    // Server name (null-terminated)
    data.extend_from_slice(b"Test Server\0");
    
    // Client name (null-terminated)
    data.extend_from_slice(b"Test Client\0");
    
    // Map name (null-terminated)
    data.extend_from_slice(b"de_dust2\0");
    
    // Game directory (null-terminated)
    data.extend_from_slice(b"csgo\0");
    
    // Playback time (f32)
    data.extend_from_slice(&60.0f32.to_le_bytes());
    
    // Playback ticks
    data.extend_from_slice(&64000u32.to_le_bytes());
    
    // Playback frames
    data.extend_from_slice(&32000u32.to_le_bytes());
    
    // Signon length
    data.extend_from_slice(&1024u32.to_le_bytes());
    
    data
}

/// Create test frame data for benchmarking
fn create_test_frame_data(size: usize) -> Vec<u8> {
    let mut data = create_test_header_data();
    
    // Add a test frame
    data.push(2); // Packet command
    data.extend_from_slice(&1000u32.to_le_bytes()); // Tick
    data.extend_from_slice(&(size as u32).to_le_bytes()); // Data length
    data.extend(vec![0xAB; size]); // Frame data
    
    data
}

/// Create test demo data of specified size
fn create_test_demo_data(total_size: usize) -> Vec<u8> {
    let header = create_test_header_data();
    let mut data = header;
    
    // Fill remaining space with test frames
    while data.len() < total_size {
        let remaining = total_size - data.len();
        let frame_size = std::cmp::min(remaining.saturating_sub(9), 1024); // Leave room for frame header
        
        if frame_size < 9 {
            break;
        }
        
        // Add frame header
        data.push(2); // Packet command
        data.extend_from_slice(&(data.len() as u32).to_le_bytes()); // Tick
        data.extend_from_slice(&((frame_size - 9) as u32).to_le_bytes()); // Data length
        data.extend(vec![0xCD; frame_size - 9]); // Frame data
    }
    
    data
}

criterion_group!(
    benches,
    benchmark_header_parsing,
    benchmark_frame_parsing,
    benchmark_complete_parsing,
    benchmark_memory_usage
);
criterion_main!(benches);
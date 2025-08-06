use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use std::path::Path;

/// Benchmark real demo file processing
fn benchmark_real_demo_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_demo_processing");
    group.measurement_time(Duration::from_secs(120)); // 2 minutes for real demo
    group.sample_size(3); // Only 3 samples since this is expensive

    let demo_path = Path::new("../test_data/vitality-vs-spirit-m1-dust2.dem");

    if !demo_path.exists() {
        eprintln!("Real demo file not found at: {:?}", demo_path);
        return;
    }

    group.bench_function("vitality_vs_spirit_parsing", |b| {
        b.iter(|| {
            let result = cs2_ml::data::vectors_from_demo(demo_path);
            match result {
                Ok(vectors) => {
                    println!("Successfully parsed {} behavioral vectors from real demo", vectors.len());
                    black_box(vectors.len())
                }
                Err(e) => {
                    eprintln!("Error parsing real demo: {}", e);
                    black_box(0)
                }
            }
        })
    });

    group.finish();
}

/// Benchmark different demo file sizes
fn benchmark_demo_size_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("demo_size_comparison");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(5);

    let demo_files = [
        ("test_demo", "../test_data/test_demo.dem"),
        ("vitality_vs_spirit", "../test_data/vitality-vs-spirit-m1-dust2.dem"),
    ];

    for (name, path) in demo_files.iter() {
        let demo_path = Path::new(path);
        if demo_path.exists() {
            group.bench_function(*name, |b| {
                b.iter(|| {
                    let result = cs2_ml::data::vectors_from_demo(demo_path);
                    match result {
                        Ok(vectors) => {
                            println!("Parsed {} vectors from {}", vectors.len(), name);
                            black_box(vectors.len())
                        }
                        Err(e) => {
                            eprintln!("Error parsing {}: {}", name, e);
                            black_box(0)
                        }
                    }
                })
            });
        } else {
            eprintln!("Demo file not found: {}", path);
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_real_demo_processing,
    benchmark_demo_size_comparison
);
criterion_main!(benches);

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tokio::runtime::Runtime;

use cs2_integration_tests::test_infrastructure::{TestInfrastructure, TestDataFactory};
use testcontainers::clients;

/// Benchmark the complete demo processing pipeline
fn benchmark_pipeline_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("pipeline_processing", |b| {
        b.to_async(&rt).iter(|| async {
            let docker = clients::Cli::default();
            let infra = TestInfrastructure::new(&docker).await.unwrap();
            let processor = infra.create_demo_processor().await.unwrap();
            
            // Create test match data
            let test_match = TestDataFactory::create_test_match("benchmark_001");
            let match_id = infra.db_manager.postgres.insert_match(&test_match).await.unwrap();
            
            // Generate realistic test data
            let snapshots = TestDataFactory::create_player_snapshots(
                black_box(match_id),
                black_box(1000), // 1000 ticks
                black_box(10),   // 10 players
            );
            
            // Benchmark batch insert
            infra.db_manager.timescale
                .insert_snapshots_batch(&snapshots)
                .await
                .unwrap();
            
            snapshots.len()
        })
    });
}

/// Benchmark database operations
fn benchmark_database_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("database_operations");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("batch_insert_1k", |b| {
        b.to_async(&rt).iter(|| async {
            let docker = clients::Cli::default();
            let infra = TestInfrastructure::new(&docker).await.unwrap();
            
            let test_match = TestDataFactory::create_test_match("bench_1k");
            let match_id = infra.db_manager.postgres.insert_match(&test_match).await.unwrap();
            
            let snapshots = TestDataFactory::create_player_snapshots(
                match_id,
                black_box(1000),
                black_box(5),
            );
            
            infra.db_manager.timescale
                .insert_snapshots_batch(&snapshots)
                .await
                .unwrap();
        })
    });
    
    group.bench_function("batch_insert_10k", |b| {
        b.to_async(&rt).iter(|| async {
            let docker = clients::Cli::default();
            let infra = TestInfrastructure::new(&docker).await.unwrap();
            
            let test_match = TestDataFactory::create_test_match("bench_10k");
            let match_id = infra.db_manager.postgres.insert_match(&test_match).await.unwrap();
            
            let snapshots = TestDataFactory::create_player_snapshots(
                match_id,
                black_box(10000),
                black_box(5),
            );
            
            infra.db_manager.timescale
                .insert_snapshots_batch(&snapshots)
                .await
                .unwrap();
        })
    });
    
    group.finish();
}

/// Benchmark ML data processing
fn benchmark_ml_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_operations");
    
    group.bench_function("behavioral_vector_creation", |b| {
        b.iter(|| {
            let vectors: Vec<_> = (0..black_box(1000))
                .map(|i| TestDataFactory::create_behavioral_vector(76561198034202275, i))
                .collect();
            vectors.len()
        })
    });
    
    group.bench_function("parquet_export", |b| {
        b.iter(|| {
            let vectors: Vec<_> = (0..1000)
                .map(|i| TestDataFactory::create_behavioral_vector(76561198034202275, i))
                .collect();
            
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            cs2_ml::data::write_to_parquet(&vectors, temp_file.path()).unwrap();
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_pipeline_processing,
    benchmark_database_operations,
    benchmark_ml_operations
);
criterion_main!(benches);

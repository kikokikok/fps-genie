use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use tokio::runtime::Runtime;

use cs2_integration_tests::test_infrastructure::{TestInfrastructure, TestDataFactory};

/// Benchmark the complete demo processing pipeline
fn benchmark_pipeline_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Set up infrastructure once for all benchmarks
    let infra = rt.block_on(async {
        TestInfrastructure::new().await.expect("Failed to create test infrastructure")
    });

    let mut group = c.benchmark_group("pipeline_processing");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(5);

    // Benchmark different batch sizes
    for batch_size in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_insert", batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    rt.block_on(async {
                        // Create test match data
                        let test_match = TestDataFactory::create_test_match(&format!("benchmark_{}", uuid::Uuid::new_v4()));
                        let match_id = infra.db_manager().postgres.insert_match(&test_match).await.unwrap();

                        // Generate realistic test data
                        let snapshots = TestDataFactory::create_player_snapshots(
                            black_box(match_id),
                            black_box(batch_size),
                            black_box(10),   // 10 players
                        );

                        // Benchmark batch insert
                        infra.db_manager().timescale
                            .insert_snapshots_batch(&snapshots)
                            .await
                            .unwrap();

                        black_box(snapshots.len())
                    })
                })
            },
        );
    }
    group.finish();
}

/// Benchmark database operations
fn benchmark_database_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Set up infrastructure once
    let infra = rt.block_on(async {
        TestInfrastructure::new().await.expect("Failed to create test infrastructure")
    });

    let mut group = c.benchmark_group("database_operations");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(10);

    // Benchmark match operations
    group.bench_function("match_insert", |b| {
        b.iter(|| {
            rt.block_on(async {
                let test_match = TestDataFactory::create_test_match(&format!("db_bench_{}", uuid::Uuid::new_v4()));
                let match_id = infra.db_manager().postgres.insert_match(&test_match).await.unwrap();
                black_box(match_id)
            })
        })
    });

    group.bench_function("match_query_unprocessed", |b| {
        b.iter(|| {
            rt.block_on(async {
                let matches = infra.db_manager().postgres.get_unprocessed_matches().await.unwrap();
                black_box(matches.len())
            })
        })
    });

    // Benchmark player snapshot queries
    group.bench_function("snapshot_query", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Create some test data first
                let test_match = TestDataFactory::create_test_match(&format!("query_bench_{}", uuid::Uuid::new_v4()));
                let match_id = infra.db_manager().postgres.insert_match(&test_match).await.unwrap();
                let snapshots = TestDataFactory::create_player_snapshots(match_id, 100, 5);
                infra.db_manager().timescale.insert_snapshots_batch(&snapshots).await.unwrap();

                // Benchmark the query
                let result = infra.db_manager().timescale
                    .get_player_snapshots(match_id, 76561198034202275, Some(50))
                    .await
                    .unwrap();
                black_box(result.len())
            })
        })
    });

    group.finish();
}

/// Benchmark vector operations
fn benchmark_vector_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Set up infrastructure once
    let infra = rt.block_on(async {
        TestInfrastructure::new().await.expect("Failed to create test infrastructure")
    });

    let mut group = c.benchmark_group("vector_operations");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(5);

    // Benchmark vector storage
    group.bench_function("vector_store", |b| {
        b.iter(|| {
            rt.block_on(async {
                let embedding = cs2_data_pipeline::models::BehavioralEmbedding {
                    id: format!("bench_embedding_{}", uuid::Uuid::new_v4()),
                    match_id: uuid::Uuid::new_v4().to_string(),
                    moment_id: format!("moment_{}", uuid::Uuid::new_v4()),
                    player_steamid: 76561198034202275,
                    moment_type: "clutch".to_string(),
                    vector: (0..256).map(|i| (i as f32) * 0.01).collect(),
                    metadata: serde_json::json!({"benchmark": true}),
                };

                infra.db_manager().vector.store_behavioral_vector(&embedding).await.unwrap();
                black_box(embedding.vector.len())
            })
        })
    });

    // Benchmark vector search
    group.bench_function("vector_search", |b| {
        b.iter(|| {
            rt.block_on(async {
                let query_vector: Vec<f32> = (0..256).map(|i| (i as f32) * 0.01).collect();
                let results = infra.db_manager().vector
                    .search_similar_behaviors(&query_vector, 10)
                    .await
                    .unwrap();
                black_box(results.len())
            })
        })
    });

    group.finish();
}

/// Benchmark ML data processing
fn benchmark_ml_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_processing");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(10);

    // Benchmark behavioral vector creation
    group.bench_function("behavioral_vector_creation", |b| {
        b.iter(|| {
            let vectors: Vec<_> = (0..100)
                .map(|i| TestDataFactory::create_behavioral_vector(76561198034202275, i))
                .collect();
            black_box(vectors.len())
        })
    });

    // Benchmark parquet export
    group.bench_function("parquet_export", |b| {
        b.iter(|| {
            let vectors: Vec<_> = (0..50)
                .map(|i| TestDataFactory::create_behavioral_vector(76561198034202275, i))
                .collect();

            let temp_file = tempfile::NamedTempFile::new().unwrap();
            cs2_ml::data::write_to_parquet(&vectors, temp_file.path()).unwrap();
            black_box(vectors.len())
        })
    });

    group.finish();
}

/// Benchmark concurrent operations
fn benchmark_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let infra = rt.block_on(async {
        TestInfrastructure::new().await.expect("Failed to create test infrastructure")
    });

    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(5);

    // Benchmark concurrent match inserts
    for concurrency in [1, 2, 4].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_match_inserts", concurrency),
            concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    rt.block_on(async {
                        let tasks: Vec<_> = (0..concurrency).map(|i| {
                            let infra = &infra;
                            async move {
                                let test_match = TestDataFactory::create_test_match(&format!("concurrent_{}_{}", concurrency, i));
                                infra.db_manager().postgres.insert_match(&test_match).await.unwrap()
                            }
                        }).collect();

                        let results = futures::future::join_all(tasks).await;
                        black_box(results.len())
                    })
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_pipeline_processing,
    benchmark_database_operations,
    benchmark_vector_operations,
    benchmark_ml_processing,
    benchmark_concurrent_operations
);
criterion_main!(benches);

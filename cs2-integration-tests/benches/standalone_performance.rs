use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use cs2_integration_tests::test_infrastructure::TestDataFactory;

/// Benchmark ML data processing without database dependencies
fn benchmark_standalone_ml_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("standalone_ml_processing");
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

/// Benchmark different vector sizes for performance scaling
fn benchmark_vector_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_scaling");
    group.measurement_time(Duration::from_secs(3));

    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("vector_creation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let vectors: Vec<_> = (0..size)
                        .map(|i| TestDataFactory::create_behavioral_vector(76561198034202275, i))
                        .collect();
                    black_box(vectors.len())
                })
            },
        );
    }

    group.finish();
}

/// Benchmark data serialization performance
fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    group.measurement_time(Duration::from_secs(3));

    let test_vectors: Vec<_> = (0..100)
        .map(|i| TestDataFactory::create_behavioral_vector(76561198034202275, i))
        .collect();

    group.bench_function("json_serialization", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&test_vectors).unwrap();
            black_box(json.len())
        })
    });

    group.bench_function("parquet_serialization", |b| {
        b.iter(|| {
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            cs2_ml::data::write_to_parquet(&test_vectors, temp_file.path()).unwrap();
            black_box(temp_file.path().to_string_lossy().len())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_standalone_ml_processing,
    benchmark_vector_scaling,
    benchmark_serialization
);
criterion_main!(benches);

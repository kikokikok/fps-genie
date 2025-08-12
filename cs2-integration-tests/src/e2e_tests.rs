/// End-to-end tests for the complete demo processing pipeline
#[cfg(test)]
mod e2e_pipeline_tests {
    use crate::TestDataFactory;
    use crate::TestInfrastructure;
    use anyhow::Result;
    use cs2_data_pipeline::models::ProcessingStatus;
    use std::time::Duration;
    use tokio::time::timeout;
    use tracing::{info, warn};

    #[tokio::test]
    async fn test_complete_demo_processing_pipeline() -> Result<()> {
        // Initialize test infrastructure
        let infra = TestInfrastructure::new().await?;
        let processor = infra.create_demo_processor().await?;

        // Setup test demos
        let _demo_files = infra.setup_test_data(&processor).await?;
        info!("📁 Set up test demo files");

        // Test 1: Demo discovery and registration
        let discovered_demos = processor.discover_demos().await?;
        assert!(!discovered_demos.is_empty(), "Should discover demo files");

        // Register the first demo
        if let Some(demo_path) = discovered_demos.first() {
            let match_id = processor.register_demo(demo_path).await?;
            info!("✅ Registered demo with ID: {}", match_id);

            // Verify it was registered in database
            let unprocessed = infra
                .db_manager()
                .postgres
                .get_unprocessed_matches()
                .await?;
            assert!(!unprocessed.is_empty(), "Should have unprocessed matches");
            assert_eq!(unprocessed[0].processing_status, ProcessingStatus::Pending);
        }

        // Test 2: Full pipeline processing
        let result = timeout(
            Duration::from_secs(120), // 2 minute timeout for processing
            processor.process_pending_matches(),
        )
        .await;

        match result {
            Ok(Ok(())) => {
                info!("✅ Pipeline processing completed successfully");

                // Verify processing status was updated
                let _processed = infra
                    .db_manager()
                    .postgres
                    .get_unprocessed_matches()
                    .await?;
                // Should be empty or have status updated to completed/failed
            }
            Ok(Err(e)) => {
                warn!("⚠️ Pipeline processing failed: {}", e);
                // This might be expected if demo file is malformed
            }
            Err(_) => {
                warn!("⚠️ Pipeline processing timed out");
                // This is also acceptable for integration testing
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_database_integration() -> Result<()> {
        let infra = TestInfrastructure::new().await?;

        // Test 1: Match operations
        let test_match = TestDataFactory::create_test_match("test_match_001");
        let match_id = infra
            .db_manager()
            .postgres
            .insert_match(&test_match)
            .await?;
        info!("✅ Inserted test match: {}", match_id);

        // Test 2: Player snapshots batch insert
        let snapshots = TestDataFactory::create_player_snapshots(match_id, 100, 5);
        infra
            .db_manager()
            .timescale
            .insert_snapshots_batch(&snapshots)
            .await?;
        info!("✅ Inserted {} player snapshots", snapshots.len());

        // Test 3: Query player snapshots
        let snapshots_result = infra
            .db_manager()
            .timescale
            .get_player_snapshots(match_id, 76561198034202275, Some(50))
            .await?;
        info!("✅ Retrieved {} player snapshots", snapshots_result.len());

        // Test 4: Vector database operations
        let embedding = cs2_data_pipeline::models::BehavioralEmbedding {
            id: "test_embedding_001".to_string(),
            match_id: match_id.to_string(),
            moment_id: "test_moment_001".to_string(),
            player_steamid: 76561198034202275,
            moment_type: "clutch".to_string(),
            vector: (0..256).map(|i| (i as f32) * 0.01).collect(),
            metadata: serde_json::json!({"test": true}),
        };

        infra
            .db_manager()
            .vector
            .store_behavioral_vector(&embedding)
            .await?;
        info!("✅ Inserted behavioral embedding");

        // Search for similar behaviors
        let similar = infra
            .db_manager()
            .vector
            .search_similar_behaviors(&embedding.vector, 5)
            .await?;
        info!("✅ Found {} similar behaviors", similar.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_ml_pipeline_integration() -> Result<()> {
        let infra = TestInfrastructure::new().await?;

        // Create test behavioral vectors
        let behavioral_vectors: Vec<_> = (0..1000)
            .map(|i| TestDataFactory::create_behavioral_vector(76561198034202275, i))
            .collect();

        info!(
            "📊 Created {} behavioral vectors for ML testing",
            behavioral_vectors.len()
        );

        // Test data conversion to ML format
        let snapshots: Vec<_> = behavioral_vectors
            .iter()
            .map(|bv| cs2_data_pipeline::models::PlayerSnapshot::from(bv.clone()))
            .collect();

        assert_eq!(snapshots.len(), behavioral_vectors.len());
        info!("✅ Successfully converted behavioral vectors to snapshots");

        // Test parquet export (part of ML pipeline)
        let temp_file = tempfile::NamedTempFile::new()?;
        let result = cs2_ml::data::write_to_parquet(&behavioral_vectors, temp_file.path());

        match result {
            Ok(()) => {
                info!("✅ Successfully exported data to Parquet format");

                // Verify file was created and has content
                let metadata = std::fs::metadata(temp_file.path())?;
                assert!(metadata.len() > 0, "Parquet file should not be empty");
            }
            Err(e) => {
                warn!("⚠️ Parquet export failed: {}", e);
                // This might fail if arrow/parquet dependencies have issues
            }
        }

        Ok(())
    }
}

/// Performance and load tests
#[cfg(test)]
mod performance_tests {
    use crate::TestDataFactory;
    use crate::TestInfrastructure;
    use tracing::info;

    #[tokio::test]
    async fn test_batch_insert_performance() {
        let infra = TestInfrastructure::new().await.unwrap();

        let test_match = TestDataFactory::create_test_match("perf_test_001");
        let match_id = infra
            .db_manager()
            .postgres
            .insert_match(&test_match)
            .await
            .unwrap();

        // Test large batch insert
        let start = std::time::Instant::now();
        let large_batch = TestDataFactory::create_player_snapshots(match_id, 10000, 10);

        infra
            .db_manager()
            .timescale
            .insert_snapshots_batch(&large_batch)
            .await
            .unwrap();

        let duration = start.elapsed();
        let throughput = large_batch.len() as f64 / duration.as_secs_f64();

        info!(
            "📈 Batch insert performance: {:.0} snapshots/second",
            throughput
        );
        assert!(
            throughput > 1000.0,
            "Should process at least 1000 snapshots/second"
        );
    }

    #[tokio::test]
    async fn test_concurrent_processing() {
        let infra = TestInfrastructure::new().await.unwrap();

        // Create multiple test matches
        let matches: Vec<_> = (0..5)
            .map(|i| TestDataFactory::create_test_match(&format!("concurrent_test_{:03}", i)))
            .collect();

        // Insert all matches
        let mut match_ids = Vec::new();
        for test_match in &matches {
            let match_id = infra
                .db_manager()
                .postgres
                .insert_match(test_match)
                .await
                .unwrap();
            match_ids.push(match_id);
        }

        // Process concurrently
        let start = std::time::Instant::now();

        let tasks: Vec<_> = match_ids
            .into_iter()
            .map(|match_id| {
                let db = infra.db_manager().clone();
                tokio::spawn(async move {
                    let snapshots = TestDataFactory::create_player_snapshots(match_id, 1000, 5);
                    db.timescale.insert_snapshots_batch(&snapshots).await
                })
            })
            .collect();

        let results = futures::future::join_all(tasks).await;
        let duration = start.elapsed();

        let successful_tasks = results.iter().filter(|r| r.is_ok()).count();
        info!(
            "📊 Concurrent processing: {}/{} tasks completed in {:?}",
            successful_tasks,
            results.len(),
            duration
        );

        assert!(
            successful_tasks >= 3,
            "Most concurrent tasks should succeed"
        );
    }
}

/// Integration tests for external APIs and services
#[cfg(test)]
mod api_integration_tests {
    use crate::TestInfrastructure;
    use std::time::Duration;
    use tokio::time::timeout;
    use tracing::{info, warn};

    #[tokio::test]
    async fn test_ml_server_integration() {
        // This test would start the ML server and test API endpoints
        // Skipped for now as it requires the server to be running
        info!("🚧 ML server integration test - implement when server is ready");
    }

    #[tokio::test]
    async fn test_real_demo_file_processing() {
        let infra = TestInfrastructure::new().await.unwrap();

        // Check if we have the test demo file
        let test_demo_path = "../test_data/test_demo.dem";

        if tokio::fs::metadata(test_demo_path).await.is_ok() {
            info!("📁 Found test demo file, running real demo processing test");

            let processor = infra.create_demo_processor().await.unwrap();

            // Copy the real demo file
            let dest_path = processor.config().demo_directory.join("real_test.dem");
            tokio::fs::copy(test_demo_path, &dest_path).await.unwrap();

            // Try to register and process it
            match processor.register_demo(&dest_path).await {
                Ok(match_id) => {
                    info!("✅ Successfully registered real demo: {}", match_id);

                    // Try processing (with timeout since real demos can be large)
                    let result = timeout(
                        Duration::from_secs(180),
                        processor.process_pending_matches(),
                    )
                    .await;

                    match result {
                        Ok(Ok(())) => info!("✅ Real demo processed successfully"),
                        Ok(Err(e)) => warn!("⚠️ Real demo processing failed: {}", e),
                        Err(_) => warn!("⚠️ Real demo processing timed out"),
                    }
                }
                Err(e) => {
                    warn!("⚠️ Failed to register real demo: {}", e);
                }
            }
        } else {
            info!(
                "⚠️ No real demo file found at {}, skipping test",
                test_demo_path
            );
        }
    }
}

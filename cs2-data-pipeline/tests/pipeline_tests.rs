use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

use cs2_data_pipeline::database::DatabaseManager;
use cs2_data_pipeline::models::ProcessingStatus;
use cs2_data_pipeline::pipeline::{DemoProcessor, PipelineConfig};

#[cfg(test)]
mod pipeline_tests {
    use super::*;

    async fn setup_test_environment() -> Result<(TempDir, PipelineConfig, DatabaseManager)> {
        let temp_dir = TempDir::new()?;
        let demo_dir = temp_dir.path().join("demos");
        let temp_demo_dir = temp_dir.path().join("temp");

        fs::create_dir_all(&demo_dir).await?;
        fs::create_dir_all(&temp_demo_dir).await?;

        let config = PipelineConfig {
            max_concurrent_jobs: 2,
            batch_size: 100,
            demo_directory: demo_dir,
            temp_directory: temp_demo_dir,
            enable_ai_analysis: false, // Disable for testing
            chunk_size_ticks: 64 * 10, // 10 seconds
        };

        // Use test database URLs
        let postgres_url = "postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis_test";
        let timescale_url = "postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis_test";
        let qdrant_url = "http://localhost:6334";

        let db = DatabaseManager::new(postgres_url, timescale_url, qdrant_url).await?;
        db.postgres.initialize_schema().await?;
        db.timescale.initialize_schema().await?;
        db.vector.initialize_collections().await?;

        Ok((temp_dir, config, db))
    }

    async fn create_mock_demo_file(demo_dir: &PathBuf, filename: &str) -> Result<PathBuf> {
        let demo_path = demo_dir.join(filename);
        // Create a mock demo file with some binary content
        let mock_content = vec![0u8; 1024]; // 1KB of zeros as mock demo data
        fs::write(&demo_path, mock_content).await?;
        Ok(demo_path)
    }

    #[tokio::test]
    async fn test_pipeline_config_creation() {
        let config = PipelineConfig::default();

        assert_eq!(config.max_concurrent_jobs, 4);
        assert_eq!(config.batch_size, 1000);
        assert!(config.enable_ai_analysis);
        assert_eq!(config.chunk_size_ticks, 64 * 60);

        println!("Pipeline config creation test passed");
    }

    #[tokio::test]
    async fn test_demo_discovery() -> Result<()> {
        let (temp_dir, config, db) = setup_test_environment().await?;
        let processor = DemoProcessor::new(db, config.clone());

        // Create some mock demo files
        create_mock_demo_file(&config.demo_directory, "match1.dem").await?;
        create_mock_demo_file(&config.demo_directory, "match2.dem").await?;
        create_mock_demo_file(&config.demo_directory, "not_demo.txt").await?; // Should be ignored

        // Test demo discovery
        let discovered_demos = processor.discover_demos().await?;

        assert_eq!(
            discovered_demos.len(),
            2,
            "Should discover exactly 2 .dem files"
        );
        assert!(discovered_demos
            .iter()
            .any(|p| p.file_name().unwrap() == "match1.dem"));
        assert!(discovered_demos
            .iter()
            .any(|p| p.file_name().unwrap() == "match2.dem"));

        println!(
            "Demo discovery test passed - found {} demo files",
            discovered_demos.len()
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_demo_registration() -> Result<()> {
        let (temp_dir, config, db) = setup_test_environment().await?;
        let processor = DemoProcessor::new(db, config.clone());

        // Create a mock demo file
        let demo_path = create_mock_demo_file(
            &config.demo_directory,
            "tournament_teamA_vs_teamB_de_dust2_2024.dem",
        )
        .await?;

        // Test demo registration
        let match_id = processor.register_demo(&demo_path).await?;
        println!("Registered demo with match ID: {}", match_id);

        // Verify the demo was registered in the database
        let pending_matches = processor.db().postgres.get_unprocessed_matches().await?;
        let registered_match = pending_matches
            .iter()
            .find(|m| m.match_id == "tournament_teamA_vs_teamB_de_dust2_2024")
            .expect("Should find the registered match");

        assert_eq!(registered_match.tournament, Some("tournament".to_string()));
        assert_eq!(registered_match.team1, "teamA");
        assert_eq!(registered_match.team2, "teamB"); // Note: should skip "vs"
        assert_eq!(registered_match.map_name, "de_dust2");
        assert_eq!(
            registered_match.processing_status,
            ProcessingStatus::Pending
        );
        assert_eq!(registered_match.demo_file_size, 1024); // Our mock file size

        println!("Demo registration test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_demo_registration_simple_filename() -> Result<()> {
        let (temp_dir, config, db) = setup_test_environment().await?;
        let processor = DemoProcessor::new(db, config.clone());

        // Create a demo with simple filename
        let demo_path = create_mock_demo_file(&config.demo_directory, "simple_demo.dem").await?;

        let match_id = processor.register_demo(&demo_path).await?;

        let pending_matches = processor.db().postgres.get_unprocessed_matches().await?;
        let registered_match = pending_matches
            .iter()
            .find(|m| m.match_id == "simple_demo")
            .expect("Should find the registered match");

        // Should use default values for simple filenames
        assert_eq!(registered_match.tournament, None);
        assert_eq!(registered_match.team1, "Team1");
        assert_eq!(registered_match.team2, "Team2");
        assert_eq!(registered_match.map_name, "unknown");

        println!("Simple filename registration test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_pipeline_workflow_with_mock_data() -> Result<()> {
        let (temp_dir, config, db) = setup_test_environment().await?;
        let processor = DemoProcessor::new(db, config.clone());

        // Create multiple demo files
        let demo_files = vec![
            "esl_navi_vs_astralis_de_inferno_2024.dem",
            "blast_g2_vs_vitality_de_mirage_2024.dem",
            "simple_match.dem",
        ];

        for demo_file in &demo_files {
            create_mock_demo_file(&config.demo_directory, demo_file).await?;
        }

        // Test the full discovery and registration workflow
        let discovered_demos = processor.discover_demos().await?;
        assert_eq!(discovered_demos.len(), 3);

        // Register all discovered demos
        for demo_path in discovered_demos {
            let match_id = processor.register_demo(&demo_path).await?;
            println!(
                "Registered demo: {:?} with ID: {}",
                demo_path.file_name(),
                match_id
            );
        }

        // Verify all demos were registered
        let pending_matches = processor.db().postgres.get_unprocessed_matches().await?;
        assert!(
            pending_matches.len() >= 3,
            "Should have at least 3 pending matches"
        );

        // Check specific matches were registered correctly
        let navi_match = pending_matches
            .iter()
            .find(|m| m.match_id == "esl_navi_vs_astralis_de_inferno_2024")
            .expect("Should find NAVI vs Astralis match");
        assert_eq!(navi_match.tournament, Some("esl".to_string()));
        assert_eq!(navi_match.team1, "navi");
        assert_eq!(navi_match.team2, "astralis");
        assert_eq!(navi_match.map_name, "de_inferno");

        let g2_match = pending_matches
            .iter()
            .find(|m| m.match_id == "blast_g2_vs_vitality_de_mirage_2024")
            .expect("Should find G2 vs Vitality match");
        assert_eq!(g2_match.tournament, Some("blast".to_string()));
        assert_eq!(g2_match.team1, "g2");
        assert_eq!(g2_match.team2, "vitality");
        assert_eq!(g2_match.map_name, "de_mirage");

        println!(
            "Pipeline workflow test passed - processed {} matches",
            pending_matches.len()
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_match_status_updates() -> Result<()> {
        let (temp_dir, config, db) = setup_test_environment().await?;
        let processor = DemoProcessor::new(db, config.clone());

        // Create and register a demo
        let demo_path = create_mock_demo_file(&config.demo_directory, "status_test.dem").await?;
        let match_id = processor.register_demo(&demo_path).await?;

        // Test status progression: Pending -> Processing -> Completed
        processor
            .db()
            .postgres
            .update_match_status("status_test", ProcessingStatus::Processing)
            .await?;

        // Verify it's no longer in unprocessed matches
        let pending_matches = processor.db().postgres.get_unprocessed_matches().await?;
        let still_pending = pending_matches.iter().any(|m| m.match_id == "status_test");
        assert!(
            !still_pending,
            "Match should not be in pending list when processing"
        );

        // Update to completed
        processor
            .db()
            .postgres
            .update_match_status("status_test", ProcessingStatus::Completed)
            .await?;

        // Test failed status as well
        processor
            .db()
            .postgres
            .update_match_status("status_test", ProcessingStatus::Failed)
            .await?;

        println!("Match status updates test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_duplicate_demo_registration() -> Result<()> {
        let (temp_dir, config, db) = setup_test_environment().await?;
        let processor = DemoProcessor::new(db, config.clone());

        let demo_path = create_mock_demo_file(&config.demo_directory, "duplicate_test.dem").await?;

        // Register the same demo twice
        let first_id = processor.register_demo(&demo_path).await?;
        let second_id = processor.register_demo(&demo_path).await?;

        // Should return the same ID (due to ON CONFLICT DO UPDATE)
        assert_eq!(first_id, second_id);

        // Should only have one match in the database
        let pending_matches = processor.db().postgres.get_unprocessed_matches().await?;
        let duplicate_matches: Vec<_> = pending_matches
            .iter()
            .filter(|m| m.match_id == "duplicate_test")
            .collect();
        assert_eq!(
            duplicate_matches.len(),
            1,
            "Should only have one match record"
        );

        println!("Duplicate demo registration test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_invalid_demo_path() -> Result<()> {
        let (temp_dir, config, db) = setup_test_environment().await?;
        let processor = DemoProcessor::new(db, config.clone());

        let non_existent_path = config.demo_directory.join("does_not_exist.dem");

        // This should fail gracefully
        let result = processor.register_demo(&non_existent_path).await;
        assert!(result.is_err(), "Should fail for non-existent file");

        println!("Error handling test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_pipeline_performance_simulation() -> Result<()> {
        let (temp_dir, config, db) = setup_test_environment().await?;
        let processor = DemoProcessor::new(db, config.clone());

        // Create multiple demo files to test concurrent processing capability
        let demo_count = 10;
        for i in 0..demo_count {
            let filename = format!("perf_test_{:03}.dem", i);
            create_mock_demo_file(&config.demo_directory, &filename).await?;
        }

        let start = std::time::Instant::now();

        // Discover and register all demos
        let discovered_demos = processor.discover_demos().await?;
        assert_eq!(discovered_demos.len(), demo_count);

        for demo_path in discovered_demos {
            processor.register_demo(&demo_path).await?;
        }

        let elapsed = start.elapsed();
        println!("Processed {} demos in {:?}", demo_count, elapsed);

        // Verify all were registered
        let pending_matches = processor.db().postgres.get_unprocessed_matches().await?;
        let perf_test_matches: Vec<_> = pending_matches
            .iter()
            .filter(|m| m.match_id.starts_with("perf_test_"))
            .collect();
        assert_eq!(perf_test_matches.len(), demo_count);

        println!("Pipeline performance simulation test passed");
        Ok(())
    }
}

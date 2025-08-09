use anyhow::Result;
use chrono::Utc;
use cs2_data_pipeline::database::DatabaseManager;
use cs2_data_pipeline::models::{BehavioralEmbedding, Match, PlayerSnapshot, ProcessingStatus};
use uuid::Uuid;

/// Integration tests for database managers
/// These tests require running database instances
#[cfg(test)]
mod database_integration_tests {
    use super::*;

    async fn setup_test_databases() -> Result<DatabaseManager> {
        let postgres_url = "postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis_test";
        let timescale_url = "postgresql://cs2_user:cs2_password@localhost:5432/cs2_analysis_test";
        let qdrant_url = "http://localhost:6334";

        let db = DatabaseManager::new(postgres_url, timescale_url, qdrant_url).await?;

        // Initialize schemas
        db.postgres.initialize_schema().await?;
        db.timescale.initialize_schema().await?;
        db.vector.initialize_collections().await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_postgres_match_operations() -> Result<()> {
        let db = setup_test_databases().await?;

        // Create a test match
        let test_match = Match {
            id: Uuid::new_v4(),
            match_id: "test_match_001".to_string(),
            tournament: Some("Test Tournament".to_string()),
            map_name: "de_dust2".to_string(),
            team1: "Team Alpha".to_string(),
            team2: "Team Beta".to_string(),
            score_team1: 16,
            score_team2: 14,
            demo_file_path: "/path/to/test.dem".to_string(),
            demo_file_size: 1024 * 1024, // 1MB
            tick_rate: 64,
            duration_seconds: 1800, // 30 minutes
            created_at: Utc::now(),
            processed_at: None,
            processing_status: ProcessingStatus::Pending,
        };

        // Test inserting a match
        let match_id = db.postgres.insert_match(&test_match).await?;
        println!("Inserted match with ID: {}", match_id);

        // Test retrieving unprocessed matches
        let pending_matches = db.postgres.get_unprocessed_matches().await?;
        assert!(
            !pending_matches.is_empty(),
            "Should have at least one pending match"
        );

        // Test updating match status
        db.postgres
            .update_match_status(&test_match.match_id, ProcessingStatus::Processing)
            .await?;
        db.postgres
            .update_match_status(&test_match.match_id, ProcessingStatus::Completed)
            .await?;

        println!("PostgreSQL match operations test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_timescale_player_snapshots() -> Result<()> {
        let db = setup_test_databases().await?;

        // Create test player snapshots
        let snapshots = vec![
            PlayerSnapshot {
                timestamp: Utc::now(),
                match_id: Uuid::new_v4(),
                tick: 100,
                steamid: 76561198000000001,
                round_number: 1,
                health: 100.0,
                armor: 50.0,
                pos_x: 100.0,
                pos_y: 200.0,
                pos_z: 30.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                yaw: 90.0,
                pitch: 0.0,
                weapon_id: 7, // AK-47
                ammo_clip: 30,
                ammo_reserve: 120,
                is_alive: true,
                is_airborne: false,
                is_scoped: false,
                is_walking: false,
                flash_duration: 0.0,
                money: 2700,
                equipment_value: 2700,
            },
            PlayerSnapshot {
                timestamp: Utc::now(),
                match_id: Uuid::new_v4(),
                tick: 101,
                steamid: 76561198000000001,
                round_number: 1,
                health: 95.0,
                armor: 45.0,
                pos_x: 105.0,
                pos_y: 205.0,
                pos_z: 30.0,
                vel_x: 250.0,
                vel_y: 0.0,
                vel_z: 0.0,
                yaw: 85.0,
                pitch: -5.0,
                weapon_id: 7,
                ammo_clip: 29,
                ammo_reserve: 120,
                is_alive: true,
                is_airborne: false,
                is_scoped: false,
                is_walking: true,
                flash_duration: 0.0,
                money: 2700,
                equipment_value: 2700,
            },
        ];

        // Test batch insertion
        db.timescale.insert_snapshots_batch(&snapshots).await?;
        println!("Successfully inserted {} player snapshots", snapshots.len());

        // Test retrieval
        let retrieved_snapshots = db
            .timescale
            .get_player_snapshots(snapshots[0].match_id, snapshots[0].steamid, Some(10))
            .await?;

        assert!(
            !retrieved_snapshots.is_empty(),
            "Should retrieve at least one snapshot"
        );
        println!("Retrieved {} snapshots", retrieved_snapshots.len());

        println!("TimescaleDB player snapshots test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_vector_behavioral_embeddings() -> Result<()> {
        let db = setup_test_databases().await?;

        // Create test behavioral embedding
        let embedding = BehavioralEmbedding {
            id: "test_behavior_001".to_string(),
            match_id: "test_match_001".to_string(),
            moment_id: "clutch_moment_001".to_string(),
            player_steamid: 76561198000000001,
            moment_type: "clutch".to_string(),
            vector: (0..512).map(|i| (i as f32) / 512.0).collect(), // Normalized test vector
            metadata: serde_json::json!({
                "round": 15,
                "enemies_remaining": 3,
                "time_left": 25.5,
                "bomb_planted": true
            }),
        };

        // Test storing behavioral vector
        db.vector.store_behavioral_vector(&embedding).await?;
        println!("Successfully stored behavioral vector");

        // Test similarity search
        let query_vector: Vec<f32> = (0..512).map(|i| (i as f32) / 512.0).collect();
        let similar_behaviors = db.vector.search_similar_behaviors(&query_vector, 5).await?;

        assert!(
            !similar_behaviors.is_empty(),
            "Should find similar behaviors"
        );
        println!("Found {} similar behaviors", similar_behaviors.len());

        println!("Qdrant vector operations test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_full_database_workflow() -> Result<()> {
        let db = setup_test_databases().await?;

        // 1. Create and insert a match
        let test_match = Match {
            id: Uuid::new_v4(),
            match_id: "workflow_test_001".to_string(),
            tournament: Some("Integration Test Tournament".to_string()),
            map_name: "de_mirage".to_string(),
            team1: "Team Integration".to_string(),
            team2: "Team Test".to_string(),
            score_team1: 16,
            score_team2: 12,
            demo_file_path: "/path/to/workflow_test.dem".to_string(),
            demo_file_size: 2 * 1024 * 1024, // 2MB
            tick_rate: 128,
            duration_seconds: 2100, // 35 minutes
            created_at: Utc::now(),
            processed_at: None,
            processing_status: ProcessingStatus::Pending,
        };

        let match_uuid = db.postgres.insert_match(&test_match).await?;
        println!("Created match: {}", match_uuid);

        // 2. Update status to processing
        db.postgres
            .update_match_status(&test_match.match_id, ProcessingStatus::Processing)
            .await?;

        // 3. Insert player snapshots for this match
        let snapshots: Vec<PlayerSnapshot> = (0..100)
            .map(|i| {
                PlayerSnapshot {
                    timestamp: Utc::now(),
                    match_id: match_uuid,
                    tick: 1000 + i,
                    steamid: 76561198000000001 + (i as i64 % 10), // Simulate 10 different players
                    round_number: ((i / 10) + 1) as i32,
                    health: 100.0 - (i as f32 * 0.5),
                    armor: 100.0 - (i as f32 * 0.3),
                    pos_x: 100.0 + (i as f32 * 2.0),
                    pos_y: 200.0 + (i as f32 * 1.5),
                    pos_z: 30.0,
                    vel_x: if i % 3 == 0 { 250.0 } else { 0.0 },
                    vel_y: 0.0,
                    vel_z: 0.0,
                    yaw: (i as f32 * 3.6) % 360.0,
                    pitch: ((i as f32 * 1.8) % 180.0) - 90.0,
                    weapon_id: 7 + (i % 5) as u16, // Different weapons
                    ammo_clip: 30 - (i % 31) as i32,
                    ammo_reserve: 120,
                    is_alive: i % 20 != 19, // Some players dead
                    is_airborne: i % 15 == 0,
                    is_scoped: i % 25 == 0,
                    is_walking: i % 3 == 0,
                    flash_duration: if i % 30 == 0 { 2.5 } else { 0.0 },
                    money: 2700 - (i as i32 * 10),
                    equipment_value: 2700 - (i as i32 * 5),
                }
            })
            .collect();

        db.timescale.insert_snapshots_batch(&snapshots).await?;
        println!("Inserted {} player snapshots", snapshots.len());

        // 4. Create and store behavioral embeddings
        for i in 0..5 {
            let embedding = BehavioralEmbedding {
                id: format!("workflow_behavior_{:03}", i),
                match_id: test_match.match_id.clone(),
                moment_id: format!("moment_{:03}", i),
                player_steamid: 76561198000000001 + i,
                moment_type: if i % 2 == 0 { "clutch" } else { "entry_frag" }.to_string(),
                vector: (0..512).map(|j| ((i + j) as f32) / 512.0).collect(),
                metadata: serde_json::json!({
                    "round": i + 10,
                    "importance": (i as f32 + 1.0) / 5.0
                }),
            };
            db.vector.store_behavioral_vector(&embedding).await?;
        }
        println!("Stored 5 behavioral embeddings");

        // 5. Update match status to completed
        db.postgres
            .update_match_status(&test_match.match_id, ProcessingStatus::Completed)
            .await?;

        // 6. Verify the workflow by querying data
        let completed_matches = db.postgres.get_unprocessed_matches().await?;
        let workflow_match_completed = !completed_matches
            .iter().any(|m| m.match_id == test_match.match_id); // Should not be in unprocessed list
        assert!(
            workflow_match_completed,
            "Match should be marked as completed"
        );

        let retrieved_snapshots = db
            .timescale
            .get_player_snapshots(match_uuid, 76561198000000001, Some(50))
            .await?;
        assert!(
            !retrieved_snapshots.is_empty(),
            "Should retrieve player snapshots"
        );

        let similar_behaviors = db
            .vector
            .search_similar_behaviors(
                &(0..512).map(|i| (i as f32) / 512.0).collect::<Vec<f32>>(),
                3,
            )
            .await?;
        assert!(
            !similar_behaviors.is_empty(),
            "Should find similar behaviors"
        );

        println!("Full database workflow test completed successfully");
        println!("Match processed: {}", test_match.match_id);
        println!("Snapshots stored: {}", snapshots.len());
        println!("Embeddings stored: 5");
        println!("Similar behaviors found: {}", similar_behaviors.len());

        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_player_snapshot_creation() {
        let snapshot = PlayerSnapshot {
            timestamp: Utc::now(),
            match_id: Uuid::new_v4(),
            tick: 12800, // 200 seconds at 64 tick rate
            steamid: 76561198000000001,
            round_number: 15,
            health: 87.0,
            armor: 45.0,
            pos_x: 1024.5,
            pos_y: -512.25,
            pos_z: 64.0,
            vel_x: 250.0,
            vel_y: 0.0,
            vel_z: 0.0,
            yaw: 145.5,
            pitch: -12.3,
            weapon_id: 7, // AK-47
            ammo_clip: 25,
            ammo_reserve: 90,
            is_alive: true,
            is_airborne: false,
            is_scoped: false,
            is_walking: true,
            flash_duration: 1.2,
            money: 2350,
            equipment_value: 2700,
        };

        assert_eq!(snapshot.tick, 12800);
        assert_eq!(snapshot.steamid, 76561198000000001);
        assert_eq!(snapshot.round_number, 15);
        assert!(snapshot.is_alive);
        assert!(snapshot.is_walking);
        assert!(!snapshot.is_airborne);
        assert_eq!(snapshot.weapon_id, 7);
        println!("PlayerSnapshot unit test passed");
    }

    #[test]
    fn test_behavioral_embedding_creation() {
        let embedding = BehavioralEmbedding {
            id: "unit_test_embedding".to_string(),
            match_id: "unit_test_match".to_string(),
            moment_id: "unit_test_moment".to_string(),
            player_steamid: 76561198000000001,
            moment_type: "ace".to_string(),
            vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            metadata: serde_json::json!({
                "weapon": "ak47",
                "enemies_killed": 5,
                "time_taken": 8.7
            }),
        };

        assert_eq!(embedding.id, "unit_test_embedding");
        assert_eq!(embedding.moment_type, "ace");
        assert_eq!(embedding.vector.len(), 5);
        assert_eq!(embedding.player_steamid, 76561198000000001);
        println!("BehavioralEmbedding unit test passed");
    }
}

use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use cs2_common::BehavioralVector;
use cs2_data_pipeline::{DatabaseManager, DemoProcessor, PipelineConfig};

/// Test infrastructure that manages all required services
/// For now, we'll use a simplified approach without containers for integration tests
pub struct TestInfrastructure {
    db_manager: Arc<DatabaseManager>,
}

impl TestInfrastructure {
    /// Set up complete testing infrastructure with all databases
    /// This assumes databases are already running locally
    pub async fn new() -> Result<Self> {
        // Use default local database URLs for testing
        let postgres_url = "postgresql://postgres:password@localhost:5432/cs2_test";
        let qdrant_url = "http://localhost:6333";

        // Initialize database manager
        let db_manager =
            Arc::new(DatabaseManager::new(postgres_url, postgres_url, qdrant_url).await?);

        Ok(Self { db_manager })
    }

    pub fn db_manager(&self) -> Arc<DatabaseManager> {
        self.db_manager.clone()
    }

    /// Create a test demo processor with temporary directories
    pub async fn create_demo_processor(&self) -> Result<DemoProcessor> {
        let config = PipelineConfig {
            max_concurrent_jobs: 2,
            batch_size: 100,
            demo_directory: std::path::PathBuf::from("./test_data"),
            temp_directory: std::path::PathBuf::from("./temp_test"),
            enable_ai_analysis: false,
            chunk_size_ticks: 64 * 10, // 10 seconds for testing
        };

        Ok(DemoProcessor::new((*self.db_manager).clone(), config))
    }

    /// Copy test demo files to the processor's demo directory
    pub async fn setup_test_data(&self, processor: &DemoProcessor) -> Result<()> {
        // Copy test demo file to processor's demo directory
        let demo_file = "test_demo.dem";
        let src = std::path::Path::new("./test_data").join(demo_file);
        if src.exists() {
            let dest = processor.config().demo_directory.join(demo_file);
            std::fs::create_dir_all(processor.config().demo_directory.parent().unwrap())?;
            std::fs::copy(&src, &dest)?;
        }
        Ok(())
    }

    /// Get connection info for testing
    pub fn get_connection_info(&self) -> TestConnectionInfo {
        TestConnectionInfo {
            postgres_url: "postgresql://postgres:password@localhost:5432/cs2_test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            qdrant_url: "http://localhost:6333".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestConnectionInfo {
    pub postgres_url: String,
    pub redis_url: String,
    pub qdrant_url: String,
}

/// Test data factory for creating realistic test scenarios
pub struct TestDataFactory;

impl TestDataFactory {
    /// Create a realistic behavioral vector for testing
    pub fn create_behavioral_vector(steamid: u64, tick: u32) -> BehavioralVector {
        BehavioralVector {
            tick,
            steamid,
            health: 85.0 + (tick % 15) as f32, // Varying health
            armor: 50.0 + (tick % 50) as f32,  // Varying armor
            pos_x: 100.0 + (tick as f32 * 0.1),
            pos_y: 200.0 + (tick as f32 * 0.05),
            pos_z: 128.0 + ((tick as f32 * 0.01).sin() * 10.0),
            vel_x: (tick as f32 * 0.02).cos() * 250.0,
            vel_y: (tick as f32 * 0.02).sin() * 250.0,
            vel_z: if tick % 64 < 10 { 50.0 } else { 0.0 }, // Occasional jumping
            yaw: (tick as f32 * 0.1) % 360.0,
            pitch: ((tick as f32 * 0.05).sin() * 15.0),
            weapon_id: 7 + (tick % 3) as u16, // Cycling through weapons
            ammo: 30.0 - (tick % 31) as f32,  // Decreasing ammo
            is_airborne: if tick % 64 < 10 { 1.0 } else { 0.0 },
            delta_yaw: ((tick as f32 * 0.01).cos() * 5.0),
            delta_pitch: ((tick as f32 * 0.01).sin() * 2.0),
        }
    }

    /// Create a set of professional player steamids for testing
    pub fn pro_player_steamids() -> Vec<u64> {
        vec![
            76561198034202275, // s1mple
            76561198010511021, // ZywOo
            76561198044045107, // sh1ro
            76561197960265728, // NiKo
            76561197987713664, // device
        ]
    }

    /// Create test match data
    pub fn create_test_match(match_id: &str) -> cs2_data_pipeline::Match {
        cs2_data_pipeline::Match {
            id: Uuid::new_v4(),
            match_id: match_id.to_string(),
            tournament: Some("Test Tournament 2024".to_string()),
            map_name: "de_dust2".to_string(),
            team1: "Team Liquid".to_string(),
            team2: "NAVI".to_string(),
            score_team1: 16,
            score_team2: 14,
            demo_file_path: format!("./test_data/{}.dem", match_id),
            demo_file_size: 52428800, // ~50MB
            tick_rate: 64,
            duration_seconds: 2400, // 40 minutes
            created_at: chrono::Utc::now(),
            processed_at: None,
            processing_status: cs2_data_pipeline::ProcessingStatus::Pending,
        }
    }

    /// Create a batch of realistic player snapshots
    pub fn create_player_snapshots(
        match_id: Uuid,
        num_ticks: u32,
        num_players: usize,
    ) -> Vec<cs2_data_pipeline::PlayerSnapshot> {
        let mut snapshots = Vec::new();
        let steamids = Self::pro_player_steamids();

        for tick in 1..=num_ticks {
            for (i, &steamid) in steamids.iter().take(num_players).enumerate() {
                let base_time = chrono::Utc::now();
                let timestamp = base_time + chrono::Duration::seconds((tick / 64) as i64);

                snapshots.push(cs2_data_pipeline::PlayerSnapshot {
                    timestamp,
                    match_id,
                    tick,
                    steamid: steamid as i64,
                    round_number: (tick / (64 * 120)) as i32 + 1, // ~2 min rounds
                    health: 100.0 - (tick % 100) as f32,
                    armor: 100.0 - (tick % 150) as f32,
                    pos_x: 100.0 + (i as f32 * 50.0) + (tick as f32 * 0.1),
                    pos_y: 200.0 + (i as f32 * 30.0) + (tick as f32 * 0.05),
                    pos_z: 128.0 + ((tick as f32 * 0.01).sin() * 5.0),
                    vel_x: (tick as f32 * 0.02).cos() * 200.0,
                    vel_y: (tick as f32 * 0.02).sin() * 200.0,
                    vel_z: if tick % 128 < 15 { 40.0 } else { 0.0 },
                    yaw: (tick as f32 * 0.1 + i as f32 * 60.0) % 360.0,
                    pitch: ((tick as f32 * 0.05).sin() * 10.0),
                    weapon_id: (7 + i % 5) as u16,
                    ammo_clip: 30 - (tick % 31) as i32,
                    ammo_reserve: 120 - (tick % 121) as i32,
                    is_alive: tick % 200 > 10, // Occasional deaths
                    is_airborne: tick % 64 < 8,
                    is_scoped: tick % 100 < 5,
                    is_walking: tick % 80 < 20,
                    flash_duration: if tick % 300 < 5 { 2.5 } else { 0.0 },
                    money: 16000 - (tick % 16000) as i32,
                    equipment_value: 4000 + (tick % 2000) as i32,
                });
            }
        }

        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_infrastructure_setup() {
        let infra = TestInfrastructure::new().await.unwrap();

        // Test database connections
        let conn_info = infra.get_connection_info();
        assert!(conn_info.postgres_url.contains("postgresql://"));
        assert!(conn_info.qdrant_url.contains("http://"));
    }

    #[tokio::test]
    async fn test_data_factory() {
        let bv = TestDataFactory::create_behavioral_vector(76561198034202275, 1000);
        assert_eq!(bv.steamid, 76561198034202275);
        assert_eq!(bv.tick, 1000);
        assert!(bv.health > 0.0);
        assert!(bv.yaw >= 0.0 && bv.yaw < 360.0);
    }
}

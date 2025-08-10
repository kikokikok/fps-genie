use anyhow::Result;
use chrono::Utc;
use futures::stream::{self, StreamExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info};
use uuid::Uuid;

use crate::database::DatabaseManager;
use crate::models::{Match, PlayerSnapshot, ProcessingStatus};
use cs2_demo_parser::first_pass::parser_settings::ParserInputs;
use cs2_demo_parser::parse_demo::DemoOutput;

/// Configuration for the demo processing pipeline
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub max_concurrent_jobs: usize,
    pub batch_size: usize,
    pub demo_directory: PathBuf,
    pub temp_directory: PathBuf,
    pub enable_ai_analysis: bool,
    pub chunk_size_ticks: u32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 4, // Adjust based on your hardware
            batch_size: 1000,       // Player snapshots per batch
            demo_directory: PathBuf::from("./demos"),
            temp_directory: PathBuf::from("./temp"),
            enable_ai_analysis: true,
            chunk_size_ticks: 64 * 60, // 1 minute at 64 tick rate
        }
    }
}

/// Main demo processing pipeline
pub struct DemoProcessor {
    db: Arc<DatabaseManager>,
    config: PipelineConfig,
    semaphore: Arc<Semaphore>,
}

impl DemoProcessor {
    pub fn new(db: DatabaseManager, config: PipelineConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));

        Self {
            db: Arc::new(db),
            config,
            semaphore,
        }
    }

    /// Get a reference to the database manager
    pub fn db(&self) -> &DatabaseManager {
        &self.db
    }

    /// Get a reference to the pipeline configuration
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Discover and register demo files in the configured directory
    pub async fn discover_demos(&self) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut demo_files = Vec::new();

        for entry in WalkDir::new(&self.config.demo_directory) {
            let entry = entry?;
            if let Some(extension) = entry.path().extension() {
                if extension == "dem" {
                    demo_files.push(entry.path().to_path_buf());
                }
            }
        }

        info!("Discovered {} demo files", demo_files.len());
        Ok(demo_files)
    }

    /// Register a demo file in the database for processing
    pub async fn register_demo(&self, demo_path: &Path) -> Result<Uuid> {
        // Extract basic metadata from filename and file stats
        let filename = demo_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let file_size = std::fs::metadata(demo_path)?.len() as i64;

        // Parse tournament/match info from filename if available
        // Format: tournament_team1_vs_team2_map_date.dem
        let parts: Vec<&str> = filename.split('_').collect();
        let (tournament, team1, team2, map_name) = if parts.len() >= 4 {
            (
                Some(parts[0].to_string()),
                parts[1].to_string(),
                parts[3].to_string(), // Skip "vs"
                parts[4].to_string(),
            )
        } else {
            (
                None,
                "Team1".to_string(),
                "Team2".to_string(),
                "unknown".to_string(),
            )
        };

        let match_data = Match {
            id: Uuid::new_v4(),
            match_id: filename.to_string(),
            tournament,
            map_name,
            team1,
            team2,
            score_team1: 0, // Will be updated after parsing
            score_team2: 0,
            demo_file_path: demo_path.to_string_lossy().to_string(),
            demo_file_size: file_size,
            tick_rate: 64,       // Default, will be updated
            duration_seconds: 0, // Will be calculated
            created_at: Utc::now(),
            processed_at: None,
            processing_status: ProcessingStatus::Pending,
        };

        let match_id = self.db.postgres.insert_match(&match_data).await?;
        info!("Registered demo {} with ID {}", filename, match_id);

        Ok(match_id)
    }

    /// Process all pending matches
    pub async fn process_pending_matches(&self) -> Result<()> {
        let matches = self.db.postgres.get_unprocessed_matches().await?;
        info!("Found {} pending matches to process", matches.len());

        let semaphore = self.semaphore.clone();
        let db = self.db.clone();
        let config = Arc::new(self.config.clone());

        // Process matches concurrently with semaphore limiting
        stream::iter(matches)
            .map(|match_data| {
                let semaphore = semaphore.clone();
                let db = db.clone();
                let config = config.clone();

                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    Self::process_single_match(db, config, match_data).await
                }
            })
            .buffer_unordered(self.config.max_concurrent_jobs)
            .for_each(|result| async {
                if let Err(e) = result {
                    error!("Failed to process match: {}", e);
                }
            })
            .await;

        Ok(())
    }

    /// Process a single match demo file
    async fn process_single_match(
        db: Arc<DatabaseManager>,
        config: Arc<PipelineConfig>,
        mut match_data: Match,
    ) -> Result<()> {
        info!("Processing match: {}", match_data.match_id);

        // Update status to processing
        db.postgres
            .update_match_status(&match_data.match_id, ProcessingStatus::Processing)
            .await?;

        let result = Self::parse_demo_file(&db, &config, &mut match_data).await;

        match result {
            Ok(_) => {
                db.postgres
                    .update_match_status(&match_data.match_id, ProcessingStatus::Completed)
                    .await?;
                info!("Successfully processed match: {}", match_data.match_id);
            }
            Err(e) => {
                error!("Failed to process match {}: {}", match_data.match_id, e);
                db.postgres
                    .update_match_status(&match_data.match_id, ProcessingStatus::Failed)
                    .await?;
            }
        }

        Ok(())
    }

    /// Parse a demo file and extract all data
    async fn parse_demo_file(
        db: &Arc<DatabaseManager>,
        config: &PipelineConfig,
        match_data: &mut Match,
    ) -> Result<()> {
        let demo_path = Path::new(&match_data.demo_file_path);

        // Read demo file
        let demo_bytes = tokio::fs::read(demo_path).await?;
        info!("Read demo file: {} MB", demo_bytes.len() / 1024 / 1024);

        // Create parser with comprehensive settings
        let parser_inputs = ParserInputs {
            real_name_to_og_name: ahash::AHashMap::new(),
            wanted_players: Vec::new(),
            wanted_player_props: vec![
                "X".to_string(),
                "Y".to_string(),
                "Z".to_string(),
                "health".to_string(),
                "armor_value".to_string(),
                "velocity[0]".to_string(),
                "velocity[1]".to_string(),
                "velocity[2]".to_string(),
                "m_angEyeAngles[0]".to_string(),
                "m_angEyeAngles[1]".to_string(),
                "m_hActiveWeapon".to_string(),
                "m_iClip1".to_string(),
                "m_lifeState".to_string(),
                "m_hGroundEntity".to_string(),
                "m_bIsScoped".to_string(),
                "m_bIsWalking".to_string(),
                "m_flFlashDuration".to_string(),
                "m_iAccount".to_string(),
            ],
            wanted_other_props: vec![],
            wanted_prop_states: ahash::AHashMap::new(), // Empty AHashMap for now
            wanted_ticks: vec![],
            wanted_events: vec![
                "round_start".to_string(),
                "round_end".to_string(),
                "player_death".to_string(),
                "weapon_fire".to_string(),
                "player_hurt".to_string(),
                "bomb_planted".to_string(),
                "bomb_defused".to_string(),
                "bomb_exploded".to_string(),
            ],
            parse_ents: true,
            parse_projectiles: false,
            parse_grenades: true,
            only_header: false,
            only_convars: false,
            huffman_lookup_table: &vec![],
            order_by_steamid: true,
            list_props: false,
            fallback_bytes: Some(demo_bytes.clone()),
        };

        // Parse demo using the cs2-demo-parser
        let mut parser = cs2_demo_parser::parse_demo::Parser::new(
            parser_inputs,
            cs2_demo_parser::parse_demo::ParsingMode::Normal,
        );
        let demo_output = parser.parse_demo(&demo_bytes)?;

        // Update match metadata from demo header
        if let Some(_header) = &demo_output.header {
            match_data.tick_rate = 64; // Default tick rate, extract from header if available
                                       // Calculate duration from game events or other available data
            match_data.duration_seconds = (demo_output.game_events.len() as f32 / 64.0) as i32;
        }

        // Extract round events to determine round numbers
        let _round_events: std::collections::HashMap<u32, bool> = demo_output
            .game_events
            .iter()
            .filter(|event| event.name == "round_start" || event.name == "round_end")
            .map(|event| (event.tick as u32, event.name == "round_start"))
            .collect();

        // Process player data from the df (dataframe) structure
        let mut snapshots = Vec::new();
        let current_round = 1;

        // Extract player snapshots from the parsed data (demo_output.df contains player data)
        for (_player_id, player_data) in &demo_output.df {
            let batch = Self::extract_player_snapshots_from_player_data(
                player_data,
                match_data.id,
                current_round,
            )?;
            snapshots.extend(batch);

            // Process in batches to avoid memory issues
            if snapshots.len() >= config.batch_size {
                db.timescale.insert_snapshots_batch(&snapshots).await?;
                snapshots.clear();
            }
        }

        // Insert remaining snapshots
        if !snapshots.is_empty() {
            db.timescale.insert_snapshots_batch(&snapshots).await?;
        }

        // Extract key moments
        Self::extract_key_moments(db, match_data, &demo_output).await?;

        info!("Completed processing match: {}", match_data.match_id);
        Ok(())
    }

    /// Extract player snapshots from a single tick
    fn extract_player_snapshots_from_tick(
        _tick_data: &cs2_demo_parser::second_pass::variants::PropColumn,
        _match_id: Uuid,
        _tick: u32,
        _round_number: i32,
    ) -> Result<Vec<PlayerSnapshot>> {
        // This is a placeholder implementation
        // In a real implementation, this would parse the tick data
        // and extract all player states
        let snapshots = Vec::new();
        Ok(snapshots)
    }

    /// Extract player snapshots from player data
    fn extract_player_snapshots_from_player_data(
        _player_data: &cs2_demo_parser::second_pass::variants::PropColumn,
        _match_id: Uuid,
        _round_number: i32,
    ) -> Result<Vec<PlayerSnapshot>> {
        // This is a placeholder implementation
        // In a real implementation, this would parse the PropColumn data
        // and extract snapshots for each player
        let snapshots = Vec::new();
        Ok(snapshots)
    }

    /// Extract key moments from demo events
    async fn extract_key_moments(
        _db: &Arc<DatabaseManager>,
        _match_data: &Match,
        _demo_output: &DemoOutput,
    ) -> Result<()> {
        // Analyze game events to identify clutches, aces, etc.
        let death_events: Vec<String> = vec![]; // Extract from demo_output.game_events

        // Example: Identify clutch scenarios (1vX situations)
        for _window in death_events.windows(3) {
            // Process death events to identify clutch scenarios
            // Store as KeyMoment in database
        }

        Ok(())
    }

    /// Run the complete pipeline
    pub async fn run(&self) -> Result<()> {
        info!("Starting demo processing pipeline");

        // Initialize database schemas
        self.db.postgres.initialize_schema().await?;
        self.db.timescale.initialize_schema().await?;
        self.db.vector.initialize_collections().await?;

        // Discover and register new demos
        let demo_files = self.discover_demos().await?;

        for demo_path in demo_files {
            // Check if already registered
            if (self.register_demo(&demo_path).await).is_err() {
                // Already exists or error - skip
                continue;
            }
        }

        // Process pending matches
        self.process_pending_matches().await?;

        info!("Pipeline completed successfully");
        Ok(())
    }
}

use anyhow::Result;
use clap::{Parser, Subcommand};
use sqlx::Row; // Add this import to fix the Row trait issues
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber;

use cs2_data_pipeline::{DatabaseManager, DemoProcessor, PipelineConfig};

#[derive(Parser)]
#[command(name = "cs2-pipeline")]
#[command(about = "CS2 Demo Analysis & AI Training System - Data Pipeline")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// PostgreSQL connection string
    #[arg(long, env = "DATABASE_URL")]
    postgres_url: String,

    /// TimescaleDB connection string (can be same as postgres_url if using extensions)
    #[arg(long, env = "TIMESCALE_URL")]
    timescale_url: Option<String>,

    /// Qdrant vector database URL
    #[arg(long, env = "QDRANT_URL", default_value = "http://localhost:6334")]
    qdrant_url: String,

    /// Demo files directory
    #[arg(long, env = "DEMO_DIR", default_value = "./demos")]
    demo_dir: PathBuf,

    /// Maximum concurrent processing jobs
    #[arg(long, default_value = "4")]
    max_jobs: usize,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize database schemas
    Init,

    /// Discover and register demo files
    Discover {
        /// Recursively scan subdirectories
        #[arg(short, long)]
        recursive: bool,
    },

    /// Process pending matches
    Process {
        /// Only process this many matches
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Run the complete pipeline (discover + process)
    Run {
        /// Run continuously, checking for new demos every N seconds
        #[arg(long)]
        watch_interval: Option<u64>,
    },

    /// Show pipeline statistics
    Stats,

    /// Reprocess failed matches
    Retry {
        /// Maximum number of retry attempts
        #[arg(long, default_value = "3")]
        max_retries: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    // Set up database connections
    let timescale_url = cli.timescale_url
        .as_ref()
        .unwrap_or(&cli.postgres_url);

    info!("Connecting to databases...");
    let db_manager = DatabaseManager::new(
        &cli.postgres_url,
        timescale_url,
        &cli.qdrant_url,
    ).await?;

    // Configure pipeline
    let config = PipelineConfig {
        max_concurrent_jobs: cli.max_jobs,
        demo_directory: cli.demo_dir,
        enable_ai_analysis: true,
        ..Default::default()
    };

    let processor = DemoProcessor::new(db_manager, config);

    match cli.command {
        Commands::Init => {
            processor.db().postgres.initialize_schema().await?;
            processor.db().timescale.initialize_schema().await?;
            processor.db().vector.initialize_collections().await?;
            println!("Database schemas initialized successfully");
        }

        Commands::Discover { recursive: _ } => {
            info!("Discovering demo files...");
            let demo_files = processor.discover_demos().await?;

            for demo_path in demo_files {
                match processor.register_demo(&demo_path).await {
                    Ok(match_id) => {
                        info!("Registered: {} -> {}", demo_path.display(), match_id);
                    }
                    Err(e) => {
                        info!("Skipped: {} ({})", demo_path.display(), e);
                    }
                }
            }
        }

        Commands::Process { limit: _ } => {
            info!("Processing pending matches...");
            processor.process_pending_matches().await?;
        }

        Commands::Run { watch_interval } => {
            if let Some(interval) = watch_interval {
                info!("Running pipeline in watch mode (interval: {}s)", interval);
                loop {
                    if let Err(e) = processor.run().await {
                        eprintln!("Pipeline error: {}", e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
                }
            } else {
                info!("Running pipeline once...");
                processor.run().await?;
            }
        }

        Commands::Stats => {
            info!("Gathering pipeline statistics...");

            // Use regular sqlx queries instead of macros to avoid offline mode issues
            let stats_query = "
                SELECT processing_status, COUNT(*) as count
                FROM matches
                GROUP BY processing_status
            ";

            let rows = sqlx::query(stats_query)
                .fetch_all(&processor.db().postgres.pool)
                .await?;

            println!("\n📊 Pipeline Statistics:");
            for row in rows {
                let status: String = row.get("processing_status");
                let count: i64 = row.get("count");
                println!("  {:<12} {}", format!("{}:", status), count);
            }

            let snapshot_count_query = "SELECT COUNT(*) FROM player_snapshots";
            let snapshot_row = sqlx::query(snapshot_count_query)
                .fetch_one(&processor.db().timescale.pool)
                .await?;
            let snapshot_count: i64 = snapshot_row.get(0);

            let total_size_query = "SELECT SUM(demo_file_size) FROM matches WHERE processing_status = 'completed'";
            let size_row = sqlx::query(total_size_query)
                .fetch_optional(&processor.db().postgres.pool)
                .await?;

            if let Some(row) = size_row {
                if let Ok(size) = row.try_get::<Option<i64>, _>(0) {
                    if let Some(size) = size {
                        println!("Processed demo data: {:.2} GB", size as f64 / 1024.0 / 1024.0 / 1024.0);
                    }
                }
            }

            println!("Total snapshots: {}", snapshot_count);
            println!("=====================================\n");
        }

        Commands::Retry { max_retries: _ } => {
            info!("Retrying failed matches...");
            // TODO: Implement retry logic for failed matches
            todo!("Retry functionality not yet implemented");
        }
    }

    Ok(())
}

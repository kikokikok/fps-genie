use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, Level};

mod analysis;
mod models;
mod training;
mod visualization;

use analysis::AdvancedAnalytics;
use training::TrainingPipeline;
use visualization::AnalyticsVisualizer;

#[derive(Parser)]
#[command(name = "cs2-analytics")]
#[command(about = "CS2 Advanced Analytics & Training System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Database connection string
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    /// Qdrant vector database URL
    #[arg(long, env = "QDRANT_URL", default_value = "http://localhost:6334")]
    qdrant_url: String,

    /// Output directory for models and results
    #[arg(long, default_value = "./analytics_output")]
    output_dir: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Train advanced ML models for pro player analysis
    Train {
        /// Training configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Model type to train
        #[arg(long, default_value = "behavior-cloning")]
        model_type: String,

        /// Number of training epochs
        #[arg(long, default_value = "100")]
        epochs: usize,
    },

    /// Run advanced analytics on processed demo data
    Analyze {
        /// Match IDs to analyze (comma-separated)
        #[arg(long)]
        matches: Option<String>,

        /// Player SteamIDs to focus on (comma-separated)
        #[arg(long)]
        players: Option<String>,

        /// Analysis type
        #[arg(long, default_value = "playstyle")]
        analysis_type: String,
    },

    /// Generate visualizations and reports
    Visualize {
        /// Input data path (parquet files)
        #[arg(short, long)]
        input: PathBuf,

        /// Visualization type
        #[arg(long, default_value = "heatmap")]
        viz_type: String,
    },

    /// Run comprehensive pro player comparison
    Compare {
        /// Target player SteamID
        #[arg(long)]
        player: i64,

        /// Professional players to compare against
        #[arg(long)]
        pro_players: Option<String>,

        /// Generate detailed report
        #[arg(long)]
        detailed: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    // Create output directory
    tokio::fs::create_dir_all(&cli.output_dir).await?;

    // Initialize components
    let training_pipeline = TrainingPipeline::new(&cli.database_url, &cli.qdrant_url).await?;
    let analytics = AdvancedAnalytics::new(&cli.database_url, &cli.qdrant_url).await?;
    let visualizer = AnalyticsVisualizer::new(&cli.output_dir);

    match cli.command {
        Commands::Train {
            config,
            model_type,
            epochs,
        } => {
            info!("🤖 Starting advanced ML training pipeline...");
            training_pipeline
                .train_model(&model_type, epochs, config)
                .await?;
        }

        Commands::Analyze {
            matches,
            players,
            analysis_type,
        } => {
            info!("📊 Running advanced analytics...");
            analytics
                .run_analysis(&analysis_type, matches, players)
                .await?;
        }

        Commands::Visualize { input, viz_type } => {
            info!("📈 Generating visualizations...");
            visualizer.generate_visualization(&viz_type, &input).await?;
        }

        Commands::Compare {
            player,
            pro_players,
            detailed,
        } => {
            info!("🆚 Running pro player comparison for SteamID: {}", player);
            analytics
                .compare_with_pros(player, pro_players, detailed)
                .await?;
        }
    }

    Ok(())
}

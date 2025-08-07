use anyhow::Result;
use clap::{Parser, Subcommand};
use cs2_common::BehavioralVector;
use plotters::prelude::*;
use polars::prelude::*;
use std::path::{Path, PathBuf};
use tracing::info;

/// CS2 Demo Analyzer - Visualize and analyze CS2 demo files
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a single demo file
    Analyze {
        /// Path to the demo file
        #[arg(short, long)]
        demo: PathBuf,

        /// Output directory for analysis results
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
    /// Compare multiple players across demo files
    Compare {
        /// Paths to the parquet files containing behavioral vectors
        #[arg(short, long)]
        parquet_files: Vec<PathBuf>,

        /// Output file for comparison chart
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Visualize player movement and aim patterns
    Visualize {
        /// Path to the parquet file containing behavioral vectors
        #[arg(short, long)]
        parquet: PathBuf,

        /// Output file for visualization
        #[arg(short, long)]
        output: PathBuf,

        /// Type of visualization: "movement", "aim", or "both"
        #[arg(short, long, default_value = "both")]
        type_: String,
    },
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { demo, output_dir } => {
            info!("Analyzing demo file: {}", demo.display());

            // Parse the demo and extract behavioral vectors
            let vectors = cs2_ml::data::vectors_from_demo(&demo)?;
            info!("Extracted {} behavioral vectors", vectors.len());

            // Determine output directory
            let output_dir = output_dir.unwrap_or_else(|| {
                let mut dir = PathBuf::from("analysis_results");
                if let Some(stem) = demo.file_stem() {
                    dir.push(stem);
                }
                dir
            });

            // Create the output directory
            std::fs::create_dir_all(&output_dir)?;

            // Save the vectors to a parquet file
            let parquet_path = output_dir.join("vectors.parquet");
            cs2_ml::data::write_parquet(&vectors, &parquet_path)?;
            info!("Wrote behavioral vectors to {}", parquet_path.display());

            // Generate basic statistics
            generate_statistics(&vectors, &output_dir)?;

            // Generate visualizations
            generate_player_movement_chart(&vectors, &output_dir.join("movement.png"))?;
            generate_aim_patterns_chart(&vectors, &output_dir.join("aim.png"))?;

            info!(
                "Analysis complete. Results saved to {}",
                output_dir.display()
            );
        }
        Commands::Compare {
            parquet_files,
            output,
        } => {
            info!("Comparing {} parquet files", parquet_files.len());

            // Load datasets
            let mut datasets = Vec::new();
            for path in &parquet_files {
                let df = ParquetReader::new(std::fs::File::open(path)?).finish()?;
                datasets.push((
                    path.file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    df,
                ));
            }

            // Generate comparison chart
            compare_datasets(&datasets, &output)?;

            info!("Comparison complete. Chart saved to {}", output.display());
        }
        Commands::Visualize {
            parquet,
            output,
            type_,
        } => {
            info!("Visualizing data from {}", parquet.display());

            // Load vectors from parquet
            let df = ParquetReader::new(std::fs::File::open(&parquet)?).finish()?;

            match type_.as_str() {
                "movement" => {
                    generate_movement_visualization(&df, &output)?;
                }
                "aim" => {
                    generate_aim_visualization(&df, &output)?;
                }
                "both" | _ => {
                    let movement_output = output.with_file_name(format!(
                        "{}_movement.png",
                        output.file_stem().unwrap_or_default().to_string_lossy()
                    ));
                    let aim_output = output.with_file_name(format!(
                        "{}_aim.png",
                        output.file_stem().unwrap_or_default().to_string_lossy()
                    ));

                    generate_movement_visualization(&df, &movement_output)?;
                    generate_aim_visualization(&df, &aim_output)?;
                }
            }

            info!(
                "Visualization complete. Output saved to {}",
                output.display()
            );
        }
    }

    Ok(())
}

/// Generate basic statistics about the behavioral vectors
fn generate_statistics(vectors: &[BehavioralVector], output_dir: &Path) -> Result<()> {
    // Extract player statistics
    let player_ids: Vec<_> = vectors
        .iter()
        .map(|v| v.steamid)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    info!("Found {} unique players", player_ids.len());

    // For each player, calculate statistics
    for player_id in player_ids {
        let player_vectors: Vec<_> = vectors.iter().filter(|v| v.steamid == player_id).collect();

        if player_vectors.is_empty() {
            continue;
        }

        // Calculate basic statistics
        let avg_delta_yaw =
            player_vectors.iter().map(|v| v.delta_yaw).sum::<f32>() / player_vectors.len() as f32;
        let avg_delta_pitch =
            player_vectors.iter().map(|v| v.delta_pitch).sum::<f32>() / player_vectors.len() as f32;

        // Write statistics to a file
        let stats_path = output_dir.join(format!("player_{}_stats.txt", player_id));
        let mut stats = std::fs::File::create(stats_path)?;
        use std::io::Write;
        writeln!(stats, "Statistics for player {}", player_id)?;
        writeln!(stats, "Snapshots: {}", player_vectors.len())?;
        writeln!(stats, "Average delta yaw: {:.4}", avg_delta_yaw)?;
        writeln!(stats, "Average delta pitch: {:.4}", avg_delta_pitch)?;
    }

    Ok(())
}

/// Generate a player movement chart
fn generate_player_movement_chart(vectors: &[BehavioralVector], output_path: &Path) -> Result<()> {
    // Create a bitmap to render the chart
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Create chart with appropriate scales
    let mut chart = ChartBuilder::on(&root)
        .caption("Player Movement Patterns", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-3000.0..3000.0, -3000.0..3000.0)?;

    // Draw mesh grid
    chart.configure_mesh().draw()?;

    // Draw player positions
    // In real implementation, filter by player and tick range
    chart.draw_series(vectors.iter().take(1000).map(|v| {
        Circle::new(
            (v.pos_x as f64, v.pos_y as f64),
            2,
            RGBColor(0, 0, 255).filled(),
        )
    }))?;

    root.present()?;
    info!("Player movement chart saved to {}", output_path.display());

    Ok(())
}

/// Generate an aim patterns chart
fn generate_aim_patterns_chart(vectors: &[BehavioralVector], output_path: &Path) -> Result<()> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Aim Adjustment Patterns", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-5.0..5.0, -5.0..5.0)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(vectors.iter().take(1000).map(|v| {
        Circle::new(
            (v.delta_yaw as f64, v.delta_pitch as f64),
            2,
            RGBColor(255, 0, 0).filled(),
        )
    }))?;

    root.present()?;
    info!("Aim patterns chart saved to {}", output_path.display());

    Ok(())
}

/// Compare multiple datasets
fn compare_datasets(datasets: &[(String, DataFrame)], output_path: &Path) -> Result<()> {
    // A simplified example - in a real app, this would be more sophisticated
    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Player Comparison", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0..1.0, 0.0..10.0)?;

    chart.configure_mesh().draw()?;

    for (i, (name, _)) in datasets.iter().enumerate() {
        let i_f64 = i as f64 / datasets.len() as f64;
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(i_f64, 0.0), (i_f64 + 0.1, 5.0)],
                RGBColor(30 * i as u8, 144, 255).filled(),
            )))?
            .label(name.clone());
    }

    chart.configure_series_labels().draw()?;

    root.present()?;
    info!("Comparison chart saved to {}", output_path.display());

    Ok(())
}

/// Generate movement visualization from DataFrame
fn generate_movement_visualization(_df: &DataFrame, output_path: &Path) -> Result<()> {
    // This is a placeholder that would be expanded in a real implementation
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Player Movement from DataFrame",
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-3000.0..3000.0, -3000.0..3000.0)?;

    chart.configure_mesh().draw()?;

    // In real implementation, we'd extract positions from DataFrame
    // and draw movement paths

    root.present()?;
    info!("Movement visualization saved to {}", output_path.display());

    Ok(())
}

/// Generate aim visualization from DataFrame
fn generate_aim_visualization(_df: &DataFrame, output_path: &Path) -> Result<()> {
    // This is a placeholder that would be expanded in a real implementation
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Aim Patterns from DataFrame",
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-5.0..5.0, -5.0..5.0)?;

    chart.configure_mesh().draw()?;

    // In real implementation, we'd extract delta yaw/pitch from DataFrame
    // and visualize aim patterns

    root.present()?;
    info!("Aim visualization saved to {}", output_path.display());

    Ok(())
}

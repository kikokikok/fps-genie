use anyhow::Result;
use clap::{Parser, Subcommand};
use cs2_common::{
    BehavioralVector, 
    PlayerMechanicsExtractor, TeamDynamicsExtractor, DecisionMetricsExtractor, TemporalContextExtractor,
    ExtractedFeatures
};
use cs2_ml::{PlayerStyleClassifier, TeamDynamicsTransformer, DecisionQualityRNN};
use plotters::prelude::*;
use polars::prelude::*;
use std::collections::HashMap;
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
            info!("Analyzing demo file with comprehensive feature extraction: {}", demo.display());

            // Parse the demo and extract behavioral vectors
            let vectors = cs2_ml::data::vectors_from_demo(&demo)?;
            info!("Extracted {} behavioral vectors", vectors.len());

            // Group vectors by player
            let mut player_vectors: HashMap<u64, Vec<BehavioralVector>> = HashMap::new();
            for vector in vectors {
                player_vectors.entry(vector.steamid).or_default().push(vector);
            }

            info!("Found {} unique players", player_vectors.len());

            // Initialize comprehensive feature extractors
            let mechanics_extractor = PlayerMechanicsExtractor::new();
            let team_extractor = TeamDynamicsExtractor::new();
            let decision_extractor = DecisionMetricsExtractor::new();
            let temporal_extractor = TemporalContextExtractor::new();

            // Extract comprehensive features for each player
            let mut all_extracted_features: HashMap<u64, ExtractedFeatures> = HashMap::new();
            
            for (&player_id, player_vecs) in &player_vectors {
                info!("Extracting comprehensive features for player {}", player_id);
                
                // Extract all feature types
                let player_mechanics = mechanics_extractor.extract_features(player_vecs);
                let team_dynamics = team_extractor.extract_features(&player_vectors);
                let decision_metrics = decision_extractor.extract_features(player_vecs, &player_vectors);
                let temporal_context = temporal_extractor.extract_features(player_vecs, &player_vectors, Some("de_dust2"));
                
                let extracted_features = ExtractedFeatures {
                    player_mechanics,
                    team_dynamics,
                    decision_metrics,
                    temporal_context,
                };
                
                all_extracted_features.insert(player_id, extracted_features);
            }

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

            // Save the basic vectors to a parquet file (for compatibility)
            let all_vectors: Vec<BehavioralVector> = player_vectors.values().flatten().cloned().collect();
            let parquet_path = output_dir.join("vectors.parquet");
            cs2_ml::data::write_parquet(&all_vectors, &parquet_path)?;
            info!("Wrote behavioral vectors to {}", parquet_path.display());

            // Generate comprehensive analysis using ML models
            generate_ml_analysis(&all_extracted_features, &output_dir)?;

            // Generate traditional statistics and visualizations
            generate_statistics(&all_vectors, &output_dir)?;
            generate_player_movement_chart(&all_vectors, &output_dir.join("movement.png"))?;
            generate_aim_patterns_chart(&all_vectors, &output_dir.join("aim.png"))?;

            // Generate comprehensive feature analysis
            generate_comprehensive_feature_analysis(&all_extracted_features, &output_dir)?;

            info!(
                "Comprehensive analysis complete. Results saved to {}",
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

/// Generate ML-based analysis using the comprehensive feature extractors and models
fn generate_ml_analysis(features: &HashMap<u64, ExtractedFeatures>, output_dir: &Path) -> Result<()> {
    info!("Generating ML-based analysis");

    // Initialize ML models (CPU for now, would use GPU in production)
    let device = candle_core::Device::Cpu;
    
    // Create style classifier and analyze each player
    let style_classifier = PlayerStyleClassifier::new(18, 6, 5, device.clone())?;
    
    for (&player_id, player_features) in features {
        info!("Analyzing player style for player {}", player_id);
        
        match style_classifier.classify_player_style(player_features) {
            Ok(style_prediction) => {
                let analysis_path = output_dir.join(format!("player_{}_style_analysis.json", player_id));
                let analysis_json = serde_json::to_string_pretty(&style_prediction)?;
                std::fs::write(analysis_path, analysis_json)?;
                
                info!("Player {} classified as: {} (confidence: {:.2})", 
                     player_id, style_prediction.primary_style, style_prediction.confidence);
            }
            Err(e) => {
                info!("Warning: Could not classify player {} style: {}", player_id, e);
            }
        }
    }

    // Analyze team dynamics
    if features.len() >= 2 {
        info!("Analyzing team dynamics");
        let team_transformer = TeamDynamicsTransformer::new(16, 8, 4, device.clone())?;
        
        match team_transformer.analyze_team_dynamics(features) {
            Ok(team_analysis) => {
                let team_path = output_dir.join("team_dynamics_analysis.json");
                let team_json = serde_json::to_string_pretty(&team_analysis)?;
                std::fs::write(team_path, team_json)?;
                
                info!("Team coordination score: {:.2}", team_analysis.coordination_score);
                info!("Tactical cohesion: {:.2}", team_analysis.tactical_cohesion);
            }
            Err(e) => {
                info!("Warning: Could not analyze team dynamics: {}", e);
            }
        }
    }

    // Analyze decision quality over time
    info!("Analyzing decision quality patterns");
    let decision_rnn = DecisionQualityRNN::new(10, 5, 32, device)?;
    
    for (&player_id, player_features) in features {
        // Create a sequence of decision metrics (simplified - would need temporal segmentation in real implementation)
        let decision_sequence = vec![player_features.decision_metrics.clone()];
        
        match decision_rnn.evaluate_decision_quality(&decision_sequence) {
            Ok(quality_analysis) => {
                let quality_path = output_dir.join(format!("player_{}_decision_quality.json", player_id));
                let quality_json = serde_json::to_string_pretty(&quality_analysis)?;
                std::fs::write(quality_path, quality_json)?;
                
                info!("Player {} decision quality: {:.2}", player_id, quality_analysis.overall_quality);
            }
            Err(e) => {
                info!("Warning: Could not analyze decision quality for player {}: {}", player_id, e);
            }
        }
    }

    Ok(())
}

/// Generate comprehensive feature analysis reports
fn generate_comprehensive_feature_analysis(features: &HashMap<u64, ExtractedFeatures>, output_dir: &Path) -> Result<()> {
    info!("Generating comprehensive feature analysis");

    // Create detailed analysis for each player
    for (&player_id, player_features) in features {
        let mut analysis_report = Vec::new();
        
        analysis_report.push(format!("=== Player {} Comprehensive Analysis ===\n", player_id));
        
        // Player Mechanics Analysis
        analysis_report.push("## Player Mechanics Features".to_string());
        analysis_report.push(format!("Headshot Percentage: {:.1}%", player_features.player_mechanics.headshot_percentage * 100.0));
        analysis_report.push(format!("Flick Accuracy: {:.1}%", player_features.player_mechanics.flick_accuracy * 100.0));
        analysis_report.push(format!("Target Acquisition Time: {:.3}s", player_features.player_mechanics.target_acquisition_time));
        analysis_report.push(format!("Movement Efficiency: {:.1}%", player_features.player_mechanics.movement_efficiency * 100.0));
        analysis_report.push(format!("Recoil Control Consistency: {:.1}%", player_features.player_mechanics.recoil_control_consistency * 100.0));
        analysis_report.push("".to_string());
        
        // Team Dynamics Analysis
        analysis_report.push("## Team Dynamics Features".to_string());
        analysis_report.push(format!("Formation Preference: {:.1}% spread vs stack", player_features.team_dynamics.formation_spread_vs_stack * 100.0));
        analysis_report.push(format!("Map Control: {:.1}%", player_features.team_dynamics.map_control_percentage * 100.0));
        analysis_report.push(format!("Crossfire Effectiveness: {:.1}%", player_features.team_dynamics.crossfire_setup_effectiveness * 100.0));
        analysis_report.push(format!("Trade Efficiency: {:.1}%", player_features.team_dynamics.trade_efficiency * 100.0));
        analysis_report.push("".to_string());
        
        // Decision Metrics Analysis
        analysis_report.push("## Decision Making Features".to_string());
        analysis_report.push(format!("Buy Efficiency: {:.2} value/dollar", player_features.decision_metrics.buy_efficiency_value_per_dollar));
        analysis_report.push(format!("Decision Speed: {:.1}%", player_features.decision_metrics.decision_speed_after_first_contact * 100.0));
        analysis_report.push(format!("Reaction Time (Visual): {:.3}s", player_features.decision_metrics.reaction_time_visual_stimuli));
        analysis_report.push(format!("Reaction Consistency: {:.1}%", player_features.decision_metrics.reaction_consistency * 100.0));
        analysis_report.push("".to_string());
        
        // Temporal Context Analysis
        analysis_report.push("## Temporal & Contextual Features".to_string());
        analysis_report.push(format!("Clutch Performance: {:.1}%", player_features.temporal_context.clutch_performance_metrics * 100.0));
        analysis_report.push(format!("Counter-Strategy Effectiveness: {:.1}%", player_features.temporal_context.counter_strategy_effectiveness * 100.0));
        analysis_report.push(format!("Adaptation to Opponents: {:.1}%", player_features.temporal_context.adaptation_to_opponent_patterns * 100.0));
        analysis_report.push("".to_string());
        
        // Weapon Preferences
        if !player_features.player_mechanics.weapon_preference_patterns.is_empty() {
            analysis_report.push("## Weapon Preferences".to_string());
            for (weapon, preference) in &player_features.player_mechanics.weapon_preference_patterns {
                analysis_report.push(format!("{}: {:.1}%", weapon, preference * 100.0));
            }
            analysis_report.push("".to_string());
        }
        
        // Save individual player report
        let report_path = output_dir.join(format!("player_{}_comprehensive_analysis.txt", player_id));
        std::fs::write(report_path, analysis_report.join("\n"))?;
    }

    // Generate team summary
    if features.len() >= 2 {
        let mut team_summary = Vec::new();
        team_summary.push("=== Team Performance Summary ===\n".to_string());
        
        // Calculate team averages
        let avg_mechanics: f32 = features.values()
            .map(|f| (f.player_mechanics.headshot_percentage + f.player_mechanics.flick_accuracy) / 2.0)
            .sum::<f32>() / features.len() as f32;
            
        let avg_teamwork: f32 = features.values()
            .map(|f| (f.team_dynamics.crossfire_setup_effectiveness + f.team_dynamics.trade_efficiency) / 2.0)
            .sum::<f32>() / features.len() as f32;
            
        let avg_decision_quality: f32 = features.values()
            .map(|f| (f.decision_metrics.decision_speed_after_first_contact + f.decision_metrics.reaction_consistency) / 2.0)
            .sum::<f32>() / features.len() as f32;
        
        team_summary.push(format!("Team Size: {} players", features.len()));
        team_summary.push(format!("Average Mechanical Skill: {:.1}%", avg_mechanics * 100.0));
        team_summary.push(format!("Average Teamwork Quality: {:.1}%", avg_teamwork * 100.0));
        team_summary.push(format!("Average Decision Making: {:.1}%", avg_decision_quality * 100.0));
        
        let team_summary_path = output_dir.join("team_summary.txt");
        std::fs::write(team_summary_path, team_summary.join("\n"))?;
    }

    Ok(())
}

/// Enhanced Demo Analyzer - MLMOVE/CSKNOW Integration
///
/// Integrates pro similarity scoring, micro-bot inference, and comprehensive
/// analytics into the existing demo analysis pipeline.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use cs2_analytics::{EarthMoverDistanceCalculator, ProGapAnalysis, ProReferenceDataset};
use cs2_common::{feature_extraction::*, BehavioralVector};
use cs2_ml::{
    DecisionQualityAnalysis, DecisionQualityRNN, MLMOVETransformer, MovementPrediction,
    PlayerStyleClassifier, PlayerStylePrediction, TeamDynamicsAnalysis, TeamDynamicsTransformer,
};

/// Enhanced analysis result with MLMOVE/CSKNOW integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAnalysisResult {
    /// Traditional comprehensive feature analysis
    pub feature_analysis: ExtractedFeatures,
    /// Pro gap analysis using CSKNOW dataset
    pub pro_gap_analysis: ProGapAnalysis,
    /// ML-based player style prediction
    pub style_prediction: PlayerStylePrediction,
    /// Team dynamics analysis
    pub team_dynamics: TeamDynamicsAnalysis,
    /// Decision quality analysis over time
    pub decision_quality: DecisionQualityAnalysis,
    /// MLMOVE movement predictions for key moments
    pub movement_predictions: Vec<MovementAnalysis>,
    /// Performance metrics and timing
    pub performance_metrics: AnalysisPerformanceMetrics,
}

/// Movement analysis using MLMOVE transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementAnalysis {
    /// Tick number for this analysis
    pub tick: u32,
    /// MLMOVE movement prediction
    pub prediction: MovementPrediction,
    /// Comparison with actual player action
    pub actual_vs_predicted: ActionComparison,
    /// Context information
    pub context: MovementContext,
}

/// Comparison between actual and predicted actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionComparison {
    /// Similarity score (0.0 to 1.0)
    pub similarity_score: f32,
    /// Was the actual action in the top-K predictions?
    pub in_top_k: bool,
    /// Rank of actual action in predictions (1-based)
    pub actual_action_rank: usize,
    /// Description of differences
    pub difference_description: String,
}

/// Context for movement analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementContext {
    /// Map area where movement occurred
    pub map_area: String,
    /// Player health at the time
    pub player_health: f32,
    /// Enemies visible
    pub enemies_visible: u32,
    /// Utility usage nearby
    pub utility_active: Vec<String>,
}

/// Performance metrics for analysis pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPerformanceMetrics {
    /// Total analysis time in milliseconds
    pub total_time_ms: f32,
    /// Feature extraction time
    pub feature_extraction_ms: f32,
    /// Pro gap analysis time
    pub pro_gap_analysis_ms: f32,
    /// ML inference time
    pub ml_inference_ms: f32,
    /// MLMOVE predictions time
    pub mlmove_predictions_ms: f32,
    /// Memory usage in MB
    pub memory_usage_mb: f32,
}

/// Enhanced demo analyzer with MLMOVE/CSKNOW integration
pub struct EnhancedDemoAnalyzer {
    /// Pro reference dataset for gap analysis
    pro_dataset: ProReferenceDataset,
    /// EMD calculator for similarity scoring
    emd_calculator: EarthMoverDistanceCalculator,
    /// Player style classifier
    style_classifier: PlayerStyleClassifier,
    /// Team dynamics transformer
    team_transformer: TeamDynamicsTransformer,
    /// Decision quality RNN
    decision_rnn: DecisionQualityRNN,
    /// MLMOVE transformer for movement prediction
    mlmove_transformer: MLMOVETransformer,
    /// Feature extractors
    mechanics_extractor: PlayerMechanicsExtractor,
    /// Analysis configuration
    config: AnalysisConfig,
}

/// Configuration for enhanced analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Whether to run pro gap analysis
    pub enable_pro_gap_analysis: bool,
    /// Whether to run ML style classification
    pub enable_style_classification: bool,
    /// Whether to run team dynamics analysis
    pub enable_team_analysis: bool,
    /// Whether to run decision quality analysis
    pub enable_decision_analysis: bool,
    /// Whether to run MLMOVE movement predictions
    pub enable_movement_predictions: bool,
    /// Number of key moments to analyze with MLMOVE
    pub movement_analysis_points: usize,
    /// Map name for analysis
    pub map_name: String,
    /// Sampling interval for movement analysis (every N ticks)
    pub movement_sampling_interval: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            enable_pro_gap_analysis: true,
            enable_style_classification: true,
            enable_team_analysis: true,
            enable_decision_analysis: true,
            enable_movement_predictions: true,
            movement_analysis_points: 50,
            map_name: "de_dust2".to_string(),
            movement_sampling_interval: 128, // ~1 second at 128 tick rate
        }
    }
}

impl EnhancedDemoAnalyzer {
    /// Create new enhanced demo analyzer
    pub fn new(config: AnalysisConfig) -> Result<Self> {
        let device = candle_core::Device::Cpu;

        // Load pro reference dataset
        let pro_dataset = ProReferenceDataset::load_csknow_dataset()?;
        let emd_calculator = EarthMoverDistanceCalculator::new();

        // Initialize ML models
        let style_classifier = PlayerStyleClassifier::new(18, 6, 5, device.clone())?;
        let team_transformer = TeamDynamicsTransformer::new(16, 8, 4, device.clone())?;
        let decision_rnn = DecisionQualityRNN::new(10, 5, 32, device.clone())?;
        let mlmove_transformer = MLMOVETransformer::new(device.clone())?;

        // Initialize feature extractors
        let mechanics_extractor = PlayerMechanicsExtractor::new();

        Ok(Self {
            pro_dataset,
            emd_calculator,
            style_classifier,
            team_transformer,
            decision_rnn,
            mlmove_transformer,
            mechanics_extractor,
            config,
        })
    }

    /// Perform comprehensive enhanced analysis on behavioral vectors
    pub fn analyze(&self, vectors: &[BehavioralVector]) -> Result<EnhancedAnalysisResult> {
        let analysis_start = Instant::now();
        let mut performance_metrics = AnalysisPerformanceMetrics {
            total_time_ms: 0.0,
            feature_extraction_ms: 0.0,
            pro_gap_analysis_ms: 0.0,
            ml_inference_ms: 0.0,
            mlmove_predictions_ms: 0.0,
            memory_usage_mb: 0.0,
        };

        // Step 1: Extract comprehensive features
        let feature_start = Instant::now();
        let feature_analysis = self.extract_comprehensive_features(vectors)?;
        performance_metrics.feature_extraction_ms = feature_start.elapsed().as_secs_f32() * 1000.0;

        // Step 2: Pro gap analysis using CSKNOW
        let pro_gap_analysis = if self.config.enable_pro_gap_analysis {
            let pro_start = Instant::now();
            let analysis = self
                .pro_dataset
                .analyze_pro_gap(&feature_analysis, &self.config.map_name)?;
            performance_metrics.pro_gap_analysis_ms = pro_start.elapsed().as_secs_f32() * 1000.0;
            analysis
        } else {
            ProGapAnalysis {
                overall_pro_gap: 0.0,
                map_specific_gaps: HashMap::new(),
                feature_gaps: cs2_analytics::FeatureGaps {
                    aim_gap: 0.0,
                    movement_gap: 0.0,
                    decision_gap: 0.0,
                    positioning_gap: 0.0,
                    utility_gap: 0.0,
                },
                improvement_recommendations: Vec::new(),
                closest_pro_style: "Unknown".to_string(),
                style_match_confidence: 0.0,
            }
        };

        // Step 3: ML-based analysis
        let ml_start = Instant::now();
        let (style_prediction, team_dynamics, decision_quality) =
            self.run_ml_analysis(&feature_analysis, vectors)?;
        performance_metrics.ml_inference_ms = ml_start.elapsed().as_secs_f32() * 1000.0;

        // Step 4: MLMOVE movement predictions
        let movement_predictions = if self.config.enable_movement_predictions {
            let mlmove_start = Instant::now();
            let predictions = self.analyze_movement_patterns(vectors)?;
            performance_metrics.mlmove_predictions_ms =
                mlmove_start.elapsed().as_secs_f32() * 1000.0;
            predictions
        } else {
            Vec::new()
        };

        performance_metrics.total_time_ms = analysis_start.elapsed().as_secs_f32() * 1000.0;

        Ok(EnhancedAnalysisResult {
            feature_analysis,
            pro_gap_analysis,
            style_prediction,
            team_dynamics,
            decision_quality,
            movement_predictions,
            performance_metrics,
        })
    }

    /// Extract comprehensive features from behavioral vectors
    fn extract_comprehensive_features(
        &self,
        vectors: &[BehavioralVector],
    ) -> Result<ExtractedFeatures> {
        // Group vectors by player
        let mut player_vectors: HashMap<u64, Vec<BehavioralVector>> = HashMap::new();
        for vector in vectors {
            player_vectors
                .entry(vector.steamid)
                .or_default()
                .push(vector.clone());
        }

        // For this demo, analyze the first player
        let first_player_vectors = player_vectors
            .values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No player data found"))?;

        // Extract player mechanics features
        let player_mechanics = self
            .mechanics_extractor
            .extract_features(first_player_vectors);

        // Create placeholder team dynamics (would aggregate from all players)
        let team_dynamics = TeamDynamicsFeatures {
            formation_spread_vs_stack: 0.6,
            map_control_percentage: 0.7,
            defensive_setup_variations: 0.5,
            site_approach_patterns: HashMap::new(),
            rotation_timing: 0.8,
            rotation_route_efficiency: 0.75,
            crossfire_setup_effectiveness: 0.65,
            smoke_coverage_effectiveness: 0.7,
            flash_effectiveness_enemies: 0.6,
            flash_effectiveness_teammates: 0.85,
            molotov_area_denial_effectiveness: 0.55,
            grenade_damage_efficiency: 0.6,
            utility_timing_vs_executes: 0.7,
            support_utility_coordination: 0.65,
            execute_timing_consistency: 0.8,
            role_adherence: 0.9,
            trade_efficiency: 0.7,
            mid_round_adaptation_frequency: 0.4,
            default_strategy_identification: HashMap::new(),
            execute_success_rate_by_type: HashMap::new(),
        };

        // Create placeholder decision metrics
        let decision_metrics = DecisionMetricsFeatures {
            buy_efficiency_value_per_dollar: 0.8,
            save_decision_quality: 0.75,
            force_buy_success_rate: 0.6,
            investment_utility_vs_weapons: 0.7,
            economic_impact_on_strategy: 0.65,
            information_based_rotation_timing: 0.7,
            decision_speed_after_first_contact: 0.8,
            re_aggression_timing_patterns: 0.6,
            post_plant_positioning_decisions: 0.75,
            timeout_impact_on_decision_quality: 0.7,
            reaction_time_visual_stimuli: 0.25,
            reaction_time_audio_stimuli: 0.22,
            adjustment_time_after_enemy_spotted: 0.3,
            reaction_consistency: 0.8,
            threat_prioritization_under_pressure: 0.7,
        };

        // Create placeholder temporal context
        let temporal_context = TemporalContextFeatures {
            early_round_tendencies: HashMap::new(),
            mid_round_adaptations: HashMap::new(),
            late_round_decision_patterns: HashMap::new(),
            clutch_performance_metrics: 0.65,
            map_specific_tendencies: HashMap::new(),
            position_preference_by_map: HashMap::new(),
            success_rates_by_area: HashMap::new(),
            route_preference_patterns: HashMap::new(),
            counter_strategy_effectiveness: 0.7,
            adaptation_to_opponent_patterns: 0.6,
            anti_strategy_timing: 0.55,
            information_denial_effectiveness: 0.6,
        };

        Ok(ExtractedFeatures {
            player_mechanics,
            team_dynamics,
            decision_metrics,
            temporal_context,
        })
    }

    /// Run ML-based analysis (style classification, team dynamics, decision quality)
    fn run_ml_analysis(
        &self,
        features: &ExtractedFeatures,
        vectors: &[BehavioralVector],
    ) -> Result<(
        PlayerStylePrediction,
        TeamDynamicsAnalysis,
        DecisionQualityAnalysis,
    )> {
        // Player style classification
        let style_prediction = if self.config.enable_style_classification {
            self.style_classifier.classify_player_style(features)?
        } else {
            PlayerStylePrediction {
                primary_style: "Unknown".to_string(),
                confidence: 0.0,
                style_probabilities: HashMap::new(),
            }
        };

        // Team dynamics analysis (placeholder for single player)
        let team_dynamics = if self.config.enable_team_analysis {
            let mut team_features = HashMap::new();
            team_features.insert(vectors[0].steamid, features.clone());
            self.team_transformer
                .analyze_team_dynamics(&team_features)?
        } else {
            TeamDynamicsAnalysis::default()
        };

        // Decision quality analysis
        let decision_quality = if self.config.enable_decision_analysis {
            // Create decision sequence from vectors (simplified)
            let decision_sequence: Vec<DecisionMetricsFeatures> = vectors
                .chunks(32) // Group into decision windows
                .map(|_chunk| features.decision_metrics.clone())
                .collect();

            self.decision_rnn
                .evaluate_decision_quality(&decision_sequence)?
        } else {
            DecisionQualityAnalysis::default()
        };

        Ok((style_prediction, team_dynamics, decision_quality))
    }

    /// Analyze movement patterns using MLMOVE transformer
    fn analyze_movement_patterns(
        &self,
        vectors: &[BehavioralVector],
    ) -> Result<Vec<MovementAnalysis>> {
        let mut movement_analyses = Vec::new();
        let sequence_length = 32; // MLMOVE sequence length

        // Sample key moments for analysis
        let sample_points = self.select_analysis_points(vectors);

        for &tick_idx in &sample_points {
            if tick_idx < sequence_length {
                continue; // Need enough history
            }

            // Extract sequence leading up to this point
            let sequence_start = tick_idx.saturating_sub(sequence_length);
            let sequence = &vectors[sequence_start..tick_idx];

            // Get MLMOVE prediction
            let prediction = self.mlmove_transformer.predict_movement(sequence)?;

            // Compare with actual action (simplified)
            let actual_vector = &vectors[tick_idx];
            let comparison = self.compare_with_actual(actual_vector, &prediction);

            // Create context
            let context = MovementContext {
                map_area: self.determine_map_area(actual_vector),
                player_health: actual_vector.health,
                enemies_visible: 0,         // Would need game state information
                utility_active: Vec::new(), // Would need utility tracking
            };

            movement_analyses.push(MovementAnalysis {
                tick: actual_vector.tick,
                prediction,
                actual_vs_predicted: comparison,
                context,
            });
        }

        Ok(movement_analyses)
    }

    /// Select interesting points for movement analysis
    fn select_analysis_points(&self, vectors: &[BehavioralVector]) -> Vec<usize> {
        let mut points = Vec::new();
        let total_ticks = vectors.len();
        let interval = self.config.movement_sampling_interval;

        // Regular sampling
        for i in (interval..total_ticks).step_by(interval) {
            points.push(i);
            if points.len() >= self.config.movement_analysis_points {
                break;
            }
        }

        // Add interesting moments (movement changes, health changes, etc.)
        for (i, window) in vectors.windows(2).enumerate() {
            if i >= total_ticks - 1 {
                break;
            }

            let current = &window[0];
            let next = &window[1];

            // Detect significant movement changes
            let vel_change = ((next.vel_x - current.vel_x).powi(2)
                + (next.vel_y - current.vel_y).powi(2))
            .sqrt();

            // Detect significant angle changes
            let angle_change = (next.yaw - current.yaw).abs() + (next.pitch - current.pitch).abs();

            if vel_change > 100.0 || angle_change > 30.0 || current.health != next.health {
                points.push(i + 1);
            }
        }

        // Sort and deduplicate
        points.sort_unstable();
        points.dedup();
        points.truncate(self.config.movement_analysis_points);

        points
    }

    /// Compare MLMOVE prediction with actual player action
    fn compare_with_actual(
        &self,
        actual: &BehavioralVector,
        prediction: &MovementPrediction,
    ) -> ActionComparison {
        // Convert actual movement to discrete action (simplified)
        let actual_action = self.vector_to_discrete_action(actual);

        // Find rank of actual action in predictions
        let mut ranked_actions: Vec<(usize, f32)> = prediction
            .action_probabilities
            .iter()
            .enumerate()
            .map(|(i, &prob)| (i, prob))
            .collect();
        ranked_actions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let actual_action_idx = self.discrete_action_to_index(&actual_action);
        let actual_rank = ranked_actions
            .iter()
            .position(|(idx, _)| *idx == actual_action_idx)
            .map(|pos| pos + 1)
            .unwrap_or(97);

        // Calculate similarity (simplified)
        let predicted_action = &prediction.action;
        let similarity = self.calculate_action_similarity(&actual_action, predicted_action);

        ActionComparison {
            similarity_score: similarity,
            in_top_k: actual_rank <= 5, // Top-5 accuracy
            actual_action_rank: actual_rank,
            difference_description: format!(
                "Predicted: dir={} speed={} jump={}, Actual: dir={} speed={} jump={}",
                predicted_action.direction,
                predicted_action.speed,
                predicted_action.jump,
                actual_action.direction,
                actual_action.speed,
                actual_action.jump
            ),
        }
    }

    /// Convert behavioral vector to discrete action (simplified)
    fn vector_to_discrete_action(&self, vector: &BehavioralVector) -> cs2_ml::DiscreteAction {
        // Simplified conversion - in practice would need more sophisticated logic
        let vel_magnitude = (vector.vel_x.powi(2) + vector.vel_y.powi(2)).sqrt();
        let direction = if vel_magnitude < 50.0 {
            8 // No movement
        } else {
            // Determine direction from velocity vector
            let angle = vector.vel_y.atan2(vector.vel_x).to_degrees();
            ((angle + 360.0) % 360.0 / 45.0) as u8 % 8
        };

        let speed = if vel_magnitude < 50.0 {
            0
        } else if vel_magnitude < 150.0 {
            1
        } else if vel_magnitude < 250.0 {
            3
        } else {
            4
        };

        let jump = if vector.is_airborne > 0.5 { 1 } else { 0 };

        cs2_ml::DiscreteAction {
            direction,
            speed,
            jump,
        }
    }

    /// Convert discrete action to index in 97-way action space
    fn discrete_action_to_index(&self, action: &cs2_ml::DiscreteAction) -> usize {
        if action.direction == 8 {
            return 96; // No-op action
        }

        (action.direction as usize) + (action.speed as usize * 8) + (action.jump as usize * 48)
    }

    /// Calculate similarity between two discrete actions
    fn calculate_action_similarity(
        &self,
        actual: &cs2_ml::DiscreteAction,
        predicted: &cs2_ml::DiscreteAction,
    ) -> f32 {
        let dir_sim = if actual.direction == predicted.direction {
            1.0
        } else {
            0.0
        };
        let speed_sim = 1.0 - (actual.speed as f32 - predicted.speed as f32).abs() / 5.0;
        let jump_sim = if actual.jump == predicted.jump {
            1.0
        } else {
            0.0
        };

        (dir_sim + speed_sim + jump_sim) / 3.0
    }

    /// Determine map area from position (simplified)
    fn determine_map_area(&self, vector: &BehavioralVector) -> String {
        // Simplified map area detection for dust2
        match (vector.pos_x, vector.pos_y) {
            (x, y) if x > 1000.0 && y > 500.0 => "A Site".to_string(),
            (x, y) if x < -500.0 && y < -500.0 => "B Site".to_string(),
            (x, y) if x.abs() < 500.0 => "Mid".to_string(),
            _ => "Unknown Area".to_string(),
        }
    }
}

/// High-level API for running enhanced analysis
pub fn analyze_demo_enhanced(
    vectors: &[BehavioralVector],
    config: Option<AnalysisConfig>,
) -> Result<EnhancedAnalysisResult> {
    let config = config.unwrap_or_default();
    let analyzer = EnhancedDemoAnalyzer::new(config)?;
    analyzer.analyze(vectors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_analyzer_creation() -> Result<()> {
        let config = AnalysisConfig::default();
        let _analyzer = EnhancedDemoAnalyzer::new(config)?;
        Ok(())
    }

    #[test]
    fn test_analysis_config_defaults() {
        let config = AnalysisConfig::default();
        assert!(config.enable_pro_gap_analysis);
        assert!(config.enable_style_classification);
        assert!(config.enable_movement_predictions);
        assert_eq!(config.movement_analysis_points, 50);
    }

    #[test]
    fn test_discrete_action_conversion() -> Result<()> {
        let config = AnalysisConfig::default();
        let analyzer = EnhancedDemoAnalyzer::new(config)?;

        let vector = BehavioralVector {
            tick: 1,
            steamid: 123456789,
            health: 100.0,
            armor: 100.0,
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 64.0,
            vel_x: 250.0,
            vel_y: 0.0,
            vel_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            weapon_id: 7,
            ammo: 30.0,
            is_airborne: 0.0,
            delta_yaw: 0.0,
            delta_pitch: 0.0,
        };

        let action = analyzer.vector_to_discrete_action(&vector);
        assert!(action.direction < 8);
        assert!(action.speed <= 5);
        assert!(action.jump <= 1);

        Ok(())
    }

    #[test]
    fn test_enhanced_analysis() -> Result<()> {
        let vectors = vec![BehavioralVector {
            tick: 1,
            steamid: 123456789,
            health: 100.0,
            armor: 100.0,
            pos_x: 100.0,
            pos_y: 200.0,
            pos_z: 64.0,
            vel_x: 250.0,
            vel_y: 0.0,
            vel_z: 0.0,
            yaw: 90.0,
            pitch: 0.0,
            weapon_id: 7,
            ammo: 30.0,
            is_airborne: 0.0,
            delta_yaw: 0.0,
            delta_pitch: 0.0,
        }];

        let result = analyze_demo_enhanced(&vectors, None)?;

        assert!(result.performance_metrics.total_time_ms > 0.0);
        assert!(!result.pro_gap_analysis.closest_pro_style.is_empty());

        Ok(())
    }
}

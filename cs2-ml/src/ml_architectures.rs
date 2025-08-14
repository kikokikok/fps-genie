use anyhow::Result;
use candle_core::{DType, Device, Tensor};
use candle_nn::{linear, Linear, Module, VarBuilder, VarMap};
use cs2_common::feature_extraction::*;

/// Player Style Classifier - Identifies playing styles and patterns
#[derive(Debug)]
pub struct PlayerStyleClassifier {
    feature_encoder: FeatureEncoder,
    style_classifier: StyleClassificationHead,
    device: Device,
}

/// Team Dynamics Transformer - Analyzes team coordination patterns  
#[derive(Debug)]
pub struct TeamDynamicsTransformer {
    position_encoder: PositionEncoder,
    multi_head_attention: MultiHeadAttention,
    coordination_predictor: CoordinationPredictor,
    device: Device,
}

/// Decision Quality RNN - Evaluates tactical decision quality over time
#[derive(Debug)]
pub struct DecisionQualityRNN {
    decision_encoder: DecisionEncoder,
    lstm_cell: LSTMCell,
    quality_predictor: QualityPredictor,
    device: Device,
}

/// Base ML Components

/// Multi-Layer Perceptron for feature processing
#[derive(Debug)]
pub struct MLP {
    layers: Vec<Linear>,
    dropout_rate: f32,
}

/// Attention mechanism for sequence modeling
#[derive(Debug)]
pub struct AttentionMechanism {
    query_proj: Linear,
    key_proj: Linear,
    value_proj: Linear,
    output_proj: Linear,
    attention_dim: usize,
}

/// RNN Cell for temporal pattern recognition
#[derive(Debug)]
pub struct RNNCell {
    input_to_hidden: Linear,
    hidden_to_hidden: Linear,
    hidden_size: usize,
}

/// LSTM Cell for complex temporal dependencies
#[derive(Debug)]
pub struct LSTMCell {
    input_gate: Linear,
    forget_gate: Linear,
    output_gate: Linear,
    cell_gate: Linear,
    hidden_size: usize,
}

// Component implementations for PlayerStyleClassifier

#[derive(Debug)]
pub struct FeatureEncoder {
    mechanics_encoder: MLP,
    context_encoder: MLP,
    fusion_layer: Linear,
}

#[derive(Debug)]
pub struct StyleClassificationHead {
    style_mlp: MLP,
    output_layer: Linear,
    num_styles: usize,
}

// Component implementations for TeamDynamicsTransformer

#[derive(Debug)]
pub struct PositionEncoder {
    spatial_encoder: MLP,
    temporal_encoder: Linear,
    position_embedding: Linear,
}

#[derive(Debug)]
pub struct MultiHeadAttention {
    heads: Vec<AttentionMechanism>,
    num_heads: usize,
    head_dim: usize,
}

#[derive(Debug)]
pub struct CoordinationPredictor {
    coordination_mlp: MLP,
    team_dynamics_head: Linear,
}

// Component implementations for DecisionQualityRNN

#[derive(Debug)]
pub struct DecisionEncoder {
    decision_mlp: MLP,
    context_encoder: Linear,
}

#[derive(Debug)]
pub struct QualityPredictor {
    quality_mlp: MLP,
    quality_head: Linear,
}

// Implementation of base ML components

impl MLP {
    pub fn new(layer_sizes: &[usize], dropout_rate: f32, vs: VarBuilder) -> Result<Self> {
        let mut layers = Vec::new();

        for i in 0..layer_sizes.len() - 1 {
            let layer = linear(
                layer_sizes[i],
                layer_sizes[i + 1],
                vs.pp(format!("layer_{i}")),
            )?;
            layers.push(layer);
        }

        Ok(Self {
            layers,
            dropout_rate,
        })
    }

    pub fn forward(&self, input: &Tensor, _training: bool) -> Result<Tensor> {
        let mut x = input.clone();

        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(&x)?;

            // Apply activation (ReLU) for all but the last layer
            if i < self.layers.len() - 1 {
                x = x.relu()?;

                // Skip dropout for now as it's not available in basic candle
                // In production, would implement custom dropout or use candle-transformers
            }
        }

        Ok(x)
    }
}

impl AttentionMechanism {
    pub fn new(input_dim: usize, attention_dim: usize, vs: VarBuilder) -> Result<Self> {
        let query_proj = linear(input_dim, attention_dim, vs.pp("query"))?;
        let key_proj = linear(input_dim, attention_dim, vs.pp("key"))?;
        let value_proj = linear(input_dim, attention_dim, vs.pp("value"))?;
        let output_proj = linear(attention_dim, input_dim, vs.pp("output"))?;

        Ok(Self {
            query_proj,
            key_proj,
            value_proj,
            output_proj,
            attention_dim,
        })
    }

    pub fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let queries = self.query_proj.forward(input)?;
        let keys = self.key_proj.forward(input)?;
        let values = self.value_proj.forward(input)?;

        // Compute attention scores
        let scores = queries.matmul(&keys.transpose(1, 2)?)?;
        let scaled_scores = scores.div(&Tensor::new(
            (self.attention_dim as f32).sqrt(),
            input.device(),
        )?)?;

        // Simple max normalization instead of softmax for now
        let max_scores = scaled_scores
            .max(candle_core::D::Minus1)?
            .broadcast_as(scaled_scores.shape())?;
        let attention_weights = (scaled_scores - max_scores)?.exp()?;

        // Apply attention to values
        let attended = attention_weights.matmul(&values)?;
        let output = self.output_proj.forward(&attended)?;

        Ok(output)
    }
}

impl RNNCell {
    pub fn new(input_size: usize, hidden_size: usize, vs: VarBuilder) -> Result<Self> {
        let input_to_hidden = linear(input_size, hidden_size, vs.pp("input_to_hidden"))?;
        let hidden_to_hidden = linear(hidden_size, hidden_size, vs.pp("hidden_to_hidden"))?;

        Ok(Self {
            input_to_hidden,
            hidden_to_hidden,
            hidden_size,
        })
    }

    pub fn forward(&self, input: &Tensor, hidden: &Tensor) -> Result<Tensor> {
        let input_contribution = self.input_to_hidden.forward(input)?;
        let hidden_contribution = self.hidden_to_hidden.forward(hidden)?;
        let new_hidden = (input_contribution + hidden_contribution)?.tanh()?;
        Ok(new_hidden)
    }

    pub fn init_hidden(&self, batch_size: usize, device: &Device) -> Result<Tensor> {
        Ok(Tensor::zeros(
            (batch_size, self.hidden_size),
            DType::F32,
            device,
        )?)
    }
}

impl LSTMCell {
    pub fn new(input_size: usize, hidden_size: usize, vs: VarBuilder) -> Result<Self> {
        let input_gate = linear(input_size + hidden_size, hidden_size, vs.pp("input_gate"))?;
        let forget_gate = linear(input_size + hidden_size, hidden_size, vs.pp("forget_gate"))?;
        let output_gate = linear(input_size + hidden_size, hidden_size, vs.pp("output_gate"))?;
        let cell_gate = linear(input_size + hidden_size, hidden_size, vs.pp("cell_gate"))?;

        Ok(Self {
            input_gate,
            forget_gate,
            output_gate,
            cell_gate,
            hidden_size,
        })
    }

    pub fn forward(
        &self,
        input: &Tensor,
        hidden: &Tensor,
        cell: &Tensor,
    ) -> Result<(Tensor, Tensor)> {
        let combined = Tensor::cat(&[input, hidden], 1)?;

        // Simplified LSTM using tanh instead of sigmoid for basic implementation
        let input_gate = self.input_gate.forward(&combined)?.tanh()?;
        let forget_gate = self.forget_gate.forward(&combined)?.tanh()?;
        let output_gate = self.output_gate.forward(&combined)?.tanh()?;
        let cell_gate = self.cell_gate.forward(&combined)?.tanh()?;

        let new_cell = (forget_gate.mul(cell)? + input_gate.mul(&cell_gate)?)?;
        let new_hidden = output_gate.mul(&new_cell.tanh()?)?;

        Ok((new_hidden, new_cell))
    }

    pub fn init_state(&self, batch_size: usize, device: &Device) -> Result<(Tensor, Tensor)> {
        let hidden = Tensor::zeros((batch_size, self.hidden_size), DType::F32, device)?;
        let cell = Tensor::zeros((batch_size, self.hidden_size), DType::F32, device)?;
        Ok((hidden, cell))
    }
}

// Implementation of PlayerStyleClassifier

impl FeatureEncoder {
    pub fn new(mechanics_dim: usize, context_dim: usize, vs: VarBuilder) -> Result<Self> {
        let mechanics_encoder = MLP::new(&[mechanics_dim, 256, 128], 0.2, vs.pp("mechanics"))?;
        let context_encoder = MLP::new(&[context_dim, 128, 64], 0.2, vs.pp("context"))?;
        let fusion_layer = linear(128 + 64, 256, vs.pp("fusion"))?;

        Ok(Self {
            mechanics_encoder,
            context_encoder,
            fusion_layer,
        })
    }

    pub fn forward(&self, mechanics: &Tensor, context: &Tensor, training: bool) -> Result<Tensor> {
        let mechanics_encoded = self.mechanics_encoder.forward(mechanics, training)?;
        let context_encoded = self.context_encoder.forward(context, training)?;

        let combined = Tensor::cat(&[&mechanics_encoded, &context_encoded], 1)?;
        let fused = self.fusion_layer.forward(&combined)?;

        Ok(fused)
    }
}

impl StyleClassificationHead {
    pub fn new(input_dim: usize, num_styles: usize, vs: VarBuilder) -> Result<Self> {
        let style_mlp = MLP::new(&[input_dim, 128, 64], 0.3, vs.pp("style_mlp"))?;
        let output_layer = linear(64, num_styles, vs.pp("output"))?;

        Ok(Self {
            style_mlp,
            output_layer,
            num_styles,
        })
    }

    pub fn forward(&self, features: &Tensor, training: bool) -> Result<Tensor> {
        let style_features = self.style_mlp.forward(features, training)?;
        let logits = self.output_layer.forward(&style_features)?;
        Ok(logits)
    }
}

impl PlayerStyleClassifier {
    pub fn new(
        mechanics_dim: usize,
        context_dim: usize,
        num_styles: usize,
        device: Device,
    ) -> Result<Self> {
        let varmap = VarMap::new();
        let vs = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let feature_encoder = FeatureEncoder::new(mechanics_dim, context_dim, vs.pp("encoder"))?;
        let style_classifier = StyleClassificationHead::new(256, num_styles, vs.pp("classifier"))?;

        Ok(Self {
            feature_encoder,
            style_classifier,
            device,
        })
    }

    pub fn forward(
        &self,
        mechanics_features: &Tensor,
        context_features: &Tensor,
        training: bool,
    ) -> Result<Tensor> {
        let encoded_features =
            self.feature_encoder
                .forward(mechanics_features, context_features, training)?;
        let style_predictions = self.style_classifier.forward(&encoded_features, training)?;
        Ok(style_predictions)
    }

    /// Extract and classify player style from comprehensive features
    pub fn classify_player_style(
        &self,
        features: &ExtractedFeatures,
    ) -> Result<PlayerStylePrediction> {
        // Convert features to tensors
        let mechanics_tensor = self.mechanics_features_to_tensor(&features.player_mechanics)?;
        let context_tensor = self.context_features_to_tensor(&features.temporal_context)?;

        // Forward pass - simplified without softmax for now
        let predictions = self.forward(&mechanics_tensor, &context_tensor, false)?;

        // Use simple argmax instead of softmax
        let predictions_vec = predictions.to_vec2::<f32>()?;
        let predicted_style = predictions_vec[0]
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        // Create normalized probabilities (simplified)
        let max_val = predictions_vec[0]
            .iter()
            .fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let exp_vals: Vec<f32> = predictions_vec[0]
            .iter()
            .map(|&x| (x - max_val).exp())
            .collect();
        let sum_exp: f32 = exp_vals.iter().sum();
        let probabilities: Vec<f32> = exp_vals.iter().map(|&x| x / sum_exp).collect();

        Ok(PlayerStylePrediction {
            primary_style: self.style_index_to_name(predicted_style),
            confidence: probabilities[predicted_style],
            style_probabilities: self.create_style_distribution(&probabilities),
        })
    }

    fn mechanics_features_to_tensor(&self, mechanics: &PlayerMechanicsFeatures) -> Result<Tensor> {
        let features = vec![
            mechanics.headshot_percentage,
            mechanics.flick_accuracy,
            mechanics.target_acquisition_time,
            mechanics.spray_control_deviation,
            mechanics.crosshair_placement_height,
            mechanics.pre_aim_accuracy,
            mechanics.counter_strafe_effectiveness,
            mechanics.peek_technique_score,
            mechanics.movement_efficiency,
            mechanics.position_transition_smoothness,
            mechanics.crouch_usage_pattern,
            mechanics.jump_usage_pattern,
            mechanics.air_strafe_control,
            mechanics.recoil_control_consistency,
            mechanics.burst_vs_spray_preference,
            mechanics.weapon_switch_speed,
            mechanics.positioning_vs_weapon_range,
            mechanics.first_bullet_accuracy,
        ];

        Ok(Tensor::from_slice(
            &features,
            (1, features.len()),
            &self.device,
        )?)
    }

    fn context_features_to_tensor(&self, context: &TemporalContextFeatures) -> Result<Tensor> {
        let features = vec![
            context.clutch_performance_metrics,
            context.counter_strategy_effectiveness,
            context.adaptation_to_opponent_patterns,
            context.anti_strategy_timing,
            context.information_denial_effectiveness,
            // Add map-specific tendency aggregates
            context.map_specific_tendencies.values().sum::<f32>()
                / context.map_specific_tendencies.len().max(1) as f32,
        ];

        Ok(Tensor::from_slice(
            &features,
            (1, features.len()),
            &self.device,
        )?)
    }

    fn style_index_to_name(&self, index: usize) -> String {
        match index {
            0 => "Aggressive Entry Fragger".to_string(),
            1 => "Tactical Support".to_string(),
            2 => "Passive AWPer".to_string(),
            3 => "Lurker".to_string(),
            4 => "IGL (In-Game Leader)".to_string(),
            _ => "Unknown Style".to_string(),
        }
    }

    fn create_style_distribution(&self, probs: &[f32]) -> std::collections::HashMap<String, f32> {
        let mut distribution = std::collections::HashMap::new();
        for (i, &prob) in probs.iter().enumerate() {
            distribution.insert(self.style_index_to_name(i), prob);
        }
        distribution
    }
}

// Implementation of TeamDynamicsTransformer

impl PositionEncoder {
    pub fn new(spatial_dim: usize, temporal_dim: usize, vs: VarBuilder) -> Result<Self> {
        let spatial_encoder = MLP::new(&[spatial_dim, 128, 64], 0.1, vs.pp("spatial"))?;
        let temporal_encoder = linear(temporal_dim, 32, vs.pp("temporal"))?;
        let position_embedding = linear(64 + 32, 128, vs.pp("position"))?;

        Ok(Self {
            spatial_encoder,
            temporal_encoder,
            position_embedding,
        })
    }

    pub fn forward(&self, spatial: &Tensor, temporal: &Tensor, training: bool) -> Result<Tensor> {
        let spatial_encoded = self.spatial_encoder.forward(spatial, training)?;
        let temporal_encoded = self.temporal_encoder.forward(temporal)?;

        let combined = Tensor::cat(&[&spatial_encoded, &temporal_encoded], 1)?;
        let position_embedded = self.position_embedding.forward(&combined)?;

        Ok(position_embedded)
    }
}

impl MultiHeadAttention {
    pub fn new(input_dim: usize, num_heads: usize, vs: VarBuilder) -> Result<Self> {
        let head_dim = input_dim / num_heads;
        let mut heads = Vec::new();

        for i in 0..num_heads {
            let head = AttentionMechanism::new(input_dim, head_dim, vs.pp(format!("head_{i}")))?;
            heads.push(head);
        }

        Ok(Self {
            heads,
            num_heads,
            head_dim,
        })
    }

    pub fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let mut head_outputs = Vec::new();

        for head in &self.heads {
            let head_output = head.forward(input)?;
            head_outputs.push(head_output);
        }

        let concatenated = Tensor::cat(&head_outputs, 2)?;
        Ok(concatenated)
    }
}

impl CoordinationPredictor {
    pub fn new(input_dim: usize, vs: VarBuilder) -> Result<Self> {
        let coordination_mlp = MLP::new(&[input_dim, 256, 128], 0.2, vs.pp("coordination"))?;
        let team_dynamics_head = linear(128, 64, vs.pp("dynamics"))?;

        Ok(Self {
            coordination_mlp,
            team_dynamics_head,
        })
    }

    pub fn forward(&self, features: &Tensor, training: bool) -> Result<Tensor> {
        let coordination_features = self.coordination_mlp.forward(features, training)?;
        let dynamics = self.team_dynamics_head.forward(&coordination_features)?;
        Ok(dynamics)
    }
}

impl TeamDynamicsTransformer {
    pub fn new(
        spatial_dim: usize,
        temporal_dim: usize,
        num_heads: usize,
        device: Device,
    ) -> Result<Self> {
        let varmap = VarMap::new();
        let vs = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let position_encoder = PositionEncoder::new(spatial_dim, temporal_dim, vs.pp("encoder"))?;
        let multi_head_attention = MultiHeadAttention::new(128, num_heads, vs.pp("attention"))?;
        let coordination_predictor = CoordinationPredictor::new(128, vs.pp("predictor"))?;

        Ok(Self {
            position_encoder,
            multi_head_attention,
            coordination_predictor,
            device,
        })
    }

    pub fn forward(
        &self,
        spatial_features: &Tensor,
        temporal_features: &Tensor,
        training: bool,
    ) -> Result<Tensor> {
        let position_encoded =
            self.position_encoder
                .forward(spatial_features, temporal_features, training)?;
        let attention_output = self.multi_head_attention.forward(&position_encoded)?;
        let coordination_prediction = self
            .coordination_predictor
            .forward(&attention_output, training)?;
        Ok(coordination_prediction)
    }

    /// Analyze team dynamics from multiple players' features
    pub fn analyze_team_dynamics(
        &self,
        team_features: &std::collections::HashMap<u64, ExtractedFeatures>,
    ) -> Result<TeamDynamicsAnalysis> {
        if team_features.is_empty() {
            return Ok(TeamDynamicsAnalysis::default());
        }

        // Aggregate spatial features from all players
        let spatial_tensor = self.aggregate_spatial_features(team_features)?;
        let temporal_tensor = self.aggregate_temporal_features(team_features)?;

        // Forward pass
        let dynamics_prediction = self.forward(&spatial_tensor, &temporal_tensor, false)?;
        let dynamics_vec = dynamics_prediction.to_vec1::<f32>()?;

        Ok(TeamDynamicsAnalysis {
            coordination_score: dynamics_vec.first().copied().unwrap_or(0.0),
            tactical_cohesion: dynamics_vec.get(1).copied().unwrap_or(0.0),
            communication_effectiveness: dynamics_vec.get(2).copied().unwrap_or(0.0),
            role_distribution_balance: dynamics_vec.get(3).copied().unwrap_or(0.0),
        })
    }

    fn aggregate_spatial_features(
        &self,
        team_features: &std::collections::HashMap<u64, ExtractedFeatures>,
    ) -> Result<Tensor> {
        let mut spatial_data = Vec::new();

        for features in team_features.values() {
            spatial_data.extend(vec![
                features.team_dynamics.formation_spread_vs_stack,
                features.team_dynamics.map_control_percentage,
                features.team_dynamics.crossfire_setup_effectiveness,
                features.team_dynamics.rotation_route_efficiency,
            ]);
        }

        Ok(Tensor::from_slice(
            &spatial_data,
            (1, spatial_data.len()),
            &self.device,
        )?)
    }

    fn aggregate_temporal_features(
        &self,
        team_features: &std::collections::HashMap<u64, ExtractedFeatures>,
    ) -> Result<Tensor> {
        let mut temporal_data = Vec::new();

        for features in team_features.values() {
            temporal_data.extend(vec![
                features.team_dynamics.execute_timing_consistency,
                features.team_dynamics.mid_round_adaptation_frequency,
                features.decision_metrics.decision_speed_after_first_contact,
            ]);
        }

        Ok(Tensor::from_slice(
            &temporal_data,
            (1, temporal_data.len()),
            &self.device,
        )?)
    }
}

// Implementation of DecisionQualityRNN

impl DecisionEncoder {
    pub fn new(decision_dim: usize, context_dim: usize, vs: VarBuilder) -> Result<Self> {
        let decision_mlp = MLP::new(&[decision_dim, 128, 64], 0.2, vs.pp("decision"))?;
        let context_encoder = linear(context_dim, 32, vs.pp("context"))?;

        Ok(Self {
            decision_mlp,
            context_encoder,
        })
    }

    pub fn forward(&self, decisions: &Tensor, context: &Tensor, training: bool) -> Result<Tensor> {
        let decision_encoded = self.decision_mlp.forward(decisions, training)?;
        let context_encoded = self.context_encoder.forward(context)?;

        let combined = Tensor::cat(&[&decision_encoded, &context_encoded], 1)?;
        Ok(combined)
    }
}

impl QualityPredictor {
    pub fn new(input_dim: usize, vs: VarBuilder) -> Result<Self> {
        let quality_mlp = MLP::new(&[input_dim, 64, 32], 0.2, vs.pp("quality"))?;
        let quality_head = linear(32, 1, vs.pp("head"))?;

        Ok(Self {
            quality_mlp,
            quality_head,
        })
    }

    pub fn forward(&self, features: &Tensor, training: bool) -> Result<Tensor> {
        let quality_features = self.quality_mlp.forward(features, training)?;
        let quality_score = self.quality_head.forward(&quality_features)?;
        Ok(quality_score)
    }
}

impl DecisionQualityRNN {
    pub fn new(
        decision_dim: usize,
        context_dim: usize,
        hidden_size: usize,
        device: Device,
    ) -> Result<Self> {
        let varmap = VarMap::new();
        let vs = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let decision_encoder = DecisionEncoder::new(decision_dim, context_dim, vs.pp("encoder"))?;
        let lstm_cell = LSTMCell::new(96, hidden_size, vs.pp("lstm"))?; // 64 + 32 = 96
        let quality_predictor = QualityPredictor::new(hidden_size, vs.pp("predictor"))?;

        Ok(Self {
            decision_encoder,
            lstm_cell,
            quality_predictor,
            device,
        })
    }

    pub fn forward(
        &self,
        decision_sequence: &[Tensor],
        context_sequence: &[Tensor],
        training: bool,
    ) -> Result<Vec<f32>> {
        let batch_size = 1;
        let (mut hidden, mut cell) = self.lstm_cell.init_state(batch_size, &self.device)?;
        let mut quality_scores = Vec::new();

        for (decision, context) in decision_sequence.iter().zip(context_sequence.iter()) {
            // Encode decision and context
            let encoded = self.decision_encoder.forward(decision, context, training)?;

            // LSTM forward pass
            let (new_hidden, new_cell) = self.lstm_cell.forward(&encoded, &hidden, &cell)?;
            hidden = new_hidden;
            cell = new_cell;

            // Predict quality
            let quality = self.quality_predictor.forward(&hidden, training)?;
            let quality_value = quality.to_vec1::<f32>()?[0];
            quality_scores.push(quality_value);
        }

        Ok(quality_scores)
    }

    /// Evaluate decision quality over a sequence of game states
    pub fn evaluate_decision_quality(
        &self,
        decision_sequence: &[DecisionMetricsFeatures],
    ) -> Result<DecisionQualityAnalysis> {
        if decision_sequence.is_empty() {
            return Ok(DecisionQualityAnalysis::default());
        }

        // Convert decision features to tensors
        let decision_tensors = self.decisions_to_tensors(decision_sequence)?;
        let context_tensors = self.create_context_tensors(decision_sequence)?;

        // Forward pass
        let quality_scores = self.forward(&decision_tensors, &context_tensors, false)?;

        Ok(DecisionQualityAnalysis {
            overall_quality: quality_scores.iter().sum::<f32>() / quality_scores.len() as f32,
            quality_trend: self.calculate_quality_trend(&quality_scores),
            peak_decision_moments: self.identify_peak_decisions(&quality_scores),
            improvement_areas: self.identify_improvement_areas(decision_sequence),
        })
    }

    fn decisions_to_tensors(&self, decisions: &[DecisionMetricsFeatures]) -> Result<Vec<Tensor>> {
        let mut tensors = Vec::new();

        for decision in decisions {
            let features = vec![
                decision.buy_efficiency_value_per_dollar,
                decision.save_decision_quality,
                decision.force_buy_success_rate,
                decision.investment_utility_vs_weapons,
                decision.economic_impact_on_strategy,
                decision.information_based_rotation_timing,
                decision.decision_speed_after_first_contact,
                decision.re_aggression_timing_patterns,
                decision.post_plant_positioning_decisions,
                decision.timeout_impact_on_decision_quality,
            ];

            let tensor = Tensor::from_slice(&features, (1, features.len()), &self.device)?;
            tensors.push(tensor);
        }

        Ok(tensors)
    }

    fn create_context_tensors(&self, decisions: &[DecisionMetricsFeatures]) -> Result<Vec<Tensor>> {
        let mut tensors = Vec::new();

        for decision in decisions {
            let context = vec![
                decision.reaction_time_visual_stimuli,
                decision.reaction_time_audio_stimuli,
                decision.adjustment_time_after_enemy_spotted,
                decision.reaction_consistency,
                decision.threat_prioritization_under_pressure,
            ];

            let tensor = Tensor::from_slice(&context, (1, context.len()), &self.device)?;
            tensors.push(tensor);
        }

        Ok(tensors)
    }

    fn calculate_quality_trend(&self, scores: &[f32]) -> f32 {
        if scores.len() < 2 {
            return 0.0;
        }

        let first_half = &scores[0..scores.len() / 2];
        let second_half = &scores[scores.len() / 2..];

        let first_avg = first_half.iter().sum::<f32>() / first_half.len() as f32;
        let second_avg = second_half.iter().sum::<f32>() / second_half.len() as f32;

        second_avg - first_avg
    }

    fn identify_peak_decisions(&self, scores: &[f32]) -> Vec<usize> {
        let mean_score = scores.iter().sum::<f32>() / scores.len() as f32;
        let threshold = mean_score + 0.5; // One standard deviation above mean (simplified)

        scores
            .iter()
            .enumerate()
            .filter_map(|(i, &score)| if score > threshold { Some(i) } else { None })
            .collect()
    }

    fn identify_improvement_areas(&self, decisions: &[DecisionMetricsFeatures]) -> Vec<String> {
        let mut areas = Vec::new();

        let avg_reaction_time = decisions
            .iter()
            .map(|d| d.reaction_time_visual_stimuli)
            .sum::<f32>()
            / decisions.len() as f32;

        if avg_reaction_time > 0.5 {
            areas.push("Reaction Time".to_string());
        }

        let avg_decision_speed = decisions
            .iter()
            .map(|d| d.decision_speed_after_first_contact)
            .sum::<f32>()
            / decisions.len() as f32;

        if avg_decision_speed < 0.6 {
            areas.push("Decision Speed".to_string());
        }

        areas
    }
}

// Output structures for ML predictions

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerStylePrediction {
    pub primary_style: String,
    pub confidence: f32,
    pub style_probabilities: std::collections::HashMap<String, f32>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TeamDynamicsAnalysis {
    pub coordination_score: f32,
    pub tactical_cohesion: f32,
    pub communication_effectiveness: f32,
    pub role_distribution_balance: f32,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DecisionQualityAnalysis {
    pub overall_quality: f32,
    pub quality_trend: f32,
    pub peak_decision_moments: Vec<usize>,
    pub improvement_areas: Vec<String>,
}

// Import fix for candle_core::D

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlp_creation() -> Result<()> {
        let varmap = VarMap::new();
        let vs = VarBuilder::from_varmap(&varmap, DType::F32, &Device::Cpu);
        let mlp = MLP::new(&[10, 20, 5], 0.2, vs)?;
        assert_eq!(mlp.layers.len(), 2);
        Ok(())
    }

    #[test]
    fn test_attention_mechanism() -> Result<()> {
        let varmap = VarMap::new();
        let vs = VarBuilder::from_varmap(&varmap, DType::F32, &Device::Cpu);
        let attention = AttentionMechanism::new(64, 32, vs)?;

        let input = Tensor::zeros((1, 10, 64), DType::F32, &Device::Cpu)?;
        let output = attention.forward(&input)?;
        assert_eq!(output.shape().dims(), &[1, 10, 64]);
        Ok(())
    }

    #[test]
    fn test_lstm_cell() -> Result<()> {
        let varmap = VarMap::new();
        let vs = VarBuilder::from_varmap(&varmap, DType::F32, &Device::Cpu);
        let lstm = LSTMCell::new(10, 20, vs)?;

        let input = Tensor::zeros((1, 10), DType::F32, &Device::Cpu)?;
        let (hidden, cell) = lstm.init_state(1, &Device::Cpu)?;
        let (new_hidden, new_cell) = lstm.forward(&input, &hidden, &cell)?;

        assert_eq!(new_hidden.shape().dims(), &[1, 20]);
        assert_eq!(new_cell.shape().dims(), &[1, 20]);
        Ok(())
    }

    #[test]
    fn test_player_style_classifier() -> Result<()> {
        let classifier = PlayerStyleClassifier::new(18, 6, 5, Device::Cpu)?;

        let mechanics = Tensor::zeros((1, 18), DType::F32, &Device::Cpu)?;
        let context = Tensor::zeros((1, 6), DType::F32, &Device::Cpu)?;

        let predictions = classifier.forward(&mechanics, &context, false)?;
        assert_eq!(predictions.shape().dims(), &[1, 5]);
        Ok(())
    }

    #[test]
    fn test_team_dynamics_transformer() -> Result<()> {
        let transformer = TeamDynamicsTransformer::new(16, 8, 4, Device::Cpu)?;

        let spatial = Tensor::zeros((1, 16), DType::F32, &Device::Cpu)?;
        let temporal = Tensor::zeros((1, 8), DType::F32, &Device::Cpu)?;

        let dynamics = transformer.forward(&spatial, &temporal, false)?;
        assert_eq!(dynamics.shape().dims(), &[1, 64]);
        Ok(())
    }

    #[test]
    fn test_decision_quality_rnn() -> Result<()> {
        let rnn = DecisionQualityRNN::new(10, 5, 32, Device::Cpu)?;

        let decision_sequence = vec![
            Tensor::zeros((1, 10), DType::F32, &Device::Cpu)?,
            Tensor::zeros((1, 10), DType::F32, &Device::Cpu)?,
            Tensor::zeros((1, 10), DType::F32, &Device::Cpu)?,
        ];

        let context_sequence = vec![
            Tensor::zeros((1, 5), DType::F32, &Device::Cpu)?,
            Tensor::zeros((1, 5), DType::F32, &Device::Cpu)?,
            Tensor::zeros((1, 5), DType::F32, &Device::Cpu)?,
        ];

        let quality_scores = rnn.forward(&decision_sequence, &context_sequence, false)?;
        assert_eq!(quality_scores.len(), 3);
        Ok(())
    }
}

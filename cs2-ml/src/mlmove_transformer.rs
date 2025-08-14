/// MLMOVE Transformer Architecture - Professional Movement Cloning
/// 
/// Implements the 4-layer, 1-head, 256-d transformer from the MLMOVE research paper
/// for 0.5ms/tick professional player movement prediction and behavior cloning.

use anyhow::Result;
use candle_core::{DType, Device, Tensor};
use candle_nn::{embedding, linear, Linear, Module, VarBuilder, VarMap, Embedding};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use cs2_common::BehavioralVector;

/// MLMOVE Transformer Configuration
/// 
/// Matches the research paper specifications:
/// - 4 transformer layers
/// - 1 attention head  
/// - 256 dimensions
/// - 5M parameters total
/// - 0.5ms inference time per tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLMOVEConfig {
    /// Number of transformer layers (paper: 4)
    pub num_layers: usize,
    /// Number of attention heads (paper: 1)
    pub num_heads: usize,
    /// Model dimension (paper: 256)
    pub model_dim: usize,
    /// Feed-forward dimension (typically 4x model_dim)
    pub ff_dim: usize,
    /// Sequence length for transformer input
    pub sequence_length: usize,
    /// Discrete action space size (paper: 97-way)
    pub action_space_size: usize,
    /// Input feature dimension (position, velocity, etc.)
    pub input_dim: usize,
}

impl Default for MLMOVEConfig {
    fn default() -> Self {
        Self {
            num_layers: 4,
            num_heads: 1,
            model_dim: 256,
            ff_dim: 1024,
            sequence_length: 32,
            action_space_size: 97,
            input_dim: 10, // pos_x, pos_y, pos_z, vel_x, vel_y, vel_z, yaw, pitch, health, armor
        }
    }
}

/// MLMOVE Transformer for professional movement prediction
/// 
/// Implements the architecture from "Learning to Move Like Professional Counter-Strike Players"
/// Optimized for real-time inference with 0.5ms per tick performance target
#[derive(Debug)]
pub struct MLMOVETransformer {
    /// Input feature embedding layer
    input_embedding: Linear,
    /// Positional embeddings for sequence modeling
    position_embedding: Embedding,
    /// Transformer layers
    transformer_layers: Vec<TransformerLayer>,
    /// Output projection to action space
    output_projection: Linear,
    /// Configuration
    config: MLMOVEConfig,
    /// Device for tensor operations
    device: Device,
}

/// Single transformer layer with self-attention and feed-forward
#[derive(Debug)]
struct TransformerLayer {
    /// Self-attention mechanism
    self_attention: SelfAttention,
    /// Feed-forward network
    feed_forward: FeedForward,
    /// Layer normalization before attention
    ln1: LayerNorm,
    /// Layer normalization before feed-forward
    ln2: LayerNorm,
}

/// Self-attention mechanism (single head as per paper)
#[derive(Debug)]
struct SelfAttention {
    /// Query projection
    query: Linear,
    /// Key projection
    key: Linear,
    /// Value projection
    value: Linear,
    /// Output projection
    output: Linear,
    /// Model dimension
    model_dim: usize,
}

/// Feed-forward network
#[derive(Debug)]
struct FeedForward {
    /// First linear layer
    linear1: Linear,
    /// Second linear layer
    linear2: Linear,
}

/// Layer normalization (simplified)
#[derive(Debug)]
struct LayerNorm {
    /// Scale parameter
    weight: Linear,
    /// Bias parameter  
    bias: Linear,
}

/// Discrete action space for CS2 movement
/// 
/// Represents the 97-way discrete action space from the research:
/// - Movement directions (8 directions)
/// - Movement speeds (multiple levels)
/// - Jump actions (jump/no-jump)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscreteAction {
    /// Movement direction (0-7, representing 8 directions)
    pub direction: u8,
    /// Movement speed (0-5, representing speed levels)
    pub speed: u8,
    /// Jump action (0 = no jump, 1 = jump)
    pub jump: u8,
}

/// Movement prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementPrediction {
    /// Predicted discrete action
    pub action: DiscreteAction,
    /// Confidence score for the prediction
    pub confidence: f32,
    /// Full action probability distribution
    pub action_probabilities: Vec<f32>,
    /// Inference time in milliseconds
    pub inference_time_ms: f32,
}

impl MLMOVETransformer {
    /// Create new MLMOVE transformer with default configuration
    pub fn new(device: Device) -> Result<Self> {
        let config = MLMOVEConfig::default();
        Self::with_config(config, device)
    }

    /// Create MLMOVE transformer with custom configuration
    pub fn with_config(config: MLMOVEConfig, device: Device) -> Result<Self> {
        let varmap = VarMap::new();
        let vs = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        // Input embedding layer
        let input_embedding = linear(config.input_dim, config.model_dim, vs.pp("input_embedding"))?;

        // Positional embedding
        let position_embedding = embedding(config.sequence_length, config.model_dim, vs.pp("pos_embedding"))?;

        // Transformer layers
        let mut transformer_layers = Vec::new();
        for i in 0..config.num_layers {
            let layer = TransformerLayer::new(config.model_dim, config.ff_dim, vs.pp(&format!("layer_{}", i)))?;
            transformer_layers.push(layer);
        }

        // Output projection to action space
        let output_projection = linear(config.model_dim, config.action_space_size, vs.pp("output"))?;

        Ok(Self {
            input_embedding,
            position_embedding,
            transformer_layers,
            output_projection,
            config,
            device,
        })
    }

    /// Load pre-trained MLMOVE weights from safetensors file
    pub fn load_pretrained(model_path: &str, device: Device) -> Result<Self> {
        // TODO: Implement safetensors loading
        // For now, create a new model with default weights
        Self::new(device)
    }

    /// Forward pass through the transformer
    pub fn forward(&self, input_sequence: &Tensor) -> Result<Tensor> {
        let batch_size = input_sequence.shape().dims()[0];
        let seq_len = input_sequence.shape().dims()[1];

        // Input embedding
        let embedded = self.input_embedding.forward(input_sequence)?;

        // Add positional embeddings
        let positions = Tensor::arange(0u32, seq_len as u32, &self.device)?
            .unsqueeze(0)?
            .broadcast_as((batch_size, seq_len))?;
        let pos_embedded = self.position_embedding.forward(&positions)?;
        let mut x = (embedded + pos_embedded)?;

        // Pass through transformer layers
        for layer in &self.transformer_layers {
            x = layer.forward(&x)?;
        }

        // Output projection
        let output = self.output_projection.forward(&x)?;

        // Return last timestep for next action prediction
        let last_timestep = output.i((.., seq_len - 1, ..))?;
        Ok(last_timestep)
    }

    /// Predict next movement action from behavioral vector sequence
    pub fn predict_movement(&self, sequence: &[BehavioralVector]) -> Result<MovementPrediction> {
        let start_time = std::time::Instant::now();

        // Convert behavioral vectors to tensor
        let input_tensor = self.behavioral_vectors_to_tensor(sequence)?;

        // Forward pass
        let logits = self.forward(&input_tensor)?;
        let probabilities = self.softmax(&logits)?;

        // Find best action
        let probs_vec = probabilities.to_vec1::<f32>()?;
        let best_action_idx = probs_vec.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        let action = self.index_to_action(best_action_idx);
        let confidence = probs_vec[best_action_idx];

        let inference_time = start_time.elapsed().as_secs_f32() * 1000.0;

        Ok(MovementPrediction {
            action,
            confidence,
            action_probabilities: probs_vec,
            inference_time_ms: inference_time,
        })
    }

    /// Convert behavioral vectors to input tensor
    fn behavioral_vectors_to_tensor(&self, vectors: &[BehavioralVector]) -> Result<Tensor> {
        let seq_len = vectors.len().min(self.config.sequence_length);
        let mut input_data = Vec::new();

        for i in 0..seq_len {
            let vector = &vectors[vectors.len() - seq_len + i];
            
            // Extract relevant features for movement prediction
            let features = vec![
                vector.pos_x / 1000.0,    // Normalize position
                vector.pos_y / 1000.0,
                vector.pos_z / 100.0,
                vector.vel_x / 320.0,     // Normalize by max speed
                vector.vel_y / 320.0,
                vector.vel_z / 320.0,
                vector.yaw / 360.0,       // Normalize angles
                vector.pitch / 90.0,
                vector.health / 100.0,    // Normalize health/armor
                vector.armor / 100.0,
            ];
            
            input_data.extend(features);
        }

        // Pad if necessary
        while input_data.len() < self.config.sequence_length * self.config.input_dim {
            input_data.push(0.0);
        }

        let tensor = Tensor::from_slice(
            &input_data,
            (1, self.config.sequence_length, self.config.input_dim),
            &self.device
        )?;

        Ok(tensor)
    }

    /// Simplified softmax implementation
    fn softmax(&self, input: &Tensor) -> Result<Tensor> {
        let max_val = input.max(candle_core::D::Minus1)?.broadcast_as(input.shape())?;
        let shifted = input.sub(&max_val)?;
        let exp_vals = shifted.exp()?;
        let sum_exp = exp_vals.sum(candle_core::D::Minus1)?.broadcast_as(input.shape())?;
        exp_vals.div(&sum_exp)
    }

    /// Convert action index to discrete action
    fn index_to_action(&self, index: usize) -> DiscreteAction {
        // 97-way action space: 8 directions × 6 speeds × 2 jump states = 96, + 1 no-op
        if index == 96 {
            // No-op action
            return DiscreteAction { direction: 8, speed: 0, jump: 0 };
        }

        let direction = (index % 8) as u8;
        let speed = ((index / 8) % 6) as u8;
        let jump = (index / 48) as u8;

        DiscreteAction { direction, speed, jump }
    }

    /// Convert discrete action to CS2 movement commands
    pub fn action_to_movement_commands(&self, action: &DiscreteAction) -> MovementCommands {
        let direction_map = [
            (0.0, 1.0),   // Forward
            (1.0, 1.0),   // Forward-Right
            (1.0, 0.0),   // Right
            (1.0, -1.0),  // Back-Right
            (0.0, -1.0),  // Back
            (-1.0, -1.0), // Back-Left
            (-1.0, 0.0),  // Left
            (-1.0, 1.0),  // Forward-Left
        ];

        let (move_x, move_y) = if action.direction < 8 {
            direction_map[action.direction as usize]
        } else {
            (0.0, 0.0) // No movement
        };

        let speed_multiplier = match action.speed {
            0 => 0.0,   // No movement
            1 => 0.3,   // Slow walk
            2 => 0.6,   // Walk
            3 => 0.8,   // Fast walk
            4 => 1.0,   // Run
            5 => 1.0,   // Sprint (same as run in CS2)
            _ => 0.0,
        };

        MovementCommands {
            forward_move: move_y * speed_multiplier,
            side_move: move_x * speed_multiplier,
            jump: action.jump > 0,
        }
    }
}

/// CS2 Movement commands generated from discrete actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementCommands {
    /// Forward/backward movement (-1.0 to 1.0)
    pub forward_move: f32,
    /// Left/right movement (-1.0 to 1.0)
    pub side_move: f32,
    /// Jump command (true/false)
    pub jump: bool,
}

impl TransformerLayer {
    fn new(model_dim: usize, ff_dim: usize, vs: VarBuilder) -> Result<Self> {
        let self_attention = SelfAttention::new(model_dim, vs.pp("attention"))?;
        let feed_forward = FeedForward::new(model_dim, ff_dim, vs.pp("ff"))?;
        let ln1 = LayerNorm::new(model_dim, vs.pp("ln1"))?;
        let ln2 = LayerNorm::new(model_dim, vs.pp("ln2"))?;

        Ok(Self {
            self_attention,
            feed_forward,
            ln1,
            ln2,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        // Pre-norm architecture
        let normed1 = self.ln1.forward(input)?;
        let attention_out = self.self_attention.forward(&normed1)?;
        let residual1 = (input + attention_out)?;

        let normed2 = self.ln2.forward(&residual1)?;
        let ff_out = self.feed_forward.forward(&normed2)?;
        let residual2 = (residual1 + ff_out)?;

        Ok(residual2)
    }
}

impl SelfAttention {
    fn new(model_dim: usize, vs: VarBuilder) -> Result<Self> {
        let query = linear(model_dim, model_dim, vs.pp("query"))?;
        let key = linear(model_dim, model_dim, vs.pp("key"))?;
        let value = linear(model_dim, model_dim, vs.pp("value"))?;
        let output = linear(model_dim, model_dim, vs.pp("output"))?;

        Ok(Self {
            query,
            key,
            value,
            output,
            model_dim,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let queries = self.query.forward(input)?;
        let keys = self.key.forward(input)?;
        let values = self.value.forward(input)?;

        // Single-head attention (as per MLMOVE paper)
        let scores = queries.matmul(&keys.transpose(1, 2)?)?;
        let scaled_scores = scores.div(&Tensor::new((self.model_dim as f32).sqrt(), input.device())?)?;

        // Simplified attention weights (using max normalization instead of softmax)
        let max_scores = scaled_scores.max(candle_core::D::Minus1)?.broadcast_as(scaled_scores.shape())?;
        let attention_weights = (scaled_scores - max_scores)?.exp()?;

        let attended = attention_weights.matmul(&values)?;
        let output = self.output.forward(&attended)?;

        Ok(output)
    }
}

impl FeedForward {
    fn new(model_dim: usize, ff_dim: usize, vs: VarBuilder) -> Result<Self> {
        let linear1 = linear(model_dim, ff_dim, vs.pp("linear1"))?;
        let linear2 = linear(ff_dim, model_dim, vs.pp("linear2"))?;

        Ok(Self { linear1, linear2 })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let intermediate = self.linear1.forward(input)?;
        let activated = intermediate.relu()?; // ReLU activation
        let output = self.linear2.forward(&activated)?;
        Ok(output)
    }
}

impl LayerNorm {
    fn new(model_dim: usize, vs: VarBuilder) -> Result<Self> {
        let weight = linear(model_dim, model_dim, vs.pp("weight"))?;
        let bias = linear(model_dim, model_dim, vs.pp("bias"))?;

        Ok(Self { weight, bias })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        // Simplified layer normalization
        let mean = input.mean(candle_core::D::Minus1)?.broadcast_as(input.shape())?;
        let centered = input.sub(&mean)?;
        
        // Apply learned scaling and bias
        let scaled = self.weight.forward(&centered)?;
        let output = self.bias.forward(&scaled)?;
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlmove_transformer_creation() -> Result<()> {
        let transformer = MLMOVETransformer::new(Device::Cpu)?;
        assert_eq!(transformer.config.num_layers, 4);
        assert_eq!(transformer.config.num_heads, 1);
        assert_eq!(transformer.config.model_dim, 256);
        Ok(())
    }

    #[test]
    fn test_discrete_action_conversion() -> Result<()> {
        let transformer = MLMOVETransformer::new(Device::Cpu)?;
        
        // Test action index to discrete action conversion
        let action = transformer.index_to_action(0);
        assert_eq!(action.direction, 0);
        assert_eq!(action.speed, 0);
        assert_eq!(action.jump, 0);

        // Test movement command generation
        let commands = transformer.action_to_movement_commands(&DiscreteAction {
            direction: 0, // Forward
            speed: 4,     // Run
            jump: 1,      // Jump
        });
        
        assert_eq!(commands.forward_move, 1.0);
        assert_eq!(commands.side_move, 0.0);
        assert!(commands.jump);
        
        Ok(())
    }

    #[test]
    fn test_behavioral_vector_conversion() -> Result<()> {
        let transformer = MLMOVETransformer::new(Device::Cpu)?;
        
        let vectors = vec![
            BehavioralVector {
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
            }
        ];

        let tensor = transformer.behavioral_vectors_to_tensor(&vectors)?;
        assert_eq!(tensor.shape().dims(), &[1, 32, 10]); // batch, seq_len, features
        
        Ok(())
    }

    #[test]
    fn test_movement_prediction() -> Result<()> {
        let transformer = MLMOVETransformer::new(Device::Cpu)?;
        
        // Create sequence of behavioral vectors
        let vectors: Vec<BehavioralVector> = (0..10).map(|i| BehavioralVector {
            tick: i,
            steamid: 123456789,
            health: 100.0,
            armor: 100.0,
            pos_x: i as f32 * 10.0,
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
        }).collect();

        let prediction = transformer.predict_movement(&vectors)?;
        
        assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
        assert_eq!(prediction.action_probabilities.len(), 97);
        assert!(prediction.inference_time_ms >= 0.0);
        
        Ok(())
    }

    #[test]
    fn test_action_space_coverage() -> Result<()> {
        let transformer = MLMOVETransformer::new(Device::Cpu)?;
        
        // Test that we can represent all 97 actions
        for i in 0..97 {
            let action = transformer.index_to_action(i);
            if i == 96 {
                // No-op action
                assert_eq!(action.direction, 8);
            } else {
                assert!(action.direction < 8);
                assert!(action.speed < 6);
                assert!(action.jump < 2);
            }
        }
        
        Ok(())
    }

    #[test]
    fn test_transformer_forward_pass() -> Result<()> {
        let transformer = MLMOVETransformer::new(Device::Cpu)?;
        
        // Create dummy input tensor
        let input = Tensor::zeros((1, 32, 10), DType::F32, &Device::Cpu)?;
        let output = transformer.forward(&input)?;
        
        assert_eq!(output.shape().dims(), &[1, 97]); // batch, action_space
        
        Ok(())
    }
}
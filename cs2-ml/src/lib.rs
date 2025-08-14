// Re-export modules for library usage
pub mod conversion_utils;
pub mod data;
pub mod ml_architectures;
pub mod mlmove_transformer;
pub mod model;
pub mod player;
pub mod server;

// Re-export main types for convenience
pub use data::{vectors_from_demo, write_to_parquet};
pub use model::BehaviorNet;
pub use server::{serve, serve_with_model};

// Re-export advanced ML architectures
pub use ml_architectures::{
    AttentionMechanism, DecisionQualityAnalysis, DecisionQualityRNN, LSTMCell,
    PlayerStyleClassifier, PlayerStylePrediction, RNNCell, TeamDynamicsAnalysis,
    TeamDynamicsTransformer, MLP,
};

// Re-export MLMOVE transformer
pub use mlmove_transformer::{
    DiscreteAction, MLMOVEConfig, MLMOVETransformer, MovementCommands, MovementPrediction,
};

// Re-export conversion utilities
pub use conversion_utils::{
    convert_mlmove_to_candle, finetune_on_cs2_data, CS2FineTuner, CS2TrainingDataset,
    ConversionConfig, FineTuningConfig, TorchToCandleConverter, TrainingMetrics, TrainingSample,
};

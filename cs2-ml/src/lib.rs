// Re-export modules for library usage
pub mod data;
pub mod model;
pub mod player;
pub mod server;
pub mod ml_architectures;
pub mod mlmove_transformer;
pub mod conversion_utils;

// Re-export main types for convenience
pub use data::{vectors_from_demo, write_to_parquet};
pub use model::BehaviorNet;
pub use server::{serve, serve_with_model};

// Re-export advanced ML architectures
pub use ml_architectures::{
    PlayerStyleClassifier, TeamDynamicsTransformer, DecisionQualityRNN,
    PlayerStylePrediction, TeamDynamicsAnalysis, DecisionQualityAnalysis,
    MLP, AttentionMechanism, RNNCell, LSTMCell,
};

// Re-export MLMOVE transformer
pub use mlmove_transformer::{
    MLMOVETransformer, MLMOVEConfig, DiscreteAction, MovementPrediction, MovementCommands,
};

// Re-export conversion utilities
pub use conversion_utils::{
    TorchToCandleConverter, CS2FineTuner, ConversionConfig, FineTuningConfig,
    TrainingSample, CS2TrainingDataset, TrainingMetrics,
    convert_mlmove_to_candle, finetune_on_cs2_data,
};

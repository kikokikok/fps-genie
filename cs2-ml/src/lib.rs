// Re-export modules for library usage
pub mod data;
pub mod model;
pub mod player;
pub mod server;
pub mod ml_architectures;

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

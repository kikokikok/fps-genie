/// CS2 Analytics - Advanced behavioral analysis and professional comparison
/// 
/// This crate provides comprehensive analytics capabilities for CS2 demo analysis,
/// including professional player comparison, behavior cloning, and ML-driven insights.

pub mod analysis;
pub mod models;
pub mod pro_reference;
pub mod training;
pub mod visualization;

// Re-export key types for convenience
pub use pro_reference::{
    ProReferenceDataset, ProGapAnalysis, EarthMoverDistanceCalculator,
    OccupancyVector, FeatureGaps, PerformanceBenchmarks
};

pub use models::{AdvancedModels, BehaviorCloningConfig, CrosshairConfig};

pub use analysis::*;
pub use training::*;
pub use visualization::*;
/// Evaluation context module containing the EngineEvaluationContext struct
pub mod context;

/// Result module containing evaluation result types
pub mod result;

/// Segment evaluator module for evaluating segment rules
pub mod segment_evaluator;

/// Mappers module for converting between old and new types
pub mod mappers;

// Re-export commonly used types for convenience
pub use context::{EngineEvaluationContext, FeatureContext, FeatureMetadata};
pub use mappers::{add_identity_to_context, environment_to_context};
pub use result::{EvaluationResult, FlagResult, SegmentResult};
pub use segment_evaluator::is_context_in_segment;

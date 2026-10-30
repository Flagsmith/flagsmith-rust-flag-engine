use super::context::{FeatureMetadata, SegmentMetadata};
use crate::types::FlagsmithValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the result of a feature flag evaluation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EvaluationResult {
    /// Map of feature names to their evaluated flag results.
    pub flags: HashMap<String, FlagResult>,

    /// List of segments that matched during evaluation.
    #[serde(default)]
    pub segments: Vec<SegmentResult>,
}

/// Represents the evaluated result for a single feature flag.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FlagResult {
    /// Whether the feature is enabled.
    pub enabled: bool,

    /// The name of the feature.
    pub name: String,

    /// The reason for this evaluation result (e.g., "STATIC", "DEFAULT", "TARGETING_MATCH; segment=name", "SPLIT; weight=50", "ERROR; code=CIRCULAR_DEPENDENCY").
    pub reason: String,

    /// The value of the feature flag.
    pub value: FlagsmithValue,

    /// The key of the selected multivariate variant, "control" when none was selected, or `None` when no split applies.
    #[serde(default)]
    pub variant: Option<String>,

    /// Metadata about the feature.
    #[serde(default)]
    pub metadata: FeatureMetadata,
}

/// Represents a segment that matched during evaluation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SegmentResult {
    /// The segment name.
    pub name: String,

    /// Metadata about the segment.
    #[serde(default)]
    pub metadata: SegmentMetadata,
}

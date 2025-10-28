use crate::types::FlagsmithValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents metadata information about a feature.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FeatureMetadata {
    /// The feature ID.
    #[serde(default)]
    pub feature_id: u32,
}

/// Represents a multivariate value for a feature flag.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureValue {
    /// The value of the feature.
    pub value: FlagsmithValue,
    /// The weight of the feature value variant, as a percentage number (i.e. 100.0).
    pub weight: f64,
    /// Priority of the feature flag variant. Lower values indicate a higher priority when multiple variants apply to the same context key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
}

/// Represents a feature context for feature flag evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureContext {
    /// String key used for hashing in percentage splits.
    pub key: String,
    /// The name of the feature.
    pub name: String,
    /// Whether the feature is enabled.
    pub enabled: bool,
    /// The default value for the feature.
    pub value: FlagsmithValue,
    /// Priority for this feature context. Lower values indicate higher priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    /// Multivariate feature variants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<FeatureValue>,
    /// Metadata about the feature.
    #[serde(default)]
    pub metadata: FeatureMetadata,
}

/// Represents environment metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentContext {
    /// The environment API key.
    pub key: String,
    /// The environment name.
    pub name: String,
}

/// Represents identity context for feature flag evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityContext {
    /// The identity identifier.
    pub identifier: String,
    /// String key used for hashing in percentage splits.
    /// If not provided during deserialization, it will be constructed as "environment_key_identifier".
    #[serde(default)]
    pub key: String,
    /// Identity traits as a map of trait keys to values.
    #[serde(default)]
    pub traits: HashMap<String, FlagsmithValue>,
}

/// Segment rule condition operators.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConditionOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanInclusive,
    LessThan,
    LessThanInclusive,
    Contains,
    NotContains,
    In,
    Regex,
    PercentageSplit,
    Modulo,
    IsSet,
    IsNotSet,
}

// Helper function to deserialize value that can be a string or array
fn deserialize_condition_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde_json::Value;
    let value: Value = serde::Deserialize::deserialize(deserializer)?;
    Ok(match value {
        Value::String(s) => s,
        Value::Array(_) | Value::Object(_) | Value::Number(_) | Value::Bool(_) => {
            // Serialize non-string values back to JSON string
            serde_json::to_string(&value).unwrap_or_default()
        }
        Value::Null => String::new(),
    })
}

/// Represents a condition for segment rule evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Condition {
    /// The operator for this condition.
    pub operator: ConditionOperator,
    /// The property to evaluate (can be a JSONPath expression starting with $.).
    pub property: String,
    /// The value to compare against (can be a string or serialized JSON).
    #[serde(deserialize_with = "deserialize_condition_value")]
    pub value: String,
}

/// Segment rule types.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SegmentRuleType {
    All,
    Any,
    None,
}

/// Represents a segment rule (can be recursive).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentRule {
    /// The type of rule (ALL, ANY, NONE).
    #[serde(rename = "type")]
    pub rule_type: SegmentRuleType,
    /// Conditions for this rule.
    #[serde(default)]
    pub conditions: Vec<Condition>,
    /// Nested rules.
    #[serde(default)]
    pub rules: Vec<SegmentRule>,
}

/// Segment metadata.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SegmentMetadata {
    /// Segment ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<i32>,
    /// Source of the segment (api or identity_override).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Represents a segment context for feature flag evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentContext {
    /// Key used for percentage split segmentation.
    pub key: String,
    /// The name of the segment.
    pub name: String,
    /// Metadata about the segment.
    #[serde(default)]
    pub metadata: SegmentMetadata,
    /// Feature overrides for the segment.
    #[serde(default)]
    pub overrides: Vec<FeatureContext>,
    /// Rules that define the segment.
    pub rules: Vec<SegmentRule>,
}

/// Engine evaluation context that holds pre-processed environment data
/// for efficient feature flag evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineEvaluationContext {
    /// Environment metadata.
    pub environment: EnvironmentContext,

    /// Feature contexts indexed by feature name.
    #[serde(default)]
    pub features: HashMap<String, FeatureContext>,

    /// Segment contexts indexed by segment key.
    #[serde(default)]
    pub segments: HashMap<String, SegmentContext>,

    /// Optional identity context for evaluation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityContext>,
}

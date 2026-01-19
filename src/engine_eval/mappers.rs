use super::context::{
    Condition, ConditionOperator, EngineEvaluationContext, EnvironmentContext, FeatureContext,
    FeatureMetadata, FeatureValue, IdentityContext, SegmentContext, SegmentMetadata, SegmentRule,
    SegmentRuleType, SegmentSource,
};
use crate::environments::Environment;
use crate::features::{FeatureState, MultivariateFeatureStateValue};
use crate::identities::{Identity, Trait};
use crate::segments::{Segment, SegmentRule as OldSegmentRule};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Maps an Environment to an EngineEvaluationContext
///
/// # Arguments
/// * `environment` - The environment to convert
///
/// # Returns
/// A new engine evaluation context
pub fn environment_to_context(environment: Environment) -> EngineEvaluationContext {
    let mut ctx = EngineEvaluationContext {
        environment: EnvironmentContext {
            key: environment.api_key.clone(),
            name: environment.name.clone(),
        },
        features: HashMap::new(),
        segments: HashMap::new(),
        identity: None,
    };

    // Map feature states to feature contexts
    for fs in &environment.feature_states {
        let fc = map_feature_state_to_feature_context(fs);
        ctx.features.insert(fc.name.clone(), fc);
    }

    // Map project segments to segment contexts
    for segment in &environment.project.segments {
        let sc = map_segment_to_segment_context(segment);
        ctx.segments.insert(sc.key.clone(), sc);
    }

    // Map identity overrides to segments
    if !environment.identity_overrides.is_empty() {
        let identity_segments = map_identity_overrides_to_segments(&environment.identity_overrides);
        for (key, segment) in identity_segments {
            ctx.segments.insert(key, segment);
        }
    }

    ctx
}

/// Maps a FeatureState to a FeatureContext
fn map_feature_state_to_feature_context(fs: &FeatureState) -> FeatureContext {
    let key = if let Some(django_id) = fs.django_id {
        django_id.to_string()
    } else {
        fs.featurestate_uuid.clone()
    };

    let mut fc = FeatureContext {
        enabled: fs.enabled,
        key,
        name: fs.feature.name.clone(),
        value: fs.get_value(None),
        priority: None,
        variants: map_multivariate_values_to_variants(&fs.multivariate_feature_state_values),
        metadata: FeatureMetadata {
            feature_id: fs.feature.id,
            feature_type: fs
                .feature
                .feature_type
                .clone()
                .unwrap_or_else(|| "STANDARD".to_string()),
        },
    };

    // Set priority if this is a segment override
    if let Some(feature_segment) = &fs.feature_segment {
        fc.priority = Some(feature_segment.priority as f64);
    }

    fc
}

/// Maps multivariate feature state values to FeatureValue variants
fn map_multivariate_values_to_variants(
    mv_values: &[MultivariateFeatureStateValue],
) -> Vec<FeatureValue> {
    mv_values
        .iter()
        .map(|mv| FeatureValue {
            value: mv.multivariate_feature_option.value.clone(),
            weight: mv.percentage_allocation as f64,
            priority: None,
        })
        .collect()
}

/// Maps a Segment to a SegmentContext
fn map_segment_to_segment_context(segment: &Segment) -> SegmentContext {
    let mut sc = SegmentContext {
        key: segment.id.to_string(),
        name: segment.name.clone(),
        metadata: SegmentMetadata {
            segment_id: Some(segment.id as i32),
            source: SegmentSource::Api,
        },
        overrides: vec![],
        rules: vec![],
    };

    // Map feature state overrides
    for fs in &segment.feature_states {
        sc.overrides.push(map_feature_state_to_feature_context(fs));
    }

    // Map segment rules
    for rule in &segment.rules {
        sc.rules.push(map_segment_rule_to_rule(rule));
    }

    sc
}

/// Maps a legacy SegmentRule to the new SegmentRule format
fn map_segment_rule_to_rule(rule: &OldSegmentRule) -> SegmentRule {
    let rule_type = map_rule_type(&rule.segment_rule_type);

    let conditions = rule
        .conditions
        .iter()
        .map(|c| Condition {
            operator: map_operator(&c.operator),
            property: c.property.clone().unwrap_or_default(),
            value: super::context::ConditionValue::Single(c.value.clone().unwrap_or_default()),
        })
        .collect();

    let rules = rule
        .rules
        .iter()
        .map(|r| map_segment_rule_to_rule(r))
        .collect();

    SegmentRule {
        rule_type,
        conditions,
        rules,
    }
}

/// Maps a rule type string to SegmentRuleType enum
fn map_rule_type(rule_type: &str) -> SegmentRuleType {
    match rule_type {
        "ALL" => SegmentRuleType::All,
        "ANY" => SegmentRuleType::Any,
        "NONE" => SegmentRuleType::None,
        _ => SegmentRuleType::All,
    }
}

/// Maps an operator string to ConditionOperator enum
fn map_operator(operator: &str) -> ConditionOperator {
    match operator {
        "EQUAL" => ConditionOperator::Equal,
        "NOT_EQUAL" => ConditionOperator::NotEqual,
        "GREATER_THAN" => ConditionOperator::GreaterThan,
        "GREATER_THAN_INCLUSIVE" => ConditionOperator::GreaterThanInclusive,
        "LESS_THAN" => ConditionOperator::LessThan,
        "LESS_THAN_INCLUSIVE" => ConditionOperator::LessThanInclusive,
        "CONTAINS" => ConditionOperator::Contains,
        "NOT_CONTAINS" => ConditionOperator::NotContains,
        "IN" => ConditionOperator::In,
        "REGEX" => ConditionOperator::Regex,
        "PERCENTAGE_SPLIT" => ConditionOperator::PercentageSplit,
        "MODULO" => ConditionOperator::Modulo,
        "IS_SET" => ConditionOperator::IsSet,
        "IS_NOT_SET" => ConditionOperator::IsNotSet,
        _ => ConditionOperator::Equal,
    }
}

/// Helper struct for grouping identity overrides
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct OverrideKey {
    feature_name: String,
    enabled: String,
    feature_value: String,
    feature_id: u32,
    feature_type: String,
}

/// Maps identity overrides to segment contexts
fn map_identity_overrides_to_segments(identities: &[Identity]) -> HashMap<String, SegmentContext> {
    let mut features_to_identifiers: HashMap<String, Vec<String>> = HashMap::new();
    let mut overrides_key_to_list: HashMap<String, Vec<OverrideKey>> = HashMap::new();

    for identity in identities {
        if identity.identity_features.is_empty() {
            continue;
        }

        // Create override keys from features
        let mut overrides = Vec::new();
        for fs in &identity.identity_features {
            // Use proper JSON serialization instead of Debug format
            let feature_value =
                serde_json::to_string(&fs.get_value(None)).unwrap_or_else(|_| "null".to_string());
            overrides.push(OverrideKey {
                feature_name: fs.feature.name.clone(),
                enabled: fs.enabled.to_string(),
                feature_value,
                feature_id: fs.feature.id,
                feature_type: fs
                    .feature
                    .feature_type
                    .clone()
                    .unwrap_or_else(|| "STANDARD".to_string()),
            });
        }

        // Sort overrides for consistent hashing
        overrides.sort();

        // Generate hash for this set of overrides
        let overrides_hash = generate_hash(&overrides);

        // Group identifiers by their overrides
        features_to_identifiers
            .entry(overrides_hash.clone())
            .or_default()
            .push(identity.identifier.clone());

        overrides_key_to_list.insert(overrides_hash, overrides);
    }

    // Create segment contexts for each unique set of overrides
    let mut segment_contexts = HashMap::new();

    for (overrides_hash, identifiers) in features_to_identifiers {
        let overrides = overrides_key_to_list.get(&overrides_hash).unwrap();

        // Create segment context
        let mut sc = SegmentContext {
            key: String::new(), // Identity override segments never use % Split operator
            name: "identity_overrides".to_string(),
            metadata: SegmentMetadata {
                segment_id: None,
                source: SegmentSource::IdentityOverride,
            },
            overrides: vec![],
            rules: vec![SegmentRule {
                rule_type: SegmentRuleType::All,
                conditions: vec![Condition {
                    operator: ConditionOperator::In,
                    property: "$.identity.identifier".to_string(),
                    value: super::context::ConditionValue::Multiple(identifiers),
                }],
                rules: vec![],
            }],
        };

        // Create feature overrides
        for override_key in overrides {
            let priority = f64::NEG_INFINITY; // Strongest possible priority
            let feature_override = FeatureContext {
                key: String::new(), // Identity overrides never carry multivariate options
                name: override_key.feature_name.clone(),
                enabled: override_key.enabled == "true",
                value: serde_json::from_str(&override_key.feature_value).unwrap_or_default(),
                priority: Some(priority),
                variants: vec![],
                metadata: FeatureMetadata {
                    feature_id: override_key.feature_id,
                    feature_type: override_key.feature_type.clone(),
                },
            };

            sc.overrides.push(feature_override);
        }

        segment_contexts.insert(overrides_hash, sc);
    }

    segment_contexts
}

/// Generates a hash from override keys for use as segment key
fn generate_hash(overrides: &[OverrideKey]) -> String {
    let mut hasher = Sha256::new();

    for override_key in overrides {
        hasher.update(format!(
            "{}:{}:{}:{};",
            override_key.feature_id,
            override_key.feature_name,
            override_key.enabled,
            override_key.feature_value
        ));
    }

    let result = hasher.finalize();
    // Use safe slicing - take first 16 chars without panicking
    let hex = format!("{:x}", result);
    hex.chars().take(16).collect()
}

/// Adds identity data to an existing context
///
/// # Arguments
/// * `context` - The context to enrich with identity data
/// * `identifier` - The identity identifier
/// * `traits` - The identity traits
///
/// # Returns
/// A new context with identity information
pub fn add_identity_to_context(
    context: &EngineEvaluationContext,
    identifier: &str,
    traits: &[Trait],
) -> EngineEvaluationContext {
    let mut new_context = context.clone();

    // Create traits map
    let mut identity_traits = HashMap::new();
    for trait_obj in traits {
        identity_traits.insert(trait_obj.trait_key.clone(), trait_obj.trait_value.clone());
    }

    // Create identity context
    let environment_key = &new_context.environment.key;
    let identity = IdentityContext {
        identifier: identifier.to_string(),
        key: format!("{}_{}", environment_key, identifier),
        traits: identity_traits,
    };

    new_context.identity = Some(identity);
    new_context
}

use crate::engine_eval::context::{EngineEvaluationContext, FeatureContext};
use crate::engine_eval::result::{EvaluationResult, FlagResult, SegmentResult};
use crate::engine_eval::segment_evaluator::is_context_in_segment;
use crate::utils::hashing;
use std::collections::HashMap;

/// Holds a feature context with its associated segment name for priority comparison
struct FeatureContextWithSegment {
    feature_context: FeatureContext,
    segment_name: String,
}

/// Helper to get priority or default
fn get_priority_or_default(priority: Option<f64>) -> f64 {
    priority.unwrap_or(f64::INFINITY) // Weakest possible priority
}

/// Gets matching segments and their overrides
fn get_matching_segments_and_overrides(
    ec: &EngineEvaluationContext,
) -> (
    Vec<SegmentResult>,
    HashMap<String, FeatureContextWithSegment>,
) {
    let mut segments = Vec::new();
    let mut segment_feature_contexts: HashMap<String, FeatureContextWithSegment> = HashMap::new();

    // Process segments
    for segment_context in ec.segments.values() {
        if !is_context_in_segment(ec, segment_context) {
            continue;
        }

        // Add segment to results
        segments.push(SegmentResult {
            name: segment_context.name.clone(),
            metadata: segment_context.metadata.clone(),
        });

        // Process segment overrides
        for override_fc in &segment_context.overrides {
            let feature_name = &override_fc.name;

            // Check if we should update the segment feature context
            let should_update = if let Some(existing) = segment_feature_contexts.get(feature_name) {
                let existing_priority = get_priority_or_default(existing.feature_context.priority);
                let override_priority = get_priority_or_default(override_fc.priority);
                override_priority < existing_priority
            } else {
                true
            };

            if should_update {
                segment_feature_contexts.insert(
                    feature_name.clone(),
                    FeatureContextWithSegment {
                        feature_context: override_fc.clone(),
                        segment_name: segment_context.name.clone(),
                    },
                );
            }
        }
    }

    (segments, segment_feature_contexts)
}

/// Gets flag results from feature contexts and segment overrides
fn get_flag_results(
    ec: &EngineEvaluationContext,
    segment_feature_contexts: &HashMap<String, FeatureContextWithSegment>,
) -> HashMap<String, FlagResult> {
    let mut flags = HashMap::new();

    // Get identity key if identity exists
    // If identity key is not provided, construct it from environment key and identifier
    let identity_key: Option<String> = ec.identity.as_ref().map(|i| {
        if i.key.is_empty() {
            format!("{}_{}", ec.environment.key, i.identifier)
        } else {
            i.key.clone()
        }
    });

    // Process all features
    for feature_context in ec.features.values() {
        // Check if we have a segment override for this feature
        if let Some(segment_fc) = segment_feature_contexts.get(&feature_context.name) {
            // Use segment override with multivariate evaluation
            let fc = &segment_fc.feature_context;
            let reason = format!("TARGETING_MATCH; segment={}", segment_fc.segment_name);
            let flag_result = get_flag_result_from_feature_context(fc, identity_key.as_ref(), reason);
            flags.insert(feature_context.name.clone(), flag_result);
        } else {
            // Use default feature context
            let flag_result = get_flag_result_from_feature_context(
                feature_context,
                identity_key.as_ref(),
                "DEFAULT".to_string(),
            );
            flags.insert(feature_context.name.clone(), flag_result);
        }
    }

    flags
}

pub fn get_evaluation_result(ec: &EngineEvaluationContext) -> EvaluationResult {
    // Process segments
    let (segments, segment_feature_contexts) = get_matching_segments_and_overrides(ec);

    // Get flag results
    let flags = get_flag_results(ec, &segment_feature_contexts);

    EvaluationResult { flags, segments }
}

/// Creates a FlagResult from a FeatureContext
fn get_flag_result_from_feature_context(
    feature_context: &FeatureContext,
    identity_key: Option<&String>,
    default_reason: String,
) -> FlagResult {
    let mut reason = default_reason;
    let mut value = feature_context.value.clone();

    // Handle multivariate features
    if !feature_context.variants.is_empty()
        && identity_key.is_some()
        && !feature_context.key.is_empty()
    {
        // Sort variants by priority (lower priority value = higher priority)
        let mut sorted_variants = feature_context.variants.clone();
        sorted_variants.sort_by(|a, b| {
            let pa = get_priority_or_default(a.priority);
            let pb = get_priority_or_default(b.priority);
            pa.partial_cmp(&pb).unwrap()
        });

        // Calculate hash percentage for the identity and feature combination
        let object_ids = vec![feature_context.key.as_str(), identity_key.unwrap().as_str()];
        let hash_percentage = hashing::get_hashed_percentage_for_object_ids(object_ids, 1);

        // Select variant based on weighted distribution
        let mut cumulative_weight = 0.0;
        for variant in &sorted_variants {
            cumulative_weight += variant.weight;
            if (hash_percentage as f64) <= cumulative_weight {
                value = variant.value.clone();
                reason = format!("SPLIT; weight={}", variant.weight);
                break;
            }
        }
    }

    FlagResult {
        enabled: feature_context.enabled,
        name: feature_context.name.clone(),
        value,
        reason,
        metadata: feature_context.metadata.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_eval::context::EnvironmentContext;

    #[test]
    fn test_get_priority_or_default() {
        assert_eq!(get_priority_or_default(Some(1.0)), 1.0);
        assert_eq!(get_priority_or_default(None), f64::INFINITY);
    }

    #[test]
    fn test_get_evaluation_result_empty_context() {
        let ec = EngineEvaluationContext {
            environment: EnvironmentContext {
                key: "test".to_string(),
                name: "test".to_string(),
            },
            features: HashMap::new(),
            segments: HashMap::new(),
            identity: None,
        };

        let result = get_evaluation_result(&ec);
        assert_eq!(result.flags.len(), 0);
        assert_eq!(result.segments.len(), 0);
    }
}

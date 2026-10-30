use crate::engine_eval::context::{EngineEvaluationContext, FeatureContext};
use crate::engine_eval::result::{EvaluationResult, FlagResult, SegmentResult};
use crate::engine_eval::segment_evaluator::{ResolvedFlags, SegmentEvaluator};
use crate::utils::hashing;
use std::collections::{HashMap, HashSet};

/// Reason given to flags whose evaluation depends on themselves.
pub const CIRCULAR_DEPENDENCY_REASON: &str = "ERROR; code=CIRCULAR_DEPENDENCY";

/// A segment override, with the name of the segment it belongs to
pub(crate) struct SegmentOverride<'a> {
    pub(crate) feature_context: &'a FeatureContext,
    pub(crate) segment_name: &'a str,
}

/// Helper to get priority or default
fn get_priority_or_default(priority: Option<f64>) -> f64 {
    priority.unwrap_or(f64::INFINITY) // Weakest possible priority
}

/// Whether a segment override takes precedence over the one held so far; the first seen wins a tie
pub(crate) fn wins_over(candidate: &FeatureContext, incumbent: Option<&FeatureContext>) -> bool {
    incumbent.is_none_or(|incumbent| {
        get_priority_or_default(candidate.priority) < get_priority_or_default(incumbent.priority)
    })
}

/// Gets the reason for a flag evaluated to its environment default:
/// `DEFAULT` if segments override the feature, `STATIC` if none does.
pub(crate) fn get_fallback_reason(is_targeted: bool) -> String {
    if is_targeted { "DEFAULT" } else { "STATIC" }.to_string()
}

/// Matching segments, the winning override of each feature they override, and every feature any segment overrides
struct SegmentEvaluation<'a> {
    segments: Vec<SegmentResult>,
    segment_overrides: HashMap<&'a str, SegmentOverride<'a>>,
    targeted_feature_names: HashSet<&'a str>,
}

/// Gets matching segments and their overrides
fn evaluate_segments<'a>(
    ec: &'a EngineEvaluationContext,
    evaluator: &SegmentEvaluator<'a>,
) -> SegmentEvaluation<'a> {
    let mut segments = Vec::new();
    let mut segment_overrides: HashMap<&str, SegmentOverride> = HashMap::new();
    let mut targeted_feature_names = HashSet::new();

    for (segment_key, segment_context) in &ec.segments {
        targeted_feature_names.extend(segment_context.overrides.iter().map(|o| o.name.as_str()));

        if !evaluator.matches_segment_key(segment_key) {
            continue;
        }

        // Add segment to results
        segments.push(SegmentResult {
            name: segment_context.name.clone(),
            metadata: segment_context.metadata.clone(),
        });

        // Process segment overrides
        for override_fc in &segment_context.overrides {
            let incumbent = segment_overrides
                .get(override_fc.name.as_str())
                .map(|o| o.feature_context);
            if wins_over(override_fc, incumbent) {
                segment_overrides.insert(
                    &override_fc.name,
                    SegmentOverride {
                        feature_context: override_fc,
                        segment_name: &segment_context.name,
                    },
                );
            }
        }
    }

    SegmentEvaluation {
        segments,
        segment_overrides,
        targeted_feature_names,
    }
}

/// Gets flag results from feature contexts and segment overrides
fn get_flag_results(
    ec: &EngineEvaluationContext,
    segment_evaluation: &SegmentEvaluation,
    identity_key: Option<&String>,
) -> HashMap<String, FlagResult> {
    let mut flags = HashMap::new();

    // Process all features
    for feature_context in ec.features.values() {
        let feature_name = feature_context.name.as_str();
        let flag_result = match segment_evaluation.segment_overrides.get(feature_name) {
            Some(segment_override) => get_flag_result_from_feature_context(
                segment_override.feature_context,
                identity_key,
                format!("TARGETING_MATCH; segment={}", segment_override.segment_name),
            ),
            None => get_flag_result_from_feature_context(
                feature_context,
                identity_key,
                get_fallback_reason(
                    segment_evaluation
                        .targeted_feature_names
                        .contains(feature_name),
                ),
            ),
        };
        flags.insert(feature_context.name.clone(), flag_result);
    }

    flags
}

pub fn get_evaluation_result(ec: &EngineEvaluationContext) -> EvaluationResult {
    // Get identity key if identity exists
    // If identity key is not provided, construct it from environment key and identifier
    let identity_key: Option<String> = ec.identity.as_ref().map(|i| {
        if i.key.is_empty() {
            format!("{}_{}", ec.environment.key, i.identifier)
        } else {
            i.key.clone()
        }
    });

    let evaluator = SegmentEvaluator::with_flag_dependencies(ec, identity_key.as_ref());
    let segment_evaluation = evaluate_segments(ec, &evaluator);
    let mut flags = get_flag_results(ec, &segment_evaluation, identity_key.as_ref());

    // Flags read by segment conditions take precedence, as only they were resolved guarding against cycles
    let ResolvedFlags {
        flags: resolved_flags,
        cyclic,
    } = evaluator.into_resolved_flags();
    flags.extend(
        resolved_flags
            .into_iter()
            .map(|(feature_name, flag)| (feature_name.to_string(), flag)),
    );
    for feature_name in cyclic {
        if let Some(flag) = flags.get_mut(feature_name) {
            flag.reason = CIRCULAR_DEPENDENCY_REASON.to_string();
        }
    }

    EvaluationResult {
        flags,
        segments: segment_evaluation.segments,
    }
}

/// Creates a FlagResult from a FeatureContext
pub(crate) fn get_flag_result_from_feature_context(
    feature_context: &FeatureContext,
    identity_key: Option<&String>,
    default_reason: String,
) -> FlagResult {
    let mut reason = default_reason;
    let mut value = feature_context.value.clone();
    let mut variant = None;

    // Handle multivariate features
    if !feature_context.variants.is_empty()
        && identity_key.is_some()
        && !feature_context.key.is_empty()
    {
        variant = Some("control".to_string()); // Unless a variant is selected below

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
        for feature_value in &sorted_variants {
            cumulative_weight += feature_value.weight;
            if (hash_percentage as f64) <= cumulative_weight {
                value = feature_value.value.clone();
                variant = feature_value.key.clone();
                reason = format!("SPLIT; weight={}", feature_value.weight);
                break;
            }
        }
    }

    FlagResult {
        enabled: feature_context.enabled,
        name: feature_context.name.clone(),
        value,
        variant,
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
            segments: Default::default(),
            identity: None,
        };

        let result = get_evaluation_result(&ec);
        assert_eq!(result.flags.len(), 0);
        assert_eq!(result.segments.len(), 0);
    }
}

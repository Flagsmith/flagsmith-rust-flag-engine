use crate::engine_eval::context::{EngineEvaluationContext, FeatureContext};
use crate::engine_eval::result::{EvaluationResult, FlagResult, SegmentResult};
use crate::engine_eval::segment_evaluator::{
    is_context_in_segment_with_inputs, ConditionInputs, ContextJson,
};
use crate::utils::hashing;
use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{HashMap, HashSet};

/// Reason given to flags whose evaluation depends on themselves.
pub const CIRCULAR_DEPENDENCY_REASON: &str = "ERROR; code=CIRCULAR_DEPENDENCY";

/// Helper to get priority or default
fn get_priority_or_default(priority: Option<f64>) -> f64 {
    priority.unwrap_or(f64::INFINITY) // Weakest possible priority
}

/// Whether a segment override takes precedence over the one held so far; the first seen wins a tie
fn wins_over(candidate: &FeatureContext, incumbent: Option<&FeatureContext>) -> bool {
    incumbent.is_none_or(|incumbent| {
        get_priority_or_default(candidate.priority) < get_priority_or_default(incumbent.priority)
    })
}

/// Gets matching segments, and the winning override, if any, of every feature segments override
fn get_matching_segments_and_overrides<'a>(
    ec: &'a EngineEvaluationContext,
    resolver: &DependencyResolver,
) -> (
    Vec<SegmentResult>,
    HashMap<&'a str, Option<SegmentOverride<'a>>>,
) {
    let mut segments = Vec::new();
    let mut segment_feature_contexts: HashMap<&str, Option<SegmentOverride>> = HashMap::new();

    // Sort segment keys for deterministic ordering
    let mut segment_keys: Vec<_> = ec.segments.keys().collect();
    segment_keys.sort();

    // Process segments in sorted order
    for segment_key in segment_keys {
        let segment_context = &ec.segments[segment_key];

        let cycle_hits = resolver.cycle_hits.get();
        let matches = is_context_in_segment_with_inputs(ec, segment_context, resolver);
        // A verdict reached on a flag that could not be resolved is not sound either way
        if !matches || resolver.cycle_hits.get() != cycle_hits {
            for override_fc in &segment_context.overrides {
                segment_feature_contexts
                    .entry(override_fc.name.as_str())
                    .or_insert(None);
            }
            continue;
        }

        // Add segment to results
        segments.push(SegmentResult {
            name: segment_context.name.clone(),
            metadata: segment_context.metadata.clone(),
        });

        // Process segment overrides
        for override_fc in &segment_context.overrides {
            let feature_name = override_fc.name.as_str();

            let incumbent = segment_feature_contexts
                .get(feature_name)
                .and_then(Option::as_ref)
                .map(|existing| existing.feature_context);

            if wins_over(override_fc, incumbent) {
                segment_feature_contexts.insert(
                    feature_name,
                    Some(SegmentOverride {
                        feature_context: override_fc,
                        segment_name: &segment_context.name,
                    }),
                );
            }
        }
    }

    (segments, segment_feature_contexts)
}

/// Gets flag results from feature contexts and segment overrides
fn get_flag_results(
    ec: &EngineEvaluationContext,
    segment_feature_contexts: &HashMap<&str, Option<SegmentOverride>>,
    identity_key: Option<&String>,
) -> HashMap<String, FlagResult> {
    let mut flags = HashMap::new();

    // Process all features
    for feature_context in ec.features.values() {
        let segment_override = segment_feature_contexts.get(feature_context.name.as_str());
        // Check if we have a segment override for this feature
        if let Some(Some(segment_fc)) = segment_override {
            // Use segment override with multivariate evaluation
            let fc = segment_fc.feature_context;
            let reason = format!("TARGETING_MATCH; segment={}", segment_fc.segment_name);
            let flag_result = get_flag_result_from_feature_context(fc, identity_key, reason);
            flags.insert(feature_context.name.clone(), flag_result);
        } else {
            // Use default feature context
            let flag_result = get_flag_result_from_feature_context(
                feature_context,
                identity_key,
                get_fallback_reason(segment_override.is_some()),
            );
            flags.insert(feature_context.name.clone(), flag_result);
        }
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

    let resolver = DependencyResolver::new(ec, identity_key.as_ref());

    // Process segments
    let (segments, segment_feature_contexts) = get_matching_segments_and_overrides(ec, &resolver);

    // Get flag results
    let mut flags = get_flag_results(ec, &segment_feature_contexts, identity_key.as_ref());

    // Flags resolved for segment conditions take precedence, as only they are guarded against cycles
    flags.extend(resolver.resolved.take());
    for feature_name in resolver.cyclic.take() {
        if let Some(flag) = flags.get_mut(&feature_name) {
            flag.reason = CIRCULAR_DEPENDENCY_REASON.to_string();
        }
    }

    EvaluationResult { flags, segments }
}

/// Resolves, and memoises, the flags read by segment conditions through `$.flags` paths
struct DependencyResolver<'a> {
    ec: &'a EngineEvaluationContext,
    identity_key: Option<&'a String>,
    context_json: ContextJson<'a>,
    /// Segment keys, in evaluation order, of the segments overriding each feature; built when a
    /// condition first reads a flag
    segment_keys_by_feature_name: OnceCell<HashMap<&'a str, Vec<&'a String>>>,
    segment_matches: RefCell<HashMap<&'a str, bool>>,
    /// Features being resolved, outermost first
    resolving: RefCell<Vec<String>>,
    resolved: RefCell<HashMap<String, FlagResult>>,
    /// Features found to be in a dependency cycle
    cyclic: RefCell<HashSet<String>>,
    /// Incremented whenever a cycle is cut, so that evaluations reaching one can be told apart
    cycle_hits: Cell<usize>,
}

impl<'a> DependencyResolver<'a> {
    fn new(ec: &'a EngineEvaluationContext, identity_key: Option<&'a String>) -> Self {
        Self {
            ec,
            identity_key,
            context_json: ContextJson::new(ec),
            segment_keys_by_feature_name: OnceCell::new(),
            segment_matches: RefCell::new(HashMap::new()),
            resolving: RefCell::new(Vec::new()),
            resolved: RefCell::new(HashMap::new()),
            cyclic: RefCell::new(HashSet::new()),
            cycle_hits: Cell::new(0),
        }
    }

    fn resolve_feature(&self, feature_name: &str) {
        let cycle_start = self
            .resolving
            .borrow()
            .iter()
            .position(|name| name == feature_name);
        if let Some(cycle_start) = cycle_start {
            let resolving = self.resolving.borrow();
            self.cyclic
                .borrow_mut()
                .extend(resolving[cycle_start..].iter().cloned());
            self.cycle_hits.set(self.cycle_hits.get() + 1);
            return;
        }

        let Some(feature_context) = self.ec.features.get(feature_name) else {
            return;
        };

        self.resolving.borrow_mut().push(feature_name.to_string());
        let segment_override = self.get_segment_override(feature_name);
        self.resolving.borrow_mut().pop();

        if self.cyclic.borrow().contains(feature_name) {
            // Not a result another condition may match on
            return;
        }

        let flag_result = match segment_override {
            Some(segment_override) => get_flag_result_from_feature_context(
                segment_override.feature_context,
                self.identity_key,
                format!("TARGETING_MATCH; segment={}", segment_override.segment_name),
            ),
            None => get_flag_result_from_feature_context(
                feature_context,
                self.identity_key,
                get_fallback_reason(
                    self.get_segment_keys_by_feature_name()
                        .contains_key(feature_name),
                ),
            ),
        };
        self.resolved
            .borrow_mut()
            .insert(feature_name.to_string(), flag_result);
    }

    fn matches_segment(&self, segment_key: &'a str) -> bool {
        if let Some(matches) = self.segment_matches.borrow().get(segment_key) {
            return *matches;
        }

        let cycle_hits = self.cycle_hits.get();
        let matches =
            is_context_in_segment_with_inputs(self.ec, &self.ec.segments[segment_key], self);
        if self.cycle_hits.get() != cycle_hits {
            // Reached by cutting a cycle, so the verdict rests on a flag that could not be resolved
            return false;
        }

        self.segment_matches
            .borrow_mut()
            .insert(segment_key, matches);
        matches
    }

    fn get_segment_override(&self, feature_name: &str) -> Option<SegmentOverride<'a>> {
        let mut segment_override: Option<SegmentOverride<'a>> = None;

        let segment_keys = self
            .get_segment_keys_by_feature_name()
            .get(feature_name)
            .map(Vec::as_slice)
            .unwrap_or_default();
        for segment_key in segment_keys {
            if !self.matches_segment(segment_key) {
                continue;
            }
            let segment_context = &self.ec.segments[segment_key.as_str()];
            for override_fc in &segment_context.overrides {
                if override_fc.name == feature_name
                    && wins_over(
                        override_fc,
                        segment_override.as_ref().map(|o| o.feature_context),
                    )
                {
                    segment_override = Some(SegmentOverride {
                        feature_context: override_fc,
                        segment_name: &segment_context.name,
                    });
                }
            }
        }

        segment_override
    }

    fn get_segment_keys_by_feature_name(&self) -> &HashMap<&'a str, Vec<&'a String>> {
        self.segment_keys_by_feature_name.get_or_init(|| {
            let mut segment_keys: Vec<_> = self.ec.segments.keys().collect();
            segment_keys.sort();

            let mut segment_keys_by_feature_name: HashMap<&str, Vec<&String>> = HashMap::new();
            for segment_key in segment_keys {
                for override_fc in &self.ec.segments[segment_key].overrides {
                    segment_keys_by_feature_name
                        .entry(override_fc.name.as_str())
                        .or_default()
                        .push(segment_key);
                }
            }
            segment_keys_by_feature_name
        })
    }
}

impl ConditionInputs for DependencyResolver<'_> {
    fn get_context_json(&self) -> Option<&serde_json::Value> {
        self.context_json.get()
    }

    fn get_flag(&self, feature_name: &str) -> Option<FlagResult> {
        if let Some(flag) = self.resolved.borrow().get(feature_name) {
            return Some(flag.clone());
        }
        self.resolve_feature(feature_name);
        self.resolved.borrow().get(feature_name).cloned()
    }
}

/// A segment override borrowed from the evaluation context
struct SegmentOverride<'a> {
    feature_context: &'a FeatureContext,
    segment_name: &'a str,
}

/// Gets the reason for a flag evaluated to its environment default:
/// `DEFAULT` if the feature has segment overrides that did not match, `STATIC` if it has none.
fn get_fallback_reason(is_targeted: bool) -> String {
    if is_targeted { "DEFAULT" } else { "STATIC" }.to_string()
}

/// Creates a FlagResult from a FeatureContext
fn get_flag_result_from_feature_context(
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

        variant = Some("control".to_string());

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
            segments: HashMap::new(),
            identity: None,
        };

        let result = get_evaluation_result(&ec);
        assert_eq!(result.flags.len(), 0);
        assert_eq!(result.segments.len(), 0);
    }
}

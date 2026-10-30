use super::context::{
    Condition, ConditionOperator, EngineEvaluationContext, SegmentContext, SegmentRule,
    SegmentRuleType,
};
use super::result::FlagResult;
use crate::engine::{
    get_fallback_reason, get_flag_result_from_feature_context, wins_over, SegmentOverride,
};
use crate::types::{FlagsmithValue, FlagsmithValueType};
use crate::utils::hashing;
use jsonpath_rust::parser::model::{JpQuery, Segment, Selector};
use jsonpath_rust::parser::parse_json_path;
use jsonpath_rust::query::js_path_process;
use regex::Regex;
use semver::Version;
use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Evaluates segments against a single evaluation context.
///
/// The context is immutable for the lifetime of the evaluator, so work that
/// depends only on the context is computed lazily and shared across all
/// segments and conditions:
/// - the context is serialised to JSON at most once, and only if a JSONPath
///   property is evaluated;
/// - each JSONPath property is parsed at most once, and resolved at most once
///   unless it reads a flag.
///
/// Built [`with_flag_dependencies`](Self::with_flag_dependencies), conditions
/// can also read other flags through `$.flags.<feature>` paths, each flag
/// resolved on first read.
pub(crate) struct SegmentEvaluator<'a> {
    ec: &'a EngineEvaluationContext,
    context_json: OnceCell<Option<serde_json::Value>>,
    json_path_values: RefCell<HashMap<String, Option<FlagsmithValue>>>,
    flag_dependencies: Option<FlagDependencies<'a>>,
}

/// The flags read by segment conditions, resolved on first read and memoised
struct FlagDependencies<'a> {
    identity_key: Option<&'a String>,
    /// Keys of the segments overriding each feature, in evaluation order
    segment_keys_by_feature_name: OnceCell<HashMap<&'a str, Vec<&'a str>>>,
    segment_matches: RefCell<HashMap<&'a str, bool>>,
    flag_paths: RefCell<HashMap<String, Option<Rc<FlagPath>>>>,
    /// Features being resolved, outermost first
    resolving: RefCell<Vec<&'a str>>,
    resolved: RefCell<HashMap<&'a str, FlagResult>>,
    cyclic: RefCell<HashSet<&'a str>>,
    /// Incremented whenever a cycle is cut, so that verdicts resting on one can be told apart
    cycle_hits: Cell<usize>,
    /// Whether a condition has read a flag, after which segment matches are memoised
    used: Cell<bool>,
}

/// A JSONPath reading the result of the feature it names, e.g. `$.flags.my_feature.enabled`
struct FlagPath {
    feature_name: String,
    json_path: JpQuery,
}

/// Flags read by segment conditions, and the features found in a dependency cycle
#[derive(Default)]
pub(crate) struct ResolvedFlags<'a> {
    pub(crate) flags: HashMap<&'a str, FlagResult>,
    pub(crate) cyclic: HashSet<&'a str>,
}

impl<'a> SegmentEvaluator<'a> {
    pub(crate) fn new(ec: &'a EngineEvaluationContext) -> Self {
        Self {
            ec,
            context_json: OnceCell::new(),
            json_path_values: RefCell::new(HashMap::new()),
            flag_dependencies: None,
        }
    }

    pub(crate) fn with_flag_dependencies(
        ec: &'a EngineEvaluationContext,
        identity_key: Option<&'a String>,
    ) -> Self {
        Self {
            flag_dependencies: Some(FlagDependencies {
                identity_key,
                segment_keys_by_feature_name: OnceCell::new(),
                segment_matches: RefCell::new(HashMap::new()),
                flag_paths: RefCell::new(HashMap::new()),
                resolving: RefCell::new(Vec::new()),
                resolved: RefCell::new(HashMap::new()),
                cyclic: RefCell::new(HashSet::new()),
                cycle_hits: Cell::new(0),
                used: Cell::new(false),
            }),
            ..Self::new(ec)
        }
    }

    /// Whether the context belongs to a segment; one matching, or not, on a flag in a dependency cycle does not
    pub(crate) fn matches_segment_key(&self, segment_key: &'a str) -> bool {
        match &self.flag_dependencies {
            Some(dependencies) => self.matches_segment(dependencies, segment_key),
            None => self.is_context_in_segment(&self.ec.segments[segment_key]),
        }
    }

    pub(crate) fn into_resolved_flags(self) -> ResolvedFlags<'a> {
        self.flag_dependencies
            .map_or_else(ResolvedFlags::default, |dependencies| ResolvedFlags {
                flags: dependencies.resolved.into_inner(),
                cyclic: dependencies.cyclic.into_inner(),
            })
    }

    /// Determines if the evaluation context matches the segment rules
    pub(crate) fn is_context_in_segment(&self, segment: &SegmentContext) -> bool {
        if segment.rules.is_empty() {
            return false;
        }

        // All top-level rules must match
        segment
            .rules
            .iter()
            .all(|rule| context_matches_segment_rule(self, rule, &segment.key))
    }

    fn context_json(&self) -> Option<&serde_json::Value> {
        self.context_json
            .get_or_init(|| serde_json::to_value(self.ec).ok())
            .as_ref()
    }

    /// Gets a value from the context using JSONPath, caching the result per path
    fn get_value_from_jsonpath(&self, path: &str) -> Option<FlagsmithValue> {
        if let Some(value) = self.json_path_values.borrow().get(path) {
            return value.clone();
        }
        let value = self.resolve_jsonpath(path);
        self.json_path_values
            .borrow_mut()
            .insert(path.to_string(), value.clone());
        value
    }

    fn resolve_jsonpath(&self, path: &str) -> Option<FlagsmithValue> {
        // Parse the JSONPath expression
        let json_path = parse_json_path(path).ok()?;

        // Query the JSON using the path
        query_first_value(&json_path, self.context_json()?)
    }

    /// Gets a value by member names straight from the context, or `None` for members only JSONPath can read
    fn get_member_value(&self, names: &[&str]) -> Option<Option<FlagsmithValue>> {
        let string_value = |value: &String| FlagsmithValue {
            value: value.clone(),
            value_type: FlagsmithValueType::String,
        };
        let identity = self.ec.identity.as_ref();
        Some(match names {
            ["environment", "key"] => Some(string_value(&self.ec.environment.key)),
            ["environment", "name"] => Some(string_value(&self.ec.environment.name)),
            ["identity", "identifier"] => identity.map(|i| string_value(&i.identifier)),
            ["identity", "key"] => identity.map(|i| string_value(&i.key)),
            ["identity", "traits", trait_key] => identity
                .and_then(|i| i.traits.get(*trait_key))
                .filter(|value| value.value_type != FlagsmithValueType::None)
                .cloned(),
            ["flags", feature_name, field @ ("enabled" | "value" | "variant")] => {
                let dependencies = self.flag_dependencies.as_ref()?;
                let Some((feature_name, _)) = self.ec.features.get_key_value(*feature_name) else {
                    return Some(None);
                };
                self.read_flag(dependencies, feature_name, |flag| match *field {
                    "enabled" => Some(FlagsmithValue {
                        value: flag.enabled.to_string(),
                        value_type: FlagsmithValueType::Bool,
                    }),
                    "value" => Some(flag.value.clone())
                        .filter(|value| value.value_type != FlagsmithValueType::None),
                    _ => flag.variant.as_ref().map(string_value),
                })
                .flatten()
            }
            _ => return None,
        })
    }

    /// Gets a value from the result of the feature a `$.flags` JSONPath names, resolving it on first read
    fn get_value_from_flags(
        &self,
        dependencies: &FlagDependencies<'a>,
        path: &str,
    ) -> Option<FlagsmithValue> {
        let Some(flag_path) = self.get_flag_path(dependencies, path) else {
            return self.get_value_from_jsonpath(path); // Reads no flag, as the context holds none
        };
        let (feature_name, _) = self.ec.features.get_key_value(&flag_path.feature_name)?;
        let flag = self
            .read_flag(dependencies, feature_name, |flag| {
                serde_json::to_value(flag)
            })?
            .ok()?;
        let flags = serde_json::Map::from_iter([(feature_name.clone(), flag)]);
        let flags_json = serde_json::Value::Object(serde_json::Map::from_iter([(
            "flags".to_string(),
            serde_json::Value::Object(flags),
        )]));
        query_first_value(&flag_path.json_path, &flags_json)
    }

    fn get_flag_path(
        &self,
        dependencies: &FlagDependencies<'a>,
        path: &str,
    ) -> Option<Rc<FlagPath>> {
        if let Some(flag_path) = dependencies.flag_paths.borrow().get(path) {
            return flag_path.clone();
        }
        let flag_path = parse_flag_path(path).map(Rc::new);
        dependencies
            .flag_paths
            .borrow_mut()
            .insert(path.to_string(), flag_path.clone());
        flag_path
    }

    fn read_flag<T>(
        &self,
        dependencies: &FlagDependencies<'a>,
        feature_name: &'a str,
        read: impl FnOnce(&FlagResult) -> T,
    ) -> Option<T> {
        dependencies.used.set(true);
        if !dependencies.resolved.borrow().contains_key(feature_name) {
            self.resolve_feature(dependencies, feature_name);
        }
        dependencies.resolved.borrow().get(feature_name).map(read)
    }

    /// Resolves a flag, unless it depends on itself, or is absent from the context
    fn resolve_feature(&self, dependencies: &FlagDependencies<'a>, feature_name: &'a str) {
        let cycle_start = dependencies
            .resolving
            .borrow()
            .iter()
            .position(|name| *name == feature_name);
        if let Some(cycle_start) = cycle_start {
            let resolving = dependencies.resolving.borrow();
            dependencies
                .cyclic
                .borrow_mut()
                .extend(&resolving[cycle_start..]);
            dependencies
                .cycle_hits
                .set(dependencies.cycle_hits.get() + 1);
            return;
        }

        let Some(feature_context) = self.ec.features.get(feature_name) else {
            return;
        };

        dependencies.resolving.borrow_mut().push(feature_name);
        let segment_override = self.get_segment_override(dependencies, feature_name);
        dependencies.resolving.borrow_mut().pop();

        if dependencies.cyclic.borrow().contains(feature_name) {
            return; // Not a result conditions may match on
        }

        let flag_result = match segment_override {
            Some(segment_override) => get_flag_result_from_feature_context(
                segment_override.feature_context,
                dependencies.identity_key,
                format!("TARGETING_MATCH; segment={}", segment_override.segment_name),
            ),
            None => get_flag_result_from_feature_context(
                feature_context,
                dependencies.identity_key,
                get_fallback_reason(
                    self.get_segment_keys_by_feature_name(dependencies)
                        .contains_key(feature_name),
                ),
            ),
        };
        dependencies
            .resolved
            .borrow_mut()
            .insert(feature_name, flag_result);
    }

    fn get_segment_override(
        &self,
        dependencies: &FlagDependencies<'a>,
        feature_name: &str,
    ) -> Option<SegmentOverride<'a>> {
        let mut segment_override: Option<SegmentOverride<'a>> = None;

        let segment_keys = self
            .get_segment_keys_by_feature_name(dependencies)
            .get(feature_name)
            .map(Vec::as_slice)
            .unwrap_or_default();
        for &segment_key in segment_keys {
            if !self.matches_segment(dependencies, segment_key) {
                continue;
            }
            let segment_context = &self.ec.segments[segment_key];
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

    fn matches_segment(&self, dependencies: &FlagDependencies<'a>, segment_key: &'a str) -> bool {
        if let Some(&matches) = dependencies.segment_matches.borrow().get(segment_key) {
            return matches;
        }

        let cycle_hits = dependencies.cycle_hits.get();
        let matches = self.is_context_in_segment(&self.ec.segments[segment_key]);
        if dependencies.cycle_hits.get() != cycle_hits {
            return false; // The verdict rests on a flag left unresolved by a cycle
        }

        if dependencies.used.get() {
            dependencies
                .segment_matches
                .borrow_mut()
                .insert(segment_key, matches);
        }
        matches
    }

    fn get_segment_keys_by_feature_name<'d>(
        &self,
        dependencies: &'d FlagDependencies<'a>,
    ) -> &'d HashMap<&'a str, Vec<&'a str>> {
        dependencies.segment_keys_by_feature_name.get_or_init(|| {
            let mut segment_keys_by_feature_name: HashMap<&str, Vec<&str>> = HashMap::new();
            for (segment_key, segment_context) in &self.ec.segments {
                for override_fc in &segment_context.overrides {
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

/// Parses a JSONPath reading the result of a feature, i.e. one selecting a single feature by name right after `$.flags`
fn parse_flag_path(path: &str) -> Option<FlagPath> {
    if !path.starts_with("$.flags") {
        return None; // e.g. `$['flags']`, which may name a trait
    }
    let json_path = parse_json_path(path).ok()?;
    let [Segment::Selector(Selector::Name(flags)), Segment::Selector(Selector::Name(feature_name)), ..] =
        json_path.segments.as_slice()
    else {
        return None;
    };
    if flags != "flags" {
        return None;
    }
    Some(FlagPath {
        feature_name: feature_name.clone(),
        json_path,
    })
}

/// Gets the first value a JSONPath selects, if it is a scalar
fn query_first_value(json_path: &JpQuery, json: &serde_json::Value) -> Option<FlagsmithValue> {
    // Get the first match (if any)
    let value = js_path_process(json_path, json)
        .ok()?
        .into_iter()
        .next()?
        .val();

    // Convert to FlagsmithValue based on the JSON type
    match value {
        serde_json::Value::String(s) => Some(FlagsmithValue {
            value: s.clone(),
            value_type: crate::types::FlagsmithValueType::String,
        }),
        serde_json::Value::Number(n) => {
            if n.is_f64() {
                Some(FlagsmithValue {
                    value: n.to_string(),
                    value_type: crate::types::FlagsmithValueType::Float,
                })
            } else {
                Some(FlagsmithValue {
                    value: n.to_string(),
                    value_type: crate::types::FlagsmithValueType::Integer,
                })
            }
        }
        serde_json::Value::Bool(b) => Some(FlagsmithValue {
            value: b.to_string(),
            value_type: crate::types::FlagsmithValueType::Bool,
        }),
        _ => None,
    }
}

/// Determines if the given evaluation context matches the segment rules
///
/// When evaluating several segments against the same context, prefer a shared
/// [`SegmentEvaluator`] so context-derived work is not repeated per segment.
pub fn is_context_in_segment(ec: &EngineEvaluationContext, segment: &SegmentContext) -> bool {
    SegmentEvaluator::new(ec).is_context_in_segment(segment)
}

/// Checks if the context matches a segment rule
fn context_matches_segment_rule(
    evaluator: &SegmentEvaluator,
    rule: &SegmentRule,
    segment_key: &str,
) -> bool {
    // Check conditions if present
    if !rule.conditions.is_empty()
        && !matches_conditions_by_rule_type(
            evaluator,
            &rule.conditions,
            &rule.rule_type,
            segment_key,
        )
    {
        return false;
    }

    // Check nested rules using rule type
    if rule.rules.is_empty() {
        return true;
    }

    matches_sub_rules_by_rule_type(evaluator, &rule.rules, &rule.rule_type, segment_key)
}

fn matches_sub_rules_by_rule_type(
    evaluator: &SegmentEvaluator,
    sub_rules: &[SegmentRule],
    rule_type: &SegmentRuleType,
    segment_key: &str,
) -> bool {
    let mut results = sub_rules
        .iter()
        .map(|sr| context_matches_segment_rule(evaluator, sr, segment_key));
    match rule_type {
        SegmentRuleType::All => results.all(|r| r),
        SegmentRuleType::Any => results.any(|r| r),
        SegmentRuleType::None => !results.any(|r| r),
    }
}

/// Checks if conditions match according to the rule type
fn matches_conditions_by_rule_type(
    evaluator: &SegmentEvaluator,
    conditions: &[Condition],
    rule_type: &SegmentRuleType,
    segment_key: &str,
) -> bool {
    for condition in conditions {
        let condition_matches = context_matches_condition(evaluator, condition, segment_key);

        match rule_type {
            SegmentRuleType::All => {
                if !condition_matches {
                    return false; // Short-circuit: ALL requires all conditions to match
                }
            }
            SegmentRuleType::None => {
                if condition_matches {
                    return false; // Short-circuit: NONE requires no conditions to match
                }
            }
            SegmentRuleType::Any => {
                if condition_matches {
                    return true; // Short-circuit: ANY requires at least one condition to match
                }
            }
        }
    }

    // If we reach here: ALL/NONE passed all checks, ANY found no matches
    *rule_type != SegmentRuleType::Any
}

/// Checks if the context matches a specific condition
fn context_matches_condition(
    evaluator: &SegmentEvaluator,
    condition: &Condition,
    segment_key: &str,
) -> bool {
    let context_value = if !condition.property.is_empty() {
        get_context_value(evaluator, &condition.property)
    } else {
        None
    };

    match condition.operator {
        ConditionOperator::PercentageSplit => {
            match_percentage_split(evaluator.ec, condition, segment_key, context_value.as_ref())
        }
        ConditionOperator::In => match_in_operator(condition, context_value.as_ref()),
        ConditionOperator::IsNotSet => context_value.is_none(),
        ConditionOperator::IsSet => context_value.is_some(),
        _ => {
            if let Some(ref ctx_val) = context_value {
                parse_and_match(&condition.operator, ctx_val, &condition.value.as_string())
            } else {
                false
            }
        }
    }
}

/// Splits a JSONPath made of member names only, e.g. `$.identity.identifier`, into those names
fn get_member_names(path: &str) -> Option<Vec<&str>> {
    let names: Vec<&str> = path.strip_prefix("$.")?.split('.').collect();
    names
        .iter()
        .all(|name| {
            let mut chars = name.chars();
            chars
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
        .then_some(names)
}

/// Gets a value from the context by property name or JSONPath
fn get_context_value(evaluator: &SegmentEvaluator, property: &str) -> Option<FlagsmithValue> {
    // If property starts with $., try to parse it as a JSONPath expression
    if property.starts_with("$.") {
        let member_value =
            get_member_names(property).and_then(|names| evaluator.get_member_value(&names));
        let value = match (member_value, &evaluator.flag_dependencies) {
            (Some(value), _) => value,
            (None, Some(dependencies)) if property.starts_with("$.flags") => {
                evaluator.get_value_from_flags(dependencies, property)
            }
            (None, _) => evaluator.get_value_from_jsonpath(property),
        };
        if let Some(value) = value {
            return Some(value);
        }
        // If JSONPath parsing fails, fall through to treat it as a trait name
    }

    // Check traits by property name
    if let Some(ref identity) = evaluator.ec.identity {
        if let Some(trait_value) = identity.traits.get(property) {
            return Some(trait_value.clone());
        }
    }

    None
}

fn match_percentage_split(
    ec: &EngineEvaluationContext,
    condition: &Condition,
    segment_key: &str,
    context_value: Option<&FlagsmithValue>,
) -> bool {
    let float_value = match condition.value.as_string().parse::<f64>() {
        Ok(v) => v,
        Err(_) => return false,
    };

    let split_key: Option<String> = if condition.property.is_empty() {
        ec.identity.as_ref().map(|id| id.key.clone())
    } else {
        context_value.map(|v| v.value.clone())
    };

    let split_key = match split_key {
        Some(key) => key,
        None => return false,
    };

    let object_ids: Vec<&str> = vec![segment_key, &split_key];
    let hash_percentage = hashing::get_hashed_percentage_for_object_ids(object_ids, 1);
    (hash_percentage as f64) <= float_value
}

/// Matches IN operator
fn match_in_operator(condition: &Condition, context_value: Option<&FlagsmithValue>) -> bool {
    if context_value.is_none() {
        return false;
    }

    let ctx_value = context_value.unwrap();

    // IN operator only works with string values, not booleans
    use crate::types::FlagsmithValueType;
    if ctx_value.value_type == FlagsmithValueType::Bool {
        return false;
    }

    let trait_value = &ctx_value.value;

    // Use the ConditionValue's contains_string method for simple string matching
    condition.value.contains_string(trait_value)
}

/// Parses and matches values based on the operator using type-aware strategy
fn parse_and_match(
    operator: &ConditionOperator,
    trait_value: &FlagsmithValue,
    condition_value: &str,
) -> bool {
    use crate::types::FlagsmithValueType;

    // Handle special operators that work across all types
    match operator {
        ConditionOperator::Modulo => return evaluate_modulo(&trait_value.value, condition_value),
        ConditionOperator::Regex => return evaluate_regex(&trait_value.value, condition_value),
        ConditionOperator::Contains => return trait_value.value.contains(condition_value),
        ConditionOperator::NotContains => return !trait_value.value.contains(condition_value),
        _ => {}
    }

    // Use type-aware strategy based on trait value type
    match trait_value.value_type {
        FlagsmithValueType::Bool => compare_bool(operator, &trait_value.value, condition_value),
        FlagsmithValueType::Integer => {
            compare_integer(operator, &trait_value.value, condition_value)
        }
        FlagsmithValueType::Float => compare_float(operator, &trait_value.value, condition_value),
        FlagsmithValueType::String => compare_string(operator, &trait_value.value, condition_value),
        _ => false,
    }
}

/// Parses a boolean string value with optional integer conversion
/// NOTE: Historical engine behavior - only "1" is treated as true, "0" is NOT treated as false
fn parse_bool(s: &str, allow_int_conversion: bool) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" => Some(true),
        "1" if allow_int_conversion => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Compares boolean values
fn compare_bool(operator: &ConditionOperator, trait_value: &str, condition_value: &str) -> bool {
    if let (Some(b1), Some(b2)) = (
        parse_bool(trait_value, true),
        parse_bool(condition_value, true),
    ) {
        match operator {
            ConditionOperator::Equal => b1 == b2,
            ConditionOperator::NotEqual => b1 != b2,
            _ => false,
        }
    } else {
        false
    }
}

/// Compares integer values
fn compare_integer(operator: &ConditionOperator, trait_value: &str, condition_value: &str) -> bool {
    if let (Ok(i1), Ok(i2)) = (trait_value.parse::<i64>(), condition_value.parse::<i64>()) {
        dispatch_operator(operator, i1, i2)
    } else {
        false
    }
}

/// Compares float values
fn compare_float(operator: &ConditionOperator, trait_value: &str, condition_value: &str) -> bool {
    if let (Ok(f1), Ok(f2)) = (trait_value.parse::<f64>(), condition_value.parse::<f64>()) {
        dispatch_operator(operator, f1, f2)
    } else {
        false
    }
}

/// Compares string values, with special handling for semver
fn compare_string(operator: &ConditionOperator, trait_value: &str, condition_value: &str) -> bool {
    // Check for semver comparison
    if let Some(version_str) = condition_value.strip_suffix(":semver") {
        if let Ok(condition_version) = Version::parse(version_str) {
            return evaluate_semver(operator, trait_value, &condition_version);
        }
        return false;
    }

    // Try parsing as boolean for string types (strict - no integer conversion)
    if let (Some(b1), Some(b2)) = (
        parse_bool(trait_value, false),
        parse_bool(condition_value, false),
    ) {
        return match operator {
            ConditionOperator::Equal => b1 == b2,
            ConditionOperator::NotEqual => b1 != b2,
            _ => false,
        };
    }

    // Try parsing as integer
    if let (Ok(i1), Ok(i2)) = (trait_value.parse::<i64>(), condition_value.parse::<i64>()) {
        return dispatch_operator(operator, i1, i2);
    }

    // Try parsing as float
    if let (Ok(f1), Ok(f2)) = (trait_value.parse::<f64>(), condition_value.parse::<f64>()) {
        return dispatch_operator(operator, f1, f2);
    }

    // Fall back to string comparison
    dispatch_operator(operator, trait_value, condition_value)
}

/// Dispatches the operator to the appropriate comparison function
fn dispatch_operator<T: PartialOrd + PartialEq>(
    operator: &ConditionOperator,
    v1: T,
    v2: T,
) -> bool {
    match operator {
        ConditionOperator::Equal => v1 == v2,
        ConditionOperator::NotEqual => v1 != v2,
        ConditionOperator::GreaterThan => v1 > v2,
        ConditionOperator::LessThan => v1 < v2,
        ConditionOperator::GreaterThanInclusive => v1 >= v2,
        ConditionOperator::LessThanInclusive => v1 <= v2,
        _ => false,
    }
}

/// Evaluates regex matching
fn evaluate_regex(trait_value: &str, condition_value: &str) -> bool {
    if let Ok(re) = Regex::new(condition_value) {
        return re.is_match(trait_value);
    }
    false
}

/// Evaluates modulo operation
fn evaluate_modulo(trait_value: &str, condition_value: &str) -> bool {
    let values: Vec<&str> = condition_value.split('|').collect();
    if values.len() != 2 {
        return false;
    }

    let divisor = match values[0].parse::<f64>() {
        Ok(v) => v,
        Err(_) => return false,
    };

    let remainder = match values[1].parse::<f64>() {
        Ok(v) => v,
        Err(_) => return false,
    };

    let trait_value_float = match trait_value.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Use epsilon comparison for float equality to handle precision errors
    const EPSILON: f64 = 1e-10;
    ((trait_value_float % divisor) - remainder).abs() < EPSILON
}

/// Evaluates semantic version comparisons
fn evaluate_semver(
    operator: &ConditionOperator,
    trait_value: &str,
    condition_version: &Version,
) -> bool {
    let trait_version = match Version::parse(trait_value) {
        Ok(v) => v,
        Err(_) => return false,
    };

    match operator {
        ConditionOperator::Equal => trait_version == *condition_version,
        ConditionOperator::NotEqual => trait_version != *condition_version,
        ConditionOperator::GreaterThan => trait_version > *condition_version,
        ConditionOperator::LessThan => trait_version < *condition_version,
        ConditionOperator::GreaterThanInclusive => trait_version >= *condition_version,
        ConditionOperator::LessThanInclusive => trait_version <= *condition_version,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_evaluator_resolves_each_jsonpath_once() {
        // Given
        let ec: EngineEvaluationContext = serde_json::from_value(serde_json::json!({
            "environment": {"key": "env-key", "name": "Environment"},
            "identity": {"identifier": "user-1", "traits": {}},
        }))
        .unwrap();
        let segment = |identifiers: &[&str]| SegmentContext {
            key: String::new(),
            name: "identity_overrides".to_string(),
            metadata: Default::default(),
            overrides: vec![],
            rules: vec![SegmentRule {
                rule_type: SegmentRuleType::All,
                conditions: vec![Condition {
                    operator: ConditionOperator::In,
                    property: "$.identity['identifier']".to_string(),
                    value: super::super::context::ConditionValue::Multiple(
                        identifiers.iter().map(|i| i.to_string()).collect(),
                    ),
                }],
                rules: vec![],
            }],
        };
        let evaluator = SegmentEvaluator::new(&ec);

        // When
        let matches = [
            evaluator.is_context_in_segment(&segment(&["user-2"])),
            evaluator.is_context_in_segment(&segment(&["user-1", "user-3"])),
        ];

        // Then
        assert_eq!(matches, [false, true]);
        assert!(evaluator.context_json.get().is_some());
        let json_path_values = evaluator.json_path_values.borrow();
        assert_eq!(json_path_values.len(), 1);
        assert_eq!(
            json_path_values["$.identity['identifier']"]
                .as_ref()
                .map(|v| v.value.as_str()),
            Some("user-1")
        );
    }

    #[rstest::rstest]
    #[case("plan", "pro")]
    #[case("$.identity.traits.plan", "pro")]
    #[case("$.identity.identifier", "user-1")]
    #[case("$.identity.key", "env-key_user-1")]
    #[case("$.environment.name", "Environment")]
    #[case("$.environment.key", "env-key")]
    fn test_segment_evaluator_skips_serialisation_without_jsonpath(
        #[case] property: &str,
        #[case] value: &str,
    ) {
        // Given
        let ec: EngineEvaluationContext = serde_json::from_value(serde_json::json!({
            "environment": {"key": "env-key", "name": "Environment"},
            "identity": {"identifier": "user-1", "key": "env-key_user-1", "traits": {"plan": "pro"}},
        }))
        .unwrap();
        let segment = SegmentContext {
            key: String::new(),
            name: "pro".to_string(),
            metadata: Default::default(),
            overrides: vec![],
            rules: vec![SegmentRule {
                rule_type: SegmentRuleType::All,
                conditions: vec![Condition {
                    operator: ConditionOperator::Equal,
                    property: property.to_string(),
                    value: super::super::context::ConditionValue::Single(value.to_string()),
                }],
                rules: vec![],
            }],
        };
        let evaluator = SegmentEvaluator::new(&ec);

        // When
        let matches = evaluator.is_context_in_segment(&segment);

        // Then
        assert!(matches);
        assert!(evaluator.context_json.get().is_none());
    }

    #[test]
    fn test_dispatch_operator_integers() {
        assert!(dispatch_operator(&ConditionOperator::Equal, 5, 5));
        assert!(!dispatch_operator(&ConditionOperator::Equal, 5, 6));
        assert!(dispatch_operator(&ConditionOperator::GreaterThan, 6, 5));
        assert!(!dispatch_operator(&ConditionOperator::GreaterThan, 5, 6));
    }

    #[test]
    fn test_evaluate_modulo() {
        assert!(evaluate_modulo("2", "2|0"));
        assert!(!evaluate_modulo("3", "2|0"));
        assert!(evaluate_modulo("35.0", "4|3"));
    }
}

use super::context::{
    Condition, ConditionOperator, EngineEvaluationContext, SegmentContext, SegmentRule,
    SegmentRuleType,
};
use crate::types::FlagsmithValue;
use crate::utils::hashing;
use regex::Regex;
use semver::Version;
use serde_json_path::JsonPath;

/// Determines if the given evaluation context matches the segment rules
pub fn is_context_in_segment(ec: &EngineEvaluationContext, segment: &SegmentContext) -> bool {
    if segment.rules.is_empty() {
        return false;
    }

    // All top-level rules must match
    for rule in &segment.rules {
        if !context_matches_segment_rule(ec, rule, &segment.key) {
            return false;
        }
    }

    true
}

/// Checks if the context matches a segment rule
fn context_matches_segment_rule(
    ec: &EngineEvaluationContext,
    rule: &SegmentRule,
    segment_key: &str,
) -> bool {
    // Check conditions if present
    if !rule.conditions.is_empty() {
        if !matches_conditions_by_rule_type(ec, &rule.conditions, &rule.rule_type, segment_key) {
            return false;
        }
    }

    // Check nested rules
    for nested_rule in &rule.rules {
        if !context_matches_segment_rule(ec, nested_rule, segment_key) {
            return false;
        }
    }

    true
}

/// Checks if conditions match according to the rule type
fn matches_conditions_by_rule_type(
    ec: &EngineEvaluationContext,
    conditions: &[Condition],
    rule_type: &SegmentRuleType,
    segment_key: &str,
) -> bool {
    for condition in conditions {
        let condition_matches = context_matches_condition(ec, condition, segment_key);

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
    ec: &EngineEvaluationContext,
    condition: &Condition,
    segment_key: &str,
) -> bool {
    let context_value = if !condition.property.is_empty() {
        get_context_value(ec, &condition.property)
    } else {
        None
    };

    match condition.operator {
        ConditionOperator::PercentageSplit => {
            match_percentage_split(ec, condition, segment_key, context_value.as_ref())
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

/// Gets a value from the context by property name or JSONPath
fn get_context_value(ec: &EngineEvaluationContext, property: &str) -> Option<FlagsmithValue> {
    // If property starts with $., try to parse it as a JSONPath expression
    if property.starts_with("$.") {
        if let Some(value) = get_value_from_jsonpath(ec, property) {
            return Some(value);
        }
        // If JSONPath parsing fails, fall through to treat it as a trait name
    }

    // Check traits by property name
    if let Some(ref identity) = ec.identity {
        if let Some(trait_value) = identity.traits.get(property) {
            return Some(trait_value.clone());
        }
    }

    None
}

/// Gets a value from the context using JSONPath
fn get_value_from_jsonpath(ec: &EngineEvaluationContext, path: &str) -> Option<FlagsmithValue> {
    // Parse the JSONPath expression
    let json_path = match JsonPath::parse(path) {
        Ok(p) => p,
        Err(_) => return None,
    };

    // Serialize the context to JSON
    let context_json = match serde_json::to_value(ec) {
        Ok(v) => v,
        Err(_) => return None,
    };

    // Query the JSON using the path
    let result = json_path.query(&context_json);

    // Get the first match (if any)
    let node_list = result.all();
    if node_list.is_empty() {
        return None;
    }

    // Extract the value from the first match
    let value = node_list[0];

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

/// Matches percentage split condition
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

    // Build object IDs based on context
    let context_str = context_value.map(|v| v.value.clone());
    let object_ids: Vec<&str> = if let Some(ref ctx_str) = context_str {
        vec![segment_key, ctx_str.as_str()]
    } else if let Some(ref identity) = ec.identity {
        vec![segment_key, &identity.key]
    } else {
        return false;
    };

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
    if condition_value.ends_with(":semver") {
        let version_str = &condition_value[..condition_value.len() - 7];
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

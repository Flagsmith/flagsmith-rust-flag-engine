use super::features;
use super::types::{FeatureStateValue, FeatureStateValueType};
use regex::Regex;
use serde::{Deserialize, Serialize};
pub mod constants;
pub mod evaluator;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SegmentCondition {
    // Add operator method
    pub operator: String,
    pub value: String,
    #[serde(rename = "property_")]
    pub property: Option<String>,
}
impl SegmentCondition {
    pub fn matches_trait_value(&self, trait_value: &FeatureStateValue) -> bool {
        return match trait_value.value_type {
            FeatureStateValueType::Integer => {
                let trait_value: i64 = trait_value.value.parse().unwrap();
                let segment_condition_value: i64 = self.value.clone().parse().unwrap();

                self.number_operations(trait_value, segment_condition_value)
            }
            FeatureStateValueType::Float => {
                let trait_value: f64 = trait_value.value.parse().unwrap();
                let segment_condition_value: f64 = self.value.clone().parse().unwrap();
                self.number_operations(trait_value, segment_condition_value)
            }
            FeatureStateValueType::String => {
                self.string_operations(&trait_value.value, &self.value)
            }
            FeatureStateValueType::Bool => {
                let trait_value: bool = trait_value.value.parse().unwrap();
                let segment_condition_value: bool = self.value.clone().parse().unwrap();
                self.bool_operations(trait_value, segment_condition_value)
            }
            _ => false,
        };
    }
    fn string_operations(&self, trait_value: &str, segment_value: &str) -> bool {
        match self.operator.as_str() {
            constants::EQUAL => trait_value == segment_value,
            constants::NOT_EQUAL => trait_value != segment_value,
            constants::CONTAINS => trait_value.contains(&self.value),
            constants::NOT_CONTAINS => !trait_value.contains(&self.value),
            constants::REGEX => {
                let re = Regex::new(&self.value).unwrap();
                re.is_match(&trait_value)
            }
            _ => false,
        }
    }
    fn bool_operations(&self, trait_value: bool, segment_value: bool) -> bool {
        match self.operator.as_str() {
            constants::EQUAL => trait_value == segment_value,
            constants::NOT_EQUAL => trait_value != segment_value,
            _ => false,
        }
    }
    fn number_operations<T: PartialOrd + PartialEq>(
        &self,
        trait_value: T,
        segment_value: T,
    ) -> bool {
        match self.operator.as_str() {
            constants::EQUAL => trait_value == segment_value,
            constants::NOT_EQUAL => trait_value != segment_value,
            constants::GREATER_THAN => trait_value > segment_value, //#TODO : to a float maybe?
            constants::GREATER_THAN_INCLUSIVE => trait_value >= segment_value,
            constants::LESS_THAN => trait_value < segment_value,
            constants::LESS_THAN_INCLUSIVE => trait_value <= segment_value,
            _ => false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SegmentRule {
    pub r#type: String,
    pub rules: Vec<Box<SegmentRule>>,
    pub conditions: Vec<SegmentCondition>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Segment {
    pub id: u32,
    pub name: String,
    pub rules: Vec<SegmentRule>,
    pub feature_states: Vec<features::FeatureState>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_condition_matches_trait_value() {
        let test_cases = vec![
            (
                constants::EQUAL,
                "bar",
                FeatureStateValueType::String,
                "bar",
                true,
            ),
            (
                constants::EQUAL,
                "bar",
                FeatureStateValueType::String,
                "baz",
                false,
            ),
            (
                constants::EQUAL,
                "1",
                FeatureStateValueType::Integer,
                "1",
                true,
            ),
            (
                constants::EQUAL,
                "1",
                FeatureStateValueType::Integer,
                "2",
                false,
            ),
            (
                constants::EQUAL,
                "true",
                FeatureStateValueType::Bool,
                "true",
                true,
            ),
            (
                constants::EQUAL,
                "false",
                FeatureStateValueType::Bool,
                "false",
                true,
            ),
            (
                constants::EQUAL,
                "true",
                FeatureStateValueType::Bool,
                "false",
                false,
            ),
            (
                constants::EQUAL,
                "false",
                FeatureStateValueType::Bool,
                "true",
                false,
            ),
            (
                constants::EQUAL,
                "1.23",
                FeatureStateValueType::Float,
                "1.23",
                true,
            ),
            (
                constants::EQUAL,
                "1.23",
                FeatureStateValueType::Float,
                "1.25",
                false,
            ),
            (
                constants::GREATER_THAN,
                "2",
                FeatureStateValueType::Integer,
                "1",
                true,
            ),
            (
                constants::GREATER_THAN,
                "1",
                FeatureStateValueType::Integer,
                "2",
                false,
            ),
            (
                constants::GREATER_THAN,
                "1",
                FeatureStateValueType::Integer,
                "1",
                false,
            ),
            (
                constants::GREATER_THAN,
                "0",
                FeatureStateValueType::Integer,
                "1",
                false,
            ),
            (
                constants::GREATER_THAN,
                "2.1",
                FeatureStateValueType::Float,
                "2.0",
                true,
            ),
            (
                constants::GREATER_THAN,
                "2.1",
                FeatureStateValueType::Float,
                "2.2",
                false,
            ),
            (
                constants::GREATER_THAN,
                "2.0",
                FeatureStateValueType::Float,
                "2.1",
                false,
            ),
            (
                constants::GREATER_THAN,
                "2.0",
                FeatureStateValueType::Float,
                "2.1",
                false,
            ),
            (
                constants::GREATER_THAN,
                "2.0",
                FeatureStateValueType::Float,
                "2.0",
                false,
            ),
            (
                constants::GREATER_THAN_INCLUSIVE,
                "1",
                FeatureStateValueType::Integer,
                "1",
                true,
            ),
            (
                constants::GREATER_THAN_INCLUSIVE,
                "2",
                FeatureStateValueType::Integer,
                "1",
                true,
            ),
            (
                constants::GREATER_THAN_INCLUSIVE,
                "0",
                FeatureStateValueType::Integer,
                "1",
                false,
            ),
            (
                constants::GREATER_THAN_INCLUSIVE,
                "2.0",
                FeatureStateValueType::Float,
                "2.0",
                true,
            ),
            (
                constants::GREATER_THAN_INCLUSIVE,
                "2.1",
                FeatureStateValueType::Float,
                "2.0",
                true,
            ),
            (
                constants::GREATER_THAN_INCLUSIVE,
                "2.1",
                FeatureStateValueType::Float,
                "2.2",
                false,
            ),
            (
                constants::LESS_THAN,
                "2",
                FeatureStateValueType::Integer,
                "1",
                false,
            ),
            (
                constants::LESS_THAN,
                "1",
                FeatureStateValueType::Integer,
                "2",
                true,
            ),
            (
                constants::LESS_THAN,
                "1",
                FeatureStateValueType::Integer,
                "1",
                false,
            ),
            (
                constants::LESS_THAN,
                "0",
                FeatureStateValueType::Integer,
                "1",
                true,
            ),
            (
                constants::LESS_THAN,
                "2.1",
                FeatureStateValueType::Float,
                "2.0",
                false,
            ),
            (
                constants::LESS_THAN,
                "2.1",
                FeatureStateValueType::Float,
                "2.2",
                true,
            ),
            (
                constants::LESS_THAN,
                "2.0",
                FeatureStateValueType::Float,
                "2.1",
                true,
            ),
            (
                constants::LESS_THAN,
                "2.0",
                FeatureStateValueType::Float,
                "2.1",
                true,
            ),
            (
                constants::LESS_THAN,
                "2.0",
                FeatureStateValueType::Float,
                "2.0",
                false,
            ),
            (
                constants::LESS_THAN_INCLUSIVE,
                "1",
                FeatureStateValueType::Integer,
                "1",
                true,
            ),
            (
                constants::LESS_THAN_INCLUSIVE,
                "2",
                FeatureStateValueType::Integer,
                "1",
                false,
            ),
            (
                constants::LESS_THAN_INCLUSIVE,
                "1",
                FeatureStateValueType::Integer,
                "2",
                true,
            ),
            (
                constants::LESS_THAN_INCLUSIVE,
                "2.0",
                FeatureStateValueType::Float,
                "2.0",
                true,
            ),
            (
                constants::LESS_THAN_INCLUSIVE,
                "2.1",
                FeatureStateValueType::Float,
                "2.0",
                false,
            ),
            (
                constants::LESS_THAN_INCLUSIVE,
                "2.2",
                FeatureStateValueType::Float,
                "2.3",
                true,
            ),
            (
                constants::NOT_EQUAL,
                "bar",
                FeatureStateValueType::String,
                "bar",
                false,
            ),
            (
                constants::NOT_EQUAL,
                "bar",
                FeatureStateValueType::String,
                "baz",
                true,
            ),
            (
                constants::NOT_EQUAL,
                "1",
                FeatureStateValueType::Integer,
                "1",
                false,
            ),
            (
                constants::NOT_EQUAL,
                "1",
                FeatureStateValueType::Integer,
                "2",
                true,
            ),
            (
                constants::NOT_EQUAL,
                "true",
                FeatureStateValueType::Bool,
                "true",
                false,
            ),
            (
                constants::NOT_EQUAL,
                "false",
                FeatureStateValueType::Bool,
                "false",
                false,
            ),
            (
                constants::NOT_EQUAL,
                "true",
                FeatureStateValueType::Bool,
                "false",
                true,
            ),
            (
                constants::NOT_EQUAL,
                "false",
                FeatureStateValueType::Bool,
                "true",
                true,
            ),
            (
                constants::NOT_EQUAL,
                "1.23",
                FeatureStateValueType::Float,
                "1.23",
                false,
            ),
            (
                constants::NOT_EQUAL,
                "1.23",
                FeatureStateValueType::Float,
                "1.25",
                true,
            ),
            (
                constants::CONTAINS,
                "bar",
                FeatureStateValueType::String,
                "b",
                true,
            ),
            (
                constants::CONTAINS,
                "bar",
                FeatureStateValueType::String,
                "bar",
                true,
            ),
            (
                constants::CONTAINS,
                "bar",
                FeatureStateValueType::String,
                "baz",
                false,
            ),
            (
                constants::NOT_CONTAINS,
                "bar",
                FeatureStateValueType::String,
                "b",
                false,
            ),
            (
                constants::NOT_CONTAINS,
                "bar",
                FeatureStateValueType::String,
                "bar",
                false,
            ),
            (
                constants::NOT_CONTAINS,
                "bar",
                FeatureStateValueType::String,
                "baz",
                true,
            ),
            (
                constants::REGEX,
                "foo",
                FeatureStateValueType::String,
                r"[a-z]+",
                true,
            ),
            (
                constants::REGEX,
                "FOO",
                FeatureStateValueType::String,
                r"[a-z]+",
                false,
            ),
        ];
        fn check(
            operator: &str,
            trait_value: &str,
            trait_value_type: FeatureStateValueType,
            value: &str,
            result: bool,
        ) {
            let trait_value = FeatureStateValue {
                value: trait_value.to_string(),
                value_type: trait_value_type,
            };
            let segment_condition = SegmentCondition {
                operator: operator.to_string(),
                value: value.to_string(),
                property: Some("foo".to_string()),
            };
            assert_eq!(segment_condition.matches_trait_value(&trait_value), result)
        }
        for test_case in test_cases {
            check(
                test_case.0,
                test_case.1,
                test_case.2,
                test_case.3,
                test_case.4,
            );
        }
    }
}

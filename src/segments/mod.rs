use super::features;
use super::types::{FlagsmithValue, FlagsmithValueType};
use regex::Regex;
use serde::{Deserialize, Serialize};
pub mod constants;
pub mod evaluator;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SegmentCondition {
    pub operator: String,
    pub value: String,
    #[serde(rename = "property_")]
    pub property: Option<String>,
}

impl SegmentCondition {
    pub fn matches_trait_value(&self, trait_value: &FlagsmithValue) -> bool {
        return match trait_value.value_type {
            FlagsmithValueType::Integer => {
                let trait_value: i64 = trait_value.value.parse().unwrap();
                let segment_condition_value: i64 = self.value.clone().parse().unwrap();

                self.number_operations(trait_value, segment_condition_value)
            }
            FlagsmithValueType::Float => {
                let trait_value: f64 = trait_value.value.parse().unwrap();
                let segment_condition_value: f64 = self.value.clone().parse().unwrap();
                self.number_operations(trait_value, segment_condition_value)
            }
            FlagsmithValueType::String => self.string_operations(&trait_value.value, &self.value),
            FlagsmithValueType::Bool => {
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
            constants::GREATER_THAN => trait_value > segment_value,
            constants::GREATER_THAN_INCLUSIVE => trait_value >= segment_value,
            constants::LESS_THAN => trait_value < segment_value,
            constants::LESS_THAN_INCLUSIVE => trait_value <= segment_value,
            _ => false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SegmentRule {
    #[serde(rename = "type")]
    pub segment_rule_type: String,
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
    use rstest::*;

    #[rstest]
    #[case(constants::EQUAL, "bar", FlagsmithValueType::String, "bar", true)]
    #[case(constants::EQUAL, "bar", FlagsmithValueType::String, "baz", false)]
    #[case(constants::EQUAL, "1", FlagsmithValueType::Integer, "1", true)]
    #[case(constants::EQUAL, "1", FlagsmithValueType::Integer, "2", false)]
    #[case(constants::EQUAL, "true", FlagsmithValueType::Bool, "true", true)]
    #[case(constants::EQUAL, "false", FlagsmithValueType::Bool, "false", true)]
    #[case(constants::EQUAL, "true", FlagsmithValueType::Bool, "false", false)]
    #[case(constants::EQUAL, "false", FlagsmithValueType::Bool, "true", false)]
    #[case(constants::EQUAL, "1.23", FlagsmithValueType::Float, "1.23", true)]
    #[case(constants::EQUAL, "1.23", FlagsmithValueType::Float, "1.25", false)]
    #[case(constants::GREATER_THAN, "2", FlagsmithValueType::Integer, "1", true)]
    #[case(constants::GREATER_THAN, "1", FlagsmithValueType::Integer, "2", false)]
    #[case(constants::GREATER_THAN, "1", FlagsmithValueType::Integer, "1", false)]
    #[case(constants::GREATER_THAN, "0", FlagsmithValueType::Integer, "1", false)]
    #[case(constants::GREATER_THAN, "2.1", FlagsmithValueType::Float, "2.0", true)]
    #[case(
        constants::GREATER_THAN,
        "2.1",
        FlagsmithValueType::Float,
        "2.2",
        false
    )]
    #[case(
        constants::GREATER_THAN,
        "2.0",
        FlagsmithValueType::Float,
        "2.1",
        false
    )]
    #[case(
        constants::GREATER_THAN,
        "2.0",
        FlagsmithValueType::Float,
        "2.1",
        false
    )]
    #[case(
        constants::GREATER_THAN,
        "2.0",
        FlagsmithValueType::Float,
        "2.0",
        false
    )]
    #[case(
        constants::GREATER_THAN_INCLUSIVE,
        "1",
        FlagsmithValueType::Integer,
        "1",
        true
    )]
    #[case(
        constants::GREATER_THAN_INCLUSIVE,
        "2",
        FlagsmithValueType::Integer,
        "1",
        true
    )]
    #[case(
        constants::GREATER_THAN_INCLUSIVE,
        "0",
        FlagsmithValueType::Integer,
        "1",
        false
    )]
    #[case(
        constants::GREATER_THAN_INCLUSIVE,
        "2.0",
        FlagsmithValueType::Float,
        "2.0",
        true
    )]
    #[case(
        constants::GREATER_THAN_INCLUSIVE,
        "2.1",
        FlagsmithValueType::Float,
        "2.0",
        true
    )]
    #[case(
        constants::GREATER_THAN_INCLUSIVE,
        "2.1",
        FlagsmithValueType::Float,
        "2.2",
        false
    )]
    #[case(constants::LESS_THAN, "2", FlagsmithValueType::Integer, "1", false)]
    #[case(constants::LESS_THAN, "1", FlagsmithValueType::Integer, "2", true)]
    #[case(constants::LESS_THAN, "1", FlagsmithValueType::Integer, "1", false)]
    #[case(constants::LESS_THAN, "0", FlagsmithValueType::Integer, "1", true)]
    #[case(constants::LESS_THAN, "2.1", FlagsmithValueType::Float, "2.0", false)]
    #[case(constants::LESS_THAN, "2.1", FlagsmithValueType::Float, "2.2", true)]
    #[case(constants::LESS_THAN, "2.0", FlagsmithValueType::Float, "2.1", true)]
    #[case(constants::LESS_THAN, "2.0", FlagsmithValueType::Float, "2.1", true)]
    #[case(constants::LESS_THAN, "2.0", FlagsmithValueType::Float, "2.0", false)]
    #[case(
        constants::LESS_THAN_INCLUSIVE,
        "1",
        FlagsmithValueType::Integer,
        "1",
        true
    )]
    #[case(
        constants::LESS_THAN_INCLUSIVE,
        "2",
        FlagsmithValueType::Integer,
        "1",
        false
    )]
    #[case(
        constants::LESS_THAN_INCLUSIVE,
        "1",
        FlagsmithValueType::Integer,
        "2",
        true
    )]
    #[case(
        constants::LESS_THAN_INCLUSIVE,
        "2.0",
        FlagsmithValueType::Float,
        "2.0",
        true
    )]
    #[case(
        constants::LESS_THAN_INCLUSIVE,
        "2.1",
        FlagsmithValueType::Float,
        "2.0",
        false
    )]
    #[case(
        constants::LESS_THAN_INCLUSIVE,
        "2.2",
        FlagsmithValueType::Float,
        "2.3",
        true
    )]
    #[case(constants::NOT_EQUAL, "bar", FlagsmithValueType::String, "bar", false)]
    #[case(constants::NOT_EQUAL, "bar", FlagsmithValueType::String, "baz", true)]
    #[case(constants::NOT_EQUAL, "1", FlagsmithValueType::Integer, "1", false)]
    #[case(constants::NOT_EQUAL, "1", FlagsmithValueType::Integer, "2", true)]
    #[case(constants::NOT_EQUAL, "true", FlagsmithValueType::Bool, "true", false)]
    #[case(
        constants::NOT_EQUAL,
        "false",
        FlagsmithValueType::Bool,
        "false",
        false
    )]
    #[case(constants::NOT_EQUAL, "true", FlagsmithValueType::Bool, "false", true)]
    #[case(constants::NOT_EQUAL, "false", FlagsmithValueType::Bool, "true", true)]
    #[case(constants::NOT_EQUAL, "1.23", FlagsmithValueType::Float, "1.23", false)]
    #[case(constants::NOT_EQUAL, "1.23", FlagsmithValueType::Float, "1.25", true)]
    #[case(constants::CONTAINS, "bar", FlagsmithValueType::String, "b", true)]
    #[case(constants::CONTAINS, "bar", FlagsmithValueType::String, "bar", true)]
    #[case(constants::CONTAINS, "bar", FlagsmithValueType::String, "baz", false)]
    #[case(constants::NOT_CONTAINS, "bar", FlagsmithValueType::String, "b", false)]
    #[case(
        constants::NOT_CONTAINS,
        "bar",
        FlagsmithValueType::String,
        "bar",
        false
    )]
    #[case(
        constants::NOT_CONTAINS,
        "bar",
        FlagsmithValueType::String,
        "baz",
        true
    )]
    #[case(constants::REGEX, "foo", FlagsmithValueType::String, r"[a-z]+", true)]
    #[case(constants::REGEX, "FOO", FlagsmithValueType::String, r"[a-z]+", false)]
    fn segemnt_condition_matches_trait_value(
        #[case] operator: &str,
        #[case] trait_value: &str,
        #[case] trait_value_type: FlagsmithValueType,
        #[case] value: &str,
        #[case] result: bool,
    ) {
        let trait_value = FlagsmithValue {
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
}

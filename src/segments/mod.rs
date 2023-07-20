use super::features;
use super::types::{FlagsmithValue, FlagsmithValueType};
use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};
pub mod constants;
pub mod evaluator;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SegmentCondition {
    pub operator: String,
    pub value: Option<String>,
    #[serde(rename = "property_")]
    pub property: Option<String>,
}

impl SegmentCondition {
    pub fn matches_trait_value(&self, trait_value: &FlagsmithValue) -> bool {
        if self.operator.as_str() == constants::MODULO {
            return self.modulo_operations(trait_value, &self.value.as_ref().unwrap());
        }
        if self.operator.as_str() == constants::IN {
            return match trait_value.value_type {
                FlagsmithValueType::String => {
                    self.in_operations(&trait_value.value, &self.value.as_ref().unwrap())
                }
                FlagsmithValueType::Integer => {
                    let trait_value: String = trait_value.value.to_string();
                    self.in_operations(&trait_value, &self.value.as_ref().unwrap())
                }
                _ => false,
            };
        }
        return match trait_value.value_type {
            FlagsmithValueType::Integer => {
                let trait_value: i64 = trait_value.value.parse().unwrap();
                let segment_condition_value: i64 = self.value.as_ref().unwrap().parse().unwrap();

                self.number_operations(trait_value, segment_condition_value)
            }
            FlagsmithValueType::Float => {
                let trait_value: f64 = trait_value.value.parse().unwrap();
                let segment_condition_value: f64 = self.value.as_ref().unwrap().parse().unwrap();
                self.number_operations(trait_value, segment_condition_value)
            }
            FlagsmithValueType::String if self.value.as_ref().unwrap().ends_with(":semver") => {
                let trait_value = Version::parse(&trait_value.value).unwrap();
                let segment_condition_value = Version::parse(
                    &self.value.as_ref().unwrap()[..self.value.as_ref().unwrap().len() - 7],
                )
                .unwrap();
                self.semver_operations(trait_value, segment_condition_value)
            }
            FlagsmithValueType::String => {
                self.string_operations(&trait_value.value, &self.value.as_ref().unwrap())
            }
            FlagsmithValueType::Bool => {
                let trait_value: bool = trait_value.value.parse().unwrap();
                let segment_condition_value: bool = self.value.clone().unwrap().parse().unwrap();
                self.bool_operations(trait_value, segment_condition_value)
            }
            _ => false,
        };
    }
    fn string_operations(&self, trait_value: &str, segment_value: &str) -> bool {
        match self.operator.as_str() {
            constants::EQUAL => trait_value == segment_value,
            constants::NOT_EQUAL => trait_value != segment_value,
            constants::CONTAINS => trait_value.contains(segment_value),
            constants::NOT_CONTAINS => !trait_value.contains(segment_value),
            constants::REGEX => {
                let re = Regex::new(segment_value).unwrap();
                re.is_match(&trait_value)
            }
            _ => false,
        }
    }
    fn modulo_operations(&self, trait_value: &FlagsmithValue, segment_value: &str) -> bool {
        let values: Vec<&str> = segment_value.split("|").collect();
        if values.len() != 2 {
            return false;
        }
        let divisor: f64 = match values[0].parse() {
            Ok(v) => v,
            Err(_) => return false,
        };
        let remainder: f64 = match values[1].parse() {
            Ok(v) => v,
            Err(_) => return false,
        };

        let trait_value: f64 = match trait_value.value.parse() {
            Ok(v) => v,
            Err(_) => return false,
        };
        return (trait_value % divisor) == remainder;
    }
    fn semver_operations(&self, trait_value: Version, segment_value: Version) -> bool {
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
    fn in_operations(&self, trait_value: &str, segment_value: &str) -> bool {
        segment_value.split(',').any(|x| x == trait_value)
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
    #[case(constants::IN, "foo", FlagsmithValueType::String, "", false)]
    #[case(constants::IN, "foo", FlagsmithValueType::String, "foo,bar", true)]
    #[case(constants::IN, "bar", FlagsmithValueType::String, "foo,bar", true)]
    #[case(constants::IN, "ba", FlagsmithValueType::String, "foo,bar", false)]
    #[case(constants::IN, "foo", FlagsmithValueType::String, "foo", true)]
    #[case(constants::IN, "1", FlagsmithValueType::Integer, "1,2,3,4", true)]
    #[case(constants::IN, "1", FlagsmithValueType::Integer, "", false)]
    #[case(constants::IN, "1", FlagsmithValueType::Integer, "1", true)]
    // Flagsmith's engine does not evaluate `IN` condition for floats/doubles and booleans
    // due to ambiguous serialization across supported platforms.
    #[case(constants::IN, "1.5", FlagsmithValueType::Float, "1.5", false)]
    #[case(constants::IN, "false", FlagsmithValueType::Bool, "false", false)]
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
            value: Some(value.to_string()),
            property: Some("foo".to_string()),
        };
        assert_eq!(segment_condition.matches_trait_value(&trait_value), result)
    }

    #[rstest]
    #[case(
        constants::EQUAL,
        "1.0.0",
        FlagsmithValueType::String,
        "1.0.0:semver",
        true
    )]
    #[case(
        constants::EQUAL,
        "1.0.0",
        FlagsmithValueType::String,
        "1.0.1:semver",
        false
    )]
    #[case(
        constants::NOT_EQUAL,
        "1.0.0",
        FlagsmithValueType::String,
        "1.0.1:semver",
        true
    )]
    #[case(
        constants::NOT_EQUAL,
        "1.0.0",
        FlagsmithValueType::String,
        "1.0.0:semver",
        false
    )]
    #[case(
        constants::GREATER_THAN,
        "1.0.1",
        FlagsmithValueType::String,
        "1.0.0:semver",
        true
    )]
    #[case(
        constants::GREATER_THAN,
        "1.0.1",
        FlagsmithValueType::String,
        "1.0.1:semver",
        false
    )]
    #[case(
        constants::GREATER_THAN,
        "1.0.0",
        FlagsmithValueType::String,
        "1.0.0-beta:semver",
        true
    )]
    #[case(
        constants::GREATER_THAN,
        "1.0.1",
        FlagsmithValueType::String,
        "1.0.2-beta:semver",
        false
    )]
    #[case(
        constants::GREATER_THAN,
        "1.2.3",
        FlagsmithValueType::String,
        "1.2.3-pre.2+build.4:semver",
        true
    )]
    #[case(
        constants::LESS_THAN,
        "1.0.1",
        FlagsmithValueType::String,
        "1.0.2:semver",
        true
    )]
    #[case(
        constants::LESS_THAN,
        "1.0.1",
        FlagsmithValueType::String,
        "1.0.1:semver",
        false
    )]
    #[case(
        constants::LESS_THAN,
        "1.0.2",
        FlagsmithValueType::String,
        "1.0.1:semver",
        false
    )]
    #[case(
        constants::LESS_THAN,
        "1.0.0-rc.2",
        FlagsmithValueType::String,
        "1.0.0-rc.3:semver",
        true
    )]
    #[case(
        constants::GREATER_THAN_INCLUSIVE,
        "1.0.1",
        FlagsmithValueType::String,
        "1.0.0:semver",
        true
    )]
    #[case(
        constants::GREATER_THAN_INCLUSIVE,
        "1.0.1",
        FlagsmithValueType::String,
        "1.0.1:semver",
        true
    )]
    #[case(
        constants::GREATER_THAN_INCLUSIVE,
        "1.0.1",
        FlagsmithValueType::String,
        "1.0.2-beta:semver",
        false
    )]
    #[case(
        constants::GREATER_THAN_INCLUSIVE,
        "1.2.3",
        FlagsmithValueType::String,
        "1.2.3-pre.2+build.4:semver",
        true
    )]
    #[case(
        constants::LESS_THAN_INCLUSIVE,
        "1.0.1",
        FlagsmithValueType::String,
        "1.0.2:semver",
        true
    )]
    #[case(
        constants::LESS_THAN_INCLUSIVE,
        "1.0.1",
        FlagsmithValueType::String,
        "1.0.1:semver",
        true
    )]
    #[case(
        constants::LESS_THAN_INCLUSIVE,
        "1.0.2",
        FlagsmithValueType::String,
        "1.0.1:semver",
        false
    )]
    #[case(
        constants::LESS_THAN_INCLUSIVE,
        "1.0.0-rc.2",
        FlagsmithValueType::String,
        "1.0.0-rc.3:semver",
        true
    )]

    fn segemnt_condition_matches_trait_value_semver(
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
            value: Some(value.to_string()),
            property: Some("foo".to_string()),
        };
        assert_eq!(segment_condition.matches_trait_value(&trait_value), result)
    }

    #[rstest]
    #[case("1", FlagsmithValueType::Integer, "2|0", false)]
    #[case("1.1", FlagsmithValueType::Float, "2.1|1.1", true)]
    #[case("2", FlagsmithValueType::Integer, "2|0", true)]
    #[case("3", FlagsmithValueType::Integer, "2|0", false)]
    #[case("34.2", FlagsmithValueType::Float, "4|3", false)]
    #[case("35.0", FlagsmithValueType::Float, "4|3", true)]
    #[case("bar", FlagsmithValueType::String, "3|0", false)]
    #[case("1.0.0", FlagsmithValueType::String, "3|0", false)]
    #[case("false", FlagsmithValueType::Bool, "1|3", false)]
    fn segment_condition_matches_trait_value_modulo(
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
            operator: constants::MODULO.to_string(),
            value: Some(value.to_string()),
            property: Some("foo".to_string()),
        };
        assert_eq!(segment_condition.matches_trait_value(&trait_value), result)
    }
}

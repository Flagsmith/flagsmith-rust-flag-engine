use super::features;
use regex::Regex;
use serde::{Deserialize, Serialize};
mod constants;
pub mod evaluator;

#[derive(Clone, Serialize, Deserialize)]
pub struct SegmentCondition {
    // Add operator method
    pub operator: String,
    pub value: String,
    pub property: Option<String>,
}

impl SegmentCondition {
    pub fn matches_trait_value(&self, trait_value: String) -> bool {
        match self.operator.as_str() {
            constants::EQUAL => trait_value == self.value,
            constants::GREATER_THAN => trait_value > self.value, //#TODO : to a float maybe?
            constants::GREATER_THAN_INCLUSIVE => trait_value >= self.value,
            constants::LESS_THAN => trait_value < self.value,
            constants::LESS_THAN_INCLUSIVE => trait_value <= self.value,
            constants::NOT_EQUAL => trait_value != self.value,
            constants::CONTAINS => trait_value.contains(&self.value),
            constants::NOT_CONTAINS => trait_value.contains(&self.value),
            constants::REGEX => {
                let re = Regex::new(&self.value).unwrap();
                re.is_match(&trait_value)
            }
            _ => false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SegmentRule {
    pub r#type: String,
    pub rules: Vec<Box<SegmentRule>>,
    pub conditions: Vec<SegmentCondition>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: u32,
    pub name: String,
    pub rules: Vec<SegmentRule>,
    pub feature_states: Vec<features::FeatureState>,
}

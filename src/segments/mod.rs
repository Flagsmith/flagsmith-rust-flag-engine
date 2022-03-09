use serde::{Deserialize, Serialize};
use super::features;

#[derive(Serialize, Deserialize)]
pub struct SegmentCondition{
    // Add operator method
    pub operator: String,
    pub value: String,
    pub property: Option<String>

}

#[derive(Serialize, Deserialize)]
pub struct SegmentRule{
    pub r#type: String,
    pub rules: String,  // TODO: recursive
    pub conditions: Vec<SegmentCondition>
}

#[derive(Serialize, Deserialize)]
pub struct Segment{
    pub id: u32,
    pub name: String,
    pub rules: Vec<SegmentRule>,
    pub feature_states: Vec<features::FeatureState>

}

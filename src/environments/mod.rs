use serde::{Deserialize, Serialize};
use super::projects;
use super::features;

#[derive(Serialize, Deserialize)]
pub struct Environment{
    pub id: u32,
    pub api_key: String,
    pub project: projects::Project,
    pub feature_states: Vec<features::FeatureState>

}

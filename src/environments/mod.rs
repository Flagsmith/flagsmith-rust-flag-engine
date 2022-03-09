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

#[derive(Serialize, Deserialize)]
pub struct EnvironmentAPIKey{
    pub id: u32,
    pub key: String,
    pub created_at: String, // TODO datetime
    pub name: String,
    pub client_api_key: String,
    pub expires_at: Option<String>, // TODO: datetime
    pub active: Option<bool> // TODO: make default true
}

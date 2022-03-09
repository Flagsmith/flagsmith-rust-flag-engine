use serde::{Deserialize, Serialize};
use super::features;


#[derive(Serialize, Deserialize)]
pub struct Trait{
    trait_key:  String,
    trait_value: String //TODO: typing.Any
}

#[derive(Serialize, Deserialize)]
pub struct Identity{
    pub identifier: String,
    pub environment_api_key: String,
    pub created_date: String, // TODO: change to datetime
    pub identity_features: Vec<features::FeatureState>,
    pub identity_traits: Vec<Trait>,
    pub identity_uuid: String, // TODO: Add default value
    pub django_id: u32

}

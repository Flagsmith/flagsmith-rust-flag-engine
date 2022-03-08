use std::u32;

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct Feature{
    id: u32,
    name:String,
    r#type: Option<String>,
}


#[derive(Serialize, Deserialize)]
pub struct MultivariateFeatureOption{
    pub value: String, // typing Any
    pub id: u32 // default None
}


#[derive(Serialize, Deserialize)]
pub struct MultivariateFeatureStateValue{
    pub multivariate_feature_option: MultivariateFeatureOption,
    pub percentage_allocation: f32,
    pub id: u32, // Default None,
    pub mv_fs_value_uuid: String

}


#[derive(Serialize, Deserialize)]
pub struct FeatureState{
    pub feature: Feature,
    pub enabled: bool,
    pub django_id: Option<u32>,
    pub featurestate_uuid: Option<u32>, // Make this uuid by default
    pub multivariate_feature_state_values: Vec<MultivariateFeatureStateValue>,
    _value: Option<String> // typing. any

}

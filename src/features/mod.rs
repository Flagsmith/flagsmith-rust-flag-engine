use std::{ptr::hash, string, u32};
use super::utils;
use super::utils::hashing;
use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Clone)]
pub struct Feature{
    pub id: u32,
    pub name:String,
    r#type: Option<String>,
}



#[derive(Serialize, Deserialize, Clone)]
pub struct MultivariateFeatureOption{
    pub value: String, // typing Any
    pub id: Option<u32>
}



#[derive(Serialize, Deserialize, Clone)]
pub struct MultivariateFeatureStateValue{
    pub multivariate_feature_option: MultivariateFeatureOption,
    pub percentage_allocation: f32,
    pub id: Option<u32>, // Default None,
    pub mv_fs_value_uuid: String

}


#[derive(Serialize, Deserialize, Clone)]
pub struct FeatureState{
    pub feature: Feature,
    pub enabled: bool,
    pub django_id: Option<u32>,

    #[serde(default = "utils::get_uuid")]
    pub featurestate_uuid: String, // Make this uuid by default
    pub multivariate_feature_state_values: Vec<MultivariateFeatureStateValue>,
    _value: Option<String> // typing. any

}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn deserializing_fs_creates_default_uuid_if_not_present(){
        let feature_state_json =  r#"{
            "multivariate_feature_state_values": [],
            "feature_state_value": null,
            "django_id": 1,
            "feature": {
                "name": "feature1",
                "type": null,
                "id": 1
            },
            "segment_id": null,
            "enabled": false
        }"#;

        let feature_state: FeatureState = serde_json::from_str(feature_state_json).unwrap();
        assert_eq!(feature_state.featurestate_uuid.is_empty(), false)
    }


}
// impl  FeatureState {
//     pub fn set_value(& mut self, value: String) {
//         self._value = Some(value);
//     }
//     pub fn get_value(&self, identity_id: &str) {
//     }
//     fn get_multivariate_value(&self, identity_id: &str) {
//         match self.django_id {
//             Some(django_id) => django_id.to_string(),
//             None => self.featurestate_uuid
//         }
//         let percentage_value = hashing::get_hashed_percentage_for_object_ids(iterations)
//     }
// }

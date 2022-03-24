use std::{ptr::hash, string, u32};
use super::utils;
use super::utils::hashing;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq)]
#[derive(Hash)]
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
    pub id: Option<u32>,

    #[serde(default = "utils::get_uuid")]
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
    #[serde(alias = "feature_state_value")]
    value: Option<String> // TODO: typing. any

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
impl  FeatureState {
    //TODO: fix type
    pub fn get_value(&self, identity_id: Option<&str>) -> Option<String>{
        let value = match identity_id {
            Some(id) if self.multivariate_feature_state_values.len()>0 => Some(self.get_multivariate_value(id)),
            _ => self.value.clone()
        };
        return value;

    }
    fn get_multivariate_value(&self, identity_id: &str) -> String {
        // TODO: make return type generic
        let object_id = match self.django_id {
            Some(django_id) => django_id.to_string(),
            None => self.featurestate_uuid.clone()
        };
        let percentage_value = hashing::get_hashed_percentage_for_object_ids(vec![&object_id, identity_id], 1);
        let mut start_percentage = 0.0;
        // Iterate over the mv options in order of id (so we get the same value each
        // time) to determine the correct value to return to the identity based on
        // the percentage allocations of the multivariate options. This gives us a
        // way to ensure that the same value is returned every time we use the same
        // percentage value.
        let mut mv_fs_values = self.multivariate_feature_state_values.clone();
        mv_fs_values.sort_by_key(|mv_fs_value| {match mv_fs_value.id{
            Some(id) => id.to_string(),
            _ => mv_fs_value.mv_fs_value_uuid.clone(),
        }});
        for mv_value in mv_fs_values{
            let limit = mv_value.percentage_allocation + start_percentage;
            if start_percentage <= percentage_value && percentage_value< limit{
                return mv_value.multivariate_feature_option.value
            }

            start_percentage = limit
        }
        // default to return the control value if no MV values found, although this
        // should never happen
        return self.value.as_ref().unwrap().clone()

    }
}

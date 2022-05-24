use super::utils;
use super::utils::hashing;
use serde::{Deserialize, Serialize};

use super::types::FlagsmithValue;

#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Clone, Debug)]
pub struct Feature {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    feature_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MultivariateFeatureOption {
    pub value: FlagsmithValue,
    pub id: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MultivariateFeatureStateValue {
    pub multivariate_feature_option: MultivariateFeatureOption,
    pub percentage_allocation: f32,
    pub id: Option<u32>,

    #[serde(default = "utils::get_uuid")]
    pub mv_fs_value_uuid: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FeatureState {
    pub feature: Feature,
    pub enabled: bool,
    pub django_id: Option<u32>,

    #[serde(default = "utils::get_uuid")]
    pub featurestate_uuid: String,
    pub multivariate_feature_state_values: Vec<MultivariateFeatureStateValue>,
    #[serde(rename = "feature_state_value")]
    value: FlagsmithValue,
}

impl FeatureState {
    pub fn get_value(&self, identity_id: Option<&str>) -> FlagsmithValue {
        let value = match identity_id {
            Some(id) if self.multivariate_feature_state_values.len() > 0 => {
                self.get_multivariate_value(id)
            }
            _ => self.value.clone(),
        };
        return value;
    }
    fn get_multivariate_value(&self, identity_id: &str) -> FlagsmithValue {
        let object_id = match self.django_id {
            Some(django_id) => django_id.to_string(),
            None => self.featurestate_uuid.clone(),
        };
        let percentage_value =
            hashing::get_hashed_percentage_for_object_ids(vec![&object_id, identity_id], 1);
        let mut start_percentage = 0.0;
        // Iterate over the mv options in order of id (so we get the same value each
        // time) to determine the correct value to return to the identity based on
        // the percentage allocations of the multivariate options. This gives us a
        // way to ensure that the same value is returned every time we use the same
        // percentage value.
        let mut mv_fs_values = self.multivariate_feature_state_values.clone();
        mv_fs_values.sort_by_key(|mv_fs_value| match mv_fs_value.id {
            Some(id) => id.to_string(),
            _ => mv_fs_value.mv_fs_value_uuid.clone(),
        });
        for mv_value in mv_fs_values {
            let limit = mv_value.percentage_allocation + start_percentage;
            if start_percentage <= percentage_value && percentage_value < limit {
                return mv_value.multivariate_feature_option.value;
            }

            start_percentage = limit
        }
        // default to return the control value if no MV values found, although this
        // should never happen
        return self.value.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[test]
    fn deserializing_fs_creates_default_uuid_if_not_present() {
        let feature_state_json = r#"{
            "multivariate_feature_state_values": [
        {
            "id": 3404,
            "multivariate_feature_option": {
              "value": "baz"
            },
            "percentage_allocation": 30
          }
            ],
            "feature_state_value": 1,
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
        assert_eq!(feature_state.featurestate_uuid.is_empty(), false);
        assert_eq!(
            feature_state.multivariate_feature_state_values[0]
                .mv_fs_value_uuid
                .is_empty(),
            false
        );
    }

    #[test]
    fn serialize_and_deserialize_feature_state() {
        let feature_state_json = serde_json::json!(
            {
                "multivariate_feature_state_values": [],
                "feature_state_value": 1,
                "featurestate_uuid":"a6ff815f-63ed-4e72-99dc-9124c442ce4d",
                "django_id": 1,
                "feature": {
                    "name": "feature1",
                    "type": null,
                    "id": 1
                },
                "enabled": false
            }
        );
        let feature_state: FeatureState =
            serde_json::from_value(feature_state_json.clone()).unwrap();

        let given_json = serde_json::to_value(&feature_state).unwrap();
        assert_eq!(given_json, feature_state_json)
    }

    #[rstest]
    #[case("2", "foo".to_string())] // Generated hash percentage 26
    #[case("8", "bar".to_string())] // Generated hash percentage 38
    #[case("1", "control".to_string())] // Generated hash percentage 84
    fn feature_state_get_value_mv_values(
        #[case] identity_id: &str,
        #[case] expected_value: String,
    ) {
        let mv_feature_value_1 = "foo";
        let mv_feature_value_2 = "bar";
        let feature_state_json = serde_json::json!(
            {
                "multivariate_feature_state_values": [
                    {
                        "id": 1,
                        "multivariate_feature_option": {
                            "id":1,
                            "value": mv_feature_value_1
                        },
                        "percentage_allocation": 30
                    },
                    {
                        "id": 2,
                        "multivariate_feature_option": {
                            "id":2,
                            "value": mv_feature_value_2
                        },
                        "percentage_allocation": 30
                    }
                ],
                "feature_state_value": "control",
                "featurestate_uuid":"a6ff815f-63ed-4e72-99dc-9124c442ce4d",
                "django_id": 1,
                "feature": {
                    "name": "feature1",
                    "type": null,
                    "id": 1
                },
                "enabled": true
            }
        );
        let feature_state: FeatureState =
            serde_json::from_value(feature_state_json.clone()).unwrap();
        let value = feature_state.get_value(Some(identity_id));
        assert_eq!(value.value, expected_value);
    }

    #[rstest]
    fn feature_state_get_value_uses_django_id_for_multivariate_value_calculation_if_not_none() {
        let mv_feature_value_1 = "foo";
        let mv_feature_value_2 = "bar";
        let fs_uuid = "a6ff815f-63ed-4e72-99dc-9124c442ce4d";
        let feature_state_json = serde_json::json!(
            {
                "multivariate_feature_state_values": [
                    {
                        "id": 1,
                        "multivariate_feature_option": {
                            "id":1,
                            "value": mv_feature_value_1
                        },
                        "percentage_allocation": 30
                    },
                    {
                        "id": 2,
                        "multivariate_feature_option": {
                            "id":2,
                            "value": mv_feature_value_2
                        },
                        "percentage_allocation": 30
                    }
                ],
                "feature_state_value": "control",
                "featurestate_uuid":fs_uuid,
                "django_id": 1,
                "feature": {
                    "name": "feature1",
                    "type": null,
                    "id": 1
                },
                "enabled": true
            }
        );
        // When
        let feature_state: FeatureState =
            serde_json::from_value(feature_state_json.clone()).unwrap();

        // Then
        let value = feature_state.get_value(Some("1"));
        // Since hash percentage generated using fs_uuid and identity_id `13` is 9.7
        // and hash percentage generated using mv_fs django id `1` and identity_id `1` is 84
        // the value returned should be the control value if django id was used for the calculation
        assert_eq!(value.value, "control".to_string());
    }
}

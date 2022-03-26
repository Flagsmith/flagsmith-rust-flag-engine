use super::features;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub mod builders;
#[derive(Serialize, Deserialize)]
pub struct Trait {
    pub trait_key: String,
    pub trait_value: String, //TODO: typing.Any
}

#[derive(Serialize, Deserialize)]
pub struct Identity {
    pub identifier: String,
    pub environment_api_key: String,
    pub created_date: DateTime<Utc>,
    pub identity_features: Vec<features::FeatureState>,
    pub identity_traits: Vec<Trait>,
    pub identity_uuid: String, // TODO: Add default value
    pub django_id: Option<u32>,
}
impl Identity {
    pub fn composite_key(&self) -> String {
        return self.environment_api_key.clone() + "_" + &self.identifier;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_key_is_correct() {
        let expected_composite_key = "test_api_key_test_user";
        let identity_json = r#"{
            "identifier": "test_user",
            "environment_api_key": "test_api_key",
            "created_date": "2022-03-02T12:31:05.309861+00:00",
            "identity_features": [],
            "identity_traits": [],
            "identity_uuid":""
        }"#;

        let identity: Identity = serde_json::from_str(identity_json).unwrap();
        assert_eq!(identity.composite_key(), expected_composite_key)
    }
}

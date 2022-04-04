use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::features;
use super::projects;
pub mod builders;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Environment {
    pub id: u32,
    pub api_key: String,
    pub project: projects::Project,
    pub feature_states: Vec<features::FeatureState>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentAPIKey {
    pub id: u32,
    pub key: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub client_api_key: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl EnvironmentAPIKey {
    pub fn is_valid(&self) -> bool {
        if !self.active {
            return false;
        }
        match self.expires_at {
            None => return true,
            Some(expires_at) if expires_at > Utc::now() => return true,
            Some(_) => return false,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_is_valid_returns_if_active_and_expired_is_null() {
        let api_key_json = r#"{
            "key": "ser.test_key",
            "active": true,
            "created_at": "2022-03-02T12:31:05.309861+00:00",
            "client_api_key": "client_key",
            "id": 1,
            "name": "api key 1",
            "expires_at": null
        }"#;
        let api_key: EnvironmentAPIKey = serde_json::from_str(api_key_json).unwrap();

        assert!(api_key.is_valid())
    }

    #[test]
    fn api_key_is_valid_returns_false_if_active_is_false() {
        let api_key_json = r#"{
            "key": "ser.test_key",
            "active": false,
            "created_at": "2022-03-02T12:31:05.309861+00:00",
            "client_api_key": "client_key",
            "id": 1,
            "name": "api key 1",
            "expires_at": null
        }"#;
        let api_key: EnvironmentAPIKey = serde_json::from_str(api_key_json).unwrap();

        assert_eq!(api_key.is_valid(), false)
    }

    #[test]
    fn api_key_is_valid_returns_false_if_active_is_true_but_key_is_expired() {
        let api_key_json = r#"{
            "key": "ser.test_key",
            "active": true,
            "created_at": "2022-03-02T12:31:05.309861+00:00",
            "client_api_key": "client_key",
            "id": 1,
            "name": "api key 1",
            "expires_at": "2022-03-02T12:32:05.309861+00:00"
        }"#;
        let api_key: EnvironmentAPIKey = serde_json::from_str(api_key_json).unwrap();

        assert_eq!(api_key.is_valid(), false)
    }
}

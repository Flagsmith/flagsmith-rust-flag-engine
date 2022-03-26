use super::Environment;
use super::EnvironmentAPIKey;

pub fn build_environment_struct(value: serde_json::Value) -> Environment {
    let environment: Environment = serde_json::from_value(value).unwrap();
    return environment;
}

pub fn build_environment_api_key_struct(value: serde_json::Value) -> EnvironmentAPIKey {
    let environment_api_key: EnvironmentAPIKey = serde_json::from_value(value).unwrap();
    return environment_api_key;
}

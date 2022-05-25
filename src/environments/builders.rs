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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_environment_api_key_struct_returns_correct_struct() {
        // Given
        let key = "ser.test_key".to_string();
        let api_key_json = serde_json::json!({
            "key": key,
            "active": true,
            "created_at": "2022-03-02T12:31:05.309861+00:00",
            "client_api_key": "client_key",
            "id": 1,
            "name": "api key 1",
            "expires_at": null
        });
        // When
        let api_key_struct = build_environment_api_key_struct(api_key_json);
        // Then
        assert_eq!(api_key_struct.key, key);
    }
}

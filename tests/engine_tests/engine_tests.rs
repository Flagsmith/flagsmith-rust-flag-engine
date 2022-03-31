use core::panic;
use flagsmith_rust_flag_engine::engine;
use flagsmith_rust_flag_engine::environments;
use flagsmith_rust_flag_engine::environments::builders::build_environment_struct;
use flagsmith_rust_flag_engine::identities;
use flagsmith_rust_flag_engine::identities::builders::build_identity_struct;
use flagsmith_rust_flag_engine::types::{FlagsmithValue, FlagsmithValueType};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use rstest::*;

#[fixture]
fn test_json() -> serde_json::Value {
    // First, Let's convert the json file to serde value
    let file_path =
        "tests/engine_tests/engine-test-data/data/environment_n9fbf9h3v4fFgH3U3ngWhb.json";

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push(file_path);
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let value: serde_json::Value = serde_json::from_reader(reader).unwrap();
    return value;
}

#[rstest]
fn test_engine(test_json: serde_json::Value) {
    fn check(
        environment: &environments::Environment,
        identity: &identities::Identity,
        expected_response: serde_json::Value,
    ) {
        // Given
        let expected_flags = expected_response["flags"].as_array().unwrap();

        // When
        let mut engine_response =
            engine::get_identity_feature_states(&environment, &identity, None);
        // Sort the feature states so that we can iterate over it and compare them with expected response
        engine_response.sort_by_key(|fs| fs.feature.name.clone());

        // Then
        // engine returned right number of feature states
        assert_eq!(engine_response.len(), expected_flags.len());
        for (index, fs) in engine_response.iter().enumerate() {
            // and the values and enabled status of each of the feature states returned by the
            // engine is as expected
            assert_eq!(
                fs.enabled,
                expected_flags[index]["enabled"].as_bool().unwrap()
            );

            let identity_id = match identity.django_id {
                Some(id) => id.to_string(),
                None => identity.identity_uuid.clone(),
            };

            let fs_value = fs.get_value(Some(&identity_id));

            match fs_value.value_type {
                FlagsmithValueType::Bool => assert_eq!(
                    fs_value.value.parse::<bool>().unwrap(),
                    expected_flags[index]["feature_state_value"]
                        .as_bool()
                        .unwrap()
                ),
                FlagsmithValueType::Integer => assert_eq!(
                    fs_value.value.parse::<i64>().unwrap(),
                    expected_flags[index]["feature_state_value"]
                        .as_i64()
                        .unwrap()
                ),
                FlagsmithValueType::String => assert_eq!(
                    fs_value.value,
                    expected_flags[index]["feature_state_value"]
                        .as_str()
                        .unwrap()
                ),
                FlagsmithValueType::None => assert_eq!(
                    (),
                    expected_flags[index]["feature_state_value"]
                        .as_null()
                        .unwrap()
                ),
                FlagsmithValueType::Float => {
                    panic!("Floats are not allowed for feature state value")
                }
            }
        }
    }
    let environment = build_environment_struct(test_json["environment"].clone());

    for identity_and_response in test_json["identities_and_responses"].as_array().unwrap() {
        let identity = build_identity_struct(identity_and_response["identity"].clone());
        check(
            &environment,
            &identity,
            identity_and_response["response"].clone(),
        );
    }
}

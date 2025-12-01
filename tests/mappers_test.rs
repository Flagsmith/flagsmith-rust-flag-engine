use flagsmith_flag_engine::engine_eval::{
    context::{ConditionOperator, ConditionValue, SegmentRuleType, SegmentSource},
    environment_to_context,
};
use flagsmith_flag_engine::environments::Environment;

fn get_environment_fixture() -> &'static str {
    r#"{
  "api_key": "test_key",
  "name": "Test Environment",
  "project": {
    "name": "Test project",
    "organisation": {
      "feature_analytics": false,
      "name": "Test Org",
      "id": 1,
      "persist_trait_data": true,
      "stop_serving_flags": false
    },
    "id": 1,
    "hide_disabled_flags": false,
    "segments": [
      {
        "id": 1,
        "name": "Test segment",
        "rules": [
          {
            "type": "ALL",
            "rules": [
              {
                "type": "ALL",
                "rules": [],
                "conditions": [
                  {
                    "operator": "EQUAL",
                    "property_": "foo",
                    "value": "bar"
                  }
                ]
              }
            ],
            "conditions": []
          }
        ],
        "feature_states": []
      }
    ]
  },
  "segment_overrides": [],
  "id": 1,
  "feature_states": [
    {
      "multivariate_feature_state_values": [],
      "feature_state_value": "some-value",
      "id": 1,
      "featurestate_uuid": "00000000-0000-0000-0000-000000000000",
      "feature": {
        "name": "some_feature",
        "type": "STANDARD",
        "id": 1
      },
      "segment_id": null,
      "enabled": true
    },
    {
      "feature_state_value": "default_value",
      "django_id": 2,
      "featurestate_uuid": "11111111-1111-1111-1111-111111111111",
      "feature": {
        "name": "mv_feature_with_ids",
        "type": "MULTIVARIATE",
        "id": 2
      },
      "segment_id": null,
      "enabled": true,
      "multivariate_feature_state_values": [
        {
          "id": 100,
          "multivariate_feature_option": {
            "id": 10,
            "value": "variant_a"
          },
          "mv_fs_value_uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
          "percentage_allocation": 30.0
        },
        {
          "id": 200,
          "multivariate_feature_option": {
            "id": 20,
            "value": "variant_b"
          },
          "mv_fs_value_uuid": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
          "percentage_allocation": 70.0
        }
      ]
    },
    {
      "feature_state_value": "fallback_value",
      "django_id": 3,
      "featurestate_uuid": "22222222-2222-2222-2222-222222222222",
      "feature": {
        "name": "mv_feature_without_ids",
        "type": "MULTIVARIATE",
        "id": 3
      },
      "segment_id": null,
      "enabled": false,
      "multivariate_feature_state_values": [
        {
          "multivariate_feature_option": {
            "id": 40,
            "value": "option_y"
          },
          "mv_fs_value_uuid": "yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy",
          "percentage_allocation": 50.0
        },
        {
          "multivariate_feature_option": {
            "id": 30,
            "value": "option_x"
          },
          "mv_fs_value_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
          "percentage_allocation": 25.0
        },
        {
          "multivariate_feature_option": {
            "id": 50,
            "value": "option_z"
          },
          "mv_fs_value_uuid": "zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz",
          "percentage_allocation": 25.0
        }
      ]
    }
  ],
  "identity_overrides": [
    {
      "identifier": "overridden-id",
      "identity_uuid": "0f21cde8-63c5-4e50-baca-87897fa6cd01",
      "created_date": "2019-08-27T14:53:45.698555Z",
      "updated_at": "2023-07-14T16:12:00.000000",
      "environment_api_key": "test_key",
      "identity_features": [
        {
          "id": 1,
          "feature": {
            "id": 1,
            "name": "some_feature",
            "type": "STANDARD"
          },
          "featurestate_uuid": "1bddb9a5-7e59-42c6-9be9-625fa369749f",
          "feature_state_value": "some-overridden-value",
          "enabled": false,
          "django_id": null,
          "environment": 1,
          "identity": null,
          "feature_segment": null
        }
      ]
    }
  ]
}"#
}

#[test]
fn test_environment_to_context_produces_evaluation_context() {
    let json = get_environment_fixture();
    let environment: Environment = serde_json::from_str(json).expect("Failed to parse environment");

    let context = environment_to_context(environment);

    // Verify environment context
    assert_eq!(context.environment.key, "test_key");
    assert_eq!(context.environment.name, "Test Environment");
    assert!(context.identity.is_none());
    assert_eq!(context.segments.len(), 2);

    // Verify API segment
    assert!(context.segments.contains_key("1"));
    let api_segment = context.segments.get("1").unwrap();
    assert_eq!(api_segment.key, "1");
    assert_eq!(api_segment.name, "Test segment");
    assert_eq!(api_segment.rules.len(), 1);
    assert!(api_segment.overrides.is_empty());
    assert_eq!(api_segment.metadata.source, SegmentSource::Api);
    assert_eq!(api_segment.metadata.segment_id, Some(1));

    // Verify segment rule structure
    assert_eq!(api_segment.rules[0].rule_type, SegmentRuleType::All);
    assert!(api_segment.rules[0].conditions.is_empty());
    assert_eq!(api_segment.rules[0].rules.len(), 1);

    assert_eq!(
        api_segment.rules[0].rules[0].rule_type,
        SegmentRuleType::All
    );
    assert_eq!(api_segment.rules[0].rules[0].conditions.len(), 1);
    assert!(api_segment.rules[0].rules[0].rules.is_empty());

    assert_eq!(api_segment.rules[0].rules[0].conditions[0].property, "foo");
    assert_eq!(
        api_segment.rules[0].rules[0].conditions[0].operator,
        ConditionOperator::Equal
    );
    assert_eq!(
        api_segment.rules[0].rules[0].conditions[0]
            .value
            .as_string(),
        "bar"
    );

    // Verify identity override segment exists
    let override_segments: Vec<_> = context
        .segments
        .iter()
        .filter(|(_, s)| s.metadata.source == SegmentSource::IdentityOverride)
        .collect();
    assert_eq!(override_segments.len(), 1);

    let (_, override_segment) = override_segments[0];
    assert_eq!(override_segment.key, "");
    assert_eq!(override_segment.name, "identity_overrides");
    assert_eq!(override_segment.rules.len(), 1);
    assert_eq!(override_segment.overrides.len(), 1);

    assert_eq!(override_segment.rules[0].rule_type, SegmentRuleType::All);
    assert_eq!(override_segment.rules[0].conditions.len(), 1);
    assert!(override_segment.rules[0].rules.is_empty());

    assert_eq!(
        override_segment.rules[0].conditions[0].property,
        "$.identity.identifier"
    );
    assert_eq!(
        override_segment.rules[0].conditions[0].operator,
        ConditionOperator::In
    );
    match &override_segment.rules[0].conditions[0].value {
        ConditionValue::Multiple(identifiers) => {
            assert_eq!(identifiers, &vec!["overridden-id".to_string()]);
        }
        _ => panic!("Expected Multiple condition value"),
    }

    assert_eq!(override_segment.overrides[0].key, "");
    assert_eq!(override_segment.overrides[0].name, "some_feature");
    assert!(!override_segment.overrides[0].enabled);
    assert_eq!(
        override_segment.overrides[0].value.value,
        "some-overridden-value"
    );
    assert_eq!(
        override_segment.overrides[0].priority,
        Some(f64::NEG_INFINITY)
    );
    assert!(override_segment.overrides[0].variants.is_empty());
    assert_eq!(override_segment.overrides[0].metadata.feature_id, 1);

    // Verify features
    assert_eq!(context.features.len(), 3);

    // Verify some_feature
    assert!(context.features.contains_key("some_feature"));
    let some_feature = context.features.get("some_feature").unwrap();
    assert_eq!(some_feature.key, "00000000-0000-0000-0000-000000000000");
    assert_eq!(some_feature.name, "some_feature");
    assert!(some_feature.enabled);
    assert_eq!(some_feature.value.value, "some-value");
    assert!(some_feature.priority.is_none());
    assert!(some_feature.variants.is_empty());
    assert_eq!(some_feature.metadata.feature_id, 1);

    // Verify multivariate feature with IDs
    assert!(context.features.contains_key("mv_feature_with_ids"));
    let mv_feature_with_ids = context.features.get("mv_feature_with_ids").unwrap();
    assert_eq!(mv_feature_with_ids.key, "2");
    assert_eq!(mv_feature_with_ids.name, "mv_feature_with_ids");
    assert!(mv_feature_with_ids.enabled);
    assert_eq!(mv_feature_with_ids.value.value, "default_value");
    assert!(mv_feature_with_ids.priority.is_none());
    assert_eq!(mv_feature_with_ids.variants.len(), 2);
    assert_eq!(mv_feature_with_ids.metadata.feature_id, 2);

    // Verify variants
    assert_eq!(mv_feature_with_ids.variants[0].value.value, "variant_a");
    assert_eq!(mv_feature_with_ids.variants[0].weight, 30.0);

    assert_eq!(mv_feature_with_ids.variants[1].value.value, "variant_b");
    assert_eq!(mv_feature_with_ids.variants[1].weight, 70.0);

    // Verify multivariate feature without IDs
    assert!(context.features.contains_key("mv_feature_without_ids"));
    let mv_feature_without_ids = context.features.get("mv_feature_without_ids").unwrap();
    assert_eq!(mv_feature_without_ids.key, "3");
    assert_eq!(mv_feature_without_ids.name, "mv_feature_without_ids");
    assert!(!mv_feature_without_ids.enabled);
    assert_eq!(mv_feature_without_ids.value.value, "fallback_value");
    assert!(mv_feature_without_ids.priority.is_none());
    assert_eq!(mv_feature_without_ids.variants.len(), 3);
    assert_eq!(mv_feature_without_ids.metadata.feature_id, 3);

    // Verify variants preserve order from input
    assert_eq!(mv_feature_without_ids.variants[0].value.value, "option_y");
    assert_eq!(mv_feature_without_ids.variants[0].weight, 50.0);

    assert_eq!(mv_feature_without_ids.variants[1].value.value, "option_x");
    assert_eq!(mv_feature_without_ids.variants[1].weight, 25.0);

    assert_eq!(mv_feature_without_ids.variants[2].value.value, "option_z");
    assert_eq!(mv_feature_without_ids.variants[2].weight, 25.0);
}

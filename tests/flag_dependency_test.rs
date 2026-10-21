use flagsmith_flag_engine::engine::get_evaluation_result;
use flagsmith_flag_engine::engine_eval::{is_context_in_segment, EngineEvaluationContext};
use rstest::*;
use serde_json::json;

#[fixture]
fn prerequisite_name() -> &'static str {
    "prerequisite"
}

#[fixture]
fn context(
    prerequisite_name: &str,
    #[default("$.flags.prerequisite.enabled")] property: &str,
) -> EngineEvaluationContext {
    serde_json::from_value(json!({
        "environment": {"key": "key", "name": "Environment"},
        "features": {
            prerequisite_name: {
                "key": "1",
                "name": prerequisite_name,
                "enabled": true,
                "value": null
            },
            "dependent": {
                "key": "2",
                "name": "dependent",
                "enabled": false,
                "value": "off"
            }
        },
        "segments": {
            "1": {
                "key": "1",
                "name": "dependency",
                "rules": [{
                    "type": "ALL",
                    "conditions": [{
                        "operator": "EQUAL",
                        "property": property,
                        "value": "true"
                    }]
                }],
                "overrides": [{
                    "key": "2",
                    "name": "dependent",
                    "enabled": true,
                    "value": "on"
                }]
            }
        }
    }))
    .unwrap()
}

#[rstest]
#[case("it's on", r#"$.flags["it's on"].enabled"#)]
#[case("it's on", r"$.flags['it\'s on'].enabled")]
#[case(r#"say "on""#, r#"$.flags['say "on"'].enabled"#)]
#[case(r#"say "on""#, r#"$.flags["say \"on\""].enabled"#)]
#[case("café", r"$.flags['caf\u00e9'].enabled")]
#[case("café", "$.flags.café.enabled")]
#[case("prerequisite_2", "$.flags.prerequisite_2.enabled")]
#[case("prerequisite", "$.flags .prerequisite .enabled")]
#[case("prerequisite", "$.flags\t[ 'prerequisite' ]\n.enabled")]
#[case("prerequisite", "$.flags[\r\"prerequisite\" ]['enabled']")]
fn test_flag_named_in_any_jsonpath_notation_can_be_depended_on(
    #[case] prerequisite_name: &str,
    #[case] property: &str,
) {
    // Given
    let context = context(prerequisite_name, property);

    // When
    let result = get_evaluation_result(&context);

    // Then
    let dependent = &result.flags["dependent"];
    assert_eq!(dependent.value.value, "on");
    assert_eq!(dependent.reason, "TARGETING_MATCH; segment=dependency");
    assert_eq!(result.segments.len(), 1);
}

#[rstest]
#[case("$.flags[?@.enabled == true].enabled")]
#[case("$.flags['prerequisite', 'dependent'].enabled")]
#[case("$.flags..enabled")]
#[case("$.flags.prerequisite")]
#[case("$.flagsprerequisite.enabled")]
fn test_flags_not_selected_by_name_cannot_be_depended_on(#[case] property: &str) {
    // Given
    let context = context(prerequisite_name(), property);

    // When
    let result = get_evaluation_result(&context);

    // Then
    let dependent = &result.flags["dependent"];
    assert_eq!(dependent.value.value, "off");
    assert_eq!(dependent.reason, "DEFAULT");
    assert!(result.segments.is_empty());
}

#[rstest]
fn test_segment_membership_alone_cannot_read_flags(context: EngineEvaluationContext) {
    // Given
    let segment = &context.segments["1"];

    // When
    let is_member = is_context_in_segment(&context, segment);

    // Then
    assert!(!is_member);
}

use flagsmith_rust_flag_engine::identities;
use flagsmith_rust_flag_engine::segments;
use rstest::*;
mod fixtures;

#[rstest]
#[case(fixtures::empty_segment(), vec![], false)]
#[case(fixtures::segment_single_condition(), vec![], false)]
#[case(fixtures::segment_single_condition(), vec![fixtures::trait_1()], true)]
#[case(fixtures::segment_multiple_conditions_all(), vec![], false)]
#[case(fixtures::segment_multiple_conditions_all(), vec![fixtures::trait_1()], false)]
#[case(fixtures::segment_multiple_conditions_all(), vec![fixtures::trait_1(), fixtures::trait_2()], true)]
#[case(fixtures::segment_multiple_conditions_any(), vec![], false)]
#[case(fixtures::segment_multiple_conditions_any(), vec![fixtures::trait_1()], true)]
#[case(fixtures::segment_multiple_conditions_any(), vec![fixtures::trait_2()], true)]
#[case(fixtures::segment_multiple_conditions_any(), vec![fixtures::trait_1(), fixtures::trait_2()], true)]
#[case(fixtures::segment_nested_rules_all(), vec![], false)]
#[case(fixtures::segment_nested_rules_all(), vec![fixtures::trait_1()], false)]
#[case(fixtures::segment_nested_rules_all(), vec![fixtures::trait_2()], false)]
#[case(fixtures::segment_nested_rules_all(), vec![fixtures::trait_1(), fixtures::trait_2()], false)]
#[case(fixtures::segment_nested_rules_all(), vec![fixtures::trait_1(), fixtures::trait_2(), fixtures::trait_3()], true)]
#[case(fixtures::segment_conditions_and_nested_rules(), vec![], false)]
#[case(fixtures::segment_conditions_and_nested_rules(), vec![fixtures::trait_1()], false)]
#[case(fixtures::segment_conditions_and_nested_rules(), vec![fixtures::trait_2()], false)]
#[case(fixtures::segment_conditions_and_nested_rules(), vec![fixtures::trait_1(), fixtures::trait_2()], false)]
#[case(fixtures::segment_conditions_and_nested_rules(), vec![fixtures::trait_1(), fixtures::trait_2(), fixtures::trait_3()], true)]
fn test_evaluate_identity_in_segment(
    #[case] segment: segments::Segment,
    #[case] identity_traits: Vec<identities::Trait>,
    #[case] expected_result: bool,
) {
    let identity = identities::Identity {
        identifier: "foo".to_string(),
        identity_uuid: "".to_string(),
        identity_features: vec![],
        identity_traits: identity_traits,
        django_id: None,
        created_date: chrono::Utc::now(),
        environment_api_key: "test_api_key".to_string(),
    };
    assert_eq!(
        segments::evaluator::evaluate_identity_in_segment(&identity, &segment, None),
        expected_result
    );
}

use rstest::*;

use flagsmith_rust_flag_engine::segments;

use flagsmith_rust_flag_engine::features::featurestate_value;
use flagsmith_rust_flag_engine::identities;
use flagsmith_rust_flag_engine::segments::constants;

const TRAIT_KEY_1: &str = "email";
const TRAIT_VALUE_1: &str = "user@example.com";

const TRAIT_KEY_2: &str = "num_purchase";
const TRAIT_VALUE_2: &str = "12";

const TRAIT_KEY_3: &str = "date_joined";
const TRAIT_VALUE_3: &str = "2021-01-01";
#[fixture]
pub fn identity() -> identities::Identity {
    identities::Identity {
        identifier: "foo".to_string(),
        identity_uuid: "".to_string(),
        identity_features: vec![],
        identity_traits: vec![],
        django_id: None,
        created_date: chrono::Utc::now(),
        environment_api_key: "test_api_key".to_string(),
    }
}

pub fn trait_1() -> identities::Trait {
    identities::Trait {
        trait_key: TRAIT_KEY_1.to_string(),
        trait_value: featurestate_value::FeatureStateValue {
            value: TRAIT_VALUE_1.to_string(),
            value_type: featurestate_value::FeatureStateValueType::String,
        },
    }
}

pub fn trait_2() -> identities::Trait {
    identities::Trait {
        trait_key: TRAIT_KEY_2.to_string(),
        trait_value: featurestate_value::FeatureStateValue {
            value: TRAIT_VALUE_2.to_string(),
            value_type: featurestate_value::FeatureStateValueType::String,
        },
    }
}

pub fn trait_3() -> identities::Trait {
    identities::Trait {
        trait_key: TRAIT_KEY_3.to_string(),
        trait_value: featurestate_value::FeatureStateValue {
            value: TRAIT_VALUE_3.to_string(),
            value_type: featurestate_value::FeatureStateValueType::String,
        },
    }
}

#[fixture]
pub fn empty_segment() -> segments::Segment {
    segments::Segment {
        id: 1,
        name: "empty_segment".to_string(),
        rules: vec![],
        feature_states: vec![],
    }
}

pub fn segment_single_condition() -> segments::Segment {
    segments::Segment {
        id: 2,
        name: "segment_one_condition".to_string(),
        rules: vec![segments::SegmentRule {
            r#type: constants::ALL_RULE.to_string(),
            rules: vec![],
            conditions: vec![segments::SegmentCondition {
                operator: constants::EQUAL.to_string(),
                property: Some(TRAIT_KEY_1.to_string()),
                value: TRAIT_VALUE_1.to_string(),
            }],
        }],
        feature_states: vec![],
    }
}

pub fn segment_multiple_conditions_all() -> segments::Segment {
    segments::Segment {
        id: 3,
        name: "segment_multiple_conditions_all".to_string(),
        rules: vec![segments::SegmentRule {
            r#type: constants::ALL_RULE.to_string(),
            rules: vec![],
            conditions: vec![
                segments::SegmentCondition {
                    operator: constants::EQUAL.to_string(),
                    property: Some(TRAIT_KEY_1.to_string()),
                    value: TRAIT_VALUE_1.to_string(),
                },
                segments::SegmentCondition {
                    operator: constants::EQUAL.to_string(),
                    property: Some(TRAIT_KEY_2.to_string()),
                    value: TRAIT_VALUE_2.to_string(),
                },
            ],
        }],
        feature_states: vec![],
    }
}

pub fn segment_multiple_conditions_any() -> segments::Segment {
    segments::Segment {
        id: 4,
        name: "segment_multiple_conditions_any".to_string(),
        rules: vec![segments::SegmentRule {
            r#type: constants::ANY_RULE.to_string(),
            rules: vec![],
            conditions: vec![
                segments::SegmentCondition {
                    operator: constants::EQUAL.to_string(),
                    property: Some(TRAIT_KEY_1.to_string()),
                    value: TRAIT_VALUE_1.to_string(),
                },
                segments::SegmentCondition {
                    operator: constants::EQUAL.to_string(),
                    property: Some(TRAIT_KEY_2.to_string()),
                    value: TRAIT_VALUE_2.to_string(),
                },
            ],
        }],
        feature_states: vec![],
    }
}

pub fn segment_nested_rules_all() -> segments::Segment {
    segments::Segment {
        id: 5,
        name: "segment_nested_rules_all".to_string(),
        rules: vec![segments::SegmentRule {
            r#type: constants::ALL_RULE.to_string(),
            conditions: vec![],
            rules: vec![
                Box::new({
                    segments::SegmentRule {
                        r#type: constants::ALL_RULE.to_string(),
                        rules: vec![],
                        conditions: vec![
                            segments::SegmentCondition {
                                operator: constants::EQUAL.to_string(),
                                property: Some(TRAIT_KEY_1.to_string()),
                                value: TRAIT_VALUE_1.to_string(),
                            },
                            segments::SegmentCondition {
                                operator: constants::EQUAL.to_string(),
                                property: Some(TRAIT_KEY_2.to_string()),
                                value: TRAIT_VALUE_2.to_string(),
                            },
                        ],
                    }
                }),
                Box::new(segments::SegmentRule {
                    r#type: constants::ALL_RULE.to_string(),
                    rules: vec![],
                    conditions: vec![segments::SegmentCondition {
                        operator: constants::EQUAL.to_string(),
                        property: Some(TRAIT_KEY_3.to_string()),
                        value: TRAIT_VALUE_3.to_string(),
                    }],
                }),
            ],
        }],
        feature_states: vec![],
    }
}

pub fn segment_conditions_and_nested_rules() -> segments::Segment {
    segments::Segment {
        id: 6,
        name: "segment_multiple_conditions_all_and_nested_rules".to_string(),
        rules: vec![segments::SegmentRule {
            r#type: constants::ALL_RULE.to_string(),
            conditions: vec![segments::SegmentCondition {
                operator: constants::EQUAL.to_string(),
                property: Some(TRAIT_KEY_1.to_string()),
                value: TRAIT_VALUE_1.to_string(),
            }],
            rules: vec![
                Box::new({
                    segments::SegmentRule {
                        r#type: constants::ALL_RULE.to_string(),
                        rules: vec![],
                        conditions: vec![segments::SegmentCondition {
                            operator: constants::EQUAL.to_string(),
                            property: Some(TRAIT_KEY_2.to_string()),
                            value: TRAIT_VALUE_2.to_string(),
                        }],
                    }
                }),
                Box::new({
                    segments::SegmentRule {
                        r#type: constants::ALL_RULE.to_string(),
                        rules: vec![],
                        conditions: vec![segments::SegmentCondition {
                            operator: constants::EQUAL.to_string(),
                            property: Some(TRAIT_KEY_3.to_string()),
                            value: TRAIT_VALUE_3.to_string(),
                        }],
                    }
                }),
            ],
        }],
        feature_states: vec![],
    }
}

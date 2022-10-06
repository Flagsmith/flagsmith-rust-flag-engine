use super::constants;
use super::Segment;
use super::SegmentCondition;
use super::SegmentRule;
use crate::environments;
use crate::identities;

use crate::utils::hashing::get_hashed_percentage_for_object_ids;

pub fn get_identity_segments(
    environment: &environments::Environment,
    identity: &identities::Identity,
    override_traits: Option<&Vec<identities::Trait>>,
) -> Vec<Segment> {
    environment
        .project
        .segments
        .clone()
        .into_iter()
        .filter(|segment| evaluate_identity_in_segment(&identity, &segment, override_traits))
        .collect()
}

pub fn evaluate_identity_in_segment(
    identity: &identities::Identity,
    segment: &Segment,
    override_traits: Option<&Vec<identities::Trait>>,
) -> bool {
    let traits = override_traits.unwrap_or(&identity.identity_traits);
    let identity_id = match identity.django_id {
        Some(django_id) => django_id.to_string(),
        None => identity.composite_key(),
    };
    segment.rules.len() > 0
        && segment
            .rules
            .iter()
            .map(|rule| {
                traits_match_segment_rule(traits, rule, &segment.id.to_string(), &identity_id)
            })
            .all(|result| result)
}

fn traits_match_segment_rule(
    identity_traits: &Vec<identities::Trait>,
    rule: &SegmentRule,
    segment_id: &str,
    identity_id: &str,
) -> bool {
    let mut rules_iterator = rule.conditions.iter().map(|condition| {
        traits_match_segment_condition(&identity_traits, condition, segment_id, identity_id)
    });
    let matches_condtion = match rule.segment_rule_type.as_str() {
        constants::ANY_RULE => rules_iterator.any(|result| result == true),
        constants::ALL_RULE => rules_iterator.all(|result| result == true),
        constants::NONE_RULE => true,
        _ => false,
    };
    return matches_condtion
        && rule
            .rules
            .iter()
            .map(|rule| {
                traits_match_segment_rule(&identity_traits, rule.as_ref(), segment_id, identity_id)
            })
            .all(|result| result == true);
}

fn traits_match_segment_condition(
    identity_traits: &Vec<identities::Trait>,
    condition: &SegmentCondition,
    segment_id: &str,
    identity_id: &str,
) -> bool {
    if condition.operator == constants::PERCENTAGE_SPLIT {
        let float_value: f32 = condition.value.as_ref().unwrap().parse().unwrap();
        return get_hashed_percentage_for_object_ids(vec![segment_id, identity_id], 1)
            <= float_value;
    }
    match condition.property.clone() {
        Some(property) => {
            let identity_trait = identity_traits
                .iter()
                .filter(|identity_trait| identity_trait.trait_key == property)
                .next();
            if condition.operator == constants::IS_SET {
                return identity_trait.is_some();
            }

            if condition.operator == constants::IS_NOT_SET {
                return identity_trait.is_none();
            }

            if identity_trait.is_some() {
                return condition.matches_trait_value(&identity_trait.unwrap().trait_value);
            }

            return false;
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::types::FlagsmithValue;

    use super::*;
    use rstest::*;

    #[rstest]
    #[case(constants::IS_SET, "foo", "foo", true)]
    #[case(constants::IS_SET, "foo", "bar", false)]
    #[case(constants::IS_NOT_SET, "foo", "foo", false)]
    #[case(constants::IS_NOT_SET, "foo", "bar", true)]
    fn trait_matches_segmnt_condition_is_and_is_not(
        #[case] operator: &str,
        #[case] property: &str,
        #[case] trait_key: &str,
        #[case] expected_result: bool,
    ) {
        let condition = SegmentCondition {
            property: Some(property.to_string()),
            operator: operator.to_string(),
            value: None,
        };
        let traits = vec![identities::Trait {
            trait_key: trait_key.to_string(),
            trait_value: FlagsmithValue {
                value: "".to_string(),
                value_type: crate::types::FlagsmithValueType::None,
            },
        }];
        let result = traits_match_segment_condition(&traits, &condition, "1", "1");
        assert_eq!(result, expected_result);
    }
}

use super::constants;
use super::Segment;
use super::SegmentCondition;
use super::SegmentRule;
use crate::environments;
use crate::identities;

use crate::utils::hashing;
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
    segment.rules.len() > 0
        && segment
            .rules
            .iter()
            .map(|rule| {
                traits_match_segment_rule(
                    traits,
                    rule,
                    &segment.id.to_string(),
                    &identity.composite_key(),
                )
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
    // TODO: true if rules.conditions.len == 0
    let matches_condtion = match rule.r#type.as_str() {
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
        let float_value: f32 = condition.value.parse().unwrap();
        return get_hashed_percentage_for_object_ids(vec![segment_id, identity_id], 1)
            <= float_value;
    }
    match condition.property.clone() {
        Some(property) => {
            let identity_trait = identity_traits
                .iter()
                .filter(|identity_trait| identity_trait.trait_key == property)
                .next();
            match identity_trait {
                Some(_trait) => condition.matches_trait_value(_trait.trait_value.clone()),
                None => false,
            }
        }
        None => false,
    }
}

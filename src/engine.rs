use super::environments;
use super::features;
use super::identities;
use super::segments::evaluator;
use crate::features::Feature;
use crate::features::FeatureState;
use std::collections::HashMap;

pub fn get_environment_feature_states(
    environment: environments::Environment,
) -> Vec<features::FeatureState> {
    if environment.project.hide_disabled_flags {
        return environment
            .feature_states
            .iter()
            .filter(|fs| fs.enabled)
            .map(|fs| fs.clone())
            .collect();
    }
    return environment.feature_states;
}

pub fn get_environment_feature_state(
    environment: environments::Environment,
    feature_name: &str,
) -> features::FeatureState {
    // TODO handle error here
    return environment
        .feature_states
        .iter()
        .filter(|fs| fs.feature.name == feature_name)
        .next()
        .unwrap()
        .clone();
}

pub fn get_identity_feature_states(
    environment: &environments::Environment,
    identity: &identities::Identity,
    override_traits: Option<&Vec<identities::Trait>>,
) -> Vec<features::FeatureState> {
    let feature_states =
        get_identity_feature_states_map(environment, identity, override_traits).into_values();
    if environment.project.hide_disabled_flags {
        return feature_states.filter(|fs| fs.enabled).collect();
    }
    return feature_states.collect();
}

pub fn get_identity_feature_state(
    environment: &environments::Environment,
    identity: &identities::Identity,
    feature_name: &str,
    override_traits: Option<&Vec<identities::Trait>>,
) -> features::FeatureState {
    let feature_states =
        get_identity_feature_states_map(environment, identity, override_traits).into_values();
    feature_states
        .filter(|fs| fs.feature.name == feature_name)
        .next()
        .unwrap()
}

fn get_identity_feature_states_map(
    environment: &environments::Environment,
    identity: &identities::Identity,
    override_traits: Option<&Vec<identities::Trait>>,
) -> HashMap<Feature, FeatureState> {
    let mut feature_states: HashMap<Feature, FeatureState> = HashMap::new();

    // Get feature states from the environment
    for fs in environment.feature_states.clone() {
        feature_states.insert(fs.feature.clone(), fs);
    }

    // Override with any feature states defined by matching segments
    let identity_segments =
        evaluator::get_identity_segments(environment, identity, override_traits);
    for matching_segments in identity_segments {
        for feature_state in matching_segments.feature_states {
            feature_states.insert(feature_state.feature.clone(), feature_state);
        }
    }
    // Override with any feature states defined directly the identity
    for feature_state in identity.identity_features.clone() {
        feature_states.insert(feature_state.feature.clone(), feature_state);
    }
    return feature_states;
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENVIRONMENT_JSON: &str = r#"
        {
 "api_key": "test_key",
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
  "hide_disabled_flags": true,
  "segments": []
 },
 "segment_overrides": [],
 "id": 1,
 "feature_states": [
  {
   "multivariate_feature_state_values": [],
   "feature_state_value": true,
   "django_id": 1,
   "feature": {
    "name": "feature1",
    "type": null,
    "id": 1
   },
   "enabled": false
  },
  {
   "multivariate_feature_state_values": [],
   "feature_state_value": null,
   "django_id": 2,
   "feature": {
    "name": "feature_2",
    "type": null,
    "id": 2
   },
   "enabled": true
  }
 ]
}"#;

    #[test]
    fn get_environment_feature_states_only_return_enabled_fs_if_hide_disabled_flags_is_true() {
        let environment: environments::Environment =
            serde_json::from_str(ENVIRONMENT_JSON).unwrap();

        let environment_feature_states = get_environment_feature_states(environment);
        assert_eq!(environment_feature_states.len(), 1);
        assert_eq!(environment_feature_states[0].django_id.unwrap(), 2);
    }

    #[test]
    fn get_environment_feature_state_returns_correct_feature_state() {
        let environment: environments::Environment =
            serde_json::from_str(ENVIRONMENT_JSON).unwrap();
        let feature_name = "feature_2";
        let feature_state = get_environment_feature_state(environment, feature_name);
        assert_eq!(feature_state.feature.name, feature_name)
    }
}

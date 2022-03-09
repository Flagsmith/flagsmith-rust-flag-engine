use super::environments;
use super::features;
use super::identities;

pub fn get_environment_feature_states(environment: environments::Environment) -> Vec<features::FeatureState> {
    return environment.feature_states

}

pub fn get_environment_feature_state(environment: environments::Environment, feature_name: String)
                                     -> features::FeatureState {
    return environment.feature_states[0].clone()

}

pub fn get_identity_feature_states(environment: environments::Environment,
                                   identity: identities::Identity,
                                   override_traits: Vec<identities::Trait>
) -> Vec<features::FeatureState> {

        return environment.feature_states
}
pub fn get_identity_feature_state(environment: environments::Environment,
                                  identity: identities::Identity,
                                  feature_name: String,
                                  override_traits: Vec<identities::Trait>
) ->features::FeatureState {

    return environment.feature_states[0].clone()
}

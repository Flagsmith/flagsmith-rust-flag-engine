use super::Identity;

pub fn build_identity_struct(value: serde_json::Value) -> Identity {
    let identity: Identity = serde_json::from_value(value).unwrap();
    return identity;
}

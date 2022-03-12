use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Organisation{
    pub id: u32,
    pub name: String,
    pub feature_analytics: bool,
    pub stop_serving_flags: bool,
    pub persist_trait_data: bool
}

impl Organisation{
    pub fn unique_slug(&self) -> String {
        return self.id.to_string() + "_" + &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unqiue_slug_is_correct(){
        let expected_slug = "1_test_org";
        let organisation_json = r#"{
            "id": 1,
            "name": "test_org",
            "feature_analytics": true,
            "stop_serving_flags": false,
            "persist_trait_data": true
        }"#;

        let organisation: Organisation = serde_json::from_str(organisation_json).unwrap();
        assert_eq!(organisation.unique_slug(), expected_slug)
    }
}

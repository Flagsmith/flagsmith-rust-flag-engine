mod features;
mod projects;
mod organisations;
mod environments;
mod segments;


fn main(){
    let feature_json = r#"
        {
 "api_key": "B62qaMZNwfiqT76p38ggrQ",
 "project": {
  "name": "Test project",
  "organisation": {
   "feature_analytics": false,
   "name": "Test Org",
   "id": 12,
   "persist_trait_data": true,
   "stop_serving_flags": false
  },
  "id": 12,
  "hide_disabled_flags": false,
  "segments": []
 },
 "segment_overrides": [
  {
   "multivariate_feature_state_values": [],
   "feature_state_value": null,
   "id": 3,
   "feature": {
    "name": "feature1",
    "type": null,
    "id": 2
   },
   "segment_id": null,
   "enabled": false
  },
  {
   "multivariate_feature_state_values": [],
   "feature_state_value": null,
   "id": 4,
   "feature": {
    "name": "feature1",
    "type": null,
    "id": 2
   },
   "segment_id": null,
   "enabled": true
  }
 ],
 "id": 12,
 "feature_states": [
  {
   "multivariate_feature_state_values": [],
   "feature_state_value": null,
   "id": 3,
   "feature": {
    "name": "feature1",
    "type": null,
    "id": 2
   },
   "segment_id": null,
   "enabled": false
  },
  {
   "multivariate_feature_state_values": [],
   "feature_state_value": null,
   "id": 4,
   "feature": {
    "name": "feature1",
    "type": null,
    "id": 2
   },
   "segment_id": null,
   "enabled": true
  }
 ]
}"#;
    let env: environments::Environment = serde_json::from_str(feature_json).unwrap();
    println!(" Project name{}",env.project.name );
}

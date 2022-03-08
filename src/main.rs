use std::vec;

use serde::{Deserialize, Serialize};
use serde_json::Result;
mod features;
#[derive(Serialize, Deserialize)]
struct Organisation{
    id: u32,
    name: String,
    feature_analytics: bool,
    stop_serving_flags: bool,
    persist_trait_data: bool
}

#[derive(Serialize, Deserialize)]
struct Segment{
    id: u32,
    name: String, //Add rest of it
}

#[derive(Serialize, Deserialize)]
struct Project {
    id: u32,
    name: String,
    organisation: Organisation,
    hide_disabled_flags: bool,
    segments: Vec<Segment>
}

#[derive(Serialize, Deserialize)]
struct Environment{
    id: u32,
    api_key: String,
    project: Project,
    feature_states: Vec<features::FeatureState>

}
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
    let env: Environment = serde_json::from_str(feature_json).unwrap();
    println!(" Project name{}",env.project.name );
     
}

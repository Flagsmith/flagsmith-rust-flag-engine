use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct FeatureModel{
    id: u32,
    name:String,
    _type: String,


}



use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Organisation{
    pub id: u32,
    pub name: String,
    pub feature_analytics: bool,
    pub stop_serving_flags: bool,
    pub persist_trait_data: bool
}

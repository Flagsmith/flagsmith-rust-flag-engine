use super::organisations;
use super::segments;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub organisation: organisations::Organisation,
    pub hide_disabled_flags: bool,
    pub segments: Vec<segments::Segment>,
}

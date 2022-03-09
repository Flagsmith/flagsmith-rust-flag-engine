use serde::{Deserialize, Serialize};
use super::organisations;

#[derive(Serialize, Deserialize)]
pub struct Segment{
    pub id: u32,
    pub name: String, //Add rest of it
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub organisation: organisations::Organisation,
    pub hide_disabled_flags: bool,
    pub segments: Vec<Segment>
}

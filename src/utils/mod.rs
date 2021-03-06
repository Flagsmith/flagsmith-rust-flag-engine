pub mod hashing;
use uuid::Uuid;
pub mod datetime;

pub fn get_uuid() -> String {
    return Uuid::new_v4().to_hyphenated().to_string();
}

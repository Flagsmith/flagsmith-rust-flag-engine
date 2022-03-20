use md5::{Md5, Digest};
use num_traits::cast::ToPrimitive;

use num_bigint;

use num_bigint::Sign::{Minus, NoSign, Plus};
fn to_u32(slice: &[u8]) -> u32 {
    slice.iter().rev().fold(0, |acc, &b| acc*2 + b as u32)
}
pub fn get_hashed_percentage_for_object_ids(object_ids: Vec<&str>, iterations: u32) -> f32{
    let mut to_hash = object_ids.join(",");
    let mut hasher = Md5::new();
    for _ in 1..iterations{
        to_hash.push_str(&object_ids.join(","))
    }
    hasher.update(to_hash);
    let hashed_value = hasher.finalize();
    let hashed_value_as_bigint = num_bigint::BigUint::from_bytes_be(&hashed_value);
    let hashed_value_as_int = (hashed_value_as_bigint%9999 as u32).to_u32().unwrap();
    let value = (hashed_value_as_int as f32 /9998.0)*100.0;
    println!("hello hash {:?}", value);
    return value
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_hashed_percentage_for_object_ids_is_the_same_each_time() {
        get_hashed_percentage_for_object_ids(vec!["1", "2"], 1);
    }



}


use md5::{Md5, Digest};
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
    // let hashed_value = md5::compute(to_hash);
    let hashed_value = hasher.finalize();
    let hashed_value_as_int = to_u32(&hashed_value);
    let value = ((hashed_value_as_int%9999) as f32 /9998.0)*100.0;
    println!("hello hash {:?}", value);
    return value
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_hashed_percentage_for_object_ids_is_the_same_each_time() {
        get_hashed_percentage_for_object_ids(vec!["boy", "you", "suck"], 1);
    }



}


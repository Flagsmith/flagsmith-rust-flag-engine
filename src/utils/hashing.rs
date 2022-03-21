use md5::{Digest, Md5};
use num_bigint;
use num_traits::cast::ToPrimitive;

pub fn get_hashed_percentage_for_object_ids(object_ids: &Vec<&str>, iterations: u32) -> f32 {
    let mut to_hash = object_ids.join(",");
    let mut hasher = Md5::new();
    for _ in 1..iterations {
        to_hash.push_str(&object_ids.join(","))
    }
    hasher.update(to_hash);
    let hash = hasher.finalize();
    let hash_as_bigint = num_bigint::BigUint::from_bytes_be(&hash);
    let hash_as_int = (hash_as_bigint % 9999 as u32).to_u32().unwrap();
    let value = (hash_as_int as f32 / 9998.0) * 100.0;
    return value;
}
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn hashed_percentage_for_object_ids_is_same_for_same_ids() {
        fn check(object_ids: &Vec<&str>) {
            let first_hashed_percetnage = get_hashed_percentage_for_object_ids(&object_ids, 1);
            let second_hashed_percetnage = get_hashed_percentage_for_object_ids(&object_ids, 1);
            assert_eq!(first_hashed_percetnage, second_hashed_percetnage);
        }
        check(&vec!["1", "2"]);
        check(&vec![&Uuid::new_v4().to_hyphenated().to_string(), "2"])
    }

    #[test]
    fn hashed_percentage_for_object_ids_is_different_for_different_ids() {
        let first_object_ids = vec!["1", "2"];
        let second_object_ids = vec!["9", "10"];
        let first_hashed_percetnage = get_hashed_percentage_for_object_ids(&first_object_ids, 1);
        let second_hashed_percetnage = get_hashed_percentage_for_object_ids(&second_object_ids, 1);
        assert!(first_hashed_percetnage != second_hashed_percetnage);
    }
}

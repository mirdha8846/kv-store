use sha2::{Sha256, Digest};
// use sled::Db;
// use super::config::{NODES};
use super::hashring::Hash;

//we use hashing because this is determinstic and for same key always genrate same number
// pub fn get_node_for_key(key: &str,total_nodes: usize) -> usize {
//     let mut hasher = Sha256::new();
//     hasher.update(key.as_bytes());
//     let result = hasher.finalize();
//     //here form 32bytes array we only take first 8 bytes because ..we just want to create a u64 number
//     //and 8 bytes are enough for that
//     //try_into()-> in rust we use to conevert any vector/array to fixed sized array
//     //from_be_bytes->8-byte array ko ek single u64 number me convert karta hai.
//     let hash_num = u64::from_be_bytes(result[..8].try_into().unwrap());

//     (hash_num % total_nodes as u64) as usize
// }

pub fn get_node_for_key(input: &str) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    u64::from_be_bytes(result[..8].try_into().unwrap())
}

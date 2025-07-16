use sha2::{Sha256, Digest};
use sled::Db;
use super::config::NODES;
pub fn get_node_for_key(key: &str) -> &'static Db {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    let hash_num = u64::from_be_bytes(result[..8].try_into().unwrap());

    let node_index = (hash_num % NODES.len() as u64) as usize;
    NODES[node_index]
}
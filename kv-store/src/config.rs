use once_cell::sync::Lazy;
use super::hashring::HashRing;
use std::{collections::HashMap, sync::RwLock};


#[derive(Debug, Clone)]
pub struct NodeHealth {
    pub id: String,
    pub last_heartbeat: u64, // epoch milliseconds
    pub is_alive: bool,
}

// Global Health Table
pub static HEALTH_TABLE: Lazy<RwLock<HashMap<String, NodeHealth>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

pub static HASH_RING: Lazy<RwLock<HashRing>> = Lazy::new(|| {
    let mut ring = HashRing::new(100); // 100 vnodes per node
    ring.add_node("node0", sled::open("db/node0").unwrap());
    ring.add_node("node1", sled::open("db/node1").unwrap());
    ring.add_node("node2", sled::open("db/node2").unwrap());
    ring.add_node("node3", sled::open("db/node3").unwrap());
    ring.add_node("node4", sled::open("db/node4").unwrap());
    RwLock::new(ring)
});


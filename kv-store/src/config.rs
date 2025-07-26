use once_cell::sync::Lazy;
use super::hashring::HashRing;
use std::sync::RwLock;

pub static HASH_RING: Lazy<RwLock<HashRing>> = Lazy::new(|| {
    let mut ring = HashRing::new(100); // 100 vnodes per node
    ring.add_node("node0", sled::open("db/node0").unwrap());
    ring.add_node("node1", sled::open("db/node1").unwrap());
    ring.add_node("node2", sled::open("db/node2").unwrap());
    ring.add_node("node3", sled::open("db/node3").unwrap());
    ring.add_node("node4", sled::open("db/node4").unwrap());
    RwLock::new(ring)
});


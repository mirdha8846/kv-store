use std::collections::BTreeMap;
// use std::sync::RwLock;
use sled::Db;
use super::ring::get_node_for_key;

pub type Hash = u64;

pub struct Node {
   pub id: String,
   pub db: Db,
}
/*
ring = {
    110000000   => "nodeC",
    1230000000  => "nodeA",
    1500000000  => "nodeB",
    2300000000  => "nodeB",
    3300000000  => "nodeC",
    4200000000  => "nodeA",
}

node_map = {
    "nodeA" => Node { id: "nodeA", db: sled::open("db/nodeA").unwrap() },
    "nodeB" => Node { id: "nodeB", db: sled::open("db/nodeB").unwrap() },
    "nodeC" => Node { id: "nodeC", db: sled::open("db/nodeC").unwrap() },
}

and vnode_count is ki ek node like(nodeA) ko kitne parts me divide krenge
*/
pub struct HashRing {
    ring: BTreeMap<Hash, String>, // hash → node_id
    vnode_count: usize,
    node_map: BTreeMap<String, Node>, // node_id → Node
}

impl HashRing {
    pub fn new(vnode_count: usize) -> Self {
        HashRing {
            ring: BTreeMap::new(),
            vnode_count,
            node_map: BTreeMap::new(),
        }
    }
    //placing all nodes in ring
    pub fn add_node(&mut self, node_id: &str, db: sled::Db) {
        let node = Node { id: node_id.into(), db };
        self.node_map.insert(node_id.into(), node);

        for i in 0..self.vnode_count {
            let vnode_key = format!("{}-{}", node_id, i);//eg. nodeA-1
            let hash = get_node_for_key(&vnode_key); //get hash for nodeA-1
            self.ring.insert(hash, node_id.to_string());//store in ring ,hash->nodeA
        }
    }

    pub fn remove_node(&mut self, node_id: &str) {
        self.node_map.remove(node_id);
        //ring.retain->work like filter
        //Jo entry "hash => node_id" match karti ho, usko ring se hata do
        self.ring.retain(|_, id| id != node_id);
    }
     // get node where data will store
    pub fn get_node(&self, key: &str) -> Option<&Node> {
        let hash = get_node_for_key(key);
        // clockwise search in ring
        let node_id = self.ring.range(hash..)
            .next()
            .or_else(|| self.ring.iter().next())  // wrap-around(ring ke start se wapas search start.)
            .map(|(_, id)| id)?;//Ring me (hash, id) pair return hota hai,but hame bas id chahiye, isliye 

        self.node_map.get(node_id)
    }

    // Get node by ID for replica access
  pub fn get_follower_node_ids(&self, key: &str) -> Vec<String> {
    let leader = match self.get_node(key) {
        Some(node) => node,
        None => return Vec::new(),
    };
    let leader_id = &leader.id;

    let mut node_ids: Vec<String> = self.node_map.keys().cloned().collect();
    node_ids.sort();

    let leader_index = match node_ids.iter().position(|id| id == leader_id) {
        Some(idx) => idx,
        None => return Vec::new(),
    };

    let total = node_ids.len();
    let follower1 = node_ids[(leader_index + 1) % total].clone();
    let follower2 = node_ids[(leader_index + 2) % total].clone();

    vec![follower1, follower2]
}
pub fn get_all_node_ids(&self) -> Vec<String> {
        self.node_map.keys().cloned().collect()
    }
     pub fn get_node_by_id(&self, id: &str) -> Option<&Node> {
        self.node_map.get(id)
    }
}

use once_cell::sync::Lazy;
use sled::Db;


pub struct Node {
   pub id: String,
   pub db: Db,
}

pub static NODES: Lazy<Vec<Node>> = Lazy::new(|| {
    vec![
        Node { id: "node0".into(), db: sled::open("db/node0").unwrap() },
        Node { id: "node1".into(), db: sled::open("db/node1").unwrap() },
        Node { id: "node2".into(), db: sled::open("db/node2").unwrap() },
        Node { id: "node3".into(), db: sled::open("db/node3").unwrap() },
        Node { id: "node4".into(), db: sled::open("db/node4").unwrap() },
    ]
});
